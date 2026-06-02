# VM Evaluator Conformance Plan

This document tracks the gaps between the new VM evaluator (`partiql-eval/src/engine/`) and the legacy evaluator, prioritized by impact on the conformance test suite.

## How to Run the Analysis

```bash
# Run VM conformance tests and capture JSON output
cargo +nightly test --package partiql-conformance-tests \
  --features "conformance_test, eval_vm, experimental" --release \
  -- -Z unstable-options --format json > vm_cargo_test_results.json

# The JSON is line-delimited. Each failed test has:
# {"type":"test","event":"failed","name":"...","stdout":"<panic message>"}
```

The `stdout` field of failed tests contains the error classification:
- `unsupported operator: Discriminant(N)` → operator not compiled (see mapping below)
- `UnsupportedExpr("...")` → expression type not compiled
- `UdfNotFound` or `not found` → built-in function not registered
- `unresolved DB object` → global environment binding issue
- `assertion ... failed` / `left == right` → ran but produced wrong result
- `IllegalState` → internal bug

### BindingsOp Discriminant Mapping

| Discriminant | Operator | Status |
|--:|:--|:--|
| 0 | Scan | Supported |
| 1 | Pivot | Not supported |
| 2 | Unpivot | Not supported |
| 3 | Filter | Supported |
| 4 | OrderBy | Not supported |
| 5 | LimitOffset | Supported (LIMIT only, no OFFSET) |
| 6 | Join | Not supported |
| 7 | BagOp (UNION/INTERSECT/EXCEPT) | Not supported |
| 8 | Project | Supported |
| 9 | ProjectAll | Supported |
| 10 | ProjectValue | Supported |
| 11 | ExprQuery | Supported |
| 12 | Distinct | Not supported |
| 13 | GroupBy | Not supported |
| 14 | Having | Not supported |

## Current State (2026-06-02)

- **3,775 passing / 2,896 failing / 6,671 total (56.6% conformant)**
- Previously 3,559 (53.4%) before inline scans + unary negation
- Previously 2,587 (38.8%) before DBRef/global resolution was implemented
- Previously 1,117 (16.7%) before built-in functions were implemented
- Only ~24 tests produce a wrong result — correctness is high when coverage exists
- The problem is feature coverage, not logic bugs

## Priority Ranking

| # | Root Cause | Tests Blocked | Cumulative | Key Files |
|--:|:--|--:|--:|:--|
| 1 | Missing built-in functions | ~338 remaining | — | `engine/builtins.rs` (DONE for top functions) |
| 2 | Unresolved DB objects / globals | ~126 remaining | — | `engine/compiler.rs` (DONE: implicit scans for ExprQuery) |
| 3 | GROUP BY operator | 688 (12.4%) | 64.7% | `engine/compiler.rs` |
| 4 | Scan expression types | ~57 remaining | — | `engine/compiler.rs` (DONE: inline scans for literals) |
| 5 | Unary negation (`-x`) | 0 remaining | — | `engine/expr.rs` (DONE) |
| 6 | Variant/Ion literals | 162 (2.9%) | 75.9% | `engine/expr.rs:2285` |
| 7 | LIKE (PatternMatchExpr) | 100 (1.8%) | 77.7% | `engine/expr.rs:2214` |
| 8 | JOIN operator | 84 (1.5%) | 79.2% | `engine/compiler.rs` |
| 9 | ORDER BY operator | 60 (1.1%) | 80.3% | `engine/compiler.rs` |
| 10 | UNION/INTERSECT/EXCEPT (BagOp) | 59 (1.1%) | 81.4% | `engine/compiler.rs` |
| 11 | Typed literals | 58 (1.0%) | 82.4% | `engine/expr.rs` |
| 12 | Path traversal (PathForEach/Unpivot) | 54+50 (1.9%) | 84.3% | `engine/expr.rs` |
| 13 | IS TYPE expression | 46 (0.8%) | 85.1% | `engine/expr.rs:2226` |
| 14 | IllegalState bugs | 44 (0.8%) | 85.9% | Various |
| 15 | DISTINCT operator | 34 (0.6%) | 86.5% | `engine/compiler.rs` |
| 16 | NULLIF expression | 26 (0.5%) | 87.0% | `engine/expr.rs:2229` |
| 17 | BETWEEN expression | 24 (0.4%) | 87.5% | `engine/expr.rs:2211` |
| 18 | COALESCE expression | 15 (0.3%) | 87.7% | `engine/expr.rs:2232` |
| 19 | Subquery in project | 15 (0.3%) | 88.0% | `engine/compiler.rs` |

---

## Task Breakdown

### Task 1: Built-in Functions (~338 tests remaining)

The VM dispatches function calls via `UdfRegistry` trait in `engine/builtins.rs`. The `BuiltinFunctions` struct implements this and is wired into `PartiQLVM` (passed to `eval_inst` in the dispatch loop at `plan.rs`).

**Completed:**

- [x] `char_length` / `character_length` — 583 tests
- [x] `lower` — 274 tests
- [x] `upper` — 274 tests
- [x] `substring` — 188 tests
- [x] `overlay` — 44 tests
- [x] `cardinality` — 35 tests
- [x] `mod` — 30 tests
- [x] `position` — 28 tests
- [x] `abs` — 11 tests
- [x] `bit_length` — 11 tests
- [x] `octet_length` — 11 tests

**Remaining (not yet implemented in `builtins.rs`):**

- [ ] `trim` (LTrim/BTrim/RTrim) — 120 tests
- [ ] `extract` (ExtractYear/Month/Day/etc.) — 32 tests
- [ ] `exists` — 18 tests

**Aggregate functions (require GROUP BY first):**

- [ ] `coll_any` / `coll_some` — 52 tests
- [ ] `coll_every` — 26 tests
- [ ] `coll_avg` — 16 tests
- [ ] `coll_count` — 15 tests
- [ ] `coll_sum` — 14 tests
- [ ] `coll_max` — 13 tests
- [ ] `coll_min` — 13 tests

**Key files:**
- VM built-in implementations: `partiql-eval/src/engine/builtins.rs`
- Legacy implementations (reference): `partiql-eval/src/eval/expr/strings.rs`, `functions.rs`, `datetime.rs`
- VM UDF dispatch: `partiql-eval/src/engine/expr.rs` (search for `CallUdf`)
- VM wiring: `partiql-eval/src/engine/plan.rs` (builtins field on `PartiQLVM`)

**How to add a new built-in:** Add a match arm in `BuiltinFunctions::call()` mapping the `CallName` debug string (e.g., `"LTrim"`) to a handler function. The function receives `&[ValueRef<'a>]` args and returns `Result<ValueRef<'a>>`.

### Task 2: Unresolved DB Objects / Global Environment (1,098 tests)

Many conformance tests pass an environment like `{i: 1, f: 2.0, s: "hello"}` and then query `SELECT i` or just `i`. The VM fails with "unsupported expression: unresolved DB object default.i".

**Root cause:** The VM's `PlanCompiler` doesn't know how to resolve variables that come from the test environment bindings. The logical planner lowers these to `VarRef` with scope `DbId::default()`, and the VM compiler doesn't handle this DB object type.

**Approach:** Look at how `eval_vm.rs` passes the environment into the VM (via `ExecutionCatalog` / `DataSource`). The issue is that expression-only queries (no FROM clause) still reference these bindings, but the compiler expects them to come from a scan.

**Key files:**
- `partiql-conformance-tests/tests/support/eval_vm.rs` — how environment is provided
- `partiql-eval/src/engine/compiler.rs` — where "unresolved DB object" error occurs
- `partiql-logical-planner/src/lower.rs` — how variable references are lowered

### Task 3: GROUP BY Operator (688 tests)

The `GroupBy` and `Having` BindingsOp variants (discriminants 13, 14) aren't compiled.

**Approach:** Implement a grouping operator in the VM that:
1. Consumes all input rows
2. Groups them by the key expressions
3. Applies aggregate functions to each group
4. Produces one output row per group

**Dependencies:** Aggregate functions (Task 1 sub-tasks) are needed for GROUP BY to be useful.

**Key files:**
- Legacy implementation: `partiql-eval/src/eval/evaluable.rs` (search for `EvalGroupBy`)
- VM compiler: `partiql-eval/src/engine/compiler.rs:667` (the catch-all for unsupported ops)

### Task 4: Scan Expression Types (273 tests)

Certain scan source expressions aren't handled. For example, scanning from a literal collection (`SELECT * FROM [1, 2, 3]`) or other non-variable scan sources.

**Key files:**
- `partiql-eval/src/engine/compiler.rs:813-814` — "unsupported scan expression type"

### Task 5: Unary Negation (191 tests)

The unary `-` operator on expressions isn't compiled. This affects integer/float literals like `-1`, `-3.14` and expressions like `-x`.

**Key files:**
- `partiql-eval/src/engine/expr.rs:2171` — `Err(EngineError::UnsupportedExpr(format!("unary op {op:?}")))`

### Task 6: Variant/Ion Literals (162 tests)

A `todo!()` panic at `engine/expr.rs:2285` for `Lit::Variant(_, _)`.

### Task 7: LIKE / Pattern Match (100 tests)

`PatternMatchExpr` isn't compiled. This covers SQL `LIKE` and `SIMILAR TO` patterns.

**Key files:**
- Legacy implementation: `partiql-eval/src/eval/expr/pattern_match.rs`
- VM: `partiql-eval/src/engine/expr.rs:2214`

### Task 8: JOIN Operator (84 tests)

The `Join` BindingsOp (discriminant 6) isn't compiled. Covers INNER JOIN, LEFT JOIN, CROSS JOIN.

**Key files:**
- Legacy implementation: `partiql-eval/src/eval/evaluable.rs` (search for `EvalJoin`)

### Task 9: ORDER BY Operator (60 tests)

The `OrderBy` BindingsOp (discriminant 4) isn't compiled.

### Task 10: Set Operations (59 tests)

`BagOp` (discriminant 7) covers UNION, INTERSECT, EXCEPT.

---

## Approach for Working on a Task

1. **Find the relevant error** in `engine/compiler.rs` or `engine/expr.rs`
2. **Look at the legacy implementation** for the same feature (paths listed above)
3. **Implement the feature** in the VM, following the existing VM patterns:
   - Operators → compile to bytecode instructions in `compiler.rs`
   - Expressions → add `Inst` variants in `expr.rs` and `Expr` enum cases
4. **Run the conformance tests** to verify:
   ```bash
   cargo +nightly test --package partiql-conformance-tests \
     --features "conformance_test, eval_vm, experimental" --release \
     -- -Z unstable-options --format json > vm_cargo_test_results.json
   ```
5. **Re-run the gap analysis** (python script in this doc's "How to Run" section) to see updated numbers

## Gap Analysis Script

Save as `scripts/vm_gap_analysis.py` or run inline:

```python
import json, sys, re
from collections import Counter

disc_map = {
    '0': 'Scan', '1': 'Pivot', '2': 'Unpivot', '3': 'Filter',
    '4': 'OrderBy', '5': 'LimitOffset', '6': 'Join', '7': 'BagOp',
    '8': 'Project', '9': 'ProjectAll', '10': 'ProjectValue',
    '11': 'ExprQuery', '12': 'Distinct', '13': 'GroupBy', '14': 'Having'
}

def classify(stdout):
    if not stdout:
        return 'Unknown'
    m = re.search(r'unsupported operator.*?Discriminant\((\d+)\)', stdout)
    if m:
        return f'UnsupportedOp::{disc_map.get(m.group(1), "Op#"+m.group(1))}'
    if 'Variant literals' in stdout: return 'Unimpl::VariantLiterals'
    if 'UdfNotFound' in stdout or 'not found' in stdout: return 'MissingBuiltinFn'
    if 'unresolved DB ob' in stdout or 'unresolved var' in stdout: return 'Unimpl::UnresolvedDBObj'
    if 'unsupported scan expression' in stdout: return 'Unimpl::ScanExpr'
    if 'unary op Neg' in stdout: return 'Unimpl::UnaryNeg'
    if 'PatternMatchExpr' in stdout: return 'UnsupportedExpr::LIKE'
    if 'BetweenExpr' in stdout: return 'UnsupportedExpr::BETWEEN'
    if 'SearchedCase' in stdout or 'SimpleCase' in stdout: return 'UnsupportedExpr::CASE'
    if 'IsTypeExpr' in stdout: return 'UnsupportedExpr::IS_TYPE'
    if 'NullIfExpr' in stdout: return 'UnsupportedExpr::NULLIF'
    if 'CoalesceExpr' in stdout: return 'UnsupportedExpr::COALESCE'
    if 'SubQueryExpr' in stdout: return 'UnsupportedExpr::SUBQUERY'
    if 'GraphMatch' in stdout: return 'UnsupportedExpr::GRAPH'
    if 'PathUnpivot' in stdout: return 'Unimpl::PathUnpivot'
    if 'PathForEach' in stdout: return 'Unimpl::PathForEach'
    if 'dynamic lookup' in stdout: return 'Unimpl::DynamicLookup'
    if 'offset not supported' in stdout: return 'UnsupportedOp::Offset'
    if 'Lit::TypedLit' in stdout: return 'Unimpl::TypedLiteral'
    if 'Subquery within project' in stdout: return 'Unimpl::SubqueryInProject'
    if 'IllegalState' in stdout: return 'Bug::IllegalState'
    if 'lowering error' in stdout.lower(): return 'LoweringError'
    if 'assertion' in stdout and 'failed' in stdout: return 'WrongResult'
    if 'left ==' in stdout or ('left:' in stdout and 'right:' in stdout): return 'WrongResult'
    if "expected `Err(_)`" in stdout: return 'ShouldFailButPassed'
    return 'Other'

passing = failing = 0
error_counts = Counter()
for line in sys.stdin:
    try: obj = json.loads(line.strip())
    except: continue
    if obj.get('type') != 'test': continue
    if obj['event'] == 'ok': passing += 1
    elif obj['event'] == 'failed':
        failing += 1
        error_counts[classify(obj.get('stdout', ''))] += 1

total = passing + failing
print(f'{passing} passing / {failing} failing / {total} total ({100*passing/total:.1f}%)')
print()
for err, count in error_counts.most_common(25):
    print(f'  {count:5d}  {err}')
```

Usage: `cat vm_cargo_test_results.json | python3 scripts/vm_gap_analysis.py`
