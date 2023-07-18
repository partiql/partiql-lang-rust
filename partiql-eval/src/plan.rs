use itertools::Itertools;
use petgraph::prelude::StableGraph;
use regex::RegexBuilder;
use std::collections::HashMap;

use partiql_logical as logical;

use partiql_logical::{
    AggFunc, BagOperator, BinaryOp, BindingsOp, CallName, GroupingStrategy, IsTypeExpr, JoinKind,
    LogicalPlan, OpId, PathComponent, Pattern, PatternMatchExpr, SearchedCase, SetQuantifier,
    SortSpecNullOrder, SortSpecOrder, Type, UnaryOp, ValueExpr,
};

use crate::error::{ErrorNode, PlanErr, PlanningError};
use crate::eval;
use crate::eval::evaluable::{
    Avg, Count, EvalGroupingStrategy, EvalJoinKind, EvalOrderBy, EvalOrderBySortCondition,
    EvalOrderBySortSpec, EvalOuterExcept, EvalOuterIntersect, EvalOuterUnion, EvalSubQueryExpr,
    Evaluable, Max, Min, Sum,
};
use crate::eval::expr::pattern_match::like_to_re_pattern;
use crate::eval::expr::{
    EvalBagExpr, EvalBetweenExpr, EvalBinOp, EvalBinOpExpr, EvalDynamicLookup, EvalExpr, EvalFnAbs,
    EvalFnBaseTableExpr, EvalFnBitLength, EvalFnBtrim, EvalFnCardinality, EvalFnCharLength,
    EvalFnCollAvg, EvalFnCollCount, EvalFnCollMax, EvalFnCollMin, EvalFnCollSum, EvalFnExists,
    EvalFnExtractDay, EvalFnExtractHour, EvalFnExtractMinute, EvalFnExtractMonth,
    EvalFnExtractSecond, EvalFnExtractTimezoneHour, EvalFnExtractTimezoneMinute, EvalFnExtractYear,
    EvalFnLower, EvalFnLtrim, EvalFnModulus, EvalFnOctetLength, EvalFnOverlay, EvalFnPosition,
    EvalFnRtrim, EvalFnSubstring, EvalFnUpper, EvalIsTypeExpr, EvalLikeMatch,
    EvalLikeNonStringNonLiteralMatch, EvalListExpr, EvalLitExpr, EvalPath, EvalSearchedCaseExpr,
    EvalTupleExpr, EvalUnaryOp, EvalUnaryOpExpr, EvalVarRef, RE_SIZE_LIMIT,
};
use crate::eval::EvalPlan;
use partiql_catalog::Catalog;
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

        let mut graph: StableGraph<_, _> = Default::default();
        let mut seen = HashMap::new();

        for (s, d, w) in flows {
            let mut add_node = |op_id: &OpId| {
                let logical_op = lg.operator(*op_id).unwrap();
                *seen
                    .entry(*op_id)
                    .or_insert_with(|| graph.add_node(self.get_eval_node::<{ STRICT }>(logical_op)))
            };

            let (s, d) = (add_node(s), add_node(d));
            graph.add_edge(s, d, *w);
        }

        EvalPlan(graph)
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
                        self.plan_values::<{ STRICT }>(expr),
                        as_key,
                        at_key,
                    ))
                } else {
                    Box::new(eval::evaluable::EvalScan::new(
                        self.plan_values::<{ STRICT }>(expr),
                        as_key,
                    ))
                }
            }
            BindingsOp::Project(logical::Project { exprs }) => {
                let exprs: HashMap<_, _> = exprs
                    .iter()
                    .map(|(k, v)| (k.clone(), self.plan_values::<{ STRICT }>(v)))
                    .collect();
                Box::new(eval::evaluable::EvalSelect::new(exprs))
            }
            BindingsOp::ProjectAll => Box::new(eval::evaluable::EvalSelectAll::new()),
            BindingsOp::ProjectValue(logical::ProjectValue { expr }) => {
                let expr = self.plan_values::<{ STRICT }>(expr);
                Box::new(eval::evaluable::EvalSelectValue::new(expr))
            }
            BindingsOp::Filter(logical::Filter { expr }) => Box::new(
                eval::evaluable::EvalFilter::new(self.plan_values::<{ STRICT }>(expr)),
            ),
            BindingsOp::Having(logical::Having { expr }) => Box::new(
                eval::evaluable::EvalHaving::new(self.plan_values::<{ STRICT }>(expr)),
            ),
            BindingsOp::Distinct => Box::new(eval::evaluable::EvalDistinct::new()),
            BindingsOp::Sink => Box::new(eval::evaluable::EvalSink { input: None }),
            BindingsOp::Pivot(logical::Pivot { key, value }) => {
                Box::new(eval::evaluable::EvalPivot::new(
                    self.plan_values::<{ STRICT }>(key),
                    self.plan_values::<{ STRICT }>(value),
                ))
            }
            BindingsOp::Unpivot(logical::Unpivot {
                expr,
                as_key,
                at_key,
            }) => Box::new(eval::evaluable::EvalUnpivot::new(
                self.plan_values::<{ STRICT }>(expr),
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
                    .map(|on_condition| self.plan_values::<{ STRICT }>(on_condition));
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
                let exprs: HashMap<_, _> = exprs
                    .iter()
                    .map(|(k, v)| (k.clone(), self.plan_values::<{ STRICT }>(v)))
                    .collect();
                let aggregate_exprs = aggregate_exprs
                    .iter()
                    .map(|a_e| {
                        let func = match (a_e.func.clone(), a_e.setq.clone()) {
                            (AggFunc::AggAvg, logical::SetQuantifier::All) => {
                                eval::evaluable::AggFunc::Avg(Avg::new_all())
                            }
                            (AggFunc::AggCount, logical::SetQuantifier::All) => {
                                eval::evaluable::AggFunc::Count(Count::new_all())
                            }
                            (AggFunc::AggMax, logical::SetQuantifier::All) => {
                                eval::evaluable::AggFunc::Max(Max::new_all())
                            }
                            (AggFunc::AggMin, logical::SetQuantifier::All) => {
                                eval::evaluable::AggFunc::Min(Min::new_all())
                            }
                            (AggFunc::AggSum, logical::SetQuantifier::All) => {
                                eval::evaluable::AggFunc::Sum(Sum::new_all())
                            }
                            (AggFunc::AggAvg, logical::SetQuantifier::Distinct) => {
                                eval::evaluable::AggFunc::Avg(Avg::new_distinct())
                            }
                            (AggFunc::AggCount, logical::SetQuantifier::Distinct) => {
                                eval::evaluable::AggFunc::Count(Count::new_distinct())
                            }
                            (AggFunc::AggMax, logical::SetQuantifier::Distinct) => {
                                eval::evaluable::AggFunc::Max(Max::new_distinct())
                            }
                            (AggFunc::AggMin, logical::SetQuantifier::Distinct) => {
                                eval::evaluable::AggFunc::Min(Min::new_distinct())
                            }
                            (AggFunc::AggSum, logical::SetQuantifier::Distinct) => {
                                eval::evaluable::AggFunc::Sum(Sum::new_distinct())
                            }
                        };
                        eval::evaluable::AggregateExpression {
                            name: a_e.name.to_string(),
                            expr: self.plan_values::<{ STRICT }>(&a_e.expr),
                            func,
                        }
                    })
                    .collect();
                let group_as_alias = group_as_alias.as_ref().map(|alias| alias.to_string());
                Box::new(eval::evaluable::EvalGroupBy {
                    strategy,
                    exprs,
                    aggregate_exprs,
                    group_as_alias,
                    input: None,
                })
            }
            BindingsOp::ExprQuery(logical::ExprQuery { expr }) => {
                let expr = self.plan_values::<{ STRICT }>(expr);
                Box::new(eval::evaluable::EvalExprQuery::new(expr))
            }
            BindingsOp::OrderBy(logical::OrderBy { specs }) => {
                let cmp = specs
                    .iter()
                    .map(|spec| {
                        let expr = self.plan_values::<{ STRICT }>(&spec.expr);
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
                    limit: limit.as_ref().map(|e| self.plan_values::<{ STRICT }>(e)),
                    offset: offset.as_ref().map(|e| self.plan_values::<{ STRICT }>(e)),
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

    fn plan_values<const STRICT: bool>(&mut self, ve: &ValueExpr) -> Box<dyn EvalExpr> {
        match ve {
            ValueExpr::UnExpr(unary_op, operand) => {
                let operand = self.plan_values::<{ STRICT }>(operand);
                let op = match unary_op {
                    UnaryOp::Pos => EvalUnaryOp::Pos,
                    UnaryOp::Neg => EvalUnaryOp::Neg,
                    UnaryOp::Not => EvalUnaryOp::Not,
                };
                Box::new(EvalUnaryOpExpr { op, operand })
            }
            ValueExpr::BinaryExpr(binop, lhs, rhs) => {
                let lhs = self.plan_values::<{ STRICT }>(lhs);
                let rhs = self.plan_values::<{ STRICT }>(rhs);
                let op = match binop {
                    BinaryOp::And => EvalBinOp::And,
                    BinaryOp::Or => EvalBinOp::Or,
                    BinaryOp::Concat => EvalBinOp::Concat,
                    BinaryOp::Eq => EvalBinOp::Eq,
                    BinaryOp::Neq => EvalBinOp::Neq,
                    BinaryOp::Gt => EvalBinOp::Gt,
                    BinaryOp::Gteq => EvalBinOp::Gteq,
                    BinaryOp::Lt => EvalBinOp::Lt,
                    BinaryOp::Lteq => EvalBinOp::Lteq,
                    BinaryOp::Add => EvalBinOp::Add,
                    BinaryOp::Sub => EvalBinOp::Sub,
                    BinaryOp::Mul => EvalBinOp::Mul,
                    BinaryOp::Div => EvalBinOp::Div,
                    BinaryOp::Mod => EvalBinOp::Mod,
                    BinaryOp::Exp => EvalBinOp::Exp,
                    BinaryOp::In => EvalBinOp::In,
                };
                Box::new(EvalBinOpExpr { op, lhs, rhs })
            }
            ValueExpr::Lit(lit) => Box::new(EvalLitExpr { lit: lit.clone() }),
            ValueExpr::Path(expr, components) => Box::new(EvalPath {
                expr: self.plan_values::<{ STRICT }>(expr),
                components: components
                    .iter()
                    .map(|c| match c {
                        PathComponent::Key(k) => eval::expr::EvalPathComponent::Key(k.clone()),
                        PathComponent::Index(i) => eval::expr::EvalPathComponent::Index(*i),
                        PathComponent::KeyExpr(k) => eval::expr::EvalPathComponent::KeyExpr(
                            self.plan_values::<{ STRICT }>(k),
                        ),
                        PathComponent::IndexExpr(i) => eval::expr::EvalPathComponent::IndexExpr(
                            self.plan_values::<{ STRICT }>(i),
                        ),
                    })
                    .collect(),
            }),
            ValueExpr::VarRef(name) => Box::new(EvalVarRef { name: name.clone() }),
            ValueExpr::TupleExpr(expr) => {
                let attrs: Vec<Box<dyn EvalExpr>> = expr
                    .attrs
                    .iter()
                    .map(|attr| self.plan_values::<{ STRICT }>(attr))
                    .collect();
                let vals: Vec<Box<dyn EvalExpr>> = expr
                    .values
                    .iter()
                    .map(|attr| self.plan_values::<{ STRICT }>(attr))
                    .collect();
                Box::new(EvalTupleExpr { attrs, vals })
            }
            ValueExpr::ListExpr(expr) => {
                let elements: Vec<Box<dyn EvalExpr>> = expr
                    .elements
                    .iter()
                    .map(|elem| self.plan_values::<{ STRICT }>(elem))
                    .collect();
                Box::new(EvalListExpr { elements })
            }
            ValueExpr::BagExpr(expr) => {
                let elements: Vec<Box<dyn EvalExpr>> = expr
                    .elements
                    .iter()
                    .map(|elem| self.plan_values::<{ STRICT }>(elem))
                    .collect();
                Box::new(EvalBagExpr { elements })
            }
            ValueExpr::BetweenExpr(expr) => {
                let value = self.plan_values::<{ STRICT }>(expr.value.as_ref());
                let from = self.plan_values::<{ STRICT }>(expr.from.as_ref());
                let to = self.plan_values::<{ STRICT }>(expr.to.as_ref());
                Box::new(EvalBetweenExpr { value, from, to })
            }
            ValueExpr::PatternMatchExpr(PatternMatchExpr { value, pattern }) => {
                let value = self.plan_values::<{ STRICT }>(value);
                match pattern {
                    Pattern::Like(logical::LikeMatch { pattern, escape }) => {
                        // TODO statically assert escape length
                        if escape.chars().count() > 1 {
                            self.errors.push(PlanningError::IllegalState(format!(
                                "Invalid LIKE expression pattern: {escape}"
                            )));
                            return Box::new(ErrorNode::new());
                        }
                        let escape = escape.chars().next();
                        let regex = like_to_re_pattern(pattern, escape);
                        let regex_pattern =
                            RegexBuilder::new(&regex).size_limit(RE_SIZE_LIMIT).build();
                        match regex_pattern {
                            Ok(pattern) => Box::new(EvalLikeMatch::new(value, pattern)),
                            Err(err) => {
                                self.errors.push(PlanningError::IllegalState(format!(
                                    "Invalid LIKE expression pattern: {regex}. Regex error: {err}"
                                )));
                                Box::new(ErrorNode::new())
                            }
                        }
                    }
                    Pattern::LikeNonStringNonLiteral(logical::LikeNonStringNonLiteralMatch {
                        pattern,
                        escape,
                    }) => {
                        let pattern = self.plan_values::<{ STRICT }>(pattern);
                        let escape = self.plan_values::<{ STRICT }>(escape);
                        Box::new(EvalLikeNonStringNonLiteralMatch::new(
                            value, pattern, escape,
                        ))
                    }
                }
            }
            ValueExpr::SubQueryExpr(expr) => Box::new(EvalSubQueryExpr::new(
                self.plan_eval::<{ STRICT }>(&expr.plan),
            )),
            ValueExpr::SimpleCase(e) => {
                let cases = e
                    .cases
                    .iter()
                    .map(|case| {
                        (
                            self.plan_values::<{ STRICT }>(&ValueExpr::BinaryExpr(
                                BinaryOp::Eq,
                                e.expr.clone(),
                                case.0.clone(),
                            )),
                            self.plan_values::<{ STRICT }>(case.1.as_ref()),
                        )
                    })
                    .collect();
                let default = match &e.default {
                    // If no `ELSE` clause is specified, use implicit `ELSE NULL` (see section 6.9, pg 142 of SQL-92 spec)
                    None => Box::new(EvalLitExpr {
                        lit: Box::new(Null),
                    }),
                    Some(def) => self.plan_values::<{ STRICT }>(def),
                };
                // Here, rewrite `SimpleCaseExpr`s as `SearchedCaseExpr`s
                Box::new(EvalSearchedCaseExpr { cases, default })
            }
            ValueExpr::SearchedCase(e) => {
                let cases = e
                    .cases
                    .iter()
                    .map(|case| {
                        (
                            self.plan_values::<{ STRICT }>(case.0.as_ref()),
                            self.plan_values::<{ STRICT }>(case.1.as_ref()),
                        )
                    })
                    .collect();
                let default = match &e.default {
                    // If no `ELSE` clause is specified, use implicit `ELSE NULL` (see section 6.9, pg 142 of SQL-92 spec)
                    None => Box::new(EvalLitExpr {
                        lit: Box::new(Null),
                    }),
                    Some(def) => self.plan_values::<{ STRICT }>(def.as_ref()),
                };
                Box::new(EvalSearchedCaseExpr { cases, default })
            }
            ValueExpr::IsTypeExpr(i) => {
                let expr = self.plan_values::<{ STRICT }>(i.expr.as_ref());
                match i.not {
                    true => Box::new(EvalUnaryOpExpr {
                        op: EvalUnaryOp::Not,
                        operand: Box::new(EvalIsTypeExpr {
                            expr,
                            is_type: i.is_type.clone(),
                        }),
                    }),
                    false => Box::new(EvalIsTypeExpr {
                        expr,
                        is_type: i.is_type.clone(),
                    }),
                }
            }
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
                self.plan_values::<{ STRICT }>(&rewritten_as_case)
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
                self.plan_values::<{ STRICT }>(&as_case(
                    c.elements.first().unwrap(),
                    &c.elements[1..],
                ))
            }
            ValueExpr::DynamicLookup(lookups) => {
                let lookups = lookups
                    .iter()
                    .map(|lookup| self.plan_values::<{ STRICT }>(lookup))
                    .collect_vec();

                Box::new(EvalDynamicLookup { lookups })
            }
            ValueExpr::Call(logical::CallExpr { name, arguments }) => {
                let mut args = arguments
                    .iter()
                    .map(|arg| self.plan_values::<{ STRICT }>(arg))
                    .collect_vec();
                match name {
                    CallName::Lower => {
                        correct_num_args_or_err!(self, args, 1, "lower");
                        Box::new(EvalFnLower {
                            value: args.pop().unwrap(),
                        })
                    }
                    CallName::Upper => {
                        correct_num_args_or_err!(self, args, 1, "upper");
                        Box::new(EvalFnUpper {
                            value: args.pop().unwrap(),
                        })
                    }
                    CallName::CharLength => {
                        correct_num_args_or_err!(self, args, 1, "char_length");
                        Box::new(EvalFnCharLength {
                            value: args.pop().unwrap(),
                        })
                    }
                    CallName::OctetLength => {
                        correct_num_args_or_err!(self, args, 1, "octet_length");
                        Box::new(EvalFnOctetLength {
                            value: args.pop().unwrap(),
                        })
                    }
                    CallName::BitLength => {
                        correct_num_args_or_err!(self, args, 1, "bit_length");
                        Box::new(EvalFnBitLength {
                            value: args.pop().unwrap(),
                        })
                    }
                    CallName::LTrim => {
                        correct_num_args_or_err!(self, args, 2, "ltrim");
                        let value = args.pop().unwrap();
                        let to_trim = args.pop().unwrap();
                        Box::new(EvalFnLtrim { value, to_trim })
                    }
                    CallName::BTrim => {
                        correct_num_args_or_err!(self, args, 2, "btrim");
                        let value = args.pop().unwrap();
                        let to_trim = args.pop().unwrap();
                        Box::new(EvalFnBtrim { value, to_trim })
                    }
                    CallName::RTrim => {
                        correct_num_args_or_err!(self, args, 2, "rtrim");
                        let value = args.pop().unwrap();
                        let to_trim = args.pop().unwrap();
                        Box::new(EvalFnRtrim { value, to_trim })
                    }
                    CallName::Substring => {
                        correct_num_args_or_err!(self, args, 2, 3, "substring");
                        let length = if args.len() == 3 {
                            Some(args.pop().unwrap())
                        } else {
                            None
                        };
                        let offset = args.pop().unwrap();
                        let value = args.pop().unwrap();

                        Box::new(EvalFnSubstring {
                            value,
                            offset,
                            length,
                        })
                    }
                    CallName::Position => {
                        correct_num_args_or_err!(self, args, 2, "position");
                        let haystack = args.pop().unwrap();
                        let needle = args.pop().unwrap();
                        Box::new(EvalFnPosition { needle, haystack })
                    }
                    CallName::Overlay => {
                        correct_num_args_or_err!(self, args, 3, 4, "overlay");
                        let length = if args.len() == 4 {
                            Some(args.pop().unwrap())
                        } else {
                            None
                        };
                        let offset = args.pop().unwrap();
                        let replacement = args.pop().unwrap();
                        let value = args.pop().unwrap();

                        Box::new(EvalFnOverlay {
                            value,
                            replacement,
                            offset,
                            length,
                        })
                    }
                    CallName::Exists => {
                        correct_num_args_or_err!(self, args, 1, "exists");
                        Box::new(EvalFnExists {
                            value: args.pop().unwrap(),
                        })
                    }
                    CallName::Abs => {
                        correct_num_args_or_err!(self, args, 1, "abs");
                        Box::new(EvalFnAbs {
                            value: args.pop().unwrap(),
                        })
                    }
                    CallName::Mod => {
                        correct_num_args_or_err!(self, args, 2, "mod");
                        let rhs = args.pop().unwrap();
                        let lhs = args.pop().unwrap();
                        Box::new(EvalFnModulus { lhs, rhs })
                    }
                    CallName::Cardinality => {
                        correct_num_args_or_err!(self, args, 1, "cardinality");
                        Box::new(EvalFnCardinality {
                            value: args.pop().unwrap(),
                        })
                    }
                    CallName::ExtractYear => {
                        correct_num_args_or_err!(self, args, 1, "extract year");
                        Box::new(EvalFnExtractYear {
                            value: args.pop().unwrap(),
                        })
                    }
                    CallName::ExtractMonth => {
                        correct_num_args_or_err!(self, args, 1, "extract month");
                        Box::new(EvalFnExtractMonth {
                            value: args.pop().unwrap(),
                        })
                    }
                    CallName::ExtractDay => {
                        correct_num_args_or_err!(self, args, 1, "extract day");
                        Box::new(EvalFnExtractDay {
                            value: args.pop().unwrap(),
                        })
                    }
                    CallName::ExtractHour => {
                        correct_num_args_or_err!(self, args, 1, "extract hour");
                        Box::new(EvalFnExtractHour {
                            value: args.pop().unwrap(),
                        })
                    }
                    CallName::ExtractMinute => {
                        correct_num_args_or_err!(self, args, 1, "extract minute");
                        Box::new(EvalFnExtractMinute {
                            value: args.pop().unwrap(),
                        })
                    }
                    CallName::ExtractSecond => {
                        correct_num_args_or_err!(self, args, 1, "extract second");
                        Box::new(EvalFnExtractSecond {
                            value: args.pop().unwrap(),
                        })
                    }
                    CallName::ExtractTimezoneHour => {
                        correct_num_args_or_err!(self, args, 1, "extract timezone_hour");
                        Box::new(EvalFnExtractTimezoneHour {
                            value: args.pop().unwrap(),
                        })
                    }
                    CallName::ExtractTimezoneMinute => {
                        correct_num_args_or_err!(self, args, 1, "extract timezone_minute");
                        Box::new(EvalFnExtractTimezoneMinute {
                            value: args.pop().unwrap(),
                        })
                    }
                    CallName::CollAvg(setq) => {
                        correct_num_args_or_err!(self, args, 1, "coll_avg");
                        Box::new(EvalFnCollAvg {
                            setq: setq.into(),
                            elems: args.pop().unwrap(),
                        })
                    }
                    CallName::CollCount(setq) => {
                        correct_num_args_or_err!(self, args, 1, "coll_count");
                        Box::new(EvalFnCollCount {
                            setq: setq.into(),
                            elems: args.pop().unwrap(),
                        })
                    }
                    CallName::CollMax(setq) => {
                        correct_num_args_or_err!(self, args, 1, "coll_max");
                        Box::new(EvalFnCollMax {
                            setq: setq.into(),
                            elems: args.pop().unwrap(),
                        })
                    }
                    CallName::CollMin(setq) => {
                        correct_num_args_or_err!(self, args, 1, "coll_min");
                        Box::new(EvalFnCollMin {
                            setq: setq.into(),
                            elems: args.pop().unwrap(),
                        })
                    }
                    CallName::CollSum(setq) => {
                        correct_num_args_or_err!(self, args, 1, "coll_sum");
                        Box::new(EvalFnCollSum {
                            setq: setq.into(),
                            elems: args.pop().unwrap(),
                        })
                    }
                    CallName::ByName(name) => match self.catalog.get_function(name) {
                        None => {
                            self.errors.push(PlanningError::IllegalState(format!(
                                "Function to exist in catalog {name}",
                            )));
                            Box::new(ErrorNode::new())
                        }
                        Some(function) => {
                            let eval = function.plan_eval();
                            Box::new(EvalFnBaseTableExpr { args, expr: eval })
                        }
                    },
                }
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use partiql_catalog::PartiqlCatalog;
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
