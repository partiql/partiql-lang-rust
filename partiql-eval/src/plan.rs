use itertools::{Either, Itertools};
use partiql_catalog::call_defs::ScalarFnCallSpec;
use partiql_logical as logical;
use petgraph::prelude::StableGraph;
use std::collections::HashMap;

use partiql_logical::{
    AggFunc, BagOperator, BinaryOp, BindingsOp, CallName, GroupingStrategy, IsTypeExpr, JoinKind,
    LogicalPlan, OpId, PathComponent, Pattern, PatternMatchExpr, SearchedCase, SetQuantifier,
    SortSpecNullOrder, SortSpecOrder, Type, UnaryOp, ValueExpr, VarRefType,
};

use crate::error::{ErrorNode, PlanErr, PlanningError};
use crate::eval;
use crate::eval::evaluable::{
    AggregateFunction, Any, Avg, Count, EvalGroupingStrategy, EvalJoinKind, EvalOrderBy,
    EvalOrderBySortCondition, EvalOrderBySortSpec, EvalOuterExcept, EvalOuterIntersect,
    EvalOuterUnion, EvalSubQueryExpr, Evaluable, Every, Max, Min, Sum,
};
use crate::eval::expr::{
    BindError, BindEvalExpr, EvalBagExpr, EvalBetweenExpr, EvalCollFn, EvalDynamicLookup, EvalExpr,
    EvalExtractFn, EvalFnAbs, EvalFnBaseTableExpr, EvalFnCardinality, EvalFnExists, EvalFnOverlay,
    EvalFnPosition, EvalFnSubstring, EvalIsTypeExpr, EvalLikeMatch,
    EvalLikeNonStringNonLiteralMatch, EvalListExpr, EvalLitExpr, EvalOpBinary, EvalOpUnary,
    EvalPath, EvalSearchedCaseExpr, EvalStringFn, EvalTrimFn, EvalTupleExpr, EvalVarRef,
};
use crate::eval::EvalPlan;
use partiql_catalog::catalog::{Catalog, FunctionEntryFunction};
use partiql_value::Value::Null;

#[macro_export]
macro_rules! correct_num_args_or_err {
    ($self:expr, $args:expr, $exact_num:literal, $name:expr) => {
        if $args.len() != $exact_num {
            $self.errors.push(PlanningError::IllegalState(format!(
                "Wrong number of arguments for {}",
                $name.to_string()
            )));
            return Box::new(ErrorNode::new());
        }
    };
    ($self:expr, $args:expr, $min_num:literal, $max_num:literal, $name:expr) => {
        if !($min_num..=$max_num).contains(&$args.len()) {
            $self
                .errors
                .push(PlanningError::IllegalState($name.to_string()));
            return Box::new(ErrorNode::new());
        }
    };
}

#[derive(Debug, Eq, PartialEq)]
pub enum EvaluationMode {
    Strict,
    Permissive,
}

pub struct EvaluatorPlanner<'c> {
    mode: EvaluationMode,
    catalog: &'c dyn Catalog,
    errors: Vec<PlanningError>,
}

impl From<&logical::SetQuantifier> for eval::evaluable::SetQuantifier {
    fn from(setq: &SetQuantifier) -> Self {
        match setq {
            SetQuantifier::All => eval::evaluable::SetQuantifier::All,
            SetQuantifier::Distinct => eval::evaluable::SetQuantifier::Distinct,
        }
    }
}

impl From<&UnaryOp> for EvalOpUnary {
    fn from(op: &UnaryOp) -> Self {
        match op {
            UnaryOp::Pos => EvalOpUnary::Pos,
            UnaryOp::Neg => EvalOpUnary::Neg,
            UnaryOp::Not => EvalOpUnary::Not,
        }
    }
}

impl From<&BinaryOp> for EvalOpBinary {
    fn from(op: &BinaryOp) -> Self {
        match op {
            BinaryOp::And => EvalOpBinary::And,
            BinaryOp::Or => EvalOpBinary::Or,
            BinaryOp::Concat => EvalOpBinary::Concat,
            BinaryOp::Eq => EvalOpBinary::Eq,
            BinaryOp::Neq => EvalOpBinary::Neq,
            BinaryOp::Gt => EvalOpBinary::Gt,
            BinaryOp::Gteq => EvalOpBinary::Gteq,
            BinaryOp::Lt => EvalOpBinary::Lt,
            BinaryOp::Lteq => EvalOpBinary::Lteq,
            BinaryOp::Add => EvalOpBinary::Add,
            BinaryOp::Sub => EvalOpBinary::Sub,
            BinaryOp::Mul => EvalOpBinary::Mul,
            BinaryOp::Div => EvalOpBinary::Div,
            BinaryOp::Mod => EvalOpBinary::Mod,
            BinaryOp::Exp => EvalOpBinary::Exp,
            BinaryOp::In => EvalOpBinary::In,
        }
    }
}

impl<'c> EvaluatorPlanner<'c> {
    pub fn new(mode: EvaluationMode, catalog: &'c dyn Catalog) -> Self {
        EvaluatorPlanner {
            mode,
            catalog,
            errors: vec![],
        }
    }

    #[inline]
    pub fn compile(&mut self, plan: &LogicalPlan<BindingsOp>) -> Result<EvalPlan, PlanErr> {
        let plan = match self.mode {
            EvaluationMode::Strict => self.plan_eval::<true>(plan),
            EvaluationMode::Permissive => self.plan_eval::<false>(plan),
        };
        let errors = std::mem::take(&mut self.errors);
        if !errors.is_empty() {
            Err(PlanErr { errors })
        } else {
            Ok(plan)
        }
    }

    #[inline]
    fn plan_eval<const STRICT: bool>(&mut self, lg: &LogicalPlan<BindingsOp>) -> EvalPlan {
        let flows = lg.flows();

        let mut plan_graph: StableGraph<_, _> = Default::default();
        let mut seen = HashMap::new();

        for (s, d, branch_num) in flows {
            let mut add_node = |op_id: &OpId| {
                let logical_op = lg.operator(*op_id).unwrap();
                *seen.entry(*op_id).or_insert_with(|| {
                    plan_graph.add_node(self.get_eval_node::<{ STRICT }>(logical_op))
                })
            };

            let (s, d) = (add_node(s), add_node(d));
            plan_graph.add_edge(s, d, *branch_num);
        }
        let mode = if STRICT {
            EvaluationMode::Strict
        } else {
            EvaluationMode::Permissive
        };
        EvalPlan::new(mode, plan_graph)
    }

    fn get_eval_node<const STRICT: bool>(&mut self, be: &BindingsOp) -> Box<dyn Evaluable> {
        match be {
            BindingsOp::Scan(logical::Scan {
                expr,
                as_key,
                at_key,
            }) => {
                if let Some(at_key) = at_key {
                    Box::new(eval::evaluable::EvalScan::new_with_at_key(
                        self.plan_value::<{ STRICT }>(expr),
                        as_key,
                        at_key,
                    ))
                } else {
                    Box::new(eval::evaluable::EvalScan::new(
                        self.plan_value::<{ STRICT }>(expr),
                        as_key,
                    ))
                }
            }
            BindingsOp::Project(logical::Project { exprs }) => {
                let exprs: Vec<(_, _)> = exprs
                    .iter()
                    .map(|(k, v)| (k.clone(), self.plan_value::<{ STRICT }>(v)))
                    .collect();
                Box::new(eval::evaluable::EvalSelect::new(exprs))
            }
            BindingsOp::ProjectAll => Box::new(eval::evaluable::EvalSelectAll::new()),
            BindingsOp::ProjectValue(logical::ProjectValue { expr }) => {
                let expr = self.plan_value::<{ STRICT }>(expr);
                Box::new(eval::evaluable::EvalSelectValue::new(expr))
            }
            BindingsOp::Filter(logical::Filter { expr }) => Box::new(
                eval::evaluable::EvalFilter::new(self.plan_value::<{ STRICT }>(expr)),
            ),
            BindingsOp::Having(logical::Having { expr }) => Box::new(
                eval::evaluable::EvalHaving::new(self.plan_value::<{ STRICT }>(expr)),
            ),
            BindingsOp::Distinct => Box::new(eval::evaluable::EvalDistinct::new()),
            BindingsOp::Sink => Box::new(eval::evaluable::EvalSink { input: None }),
            BindingsOp::Pivot(logical::Pivot { key, value }) => {
                Box::new(eval::evaluable::EvalPivot::new(
                    self.plan_value::<{ STRICT }>(key),
                    self.plan_value::<{ STRICT }>(value),
                ))
            }
            BindingsOp::Unpivot(logical::Unpivot {
                expr,
                as_key,
                at_key,
            }) => Box::new(eval::evaluable::EvalUnpivot::new(
                self.plan_value::<{ STRICT }>(expr),
                as_key,
                at_key.clone(),
            )),
            BindingsOp::Join(logical::Join {
                kind,
                left,
                right,
                on,
            }) => {
                let kind = match kind {
                    // Model CROSS JOINs as INNER JOINs as mentioned by equivalence mentioned in
                    // section 5.3 of spec https://partiql.org/assets/PartiQL-Specification.pdf#subsection.5.3
                    JoinKind::Cross | JoinKind::Inner => EvalJoinKind::Inner,
                    JoinKind::Left => EvalJoinKind::Left,
                    JoinKind::Right => EvalJoinKind::Right,
                    JoinKind::Full => EvalJoinKind::Full,
                };
                let on = on
                    .as_ref()
                    .map(|on_condition| self.plan_value::<{ STRICT }>(on_condition));
                Box::new(eval::evaluable::EvalJoin::new(
                    kind,
                    self.get_eval_node::<{ STRICT }>(left),
                    self.get_eval_node::<{ STRICT }>(right),
                    on,
                ))
            }
            BindingsOp::GroupBy(logical::GroupBy {
                strategy,
                exprs,
                aggregate_exprs,
                group_as_alias,
            }) => {
                let strategy = match strategy {
                    GroupingStrategy::GroupFull => EvalGroupingStrategy::GroupFull,
                    GroupingStrategy::GroupPartial => EvalGroupingStrategy::GroupPartial,
                };
                let (aliases, exprs): (Vec<String>, Vec<Box<dyn EvalExpr>>) = exprs
                    .iter()
                    .map(|(k, v)| (k.clone(), self.plan_value::<{ STRICT }>(v)))
                    .unzip();

                let mut plan_agg = |a_e: &logical::AggregateExpression| {
                    let func = match &a_e.func {
                        AggFunc::AggAvg => Box::new(Avg {}) as Box<dyn AggregateFunction>,
                        AggFunc::AggCount => Box::new(Count {}) as Box<dyn AggregateFunction>,
                        AggFunc::AggMax => Box::new(Max {}) as Box<dyn AggregateFunction>,
                        AggFunc::AggMin => Box::new(Min {}) as Box<dyn AggregateFunction>,
                        AggFunc::AggSum => Box::new(Sum {}) as Box<dyn AggregateFunction>,
                        AggFunc::AggAny => Box::new(Any {}) as Box<dyn AggregateFunction>,
                        AggFunc::AggEvery => Box::new(Every {}) as Box<dyn AggregateFunction>,
                    };
                    eval::evaluable::AggregateExpression {
                        name: a_e.name.to_string(),
                        expr: self.plan_value::<{ STRICT }>(&a_e.expr),
                        func,
                    }
                };

                let (aggs, distinct_aggs) =
                    aggregate_exprs.iter().partition_map(|ae| match ae.setq {
                        SetQuantifier::All => Either::Left(plan_agg(ae)),
                        SetQuantifier::Distinct => Either::Right(plan_agg(ae)),
                    });

                let group_as_alias = group_as_alias
                    .as_ref()
                    .map(std::string::ToString::to_string);
                Box::new(eval::evaluable::EvalGroupBy::new(
                    strategy,
                    exprs,
                    aliases,
                    aggs,
                    distinct_aggs,
                    group_as_alias,
                ))
            }
            BindingsOp::ExprQuery(logical::ExprQuery { expr }) => {
                let expr = self.plan_value::<{ STRICT }>(expr);
                Box::new(eval::evaluable::EvalExprQuery::new(expr))
            }
            BindingsOp::OrderBy(logical::OrderBy { specs }) => {
                let cmp = specs
                    .iter()
                    .map(|spec| {
                        let expr = self.plan_value::<{ STRICT }>(&spec.expr);
                        let spec = match (&spec.order, &spec.null_order) {
                            (SortSpecOrder::Asc, SortSpecNullOrder::First) => {
                                EvalOrderBySortSpec::AscNullsFirst
                            }
                            (SortSpecOrder::Asc, SortSpecNullOrder::Last) => {
                                EvalOrderBySortSpec::AscNullsLast
                            }
                            (SortSpecOrder::Desc, SortSpecNullOrder::First) => {
                                EvalOrderBySortSpec::DescNullsFirst
                            }
                            (SortSpecOrder::Desc, SortSpecNullOrder::Last) => {
                                EvalOrderBySortSpec::DescNullsLast
                            }
                        };
                        EvalOrderBySortCondition { expr, spec }
                    })
                    .collect_vec();
                Box::new(EvalOrderBy { cmp, input: None })
            }
            BindingsOp::LimitOffset(logical::LimitOffset { limit, offset }) => {
                Box::new(eval::evaluable::EvalLimitOffset {
                    limit: limit.as_ref().map(|e| self.plan_value::<{ STRICT }>(e)),
                    offset: offset.as_ref().map(|e| self.plan_value::<{ STRICT }>(e)),
                    input: None,
                })
            }
            BindingsOp::BagOp(logical::BagOp {
                bag_op: setop,
                setq,
            }) => {
                let setq = setq.into();
                match setop {
                    BagOperator::Union => self.err_nyi("BagOperator::Union"),
                    BagOperator::Intersect => self.err_nyi("BagOperator::Intersect"),
                    BagOperator::Except => self.err_nyi("BagOperator::Except"),
                    BagOperator::OuterUnion => Box::new(EvalOuterUnion::new(setq)),
                    BagOperator::OuterIntersect => Box::new(EvalOuterIntersect::new(setq)),
                    BagOperator::OuterExcept => Box::new(EvalOuterExcept::new(setq)),
                }
            }
        }
    }

    #[inline]
    fn err_nyi(&mut self, feature: &str) -> Box<ErrorNode> {
        let msg = format!("{feature} not yet implemented in evaluator");
        self.err(PlanningError::NotYetImplemented(msg))
    }

    #[inline]
    fn err(&mut self, err: PlanningError) -> Box<ErrorNode> {
        self.errors.push(err);
        Box::new(ErrorNode::new())
    }

    fn unwrap_bind(
        &mut self,
        name: &str,
        op: Result<Box<dyn EvalExpr>, BindError>,
    ) -> Box<dyn EvalExpr> {
        match op {
            Ok(op) => op,
            Err(err) => {
                let err = match err {
                    BindError::ArgNumMismatch { .. } => {
                        PlanningError::IllegalState(format!("Wrong number of arguments for {name}"))
                    }
                    BindError::Unknown => {
                        PlanningError::IllegalState(format!("Unknown error binding {name}"))
                    }
                    BindError::NotYetImplemented(name) => PlanningError::NotYetImplemented(name),
                    BindError::ArgumentConstraint(msg) => PlanningError::IllegalState(msg),
                };

                self.err(err)
            }
        }
    }

    fn plan_values<'v, const STRICT: bool, I>(&mut self, vals: I) -> Vec<Box<dyn EvalExpr>>
    where
        I: Iterator<Item = &'v ValueExpr>,
    {
        vals.map(|arg| self.plan_value::<{ STRICT }>(arg))
            .collect_vec()
    }

    fn plan_value<const STRICT: bool>(&mut self, ve: &ValueExpr) -> Box<dyn EvalExpr> {
        let mut plan_args = |arguments: &[&ValueExpr]| {
            self.plan_values::<{ STRICT }, _>(arguments.iter().map(std::ops::Deref::deref))
        };

        let (name, bind) = match ve {
            ValueExpr::UnExpr(op, operand) => (
                "unary operator",
                EvalOpUnary::from(op).bind::<{ STRICT }>(plan_args(&[operand])),
            ),
            ValueExpr::BinaryExpr(op, lhs, rhs) => (
                "binary operator",
                EvalOpBinary::from(op).bind::<{ STRICT }>(plan_args(&[lhs, rhs])),
            ),
            ValueExpr::Lit(lit) => (
                "literal",
                EvalLitExpr { lit: *lit.clone() }.bind::<{ STRICT }>(vec![]),
            ),
            ValueExpr::Path(expr, components) => (
                "path",
                Ok(Box::new(EvalPath {
                    expr: self.plan_value::<{ STRICT }>(expr),
                    components: components
                        .iter()
                        .map(|c| match c {
                            PathComponent::Key(k) => eval::expr::EvalPathComponent::Key(k.clone()),
                            PathComponent::Index(i) => eval::expr::EvalPathComponent::Index(*i),
                            PathComponent::KeyExpr(k) => eval::expr::EvalPathComponent::KeyExpr(
                                self.plan_value::<{ STRICT }>(k),
                            ),
                            PathComponent::IndexExpr(i) => {
                                eval::expr::EvalPathComponent::IndexExpr(
                                    self.plan_value::<{ STRICT }>(i),
                                )
                            }
                        })
                        .collect(),
                }) as Box<dyn EvalExpr>),
            ),
            ValueExpr::VarRef(name, var_ref_type) => (
                "var ref",
                match var_ref_type {
                    VarRefType::Global => EvalVarRef::Global(name.clone()),
                    VarRefType::Local => EvalVarRef::Local(name.clone()),
                }
                .bind::<{ STRICT }>(vec![]),
            ),
            ValueExpr::TupleExpr(expr) => {
                let attrs: Vec<Box<dyn EvalExpr>> = expr
                    .attrs
                    .iter()
                    .map(|attr| self.plan_value::<{ STRICT }>(attr))
                    .collect();
                let vals: Vec<Box<dyn EvalExpr>> = expr
                    .values
                    .iter()
                    .map(|attr| self.plan_value::<{ STRICT }>(attr))
                    .collect();
                (
                    "tuple expr",
                    Ok(Box::new(EvalTupleExpr { attrs, vals }) as Box<dyn EvalExpr>),
                )
            }
            ValueExpr::ListExpr(expr) => {
                let elements: Vec<Box<dyn EvalExpr>> = expr
                    .elements
                    .iter()
                    .map(|elem| self.plan_value::<{ STRICT }>(elem))
                    .collect();
                (
                    "list expr",
                    Ok(Box::new(EvalListExpr { elements }) as Box<dyn EvalExpr>),
                )
            }
            ValueExpr::BagExpr(expr) => {
                let elements: Vec<Box<dyn EvalExpr>> = expr
                    .elements
                    .iter()
                    .map(|elem| self.plan_value::<{ STRICT }>(elem))
                    .collect();
                (
                    "bag expr",
                    Ok(Box::new(EvalBagExpr { elements }) as Box<dyn EvalExpr>),
                )
            }
            ValueExpr::BetweenExpr(logical::BetweenExpr { value, from, to }) => {
                let args = plan_args(&[value, from, to]);
                ("between", EvalBetweenExpr {}.bind::<{ STRICT }>(args))
            }
            ValueExpr::PatternMatchExpr(PatternMatchExpr { value, pattern }) => {
                let expr = match pattern {
                    Pattern::Like(logical::LikeMatch { pattern, escape }) => {
                        match EvalLikeMatch::create(pattern, escape) {
                            Ok(like) => like.bind::<{ STRICT }>(plan_args(&[value])),
                            Err(err) => {
                                self.errors.push(err);
                                Ok(Box::new(ErrorNode::new()) as Box<dyn EvalExpr>)
                            }
                        }
                    }
                    Pattern::LikeNonStringNonLiteral(logical::LikeNonStringNonLiteralMatch {
                        pattern,
                        escape,
                    }) => {
                        let args = plan_args(&[value, pattern, escape]);
                        EvalLikeNonStringNonLiteralMatch {}.bind::<{ STRICT }>(args)
                    }
                };

                ("pattern expr", expr)
            }
            ValueExpr::SubQueryExpr(expr) => (
                "subquery",
                Ok(Box::new(EvalSubQueryExpr::new(
                    self.plan_eval::<{ STRICT }>(&expr.plan),
                )) as Box<dyn EvalExpr>),
            ),
            ValueExpr::SimpleCase(e) => {
                let cases = e
                    .cases
                    .iter()
                    .map(|case| {
                        (
                            self.plan_value::<{ STRICT }>(&ValueExpr::BinaryExpr(
                                BinaryOp::Eq,
                                e.expr.clone(),
                                case.0.clone(),
                            )),
                            self.plan_value::<{ STRICT }>(case.1.as_ref()),
                        )
                    })
                    .collect();
                let default = match &e.default {
                    // If no `ELSE` clause is specified, use implicit `ELSE NULL` (see section 6.9, pg 142 of SQL-92 spec)
                    None => self.unwrap_bind(
                        "simple case default",
                        EvalLitExpr { lit: Null }.bind::<{ STRICT }>(vec![]),
                    ),
                    Some(def) => self.plan_value::<{ STRICT }>(def),
                };
                // Here, rewrite `SimpleCaseExpr`s as `SearchedCaseExpr`s
                (
                    "simple case",
                    Ok(Box::new(EvalSearchedCaseExpr { cases, default }) as Box<dyn EvalExpr>),
                )
            }
            ValueExpr::SearchedCase(e) => {
                let cases = e
                    .cases
                    .iter()
                    .map(|case| {
                        (
                            self.plan_value::<{ STRICT }>(case.0.as_ref()),
                            self.plan_value::<{ STRICT }>(case.1.as_ref()),
                        )
                    })
                    .collect();
                let default = match &e.default {
                    // If no `ELSE` clause is specified, use implicit `ELSE NULL` (see section 6.9, pg 142 of SQL-92 spec)
                    None => self.unwrap_bind(
                        "searched case default",
                        EvalLitExpr { lit: Null }.bind::<{ STRICT }>(vec![]),
                    ),
                    Some(def) => self.plan_value::<{ STRICT }>(def.as_ref()),
                };
                (
                    "searched case",
                    Ok(Box::new(EvalSearchedCaseExpr { cases, default }) as Box<dyn EvalExpr>),
                )
            }
            ValueExpr::IsTypeExpr(i) => (
                "is type",
                Ok(Box::new(EvalIsTypeExpr {
                    expr: self.plan_value::<{ STRICT }>(i.expr.as_ref()),
                    is_type: i.is_type.clone(),
                    invert: i.not,
                }) as Box<dyn EvalExpr>),
            ),
            ValueExpr::NullIfExpr(n) => {
                // NULLIF can be rewritten using CASE WHEN expressions as per section 6.9 pg 142 of SQL-92 spec:
                //     1) NULLIF (V1, V2) is equivalent to the following <case specification>:
                //         CASE WHEN V1=V2 THEN NULL ELSE V1 END
                let rewritten_as_case = ValueExpr::SearchedCase(SearchedCase {
                    cases: vec![(
                        Box::new(ValueExpr::BinaryExpr(
                            BinaryOp::Eq,
                            n.lhs.clone(),
                            n.rhs.clone(),
                        )),
                        Box::new(ValueExpr::Lit(Box::new(Null))),
                    )],
                    default: Some(n.lhs.clone()),
                });
                (
                    "null if",
                    Ok(self.plan_value::<{ STRICT }>(&rewritten_as_case)),
                )
            }
            ValueExpr::CoalesceExpr(c) => {
                // COALESCE can be rewritten using CASE WHEN expressions as per section 6.9 pg 142 of SQL-92 spec:
                //     2) COALESCE (V1, V2) is equivalent to the following <case specification>:
                //         CASE WHEN V1 IS NOT NULL THEN V1 ELSE V2 END
                //
                //     3) COALESCE (V1, V2, . . . ,n ), for n >= 3, is equivalent to the following <case specification>:
                //         CASE WHEN V1 IS NOT NULL THEN V1 ELSE COALESCE (V2, . . . ,n )
                //         END
                if c.elements.is_empty() {
                    self.errors.push(PlanningError::IllegalState(
                        "Wrong number of arguments to coalesce".to_string(),
                    ));
                    return Box::new(ErrorNode::new());
                }
                fn as_case(v: &ValueExpr, elems: &[ValueExpr]) -> ValueExpr {
                    let sc = SearchedCase {
                        cases: vec![(
                            Box::new(ValueExpr::IsTypeExpr(IsTypeExpr {
                                not: true,
                                expr: Box::new(v.clone()),
                                is_type: Type::NullType,
                            })),
                            Box::new(v.clone()),
                        )],
                        default: elems.first().map(|v2| Box::new(as_case(v2, &elems[1..]))),
                    };
                    ValueExpr::SearchedCase(sc)
                }
                (
                    "coalesce",
                    Ok(self.plan_value::<{ STRICT }>(&as_case(
                        c.elements.first().unwrap(),
                        &c.elements[1..],
                    ))),
                )
            }
            ValueExpr::DynamicLookup(lookups) => {
                let lookups = lookups
                    .iter()
                    .map(|lookup| self.plan_value::<{ STRICT }>(lookup))
                    .collect_vec();

                (
                    "dynamic lookup",
                    Ok(Box::new(EvalDynamicLookup { lookups }) as Box<dyn EvalExpr>),
                )
            }
            ValueExpr::Call(logical::CallExpr { name, arguments }) => {
                let args = self.plan_values::<{ STRICT }, _>(arguments.iter());
                match name {
                    CallName::Lower => ("lower", EvalStringFn::Lower.bind::<{ STRICT }>(args)),
                    CallName::Upper => ("upper", EvalStringFn::Upper.bind::<{ STRICT }>(args)),
                    CallName::CharLength => (
                        "char_length",
                        EvalStringFn::CharLength.bind::<{ STRICT }>(args),
                    ),
                    CallName::OctetLength => (
                        "octet_length",
                        EvalStringFn::OctetLength.bind::<{ STRICT }>(args),
                    ),
                    CallName::BitLength => (
                        "bit_length",
                        EvalStringFn::BitLength.bind::<{ STRICT }>(args),
                    ),
                    CallName::LTrim => ("ltrim", EvalTrimFn::Start.bind::<{ STRICT }>(args)),
                    CallName::BTrim => ("btrim", EvalTrimFn::Both.bind::<{ STRICT }>(args)),
                    CallName::RTrim => ("rtrim", EvalTrimFn::End.bind::<{ STRICT }>(args)),
                    CallName::Substring => {
                        ("substring", EvalFnSubstring {}.bind::<{ STRICT }>(args))
                    }
                    CallName::Position => ("position", EvalFnPosition {}.bind::<{ STRICT }>(args)),
                    CallName::Overlay => ("overlay", EvalFnOverlay {}.bind::<{ STRICT }>(args)),
                    CallName::Exists => ("exists", EvalFnExists {}.bind::<{ STRICT }>(args)),
                    CallName::Abs => ("abs", EvalFnAbs {}.bind::<{ STRICT }>(args)),
                    CallName::Mod => ("mod", EvalOpBinary::Mod.bind::<{ STRICT }>(args)),
                    CallName::Cardinality => {
                        ("cardinality", EvalFnCardinality {}.bind::<{ STRICT }>(args))
                    }
                    CallName::ExtractYear => {
                        ("extract year", EvalExtractFn::Year.bind::<{ STRICT }>(args))
                    }
                    CallName::ExtractMonth => (
                        "extract month",
                        EvalExtractFn::Month.bind::<{ STRICT }>(args),
                    ),
                    CallName::ExtractDay => {
                        ("extract day", EvalExtractFn::Day.bind::<{ STRICT }>(args))
                    }
                    CallName::ExtractHour => {
                        ("extract hour", EvalExtractFn::Hour.bind::<{ STRICT }>(args))
                    }
                    CallName::ExtractMinute => (
                        "extract minute",
                        EvalExtractFn::Minute.bind::<{ STRICT }>(args),
                    ),
                    CallName::ExtractSecond => (
                        "extract second",
                        EvalExtractFn::Second.bind::<{ STRICT }>(args),
                    ),
                    CallName::ExtractTimezoneHour => (
                        "extract timezone_hour",
                        EvalExtractFn::TzHour.bind::<{ STRICT }>(args),
                    ),
                    CallName::ExtractTimezoneMinute => (
                        "extract timezone_minute",
                        EvalExtractFn::TzMinute.bind::<{ STRICT }>(args),
                    ),

                    CallName::CollAvg(setq) => (
                        "coll_avg",
                        EvalCollFn::Avg(setq.into()).bind::<{ STRICT }>(args),
                    ),
                    CallName::CollCount(setq) => (
                        "coll_count",
                        EvalCollFn::Count(setq.into()).bind::<{ STRICT }>(args),
                    ),
                    CallName::CollMax(setq) => (
                        "coll_max",
                        EvalCollFn::Max(setq.into()).bind::<{ STRICT }>(args),
                    ),
                    CallName::CollMin(setq) => (
                        "coll_min",
                        EvalCollFn::Min(setq.into()).bind::<{ STRICT }>(args),
                    ),
                    CallName::CollSum(setq) => (
                        "coll_sum",
                        EvalCollFn::Sum(setq.into()).bind::<{ STRICT }>(args),
                    ),
                    CallName::CollAny(setq) => (
                        "coll_any",
                        EvalCollFn::Any(setq.into()).bind::<{ STRICT }>(args),
                    ),
                    CallName::CollEvery(setq) => (
                        "coll_every",
                        EvalCollFn::Every(setq.into()).bind::<{ STRICT }>(args),
                    ),
                    CallName::ByName(name) => {
                        //
                        let name = name.as_str();
                        match self.catalog.get_function(name) {
                            None => {
                                self.errors.push(PlanningError::IllegalState(format!(
                                    "Function call spec {name} does not exist in catalog",
                                )));

                                (name, Ok(Box::new(ErrorNode::new()) as Box<dyn EvalExpr>))
                            }
                            Some(function) => match function.entry() {
                                FunctionEntryFunction::Scalar(_) => {
                                    todo!("Scalar functions in catalog by name")
                                }
                                FunctionEntryFunction::Table(tbl_fn) => (
                                    name,
                                    Ok(Box::new(EvalFnBaseTableExpr {
                                        args,
                                        expr: tbl_fn.plan_eval(),
                                    }) as Box<dyn EvalExpr>),
                                ),
                                FunctionEntryFunction::Aggregate() => {
                                    todo!("Aggregate functions in catalog by name")
                                }
                            },
                        }
                    }
                    CallName::ById(name, oid, overload_idx) => {
                        let func = self.catalog.get_function_by_id(*oid);
                        let plan = match func {
                            Some(func) => match func.entry() {
                                FunctionEntryFunction::Table(_) => {
                                    todo!("table functions in catalog by id")
                                }
                                FunctionEntryFunction::Scalar(scfn) => {
                                    match scfn.get(*overload_idx) {
                                        None => {
                                            self.errors.push(PlanningError::IllegalState(format!(
                                                "Function call spec {name} overload #{overload_idx} does not exist in catalog",
                                            )));

                                            Ok(Box::new(ErrorNode::new()) as Box<dyn EvalExpr>)
                                        }
                                        Some(overload) => overload.bind::<{ STRICT }>(args),
                                    }
                                }
                                FunctionEntryFunction::Aggregate() => {
                                    todo!("Aggregate functions in catalog by id")
                                }
                            },
                            None => {
                                self.errors.push(PlanningError::IllegalState(format!(
                                    "Function call spec {name} does not exist in catalog",
                                )));

                                Ok(Box::new(ErrorNode::new()) as Box<dyn EvalExpr>)
                            }
                        };
                        (name.as_str(), plan)
                    }
                }
            }
        };

        self.unwrap_bind(name, bind)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use partiql_catalog::catalog::PartiqlCatalog;
    use partiql_logical::CallExpr;
    use partiql_logical::ExprQuery;
    use partiql_value::Value;

    #[test]
    fn test_logical_to_eval_plan_bad_num_arguments() {
        // Tests that the logical to eval plan can report multiple errors.
        // The following is a logical plan with two functions with the wrong number of arguments.
        // Equivalent query: ABS(1, 2) + MOD(3)
        // We define the logical plan manually because the AST to logical lowering will detect and
        // report the error.
        let mut logical = LogicalPlan::new();
        fn lit_int(i: usize) -> ValueExpr {
            ValueExpr::Lit(Box::new(Value::from(i)))
        }

        let expq = logical.add_operator(BindingsOp::ExprQuery(ExprQuery {
            expr: ValueExpr::BinaryExpr(
                BinaryOp::Add,
                Box::new(ValueExpr::Call(CallExpr {
                    name: CallName::Abs,
                    arguments: vec![lit_int(1), lit_int(2)],
                })),
                Box::new(ValueExpr::Call(CallExpr {
                    name: CallName::Mod,
                    arguments: vec![lit_int(3)],
                })),
            ),
        }));
        let sink = logical.add_operator(BindingsOp::Sink);
        logical.add_flow(expq, sink);

        let catalog = PartiqlCatalog::default();
        let mut planner = EvaluatorPlanner::new(EvaluationMode::Permissive, &catalog);
        let plan = planner.compile(&logical);

        assert!(plan.is_err());
        let planning_errs = plan.expect_err("Expect errs").errors;
        assert_eq!(planning_errs.len(), 2);
    }
}
