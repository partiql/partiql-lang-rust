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

## Migration

0. Land #689's VM fixes on their own (key remap, pushdown, GROUP BY registers). They
   don't depend on lowering.
1. Add a new `Lowerer` for `VarRefResolution::Static` only, the VM path. The legacy
   `Dynamic` path stays on the old visitor until the new one is ready.
2. Port in this order: SELECT/FROM/WHERE/projection and expressions; then GROUP BY,
   HAVING and aggregates; ORDER BY and LIMIT; joins and subqueries; set ops, VALUES and
   graph MATCH; DML.
   Gate each step: the VM conformance count must be at least main's (4696), with no new
   failures, and the pqlite e2e fixtures from #689 must pass.
3. Move `Dynamic` over: `Scope` lists the candidates for `DynamicLookup`. Gate: the
   legacy conformance failure set is unchanged.
4. Delete the visitor-based `AstToLogical` and `NameResolver`'s `in_scope`/`KeyRegistry`.

## Open questions

- Reuse helpers that aren't tied to the visitor (function-registry lookup, literal and
  coercion helpers) as they are, or move them into a `lower/expr.rs` module?
- Should `Scope` carry types later, so closed schemas can disambiguate the way Kotlin's
  `matchStruct` does? The design allows it, but it isn't part of this work.
