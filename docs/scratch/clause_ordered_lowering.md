# Clause-ordered lowering (AST → logical plan)

## Goal

Lower a query in **one walk, clause by clause, in scoping order** (FROM → LET → WHERE →
GROUP BY → HAVING → SELECT → ORDER BY → LIMIT). Each clause resolves its names against
the scope built by the clauses before it, then adds its own bindings. This is the model
partiql-lang-kotlin uses on its unresolved plan.

## Today (`main`)

- **Two passes.** `NameResolver` precomputes `in_scope`. That map registers every binding
  on *every* AST ancestor, so it over-approximates what is visible. `AstToLogical` (a
  `Visitor`) then lowers the query and filters that map on each lookup.
- **Visitor order is field order.** `Select` declares `project` first, so SELECT is
  visited before FROM. That is why the pre-pass exists.
- **Callback style.** There are 104 `enter_*`/`exit_*` methods. Values can't be
  returned from callbacks, so state lives in 14 parallel stacks (`id_stack`, `q_stack`,
  `ctx_stack`, `vexpr_stack`, `bexpr_stack`, `arg_stack`, `path_stack`, `plan_stack`,
  `aggregate_exprs`, ...) plus `KeyRegistry`.

## Visitor: don't use it for lowering

- Lowering becomes plain recursive functions that `match` on AST nodes. Each one calls
  its children in whatever order it needs and returns its result.
- The `Visit` derive and its field order stay as they are. `partiql-ast` is published,
  and `NameResolver` and outside users rely on the current order. Changing the order
  globally would be a breaking change with no benefit.
- `Visitor` remains for passes that really are order-free walks.

## State and scoping

```rust
struct Lowerer<'a> {
    catalog: &'a dyn SharedCatalog,
    resolution: VarRefResolution,
    plan: LogicalPlan<BindingsOp>,   // swapped out while lowering a subquery
    ids: IdGenerator,
}

/// Names visible at one query level; `parent` is the enclosing query (correlation).
/// Borrowed, so a scope lives exactly as long as the Rust call that lowers its query:
/// there is no push/pop to get wrong.
struct Scope<'p> {
    parent: Option<&'p Scope<'p>>,
    bindings: Vec<Binding>,          // declaration order
}
struct Binding { name: SymbolPrimitive, kind: BindingKind }
enum BindingKind { FromAs, FromAt, Let, GroupKey, GroupAs, SelectAlias }

#[derive(Clone, Copy, PartialEq, Eq)]
enum Clause { From, Where, GroupBy, Having, Select, OrderBy, Limit }

/// Everything an expression needs to lower itself.
struct ExprCx<'s> {
    scope: &'s Scope<'s>,
    clause: Clause,
    /// SELECT/HAVING: aggregate calls are collected here; their arguments resolve
    /// against `pre_group` (the FROM scope), everything else against `scope`.
    aggs: Option<&'s mut AggCollector<'s>>,
}
```

All resolution rules live in one function:

```rust
fn resolve(&self, v: &VarRef, cx: &ExprCx) -> Result<ValueExpr> {
    let found = match (v.qualifier, cx.clause) {
        (Unqualified, Clause::From) => self.global(v).or_else(|| cx.scope.local(v)),
        _                           => cx.scope.local(v).or_else(|| self.global(v)),
    };
    match found {
        Some(e) => Ok(e),
        None if cx.clause == Clause::From => Ok(global_varref(v)),
        None => cx.scope.implicit_attr(v),   // 1 FROM binding -> b.v; >1 -> AmbiguousReference
    }
}
```

## Function signatures

```rust
fn lower_statement(&mut self, s: &Statement) -> Result<LogicalStatement>;
fn lower_query(&mut self, q: &Query, outer: Option<&Scope>) -> Result<OpId>;
fn lower_select(&mut self, s: &Select, outer: Option<&Scope>) -> Result<(OpId, Scope)>;

// FROM: lower each item's source with the scope *so far*, then bind its alias.
// That gives no self-visibility and lateral references with no extra rules.
fn lower_from(&mut self, f: &FromSource, scope: &mut Scope) -> Result<OpId>;
fn lower_from_let(&mut self, f: &FromLet, scope: &mut Scope) -> Result<OpId>;
fn lower_join(&mut self, j: &Join, scope: &mut Scope) -> Result<OpId>;

fn lower_let(&mut self, input: OpId, l: &Let, scope: &mut Scope) -> Result<OpId>;
fn lower_where(&mut self, input: OpId, w: &WhereClause, scope: &Scope) -> Result<OpId>;
fn lower_group_by(&mut self, input: OpId, g: &GroupByExpr, from: &Scope, aggs: Vec<AggregateExpression>)
    -> Result<(OpId, Scope)>;        // output scope: keys + GROUP AS only
fn lower_having(&mut self, input: OpId, h: &HavingClause, scope: &Scope) -> Result<OpId>;
fn lower_projection(&mut self, input: OpId, p: &Projection, scope: &Scope) -> Result<(OpId, Scope)>;
fn lower_order_by(&mut self, input: OpId, o: &OrderByExpr, scope: &Scope) -> Result<OpId>;
fn lower_limit_offset(&mut self, input: OpId, l: &LimitOffsetClause, scope: &Scope) -> Result<OpId>;

fn lower_expr(&mut self, e: &Expr, cx: &mut ExprCx) -> Result<ValueExpr>;
fn lower_subquery(&mut self, q: &Query, cx: &ExprCx) -> Result<ValueExpr>; // outer = cx.scope
```

`lower_select` reads like the semantics:

```rust
let mut scope = Scope::child(outer);
let mut op = self.lower_from(&s.from, &mut scope)?;
op = self.lower_let(op, &s.from_let, &mut scope)?;
op = self.lower_where(op, &s.where_clause, &scope)?;
// SELECT/HAVING are lowered before the GroupBy node is added, so their aggregates are
// known. The plan is a graph, so edges are added afterwards; resolution still follows
// clause scoping (aggregate args see `scope`, the rest sees the group scope).
let (op, scope) = self.lower_group_by(op, &s.group_by, &scope, aggs)?;
...
```

## What gets simpler

| | Today | Proposed |
|---|---|---|
| Passes over the AST | 2 (`NameResolver` + visitor) | 1 |
| Lowering state | 14 stacks + `KeyRegistry` | `plan`, `ids`, borrowed `Scope` |
| Where scoping rules live | `in_scope` building (`name_resolver.rs`) plus filtering in `search_locals` | `Scope::local`, `implicit_attr`, `resolve` (about 60 lines) |
| Following one clause | Find its `enter_`/`exit_` pair, track pushes and pops across callbacks | Read one function top to bottom |
| Adding a clause (e.g. `QUALIFY`) | New callbacks, a `QueryContext` variant, 2-4 stacks, maybe resolver changes | One `lower_qualify(input, q, &scope)` call in `lower_select` |
| Errors | `errors.push` + `Traverse::Stop` + `eq_or_fault!` stack checks | `?` |
| Unit tests | Only through the full `lower()` | Call `resolve`/`lower_expr` with a hand-built `Scope` |
| Debugging | Inspect stacks mid-walk | Ordinary call stack and return values |

## Migration (as done)

The visitor-based `lower.rs` was replaced in one step by `lower/` (`mod.rs` for queries
and clauses, `scope.rs` for scopes and name resolution, `expr.rs` for expressions), and
both resolution modes moved at once. The planner no longer runs `NameResolver`. That
crate is unchanged, since it is published.

- **Dynamic** keeps `main`'s candidate lists for `DynamicLookup`, built from the current
  query level only. The evaluator binds an enclosing query's variables as globals of a
  subquery.
- **Static** implements #689's rules: no self-visibility, lateral FROM, globals first
  only in FROM, the single-binding implicit attribute, and `AmbiguousReference`.
- INNER, LEFT, CROSS and comma joins are lateral; RIGHT and FULL joins are not.
- Behaviour changes, each a fix:
  - A SELECT item equal to a GROUP BY key reads that key's alias. It used to read its
    own alias, which gave `unresolved var k`.
  - Aggregates in HAVING and ORDER BY reach the GROUP BY.
  - Unnamed SELECT items are named by position (`_2`).
  - Set-operation subqueries in scalar position lower.
  - Static ORDER BY sees SELECT aliases.
  - `WITH`, `LET` and `SELECT e.*` report NotYetImplemented instead of silently
    misplanning.
- Conformance against `main`:
  - Legacy: 5668 → 5670, 0 newly failing.
  - VM: 4696 → 4716, with 6 newly failing (`pg_select_01`, `select_where_string_equals_*`).
    Correct resolution of `a.name` exposes the VM pushdown bug that drops the whole row
    for `SELECT *`, which #689's VM fixes address. On `main` these passed only because
    `a.name` wrongly resolved to a global table `a`.
- Still to do: land #689's VM fixes as their own PR (key remap, pushdown, GROUP BY
  registers).

## Open questions

- Should `Scope` carry types later, so closed schemas can disambiguate the way Kotlin's
  `matchStruct` does? The design allows it, but it isn't part of this work.
