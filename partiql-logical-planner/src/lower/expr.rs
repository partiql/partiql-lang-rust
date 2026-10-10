//! Lowering of expressions.

use ordered_float::OrderedFloat;
use partiql_ast::ast::{
    self, BinOp, BinOpKind, Call, CallAgg, CallArg, CallArgNamed, Case, Expr, Like, Lit, Path,
    PathStep, ProjectionKind, QuerySet, UniOpKind,
};
use partiql_ast::visit::{Traverse, Visit, Visitor};
use partiql_ast_passes::error::AstTransformError;
use partiql_catalog::call_defs::{CallArgument, CallDef};
use partiql_logical as logical;
use partiql_logical::AggFunc::{AggAny, AggAvg, AggCount, AggEvery, AggMax, AggMin, AggSum};
use partiql_logical::{
    AggregateExpression, BagExpr, BetweenExpr, GraphMatchExpr, IsTypeExpr, LikeMatch,
    LikeNonStringNonLiteralMatch, ListExpr, PathComponent, Pattern, PatternMatchExpr, TupleExpr,
    ValueExpr, VarRefType,
};
use partiql_value::BindingsName;
use std::borrow::Cow;

use super::scope::{BindingKind, Clause, Cx, Scope};
use super::{nyi, symprim_to_binding, Lowerer, Result};
use crate::functions::Function;
use crate::graph::GraphToLogical;

impl Lowerer<'_> {
    /// Lowers `expr`, resolving its names in `cx`.
    ///
    /// `cx.coerce` says whether the position expects a single value; each operator sets it for
    /// its operands, and it decides whether a SQL `SELECT` subquery is coerced to a scalar.
    pub(super) fn lower_expr(&mut self, expr: &Expr, cx: Cx<'_>) -> Result<ValueExpr> {
        Ok(match expr {
            Expr::Lit(lit) => {
                let lit = lit_to_lit(&lit.node).unwrap_or_else(|err| {
                    self.errors.push(err);
                    logical::Lit::Missing
                });
                ValueExpr::Lit(Box::new(lit))
            }
            Expr::VarRef(var) => self.resolve_var(&var.node, &cx)?,
            Expr::BinOp(bin_op) => self.lower_bin_op(&bin_op.node, cx)?,
            Expr::UniOp(uni_op) => {
                let op = match uni_op.node.kind {
                    UniOpKind::Pos => logical::UnaryOp::Pos,
                    UniOpKind::Neg => logical::UnaryOp::Neg,
                    UniOpKind::Not => logical::UnaryOp::Not,
                    _ => return Err(nyi("Unsupported unary operator")),
                };
                let operand = self.lower_expr(&uni_op.node.expr, cx.coerce(true))?;
                ValueExpr::UnExpr(op, Box::new(operand))
            }
            Expr::Like(like) => self.lower_like(&like.node, cx.coerce(true))?,
            Expr::Between(between) => {
                let cx = cx.coerce(true);
                ValueExpr::BetweenExpr(BetweenExpr {
                    value: Box::new(self.lower_expr(&between.node.value, cx)?),
                    from: Box::new(self.lower_expr(&between.node.from, cx)?),
                    to: Box::new(self.lower_expr(&between.node.to, cx)?),
                })
            }
            Expr::In(in_expr) => {
                // The right side is a collection; neither side is coerced.
                let cx = cx.coerce(false);
                let lhs = self.lower_expr(&in_expr.node.lhs, cx)?;
                let rhs = self.lower_expr(&in_expr.node.rhs, cx)?;
                ValueExpr::BinaryExpr(logical::BinaryOp::In, Box::new(lhs), Box::new(rhs))
            }
            Expr::Case(case) => self.lower_case(&case.node, cx.coerce(false))?,
            Expr::Struct(tuple) => {
                let cx = cx.coerce(false);
                let mut attrs = Vec::with_capacity(tuple.node.fields.len());
                let mut values = Vec::with_capacity(tuple.node.fields.len());
                for field in &tuple.node.fields {
                    attrs.push(self.lower_expr(&field.first, cx)?);
                    values.push(self.lower_expr(&field.second, cx)?);
                }
                ValueExpr::TupleExpr(TupleExpr { attrs, values })
            }
            Expr::Bag(bag) => ValueExpr::BagExpr(BagExpr {
                elements: self.lower_exprs(&bag.node.values, cx.coerce(false))?,
            }),
            Expr::List(list) => ValueExpr::ListExpr(ListExpr {
                elements: self.lower_exprs(&list.node.values, cx.coerce(false))?,
            }),
            Expr::Path(path) => self.lower_path(&path.node, cx.coerce(false))?,
            Expr::Call(call) => self.lower_call(&call.node, cx.coerce(false))?,
            Expr::CallAgg(call) => self.lower_call_agg(&call.node, cx.coerce(false))?,
            Expr::GraphMatch(graph_match) => self.lower_graph_match(&graph_match.node, cx)?,
            Expr::Query(query) => {
                let subquery = self.lower_subquery(&query.node, cx.scope)?;
                // A SQL-style `SELECT` (a list or `*`) in a single-value position is coerced to
                // its single value (spec section 9.1); `SELECT VALUE` builds a collection.
                let is_sql_select = matches!(
                    &query.node.set.node,
                    QuerySet::Select(select) if matches!(
                        select.node.project.node.kind,
                        ProjectionKind::ProjectStar | ProjectionKind::ProjectList(_)
                    )
                );
                if cx.coerce && is_sql_select {
                    ValueExpr::Call(logical::CallExpr {
                        name: logical::CallName::CollToScalar,
                        arguments: vec![subquery],
                    })
                } else {
                    subquery
                }
            }
            Expr::Error => {
                return Err(AstTransformError::IllegalState(
                    "error node in expression".into(),
                ))
            }
            _ => return Err(nyi("Unsupported expression")),
        })
    }

    fn lower_exprs(&mut self, exprs: &[Box<Expr>], cx: Cx<'_>) -> Result<Vec<ValueExpr>> {
        exprs.iter().map(|e| self.lower_expr(e, cx)).collect()
    }

    fn lower_bin_op(&mut self, bin_op: &BinOp, cx: Cx<'_>) -> Result<ValueExpr> {
        // Operands are single-value contexts, except for the `IS` type predicate.
        let cx = cx.coerce(bin_op.kind != BinOpKind::Is);
        let lhs = self.lower_expr(&bin_op.lhs, cx)?;
        let rhs = self.lower_expr(&bin_op.rhs, cx)?;
        if bin_op.kind == BinOpKind::Is {
            let is_type = match &rhs {
                ValueExpr::Lit(lit) => match lit.as_ref() {
                    logical::Lit::Null => logical::Type::NullType,
                    logical::Lit::Missing => logical::Type::MissingType,
                    _ => return Err(nyi("Unsupported rhs literal for `IS`")),
                },
                _ => return Err(nyi("Unsupported rhs for `IS`")),
            };
            return Ok(ValueExpr::IsTypeExpr(IsTypeExpr {
                not: false,
                expr: Box::new(lhs),
                is_type,
            }));
        }
        let op = match bin_op.kind {
            BinOpKind::Add => logical::BinaryOp::Add,
            BinOpKind::Div => logical::BinaryOp::Div,
            BinOpKind::Exp => logical::BinaryOp::Exp,
            BinOpKind::Mod => logical::BinaryOp::Mod,
            BinOpKind::Mul => logical::BinaryOp::Mul,
            BinOpKind::Sub => logical::BinaryOp::Sub,
            BinOpKind::And => logical::BinaryOp::And,
            BinOpKind::Or => logical::BinaryOp::Or,
            BinOpKind::Concat => logical::BinaryOp::Concat,
            BinOpKind::Eq => logical::BinaryOp::Eq,
            BinOpKind::Gt => logical::BinaryOp::Gt,
            BinOpKind::Gte => logical::BinaryOp::Gteq,
            BinOpKind::Lt => logical::BinaryOp::Lt,
            BinOpKind::Lte => logical::BinaryOp::Lteq,
            BinOpKind::Ne => logical::BinaryOp::Neq,
            _ => return Err(nyi("Unsupported binary operator")),
        };
        Ok(ValueExpr::BinaryExpr(op, Box::new(lhs), Box::new(rhs)))
    }

    fn lower_like(&mut self, like: &Like, cx: Cx<'_>) -> Result<ValueExpr> {
        let value = Box::new(self.lower_expr(&like.value, cx)?);
        let pattern = self.lower_expr(&like.pattern, cx)?;
        let escape = match &like.escape {
            Some(escape) => self.lower_expr(escape, cx)?,
            None => ValueExpr::Lit(Box::new(logical::Lit::String(String::default()))),
        };
        let pattern = match (&pattern, &escape) {
            (ValueExpr::Lit(pattern_lit), ValueExpr::Lit(escape_lit)) => {
                match (pattern_lit.as_ref(), escape_lit.as_ref()) {
                    (logical::Lit::String(pattern), logical::Lit::String(escape)) => {
                        Some(Pattern::Like(LikeMatch {
                            pattern: pattern.clone(),
                            escape: escape.clone(),
                        }))
                    }
                    _ => None,
                }
            }
            _ => None,
        }
        .unwrap_or_else(|| {
            Pattern::LikeNonStringNonLiteral(LikeNonStringNonLiteralMatch {
                pattern: Box::new(pattern),
                escape: Box::new(escape),
            })
        });
        Ok(ValueExpr::PatternMatchExpr(PatternMatchExpr {
            value,
            pattern,
        }))
    }

    fn lower_case(&mut self, case: &Case, cx: Cx<'_>) -> Result<ValueExpr> {
        let (operand, pairs, default) = match case {
            Case::SimpleCase(c) => (Some(&c.expr), &c.cases, &c.default),
            Case::SearchedCase(c) => (None, &c.cases, &c.default),
            _ => return Err(nyi("Unsupported CASE")),
        };
        let operand = match operand {
            Some(operand) => Some(Box::new(self.lower_expr(operand, cx)?)),
            None => None,
        };
        let mut cases = Vec::with_capacity(pairs.len());
        for pair in pairs {
            let when = self.lower_expr(&pair.first, cx)?;
            let then = self.lower_expr(&pair.second, cx)?;
            cases.push((Box::new(when), Box::new(then)));
        }
        let default = match default {
            Some(default) => Some(Box::new(self.lower_expr(default, cx)?)),
            None => None,
        };
        Ok(match operand {
            Some(expr) => ValueExpr::SimpleCase(logical::SimpleCase {
                expr,
                cases,
                default,
            }),
            None => ValueExpr::SearchedCase(logical::SearchedCase { cases, default }),
        })
    }

    fn lower_path(&mut self, path: &Path, cx: Cx<'_>) -> Result<ValueExpr> {
        let root = self.lower_expr(&path.root, cx)?;
        let mut steps = Vec::with_capacity(path.steps.len());
        for step in &path.steps {
            let step = match step {
                PathStep::PathProject(step) | PathStep::PathIndex(step) => match &*step.index {
                    // In `a.b` and `a[b]`, `b` is an attribute name, not a variable.
                    Expr::VarRef(var) => PathComponent::Key(symprim_to_binding(&var.node.name)?),
                    index => match self.lower_expr(index, cx)? {
                        ValueExpr::Lit(lit) => match *lit {
                            logical::Lit::Int8(idx) => PathComponent::Index(idx.into()),
                            logical::Lit::Int16(idx) => PathComponent::Index(idx.into()),
                            logical::Lit::Int32(idx) => PathComponent::Index(idx.into()),
                            logical::Lit::Int64(idx) => PathComponent::Index(idx),
                            logical::Lit::String(key) => {
                                PathComponent::Key(BindingsName::CaseInsensitive(Cow::Owned(key)))
                            }
                            lit => {
                                PathComponent::IndexExpr(Box::new(ValueExpr::Lit(Box::new(lit))))
                            }
                        },
                        ValueExpr::VarRef(name, _) => PathComponent::Key(name),
                        // TODO if type is statically STRING, then use KeyExpr
                        expr => PathComponent::IndexExpr(Box::new(expr)),
                    },
                },
                PathStep::PathForEach => return Err(nyi("PathStep::PathForEach")),
                PathStep::PathUnpivot => return Err(nyi("PathStep::PathUnpivot")),
                _ => return Err(nyi("Unsupported path step")),
            };
            steps.push(step);
        }
        Ok(ValueExpr::Path(Box::new(root), steps))
    }

    fn lower_call_args(
        &mut self,
        args: &[ast::AstNode<CallArg>],
        cx: Cx<'_>,
    ) -> Result<Vec<CallArgument>> {
        let mut lowered = Vec::with_capacity(args.len());
        for arg in args {
            lowered.push(match &arg.node {
                CallArg::Star() => CallArgument::Star,
                CallArg::Positional(expr) => CallArgument::Positional(self.lower_expr(expr, cx)?),
                CallArg::Named(CallArgNamed { name, value }) => {
                    CallArgument::Named(name.value.to_lowercase(), self.lower_expr(value, cx)?)
                }
                CallArg::PositionalType(_) => return Err(nyi("PositionalType call argument")),
                CallArg::NamedType(_) => return Err(nyi("NamedType call argument")),
                _ => return Err(nyi("Unsupported call argument")),
            });
        }
        Ok(lowered)
    }

    fn lower_call(&mut self, call: &Call, cx: Cx<'_>) -> Result<ValueExpr> {
        // Call arguments may be collections (`EXISTS`, `CARDINALITY`), so are not coerced.
        let args = self.lower_call_args(&call.args, cx)?;
        let name = call.func_name.value.to_lowercase();
        let call_def_to_vexpr = |call_def: &CallDef| call_def.lookup(&args, &name);
        let expr = self
            .fnsym_tab
            .lookup(&name)
            .map(call_def_to_vexpr)
            .or_else(|| {
                self.catalog
                    .get_function(&name)
                    .map(|e| e.resolve(&name, &args))
            })
            .map(|res| res.map_err(Into::into))
            .unwrap_or_else(|| Err(AstTransformError::UnsupportedFunction(name.clone())));
        // Report the error but keep lowering, to report further errors too.
        Ok(expr.unwrap_or_else(|err| {
            self.errors.push(err);
            ValueExpr::Lit(Box::new(logical::Lit::Missing))
        }))
    }

    /// Lowers an SQL aggregate call (`SUM(b)`, not the `COLL_` functions) to a reference to the
    /// value the GROUP BY computes for it: `SELECT a, SUM(b) FROM t GROUP BY a` becomes
    /// `SELECT a, $__agg_1 FROM t GROUP BY a` with `$__agg_1 = SUM(b)` on the GROUP BY.
    fn lower_call_agg(&mut self, call: &CallAgg, cx: Cx<'_>) -> Result<ValueExpr> {
        let mut args = self.lower_call_args(&call.args, cx)?;
        let name = call.func_name.value.to_lowercase();
        let new_name = self.fresh_agg_name();

        // The set quantifier defaults to `ALL`.
        let (setq, expr) = match args.pop() {
            Some(CallArgument::Positional(expr)) => (logical::SetQuantifier::All, expr),
            Some(CallArgument::Named(quantifier, expr)) => match quantifier.as_str() {
                "all" => (logical::SetQuantifier::All, expr),
                "distinct" => (logical::SetQuantifier::Distinct, expr),
                _ => {
                    return Err(AstTransformError::IllegalState(
                        "Invalid set quantifier".to_string(),
                    ))
                }
            },
            Some(CallArgument::Star) => (
                logical::SetQuantifier::All,
                ValueExpr::Lit(Box::new(logical::Lit::Int8(1))),
            ),
            Some(_) => return Err(nyi("Unsupported aggregate argument")),
            None => return Err(AstTransformError::IllegalState("env is empty".to_string())),
        };
        let func = match name.as_str() {
            "avg" => AggAvg,
            "count" => AggCount,
            "max" => AggMax,
            "min" => AggMin,
            "sum" => AggSum,
            "any" | "some" => AggAny,
            "every" => AggEvery,
            _ => {
                // Report the error but keep lowering, to report further errors too.
                self.errors
                    .push(AstTransformError::UnsupportedFunction(name));
                AggAvg
            }
        };
        self.aggs.push(AggregateExpression {
            name: new_name.clone(),
            expr,
            func,
            setq,
        });
        Ok(ValueExpr::VarRef(
            BindingsName::CaseSensitive(Cow::Owned(new_name)),
            VarRefType::Local,
        ))
    }

    /// `<graph> MATCH <pattern>`: predicates in the pattern see its node and edge variables.
    fn lower_graph_match(
        &mut self,
        graph_match: &ast::GraphMatch,
        cx: Cx<'_>,
    ) -> Result<ValueExpr> {
        let value = Box::new(self.lower_expr(&graph_match.expr, cx)?);
        let mut pattern_scope = Scope::new(Some(cx.scope));
        let mut variables = GraphVariables::default();
        graph_match.pattern.visit(&mut variables);
        for name in variables.0 {
            pattern_scope.bind(name, false, BindingKind::Graph);
        }
        let pattern = GraphToLogical::new(|expr: &Expr| {
            self.lower_expr(expr, Cx::new(&pattern_scope, Clause::Graph))
        })
        .plan_graph_match(graph_match)?;
        Ok(ValueExpr::GraphMatch(Box::new(GraphMatchExpr {
            value,
            pattern,
        })))
    }
}

/// Collects the node and edge variables of a graph pattern.
#[derive(Default)]
struct GraphVariables(Vec<ast::SymbolPrimitive>);

impl<'ast> Visitor<'ast> for GraphVariables {
    fn exit_graph_match_node(&mut self, node: &'ast ast::GraphMatchNode) -> Traverse {
        self.0.extend(node.variable.clone());
        Traverse::Continue
    }

    fn exit_graph_match_edge(&mut self, edge: &'ast ast::GraphMatchEdge) -> Traverse {
        self.0.extend(edge.variable.clone());
        Traverse::Continue
    }
}

pub(super) fn lit_to_lit(lit: &Lit) -> Result<logical::Lit> {
    fn tuple_pair(field: &ast::LitField) -> Option<Result<(String, logical::Lit)>> {
        let key = field.first.clone();
        match &field.second.node {
            Lit::Missing => None,
            value => match lit_to_lit(value) {
                Ok(value) => Some(Ok((key, value))),
                Err(e) => Some(Err(e)),
            },
        }
    }

    let val = match lit {
        Lit::Null => logical::Lit::Null,
        Lit::Missing => logical::Lit::Missing,
        Lit::Int8Lit(n) => logical::Lit::Int8(*n),
        Lit::Int16Lit(n) => logical::Lit::Int16(*n),
        Lit::Int32Lit(n) => logical::Lit::Int32(*n),
        Lit::Int64Lit(n) => logical::Lit::Int64(*n),
        Lit::DecimalLit(d) => logical::Lit::Decimal(*d),
        Lit::NumericLit(n) => logical::Lit::Decimal(*n),
        Lit::RealLit(f) => logical::Lit::Double(OrderedFloat::from(*f as f64)),
        Lit::FloatLit(f) => logical::Lit::Double(OrderedFloat::from(*f as f64)),
        Lit::DoubleLit(f) => logical::Lit::Double(OrderedFloat::from(*f)),
        Lit::BoolLit(b) => logical::Lit::Bool(*b),
        Lit::EmbeddedDocLit(s, _typ) => {
            // TODO fix type for boxed variants
            logical::Lit::Variant(s.clone().into_bytes(), "Ion".to_string())
        }
        Lit::CharStringLit(s) => logical::Lit::String(s.clone()),
        Lit::NationalCharStringLit(s) => logical::Lit::String(s.clone()),
        Lit::BitStringLit(_) => return Err(nyi("Lit::BitStringLit")),
        Lit::HexStringLit(_) => return Err(nyi("Lit::HexStringLit")),
        Lit::BagLit(b) => {
            let bag: Result<_> = b.node.values.iter().map(lit_to_lit).collect();
            logical::Lit::Bag(bag?)
        }
        Lit::ListLit(l) => {
            let l: Result<_> = l.node.values.iter().map(lit_to_lit).collect();
            logical::Lit::List(l?)
        }
        Lit::StructLit(s) => {
            let tuple: Result<_> = s.node.fields.iter().filter_map(tuple_pair).collect();
            logical::Lit::Struct(tuple?)
        }
        Lit::TypedLit(_, _) => return Err(nyi("Lit::TypedLit")),
        _ => return Err(nyi("literal variant")),
    };
    Ok(val)
}
