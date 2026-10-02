#!/usr/bin/env bash
set -euo pipefail

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
PROJECT_ROOT="$(cd "$SCRIPT_DIR/.." && pwd)"
REPORTS_DIR="$PROJECT_ROOT/partiql-conformance-tests/reports"

usage() {
    echo "Usage: $0 <command>"
    echo ""
    echo "Commands:"
    echo "  vm       Generate/update VM evaluator conformance report"
    echo "  diff     Show tests that pass in legacy but fail in VM (the work list)"
    echo "  check    Verify committed vm.json matches actual test results"
    echo ""
    echo "The legacy.json baseline is generated from the main branch and should"
    echo "only be updated when main's legacy evaluator changes. To regenerate:"
    echo "  git checkout main && $0 vm  (but with 'conformance_test, experimental' features)"
    echo ""
    echo "Requirements: cargo +nightly, jq"
    exit 1
}

generate_report() {
    local features="$1"
    local output="$2"
    local commit_hash
    commit_hash=$(git -C "$PROJECT_ROOT" rev-parse --short HEAD)

    echo "Running conformance tests (this may take a minute)..."
    local json_output
    json_output=$(cargo +nightly test --package partiql-conformance-tests \
        --features "$features" --release \
        -- -Z unstable-options --format json 2>/dev/null || true)

    echo "Processing results..."
    echo "$json_output" | jq -s --arg hash "$commit_hash" '
        [ .[] | select(.type == "test" and (.event == "ok" or .event == "failed" or .event == "ignored")) ]
        | group_by(.event)
        | reduce .[] as $group (
            {"commit_hash": $hash, "passing": [], "failing": [], "ignored": []};
            if ($group[0].event == "ok") then .passing = [$group[].name] | .passing |= sort
            elif ($group[0].event == "failed") then .failing = [$group[].name] | .failing |= sort
            elif ($group[0].event == "ignored") then .ignored = [$group[].name] | .ignored |= sort
            else .
            end
        )
    ' > "$output"

    local passing failing total
    passing=$(jq '.passing | length' "$output")
    failing=$(jq '.failing | length' "$output")
    total=$((passing + failing))
    echo "Report written to $output"
    echo "  $passing passing / $failing failing / $total total ($(echo "scale=1; $passing * 100 / $total" | bc)%)"
}

cmd_vm() {
    generate_report "conformance_test, eval_vm, experimental" "$REPORTS_DIR/vm.json"
}

cmd_diff() {
    if [[ ! -f "$REPORTS_DIR/legacy.json" || ! -f "$REPORTS_DIR/vm.json" ]]; then
        echo "Error: both legacy.json and vm.json must exist in $REPORTS_DIR"
        exit 1
    fi

    local legacy_only
    legacy_only=$(jq -n --slurpfile legacy "$REPORTS_DIR/legacy.json" --slurpfile vm "$REPORTS_DIR/vm.json" '
        ($legacy[0].passing | sort) as $lp |
        ($vm[0].failing | sort) as $vf |
        [$lp[] | select(. as $t | $vf | bsearch($t) >= 0)]
    ')

    local count
    count=$(echo "$legacy_only" | jq 'length')
    echo "Tests passing in legacy but failing in VM: $count"
    echo ""

    if [[ "$count" -gt 0 ]]; then
        echo "Grouped by category:"
        echo "$legacy_only" | jq -r '.[]' | \
            sed 's/::permissive_.*//; s/::strict_.*//' | \
            sort | uniq -c | sort -rn | head -30
    fi
}

cmd_check() {
    if [[ ! -f "$REPORTS_DIR/vm.json" ]]; then
        echo "Error: vm.json not found. Run '$0 vm' to generate it."
        exit 1
    fi

    echo "Running VM conformance tests..."
    local json_output
    json_output=$(cargo +nightly test --package partiql-conformance-tests \
        --features "conformance_test, eval_vm, experimental" --release \
        -- -Z unstable-options --format json 2>/dev/null || true)

    local actual_passing actual_failing
    actual_passing=$(echo "$json_output" | jq -r 'select(.type == "test" and .event == "ok") | .name' | sort)
    actual_failing=$(echo "$json_output" | jq -r 'select(.type == "test" and .event == "failed") | .name' | sort)

    local committed_passing committed_failing
    committed_passing=$(jq -r '.passing[]' "$REPORTS_DIR/vm.json" | sort)
    committed_failing=$(jq -r '.failing[]' "$REPORTS_DIR/vm.json" | sort)

    local pass_diff fail_diff
    pass_diff=$(diff <(echo "$committed_passing") <(echo "$actual_passing") || true)
    fail_diff=$(diff <(echo "$committed_failing") <(echo "$actual_failing") || true)

    if [[ -z "$pass_diff" && -z "$fail_diff" ]]; then
        echo "OK: Committed vm.json matches actual test results."
        exit 0
    fi

    echo "MISMATCH: Committed vm.json does not match actual test results."
    echo ""

    if [[ -n "$pass_diff" ]]; then
        local new_passes lost_passes
        new_passes=$(echo "$pass_diff" | grep '^>' | sed 's/^> //' || true)
        lost_passes=$(echo "$pass_diff" | grep '^<' | sed 's/^< //' || true)

        if [[ -n "$new_passes" ]]; then
            echo "NEW PASSES (tests that now pass but aren't in vm.json):"
            echo "$new_passes" | sed 's/^/  + /'
            echo ""
        fi
        if [[ -n "$lost_passes" ]]; then
            echo "REGRESSIONS (tests in vm.json as passing but now fail):"
            echo "$lost_passes" | sed 's/^/  - /'
            echo ""
        fi
    fi

    echo "Run '$0 vm' to update the committed report."
    exit 1
}

if [[ $# -lt 1 ]]; then
    usage
fi

case "$1" in
    vm) cmd_vm ;;
    diff) cmd_diff ;;
    check) cmd_check ;;
    *) usage ;;
esac
