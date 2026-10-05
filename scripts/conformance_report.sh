#!/usr/bin/env bash
set -euo pipefail

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
PROJECT_ROOT="$(cd "$SCRIPT_DIR/.." && pwd)"

usage() {
    echo "Usage: $0 <command>"
    echo ""
    echo "Commands:"
    echo "  diff     Run legacy and VM conformance tests and show tests that pass in"
    echo "           legacy but fail in VM (the work list)"
    echo ""
    echo "Requirements: cargo +nightly, jq"
    exit 1
}

generate_report() {
    local features="$1"
    local output="$2"
    local commit_hash
    commit_hash=$(git -C "$PROJECT_ROOT" rev-parse --short HEAD)

    echo "Running conformance tests with features: $features (this may take a minute)..."
    local json_output
    json_output=$(cargo +nightly test --package partiql-conformance-tests \
        --features "$features" --release \
        -- -Z unstable-options --format json 2>/dev/null || true)

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
    echo "  $passing passing / $failing failing / $total total ($(echo "scale=1; $passing * 100 / $total" | bc)%)"
}

cmd_diff() {
    reports_dir=$(mktemp -d)
    trap 'rm -rf "$reports_dir"' EXIT

    echo "Legacy evaluator:"
    generate_report "conformance_test, experimental" "$reports_dir/legacy.json"
    echo "VM evaluator:"
    generate_report "conformance_test, eval_vm, experimental" "$reports_dir/vm.json"
    echo ""

    local legacy_only
    legacy_only=$(jq -n --slurpfile legacy "$reports_dir/legacy.json" --slurpfile vm "$reports_dir/vm.json" '
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

if [[ $# -lt 1 ]]; then
    usage
fi

case "$1" in
    diff) cmd_diff ;;
    *) usage ;;
esac
