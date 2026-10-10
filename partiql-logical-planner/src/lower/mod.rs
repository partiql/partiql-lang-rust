//! Lowering of the AST into a logical plan.
//!
//! Lowering is one recursive walk over the AST. A SELECT query is lowered clause by clause in
//! scoping order (FROM, WHERE, GROUP BY, HAVING, SELECT, ORDER BY, LIMIT), and each clause
//! resolves its names against the [`Scope`] that the clauses before it built. See
//! `docs/scratch/clause_ordered_lowering.md`.

mod expr;
mod scope;
#[cfg(test)]
mod tests;

use partiql_ast::ast::{
    self, BagOperator, CaseSensitivity, Expr, FromLet, FromLetKind, FromSource, GroupByExpr,
    GroupingStrategy, Join, JoinKind, JoinSpec, LimitOffsetClause, NullOrderingSpec, OrderByExpr,
    OrderingSpec, ProjectItem, Projection, ProjectionKind, Query, QuerySet, Select, SetQuantifier,
    SymbolPrimitive,
};
use partiql_ast_passes::error::{AstTransformError, AstTransformationError};
use partiql_catalog::catalog::SharedCatalog;
use partiql_logical as logical;
use partiql_logical::{
    AggregateExpression, BindingsOp, LogicalPlan, OpId, PathComponent, ProjectAllMode,
    SortSpecOrder, ValueExpr, VarRefType,
};
use partiql_value::BindingsName;
use rustc_hash::FxHashMap;
use std::borrow::Cow;

use crate::builtins::{FnSymTab, FN_SYM_TAB};
use scope::{BindingKind, Clause, Cx, Scope, SelectAlias};

pub(crate) type Result<T> = std::result::Result<T, AstTransformError>;

/// How variable references are lowered.
#[derive(Debug, Default, Clone, Copy, PartialEq, Eq)]
#[non_exhaustive]
pub enum VarRefResolution {
    /// Emit a [`ValueExpr::DynamicLookup`] over every candidate binding, resolved at evaluation
    /// time. This is what the `partiql-eval` evaluator expects.
    #[default]
    Dynamic,
    /// Resolve each reference at plan time to a single local `VarRef`/`Path` or a catalog
    /// [`logical::DBRef`]. Used by the `partiql-vm` engine.
    Static,
}

/// Lowers one query into a [`LogicalPlan`].
pub(crate) struct Lowerer<'a> {
    catalog: &'a dyn SharedCatalog,
    fnsym_tab: &'static FnSymTab,
    resolution: VarRefResolution,
    /// The plan being built; a subquery is built in a plan of its own.
    plan: LogicalPlan<BindingsOp>,
    /// Counters for generated binding names (`_1`, ...) and aggregate names (`$__agg_1`, ...).
    fresh_ids: u32,
    agg_ids: u32,
    /// The aggregate calls of the SELECT query being lowered.
    aggs: Vec<AggregateExpression>,
    /// Errors that do not stop lowering, so several can be reported together.
    errors: Vec<AstTransformError>,
}

impl<'a> Lowerer<'a> {
    pub fn new(catalog: &'a dyn SharedCatalog, resolution: VarRefResolution) -> Self {
        Lowerer {
            catalog,
            fnsym_tab: &FN_SYM_TAB,
            resolution,
            plan: LogicalPlan::default(),
            fresh_ids: 0,
            agg_ids: 0,
            aggs: vec![],
            errors: vec![],
        }
    }

    pub fn lower(
        mut self,
        query: &ast::TopLevelQuery,
    ) -> std::result::Result<LogicalPlan<BindingsOp>, AstTransformationError> {
        if let Err(err) = self.lower_top_level_query(query) {
            self.errors.push(err);
        }
        if self.errors.is_empty() {
            Ok(self.plan)
        } else {
            Err(AstTransformationError {
                errors: self.errors,
            })
        }
    }

    fn lower_top_level_query(&mut self, query: &ast::TopLevelQuery) -> Result<()> {
        if query.with.is_some() {
            return Err(nyi("WITH"));
        }
        let out = self.lower_query(&query.query.node, None)?;
        let sink = self.plan.add_operator(BindingsOp::Sink);
        self.plan.add_flow(out, sink);
        Ok(())
    }

    /// Lowers `query` into the current plan; returns the operator producing its result.
    fn lower_query(&mut self, query: &Query, outer: Option<&Scope<'_>>) -> Result<OpId> {
        let order_by = query.order_by.as_deref().map(|o| &o.node);
        let limit_offset = query.limit_offset.as_deref().map(|l| &l.node);
        if let QuerySet::Select(select) = &query.set.node {
            return self.lower_select(&select.node, order_by, limit_offset, outer);
        }

        let mut out = self.lower_query_set(&query.set.node, outer)?;
        let scope = Scope::new(outer);
        if let Some(order_by) = order_by {
            let op = self.lower_order_by(order_by, Cx::new(&scope, Clause::OrderBy))?;
            out = self.chain(out, op);
        }
        if let Some(limit_offset) = limit_offset {
            let op = self.lower_limit_offset(limit_offset, Cx::new(&scope, Clause::Limit))?;
            out = self.chain(out, op);
        }
        Ok(out)
    }

    /// Lowers a set operation or an expression query (anything but a SELECT).
    fn lower_query_set(&mut self, set: &QuerySet, outer: Option<&Scope<'_>>) -> Result<OpId> {
        match set {
            QuerySet::BagOp(bag_op) => {
                let bag_op = &bag_op.node;
                let lhs = self.lower_query(&bag_op.lhs.node, outer)?;
                let rhs = self.lower_query(&bag_op.rhs.node, outer)?;
                let op = match bag_op.bag_op {
                    BagOperator::Union => logical::BagOperator::Union,
                    BagOperator::Except => logical::BagOperator::Except,
                    BagOperator::Intersect => logical::BagOperator::Intersect,
                    BagOperator::OuterUnion => logical::BagOperator::OuterUnion,
                    BagOperator::OuterExcept => logical::BagOperator::OuterExcept,
                    BagOperator::OuterIntersect => logical::BagOperator::OuterIntersect,
                    _ => return Err(nyi("Unsupported bag operator")),
                };
                let setq = match bag_op.setq {
                    Some(SetQuantifier::All) => logical::SetQuantifier::All,
                    Some(SetQuantifier::Distinct) | None => logical::SetQuantifier::Distinct,
                    Some(_) => return Err(nyi("Unsupported set quantifier")),
                };
                let id = self
                    .plan
                    .add_operator(BindingsOp::BagOp(logical::BagOp { bag_op: op, setq }));
                self.plan.add_flow_with_branch_num(lhs, id, 0);
                self.plan.add_flow_with_branch_num(rhs, id, 1);
                Ok(id)
            }
            QuerySet::Expr(expr) => {
                let scope = Scope::new(outer);
                let expr = self.lower_expr(expr, Cx::new(&scope, Clause::Value))?;
                Ok(self
                    .plan
                    .add_operator(BindingsOp::ExprQuery(logical::ExprQuery { expr })))
            }
            QuerySet::Values(_) => Err(nyi("QuerySet::Values")),
            QuerySet::Table(_) => Err(nyi("QuerySet::Table")),
            _ => Err(nyi("Unsupported query set")),
        }
    }

    /// Lowers a SELECT query, clause by clause in scoping order.
    fn lower_select(
        &mut self,
        select: &Select,
        order_by: Option<&OrderByExpr>,
        limit_offset: Option<&LimitOffsetClause>,
        outer: Option<&Scope<'_>>,
    ) -> Result<OpId> {
        if select.having.is_some() && select.group_by.is_none() {
            return Err(AstTransformError::HavingWithoutGroupBy);
        }
        if select.exclude.is_some() {
            return Err(nyi("EXCLUDE"));
        }
        if select.from_let.is_some() {
            return Err(nyi("LET"));
        }

        let outer_aggs = std::mem::take(&mut self.aggs);
        let mut scope = Scope::new(outer);
        // Operators in evaluation order; WHERE comes right after FROM.
        let mut ops = vec![];

        // FROM binds its items' names into `scope`.
        let mut project_all_mode = ProjectAllMode::default();
        if let Some(from) = &select.from {
            let (op, mode) = self.lower_from_source(&from.node.source, &mut scope)?;
            ops.push(op);
            project_all_mode = mode;
        }

        if let Some(where_clause) = &select.where_clause {
            let cx = Cx::new(&scope, Clause::Where).coerce(true);
            let expr = self.lower_expr(&where_clause.node.expr, cx)?;
            ops.push(
                self.plan
                    .add_operator(BindingsOp::Filter(logical::Filter { expr })),
            );
        }

        // GROUP BY binds its keys and `GROUP AS` name into `scope`.
        let group_by = match &select.group_by {
            Some(group_by) => Some(self.lower_group_by(&group_by.node, &mut scope)?),
            None => None,
        };

        let having = match &select.having {
            Some(having) => {
                let cx = Cx::new(&scope, Clause::Having).coerce(true);
                Some(self.lower_expr(&having.node.expr, cx)?)
            }
            None => None,
        };

        let mut projection =
            self.lower_projection(&select.project.node, &scope, project_all_mode)?;
        if let Some(group_by) = &group_by {
            group_by.substitute_keys(&mut projection);
        }

        let select_aliases = select_aliases(&projection);
        let order_by = match order_by {
            Some(order_by) => {
                let cx = Cx {
                    select_aliases: Some(&select_aliases),
                    ..Cx::new(&scope, Clause::OrderBy)
                };
                Some(self.lower_order_by(order_by, cx)?)
            }
            None => None,
        };
        let limit_offset = match limit_offset {
            Some(limit_offset) => {
                let cx = Cx::new(&scope, Clause::Limit);
                Some(self.lower_limit_offset(limit_offset, cx)?)
            }
            None => None,
        };

        // The GROUP BY operator is built last, once every clause has added its aggregates.
        // Aggregates without a GROUP BY group the whole input (`GROUP BY true AS $__gk`).
        let aggregate_exprs = std::mem::replace(&mut self.aggs, outer_aggs);
        let group_by = match group_by {
            Some(group_by) => Some(group_by.into_op(aggregate_exprs)),
            None if !aggregate_exprs.is_empty() => Some(BindingsOp::GroupBy(logical::GroupBy {
                strategy: logical::GroupingStrategy::GroupFull,
                exprs: FxHashMap::from_iter([(
                    "$__gk".to_string(),
                    ValueExpr::Lit(Box::new(logical::Lit::Bool(true))),
                )]),
                aggregate_exprs,
                group_as_alias: None,
            })),
            None => None,
        };

        let distinct = matches!(select.project.node.setq, Some(SetQuantifier::Distinct))
            .then_some(BindingsOp::Distinct);
        let rest = [
            group_by,
            having.map(|expr| BindingsOp::Having(logical::Having { expr })),
            order_by,
            limit_offset,
            Some(projection),
            distinct,
        ];
        for op in rest.into_iter().flatten() {
            ops.push(self.plan.add_operator(op));
        }
        for pair in ops.windows(2) {
            self.plan.add_flow(pair[0], pair[1]);
        }
        Ok(*ops.last().expect("a SELECT has a projection"))
    }

    /// Lowers a FROM source, binding each item's names into `scope` right after lowering its
    /// source expression. Returns the operator and the `SELECT *` mode of its last item.
    fn lower_from_source(
        &mut self,
        source: &FromSource,
        scope: &mut Scope<'_>,
    ) -> Result<(OpId, ProjectAllMode)> {
        match source {
            FromSource::FromLet(from_let) => self.lower_from_let(&from_let.node, scope),
            FromSource::Join(join) => self.lower_join(&join.node, scope),
            _ => Err(nyi("Unsupported FROM source")),
        }
    }

    fn lower_from_let(
        &mut self,
        from_let: &FromLet,
        scope: &mut Scope<'_>,
    ) -> Result<(OpId, ProjectAllMode)> {
        // The source sees the items before this one, not this item's own names.
        let expr = match &*from_let.expr {
            Expr::Query(query) => self.lower_subquery(&query.node, scope)?,
            expr => self.lower_expr(expr, Cx::new(scope, Clause::From))?,
        };

        let (as_key, generated) = match from_let.as_alias.clone().or_else(|| infer_id(&expr)) {
            Some(name) => (name, false),
            None => (self.fresh_name(), true),
        };
        let scan = |expr| logical::Scan {
            expr,
            as_key: as_key.value.clone(),
            at_key: from_let.at_alias.as_ref().map(|at| at.value.clone()),
        };
        let (op, mode) = match from_let.kind {
            FromLetKind::Scan | FromLetKind::GraphTable => {
                (BindingsOp::Scan(scan(expr)), ProjectAllMode::Unwrap)
            }
            FromLetKind::Unpivot => {
                let logical::Scan {
                    expr,
                    as_key,
                    at_key,
                } = scan(expr);
                (
                    BindingsOp::Unpivot(logical::Unpivot {
                        expr,
                        as_key,
                        at_key,
                    }),
                    ProjectAllMode::PassThrough,
                )
            }
            _ => return Err(nyi("Unsupported FROM kind")),
        };
        let id = self.plan.add_operator(op);

        scope.bind(as_key, generated, BindingKind::FromAs);
        if let Some(at) = &from_let.at_alias {
            scope.bind(at.clone(), false, BindingKind::FromAt);
        }
        Ok((id, mode))
    }

    fn lower_join(&mut self, join: &Join, scope: &mut Scope<'_>) -> Result<(OpId, ProjectAllMode)> {
        let kind = match join.kind {
            JoinKind::Inner => logical::JoinKind::Inner,
            JoinKind::Left => logical::JoinKind::Left,
            JoinKind::Right => logical::JoinKind::Right,
            JoinKind::Full => logical::JoinKind::Full,
            JoinKind::Cross => logical::JoinKind::Cross,
            _ => return Err(nyi("Unsupported join kind")),
        };

        let before_left = scope.len();
        let (lid, _) = self.lower_from_source(&join.left, scope)?;
        // INNER, LEFT and CROSS (and comma) joins are lateral: the right side sees the left
        // side's names. RIGHT and FULL joins are not.
        let (rid, mode) = match join.kind {
            JoinKind::Right | JoinKind::Full => {
                let mut right_scope = scope.prefix(before_left);
                let right = self.lower_from_source(&join.right, &mut right_scope)?;
                for b in right_scope.bindings_from(before_left) {
                    scope.bind(b.name.clone(), b.generated, b.kind);
                }
                right
            }
            _ => self.lower_from_source(&join.right, scope)?,
        };

        let on = match join.predicate.as_ref().map(|spec| &spec.node) {
            Some(JoinSpec::On(expr)) => Some(self.lower_expr(expr, Cx::new(scope, Clause::On))?),
            Some(JoinSpec::Using(_)) => return Err(nyi("JoinSpec::Using")),
            Some(JoinSpec::Natural) => return Err(nyi("JoinSpec::Natural")),
            Some(_) => return Err(nyi("Unsupported join specification")),
            None => None,
        };

        let operator = |id| Box::new(self.plan.operator(id).expect("operator").clone());
        let op = BindingsOp::Join(logical::Join {
            kind,
            left: operator(lid),
            right: operator(rid),
            on,
        });
        let id = self.plan.add_operator(op);
        self.plan.add_flow_with_branch_num(lid, id, 0);
        self.plan.add_flow_with_branch_num(rid, id, 1);
        Ok((id, mode))
    }

    /// Lowers the GROUP BY keys and binds them (and `GROUP AS`) into `scope`. The operator is
    /// built by [`GroupKeys::into_op`] once all aggregates are known.
    fn lower_group_by(
        &mut self,
        group_by: &GroupByExpr,
        scope: &mut Scope<'_>,
    ) -> Result<GroupKeys> {
        let strategy = match group_by.strategy {
            None | Some(GroupingStrategy::GroupFull) => logical::GroupingStrategy::GroupFull,
            Some(GroupingStrategy::GroupPartial) => logical::GroupingStrategy::GroupPartial,
            Some(_) => return Err(nyi("Unsupported grouping strategy")),
        };

        // Keys see the FROM names but not each other.
        let mut keys = vec![];
        for key in &group_by.keys {
            let key = &key.node;
            let expr = self.lower_expr(&key.expr, Cx::new(scope, Clause::GroupBy))?;
            let alias = key.as_alias.clone().or_else(|| infer_alias(&key.expr));
            keys.push((alias, expr));
        }

        let mut group_keys = GroupKeys {
            strategy,
            keys: vec![],
            group_as_alias: group_by.group_as_alias.as_ref().map(|a| a.value.clone()),
        };
        for (alias, expr) in keys {
            let (alias, generated) = match alias {
                Some(alias) => (alias, false),
                None => (self.fresh_name(), true),
            };
            group_keys.keys.push((alias.value.clone(), expr));
            scope.bind(alias, generated, BindingKind::GroupKey);
        }
        if let Some(group_as) = &group_by.group_as_alias {
            scope.bind(group_as.clone(), false, BindingKind::GroupAs);
        }
        Ok(group_keys)
    }

    fn lower_projection(
        &mut self,
        projection: &Projection,
        scope: &Scope<'_>,
        project_all_mode: ProjectAllMode,
    ) -> Result<BindingsOp> {
        let cx = Cx::new(scope, Clause::Select);
        let op = match &projection.kind {
            ProjectionKind::ProjectStar => BindingsOp::ProjectAll(project_all_mode),
            ProjectionKind::ProjectList(items) => {
                let mut exprs = Vec::with_capacity(items.len());
                for (i, item) in items.iter().enumerate() {
                    let ProjectItem::ProjectExpr(item) = &item.node else {
                        return Err(nyi("SELECT <expr>.*"));
                    };
                    // A SELECT-list item is a single-value context (spec section 9.1).
                    let expr = self.lower_expr(&item.expr, cx.coerce(true))?;
                    let alias = item
                        .as_alias
                        .clone()
                        .or_else(|| infer_alias(&item.expr))
                        .map_or_else(|| format!("_{}", i + 1), |alias| alias.value);
                    exprs.push((alias, expr));
                }
                BindingsOp::Project(logical::Project { exprs })
            }
            ProjectionKind::ProjectPivot(pivot) => BindingsOp::Pivot(logical::Pivot {
                key: self.lower_expr(&pivot.key, cx)?,
                value: self.lower_expr(&pivot.value, cx)?,
            }),
            ProjectionKind::ProjectValue(expr) => BindingsOp::ProjectValue(logical::ProjectValue {
                expr: self.lower_expr(expr, cx)?,
            }),
            _ => return Err(nyi("Unsupported projection kind")),
        };
        Ok(op)
    }

    fn lower_order_by(&mut self, order_by: &OrderByExpr, cx: Cx<'_>) -> Result<BindingsOp> {
        let mut specs = Vec::with_capacity(order_by.sort_specs.len());
        for spec in &order_by.sort_specs {
            let spec = &spec.node;
            // A sort key is a single-value context (spec section 9.1).
            let expr = self.lower_expr(&spec.expr, cx.coerce(true))?;
            let order = match spec.ordering_spec.as_ref().unwrap_or(&OrderingSpec::Asc) {
                OrderingSpec::Asc => SortSpecOrder::Asc,
                OrderingSpec::Desc => SortSpecOrder::Desc,
                _ => return Err(nyi("Unsupported sort order")),
            };
            let null_order = match (&spec.null_ordering_spec, &order) {
                (Some(NullOrderingSpec::First), _) | (None, SortSpecOrder::Desc) => {
                    logical::SortSpecNullOrder::First
                }
                (Some(NullOrderingSpec::Last), _) | (None, SortSpecOrder::Asc) => {
                    logical::SortSpecNullOrder::Last
                }
                _ => return Err(nyi("Unsupported null ordering")),
            };
            specs.push(logical::SortSpec {
                expr,
                order,
                null_order,
            });
        }
        Ok(BindingsOp::OrderBy(logical::OrderBy { specs }))
    }

    fn lower_limit_offset(
        &mut self,
        limit_offset: &LimitOffsetClause,
        cx: Cx<'_>,
    ) -> Result<BindingsOp> {
        // `LIMIT` and `OFFSET` operands are single-value contexts (spec section 9.1).
        let cx = cx.coerce(true);
        let limit = match &limit_offset.limit {
            Some(limit) => Some(self.lower_expr(limit, cx)?),
            None => None,
        };
        let offset = match &limit_offset.offset {
            Some(offset) => Some(self.lower_expr(offset, cx)?),
            None => None,
        };
        Ok(BindingsOp::LimitOffset(logical::LimitOffset {
            limit,
            offset,
        }))
    }

    /// Lowers `query` into a plan of its own, as a value. `scope` is its enclosing scope.
    fn lower_subquery(&mut self, query: &Query, scope: &Scope<'_>) -> Result<ValueExpr> {
        let outer_plan = std::mem::take(&mut self.plan);
        let result = self.lower_query(query, Some(scope));
        let plan = std::mem::replace(&mut self.plan, outer_plan);
        result?;
        Ok(ValueExpr::SubQueryExpr(logical::SubQueryExpr { plan }))
    }

    fn chain(&mut self, input: OpId, op: BindingsOp) -> OpId {
        let id = self.plan.add_operator(op);
        self.plan.add_flow(input, id);
        id
    }

    /// A fresh name for a binding the query does not name.
    fn fresh_name(&mut self) -> SymbolPrimitive {
        // TODO assure non-collision with provided identifiers, e.g. `AS _1`
        self.fresh_ids += 1;
        SymbolPrimitive {
            value: format!("_{}", self.fresh_ids),
            case: CaseSensitivity::CaseInsensitive,
        }
    }

    fn fresh_agg_name(&mut self) -> String {
        self.agg_ids += 1;
        format!("$__agg_{}", self.agg_ids)
    }
}

/// A lowered GROUP BY, waiting for the query's aggregates.
struct GroupKeys {
    strategy: logical::GroupingStrategy,
    /// `(alias, key expression)`
    keys: Vec<(String, ValueExpr)>,
    group_as_alias: Option<String>,
}

impl GroupKeys {
    /// Replaces each SELECT-list expression that is a GROUP BY key by a reference to that
    /// key (spec section 11.2.1, "Direct Use of Grouping Expressions"): in
    /// `SELECT t.a + 1 AS a FROM t GROUP BY t.a + 1 AS k`, the projection becomes `k AS a`.
    /// SELECT VALUE, HAVING and ORDER BY expressions are not rewritten.
    fn substitute_keys(&self, projection: &mut BindingsOp) {
        let BindingsOp::Project(project) = projection else {
            return;
        };
        for (_, expr) in project.exprs.iter_mut() {
            if let Some((alias, _)) = self.keys.iter().find(|(_, key)| key == expr) {
                *expr = ValueExpr::VarRef(
                    BindingsName::CaseSensitive(Cow::Owned(alias.clone())),
                    VarRefType::Local,
                );
            }
        }
    }

    fn into_op(self, aggregate_exprs: Vec<AggregateExpression>) -> BindingsOp {
        BindingsOp::GroupBy(logical::GroupBy {
            strategy: self.strategy,
            exprs: self.keys.into_iter().collect(),
            aggregate_exprs,
            group_as_alias: self.group_as_alias,
        })
    }
}

/// The aliases of a SELECT list, for ORDER BY.
fn select_aliases(projection: &BindingsOp) -> Vec<SelectAlias> {
    match projection {
        BindingsOp::Project(project) => project
            .exprs
            .iter()
            .map(|(alias, expr)| SelectAlias {
                alias: alias.clone(),
                expr: expr.clone(),
            })
            .collect(),
        _ => vec![],
    }
}

fn nyi(what: &str) -> AstTransformError {
    AstTransformError::NotYetImplemented(what.to_string())
}

/// Convert a `SymbolPrimitive` into a `BindingsName`
pub(crate) fn symprim_to_binding(sym: &SymbolPrimitive) -> Result<BindingsName<'static>> {
    Ok(match sym.case {
        CaseSensitivity::CaseSensitive => {
            BindingsName::CaseSensitive(Cow::Owned(sym.value.clone()))
        }
        CaseSensitivity::CaseInsensitive => {
            BindingsName::CaseInsensitive(Cow::Owned(sym.value.clone()))
        }
        _ => return Err(nyi("case sensitivity")),
    })
}

/// The name a SELECT-list item or GROUP BY key without `AS` gets from its expression:
/// `a` for `a`, `e` for `b.c.d.e`.
fn infer_alias(expr: &Expr) -> Option<SymbolPrimitive> {
    match expr {
        Expr::VarRef(var) => Some(var.node.name.clone()),
        Expr::Path(path) => match path.node.steps.last() {
            Some(ast::PathStep::PathProject(step) | ast::PathStep::PathIndex(step)) => {
                infer_alias(&step.index)
            }
            _ => None,
        },
        _ => None,
    }
}

/// The name a FROM item without `AS` gets from its lowered source: `t` for `FROM t`, `c`
/// for `FROM a.b.c`.
fn infer_id(expr: &ValueExpr) -> Option<SymbolPrimitive> {
    let symbol = |name: &BindingsName<'_>| match name {
        BindingsName::CaseInsensitive(s) => Some(SymbolPrimitive {
            value: s.to_string(),
            case: CaseSensitivity::CaseInsensitive,
        }),
        BindingsName::CaseSensitive(s) => Some(SymbolPrimitive {
            value: s.to_string(),
            case: CaseSensitivity::CaseSensitive,
        }),
        _ => None,
    };
    match expr {
        ValueExpr::VarRef(name, _) => symbol(name),
        ValueExpr::DBRef(db_ref) => db_ref.path.last().and_then(symbol),
        ValueExpr::Path(_root, steps) => match steps.last() {
            Some(PathComponent::Key(name)) => symbol(name),
            Some(PathComponent::KeyExpr(key)) => match &**key {
                ValueExpr::VarRef(name, _) => symbol(name),
                _ => None,
            },
            _ => None,
        },
        ValueExpr::DynamicLookup(lookups) => lookups.first().and_then(infer_id),
        _ => None,
    }
}
