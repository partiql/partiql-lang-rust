# Why `SELECT e.a FROM example e` broke in the VM, and what fixes it

Branch `research/vm-aliased-refs` (off `origin/main` @ `de62020`). Everything below was
reproduced with `pqlite` against a stored (LMDB) table:

```
CREATE TABLE example
INSERT INTO example SELECT * FROM << {'a':1,'b':4,'c':'x','d':1},
                                     {'a':3,'b':6,'c':'x','d':1},
                                     {'a':5,'b':9,'c':'y','d':2} >>
```

## TL;DR

There were two independent bugs. Both had to be fixed before any of the example queries
gave the right answer.

1. **Lowering (the main bug).** In `VarRefResolution::Static`, the lowering step
   (`partiql-logical-planner/src/lower.rs`, `search_locals`) resolved the source expression
   of a FROM item with that item's *own alias* already in scope. So in
   `FROM example e`, the name `example` was taken to mean "attribute `example` of `e`", and
   the scan was lowered as `e.example`. The VM cannot find a table named `e.example`, so it
   evaluates the expression instead, gets MISSING, and scans a one-row bag that holds
   `MISSING`. This is why every aliased query returned `<< { 'a': MISSING } >>` or no rows.
   It is also the cause of the earlier report "`SELECT p.id FROM <db table> AS p` is MISSING
   while `SELECT *` works", and of the "nested path `t.a.b` fails" report.
2. **VM compilation.** `PlanCompiler::inline_program` (`partiql-vm/src/compiler.rs`) merges
   the separately compiled WHERE, projection and aggregate sub-programs. It remapped their
   key indices by adding a fixed offset, but `ProgramBuilder` *interns* keys, so a key that
   was already in the pool kept its old index. Any query that names the same field in
   WHERE and SELECT read past the end of the key pool (`invalid key index`). In aggregates
   it read the wrong field: `SUM(e.a), SUM(b)` computed `SUM(b)` twice. This bug has
   nothing to do with aliases, but every interesting aliased query also runs into it.

Both are fixed on the branch, in two separate commits. You asked for lateral joins and
ambiguity to be handled, so the lowering fix grew from "hide the item's own alias" into a
small, explicit set of scoping rules (below).

## How a query flows: AST → logical plan → VM program

**Parse and name resolution.** `partiql-parser` produces the AST. Then
`partiql-ast-passes::NameResolver` walks it once and builds a `KeyRegistry`:

- `schema[node]` holds what each FROM item, GROUP BY key or query *produces* (its binding
  names) and *consumes*.
- `in_scope[node]` lists the producer nodes visible at that node.

The important detail is how `in_scope` is built. A FROM item registers itself on *every
AST ancestor* up to the statement root. GROUP BY keys do the same. A FROM item's own
`in_scope` entry lists only the FROM items before it (the lateral ones). So `in_scope` is
a deliberate over-approximation. The resolver even says so:
`// TODO: delete in_scope for FROM sources in subsequent clauses`.

**Lowering.** `AstToLogical` (`lower.rs`) visits the AST and builds a `LogicalPlan` DAG
(`Scan → Filter → GroupBy → Project → Sink`). It resolves each `VarRef` in one of two ways:

- `Dynamic` (the legacy evaluator) emits a `DynamicLookup` over all candidates and decides
  at runtime. It looks at the `in_scope` of the *nearest* node that has a schema, which for
  a FROM source is the FROM item itself. That is why the tree-walker never had this bug.
- `Static` (the VM) must choose exactly one `VarRef`, `Path` or `DBRef` at plan time.
  `resolve_varref_static` tries globals first (`catalog.resolve_type`), then
  `search_locals`. `search_locals` walks up the `id_stack`, gathers the produced names
  from each ancestor's `in_scope`, and returns `VarRef(name)` on an exact match. If there
  is no exact match and the *first* scope produces a single name `e`, it returns `e.name`.
  That second step is your "single binding with an open type" rule.

**Compilation.** `PlanCompiler` (`partiql-vm/src/compiler.rs`) compiles each `Scan`:

- A literal becomes an inline scan.
- A `Call` becomes a table function.
- A `DBRef`, or a bare `VarRef`, is looked up by name in the `CompilationCatalog` and
  becomes a catalog scan with projection pushdown. Requested fields go to slots; otherwise
  the whole value goes to slot 0.
- Anything else becomes an *expression scan*: the expression is evaluated at runtime and
  then `MaterializeCursor` iterates over it.

A `PipelineSlotResolver` then maps the scan alias (or the table name) to the base-row slot
and pushed-down fields to their slots. Expressions such as `e.a` compile to
`GetField { base: <slot of e>, key_idx }` against a key pool.

pqlite's *planning* catalog knows only the table-function stubs. It does not know the
stored tables, so `search_globals` never matches a stored table name. Lowering always
falls through to `search_locals`, which is where things went wrong.

## What the plans looked like (main vs fixed)

Plans were dumped with `pqlite --debug plan,program exec --db … "<sql>"` and trimmed.

**No alias. This worked, but only by accident.**
```
SELECT a FROM example
main : Scan { expr: VarRef("example", Local), as_key: "example" }
       Project a := Path(VarRef("example", Local), [Key("a")])
```
The inferred alias of `FROM example` is `example`. So `example` *exactly matched its own
binding* and became `VarRef(example, Local)`. The compiler treats any bare `VarRef` scan
expression as a table name, so it worked. With the fix the scan becomes
`VarRef("example", Global)`, which compiles the same way.

**With an alias. This is the bug.**
```
SELECT e.a FROM example e
main : Scan { expr: Path(VarRef("e", Local), [Key("example")]), as_key: "e" }
       Project a := Path(VarRef("e", Local), [Key("a")])
program (main):
   0: GetField { dst: 2, base: 0, key_idx: 0 }   // slot0 (e, never loaded).example -> MISSING
   1: StoreSlot { slot: 0, src: 2 }
   2: MaterializeCursor { src: 0, cursor_id: 0 } // iterate over MISSING: one row
   ...
result: << { 'a': MISSING } >>
fixed: Scan { expr: VarRef("example", Global), as_key: "e" }    -> 3 correct rows
```
`example` doesn't match `e`, and `e` is the single name in the first scope, so the
implicit attribute rule fired *inside the FROM clause*. The same thing happens for
`SELECT a FROM example e`, `SELECT * …`, `SELECT VALUE e …`, `e.a.b`, and `LIMIT`.

**Alias plus GROUP BY. This showed that hiding only the item's own alias is not enough.**
```
SELECT c, COUNT(*) AS n FROM example GROUP BY c
after a naive fix: Scan { expr: Path(VarRef("c", Local), [Key("example")]) }
```
The GROUP BY key `c` is registered on every ancestor, including the statement root. So
once `example` was hidden, `c` became the "single binding", and the source turned into
`c.example`. On main this case worked by the same accidental exact match as above.

**Two bindings.**
```
SELECT x FROM table1 t1, t1.nodes n
main : Scan t1 := Path(VarRef("t1"), [Key("table1")])     <- same alias bug
       Scan n  := Path(VarRef("t1"), [Key("nodes")])
       Project x := Path(VarRef("t1"), [Key("x")])        <- silently picks t1
fixed: Lower error: AmbiguousReference { name: "x", candidates: ["t1", "n"] }

SELECT n.x FROM table1 t1, t1.nodes n
fixed: Scan t1 := VarRef("table1", Global); Scan n := Path(VarRef("t1"), [Key("nodes")])
```
On main the implicit attribute rule looked only at `scope_ids.first()`, so with two FROM
items it always picked the first one. Its own comment says "if multiple FROM sources,
ambiguous" but the code didn't do that. On main, `FROM t1, t2` also lowered the second
source as `t1.t2`, and in a subquery `FROM t1.nodes m` became `m.t1.nodes`.

## Results: actual vs expected

| Query | main | fixed |
|---|---|---|
| `SELECT e.a FROM example e` (also with `AS`) | `{a: MISSING}` ×1 | 1, 3, 5 ✓ |
| `SELECT a FROM example e` | `{a: MISSING}` ×1 | ✓ |
| `SELECT * FROM example e` | `{_1: MISSING}` | ✓ |
| `SELECT VALUE e / e.a / a FROM example e` | `MISSING` | ✓ |
| `SELECT a FROM example WHERE a > 2` (no alias) | `invalid key index` | 3, 5 ✓ |
| `SELECT e.a FROM example e WHERE e.a > 2` | 0 rows | ✓ |
| `SELECT a, e.b FROM example e WHERE e.a > 2 AND b > 5` | `TypeError("expected bool")` | `{3,6},{5,9}` ✓ |
| `SELECT SUM(e.a), SUM(b) FROM example e WHERE e.a > 0 GROUP BY e.c, d` | 0 rows; without the alias (`… FROM example WHERE a > 0 GROUP BY c`) `{10,10},{9,9}`, i.e. `SUM(b)` twice | `{4,10},{5,9}` ✓ |
| `SELECT e.c, COUNT(*) … GROUP BY e.c` | `{c: MISSING, n: 1}` | ✓ |
| `SELECT n.a.b, n.id FROM nested n` / `… WHERE n.a.b > 15` | MISSING / 0 rows | ✓ |
| `SELECT COUNT(*) FROM example e` | 1 | 3 ✓ |
| `SELECT x.a, b FROM <<…>> x WHERE x.a > 1 AND b > 3` (inline) | `invalid key index` | ✓ |
| `SELECT x FROM table1 t1, t1.nodes n` | silently `t1.x` | `AmbiguousReference` ✓ |
| `SELECT n.x FROM table1 t1, t1.nodes n` | plan wrong (`t1.table1`) | plan correct; **VM: Join not compiled** |
| `SELECT example.a FROM example e` | `invalid key index` | MISSING ×3 (see note) |
| `ORDER BY`, FROM/WHERE subqueries | unsupported operator | plan correct; **VM unsupported** |
| `SELECT e.c AS k, SUM(e.a) … WHERE … GROUP BY e.c` | `unresolved var k` | same (separate VM bug) |
| `COUNT(*) … WHERE e.a > 100` | no row | no row (separate VM bug) |
| `SELECT e.* FROM example e` | lowering NYI (PathUnpivot) | same |

Note on `example.a FROM example e`: in pqlite this lowers to `e.example.a`, because the
planning catalog doesn't know about stored tables. Kotlin (and Rust, once tables are
registered in the planning catalog) would resolve `example` to the global table, a bag, so
`.a` on it is MISSING in permissive mode. The result is the same either way.

## The scoping rules the fix implements

All the changes are in `search_locals` (Static mode only; Dynamic is untouched):

1. **A node never sees its own bindings.** Scopes that are on the current `id_stack`
   (ancestor-or-self) are hidden. This is what fixes `FROM example e`.
2. **A FROM source sees only the FROM items before it (lateral), plus enclosing
   queries.** When the reference is inside FROM item *F* of query *Q*, it searches
   `in_scope[F]` first, then skips the rest of *Q*. Anything *Q* registered on the
   ancestors above it is hidden: its own FROM items, its GROUP BY keys and group-as
   variables. So `FROM t1, t1.nodes n` resolves `t1`, while GROUP BY keys and later items
   are invisible.
3. **No implicit attribute inside FROM.** An unmatched name in a FROM source is a global
   (`VarRef(..., Global)`), never `prev_item.name`. `FROM t1, t2` is a join of two tables.
   A lateral reference must be written out (`t1.nodes`). This also matches SQL, and it
   matters because pqlite's planning catalog cannot yet confirm that `t2` is a table.
4. **FROM items of nested subqueries are hidden from the enclosing query.** The resolver
   now records `KeyRegistry::from_lets: FROM item → owning query`. A FROM item is visible
   only if its owner is the current query or one that encloses it. Without this,
   `SELECT a FROM e … WHERE EXISTS (SELECT 1 FROM t)` would make `a` ambiguous.
5. **Exact match first, then the implicit attribute rule.** Pass 1 looks for an exact
   binding match at every level, innermost first, so a correlated outer variable wins over
   an inner implicit-attribute guess. Pass 2 applies your rule: at the innermost level
   that has FROM bindings, if there is **exactly one**, `a` becomes `binding.a` (using its
   `AS` name; `AT` names are not candidates). If there are several, lowering fails with the
   new `AstTransformError::AmbiguousReference { name, candidates }`. GROUP BY keys are
   never candidates.

## What Kotlin does (partiql-lang-kotlin, `partiql-planner/.../internal`)

- `RexConverter.visitExprVarRef` emits `Rex.Op.Var.Unresolved(id, scope)`. A path made of
  plain field steps, like `a.b.c`, is folded into one multi-part identifier so that
  resolution can do longest-match. `@a` sets `Scope.LOCAL`.
- `PlanTyper.visitRexOpVarUnresolved` calls `TypeEnv.resolve(id, strategy)`:
  - `LOCAL`: `locals.resolveName ?: globals.resolveTable ?: locals.resolveField`
  - `GLOBAL`: `globals.resolveTable ?: locals.resolveName ?: locals.resolveField`

  SELECT and WHERE use LOCAL.
- **FROM sources are typed with `Strategy.GLOBAL` and an empty current scope**
  (`PlanTyper.kt` ~205/219/267: `node.rex.type(emptyList(), outer, GLOBAL)`). The item's
  own alias exists only in the rel's output schema, so it can't be seen inside its own
  source. That is exactly rule 1/2 above.
- **Lateral.** A comma-FROM becomes left-folded `Join(INNER, lateral)`. `visitRelOpJoin`
  types the right-hand side with `outer + Scope(lhs.schema)`, so `t1` is visible to
  `t1.nodes` as an *outer* scope. It is not visible for RIGHT/FULL joins.
- **Implicit attribute** (`Scope.resolveField` / `matchStruct`). This uses *types*: a
  closed ROW answers "has field `a`" with yes or no; STRUCT, DYNAMIC and ANY answer
  "unknown". Exactly one candidate gives `binding.a`. One known candidate beats any number
  of unknowns. Two known, or several unknown, return null, which becomes a
  `varRefNotFound` planning error (with a `TODO emit ambiguous error`). Kotlin never
  emits a runtime multi-binding lookup.
- **Globals** use longest-match through the connector (`Env.resolveTable`); leftover parts
  become path steps.

**How Rust differs:**

- Rust has no types at lowering time, so every binding counts as "unknown". The rules
  agree with Kotlin's open-type behaviour, except that Rust reports a dedicated
  `AmbiguousReference` error where Kotlin reports "not found".
- Rust tries globals first for unqualified names everywhere. Kotlin does that only in
  FROM, and tries locals first in SELECT and WHERE. This is harmless in pqlite, where
  tables are not in the planning catalog. Once they are, though,
  `SELECT example.a FROM example` would bind `example` to the global table rather than
  the local binding (see question 2).
- Rust has no closed-schema disambiguation. That is what would let `SELECT x FROM t1,
  t1.nodes n` work once `t1`'s schema is known.

## Why the fix lives in lowering (and the VM fix in the compiler)

The alias bug is a name-resolution bug. The plan itself was wrong (`Scan(e.example)`),
and the VM compiled it faithfully. Patching the compiler, for example by rewriting
`Path(alias, [Key(t)])` to a table scan when `alias` is the scan's own `as_key`, would
only treat the symptom. It would also leave every other consumer of Static plans broken,
and it couldn't fix the GROUP BY, ambiguity and subquery cases. The fix belongs in lowering
for two reasons. First, lowering is the only stage that has scope structure. Second,
Kotlin puts the same rule in the same place: its planner types FROM sources with no
current scope.

The key-remap bug, on the other hand, is purely a compiler bookkeeping bug, and it is a
one-line idea: map each sub-program key through `push_key_pub`. The change is small and
clearly correct, so it is implemented in full.

### Sketch of the changes (as committed)

```rust
// partiql-vm/src/compiler.rs — inline_program
let key_map: Vec<u16> = sub_program.keys.iter()
    .map(|k| builder.push_key_pub(k.clone()))     // interned index, not offset
    .collect();
remap_inst(inst, const_offset, &key_map)          // GetField/CallUdf use key_map[idx]

// partiql-ast-passes/src/name_resolver.rs — record ownership
pub from_lets: FnvIndexMap<NodeId /*FromLet*/, NodeId /*owning Query*/>,

// partiql-logical-planner/src/lower.rs — search_locals
hidden = id_stack ∪ { f | from_lets[f] ∉ query_id_stack }
if in FROM item F of query Q:
    levels = [in_scope[F]];  hidden ∪= in_scope[Q] \ in_scope[F];  skip id_stack up to Q
levels += in_scope[ancestor] for remaining ancestors
pass 1: exact match on produced names in visible scopes  -> VarRef(name, Local)
if in FROM: return None                                    -> Global
pass 2: innermost level with FROM bindings: 1 -> Path(b, [name]); >1 -> AmbiguousReference
```

The lowering diff is about 100 lines plus 7 unit tests in `lower.rs`. It is bigger than
"tiny", but most of it is the restructured `search_locals`. If you would prefer the
minimal version, rule 1 alone is about 8 lines and fixes the headline bug. It *regresses*
`SELECT c … FROM example GROUP BY c` (the source turns into `c.example`), though, so it
isn't safe on its own. Rule 2 is the smallest addition that makes it safe.

### Risks

- **Conformance under `eval_vm`** improves from 4696 to 4706 passing: 18 tests newly pass
  and 8 newly fail. The 8 are all of the form "GROUP BY / GROUP AS binding referenced in
  FROM / WHERE", e.g.
  `SELECT gb_binding FROM sales_report, gb_binding WHERE … GROUP BY rep AS gb_binding`.
  These are negative tests that expect `EvaluationFail`. They are still rejected, but now
  at lowering (`AmbiguousReference` for `rep`, since there are two FROM bindings), and the
  harness counts an unexpected *lowering* error as a test failure. Before, they "passed"
  by failing at runtime for the wrong reason. Kotlin would also reject them at planning
  time. Question 1 asks how you want to handle this.
- The default (non-VM) conformance run is unchanged: 5668/787, with an identical failure
  set. Dynamic mode doesn't use `search_locals`.
- `KeyRegistry` gains a public field. It isn't `#[non_exhaustive]`, so code outside the
  crate that builds it with a struct literal would break. Within the workspace it is only
  built by the resolver.
- The new ambiguity error is a behaviour change. Queries that used to silently pick the
  first FROM binding now fail at lowering. That is intended, but a user could notice it.
- `@name` (qualified) still runs locals → globals through the same code. FROM items
  inside GROUP BY or HAVING expressions, and `WITH`, were not exercised.

## Remaining gaps (not name resolution; kept as `skip::` fixtures)

All of these now produce **correct logical plans**. They fail later, in the VM:

- `Join`: comma/CROSS/lateral joins (`unsupported operator: Discriminant(6)`)
- `OrderBy` (`Discriminant(4)`)
- `SubQueryExpr` in FROM or WHERE (`IN (SELECT …)`, `EXISTS`)
- GROUP BY + WHERE + a renamed key: `SELECT e.c AS k … WHERE … GROUP BY e.c` gives
  `unresolved var k`. This happens on main too, with or without an alias.
- `COUNT(*)` over empty input returns no row instead of `{c: 0}`
- `SELECT e.*` (lowering: `PathUnpivot` not implemented)
- pqlite's planning catalog doesn't register stored tables, so FROM names resolve to a
  `Global` `VarRef` instead of a `DBRef`. It works, but no table-existence or schema
  information reaches lowering. That information is what would allow Kotlin-style
  type-based disambiguation.

## Questions for you

1. The 8 `simple_group_by_fail` conformance tests under `eval_vm` now fail at lowering
   (`AmbiguousReference`) rather than at evaluation. You can (a) accept this, (b) make the
   VM conformance harness accept a lowering error where the test expects `EvaluationFail`,
   which is arguably correct for a static planner, or (c) make ambiguity a deferred
   runtime error. Which would you prefer?
2. Should Static resolution switch to Kotlin's order for unqualified names outside FROM,
   i.e. locals before globals? It makes no difference in pqlite today, but it will once
   stored tables are registered in the planning catalog.
3. Do you want the full lowering fix kept as is, or split further (for example, land
   rules 1–2 first and rules 4–5, the ambiguity error, separately)?

## Tests added

- `partiql-tools/tests/pqlite/cases/query/aliased_refs.test.ion`: alias / no alias / AS,
  qualified and unqualified, SELECT VALUE, WHERE, GROUP BY, LIMIT
- `…/query/aliased_nested_paths.test.ion`: `n.a.b` alone, combined with another field,
  in WHERE, in SELECT VALUE
- `…/query/aliased_inline_sources.test.ion`: inline bags and `mem()` with aliases;
  WHERE+SELECT sharing a field
- `…/errors/ambiguous_reference.test.ion`: `SELECT x FROM t1, t1.nodes n` and
  `t1, t2` both give `Ambiguous`
- `…/query/aliased_known_gaps.test.ion`: `skip::` steps for the gaps above
- `lower.rs` unit tests `test_static_*` (7): plan-level checks for alias, implicit
  attribute, GROUP BY, lateral, non-lateral global, ambiguity, subquery isolation. These
  cover joins even though the VM can't run them yet.

On `main`, `aliased_refs`, `aliased_nested_paths`, `aliased_inline_sources` and
`ambiguous_reference` all fail. On the branch, all fixtures pass and `make ci-check` passes.

Run them with:

```
cargo test -p partiql-tools --test pqlite_e2e aliased
cargo test -p partiql-logical-planner test_static_
```
