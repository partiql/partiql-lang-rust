use itertools::Itertools;
use petgraph::prelude::StableGraph;
use std::collections::HashMap;

use partiql_logical as logical;

use partiql_logical::{
    AggFunc, BinaryOp, BindingsOp, CallName, GroupingStrategy, IsTypeExpr, JoinKind, LogicalPlan,
    OpId, PathComponent, Pattern, PatternMatchExpr, SearchedCase, SortSpecNullOrder, SortSpecOrder,
    Type, UnaryOp, ValueExpr,
};

use crate::eval;
use crate::eval::evaluable::{
    Avg, Count, EvalGroupingStrategy, EvalJoinKind, EvalOrderBy, EvalOrderBySortCondition,
    EvalOrderBySortSpec, EvalSubQueryExpr, Evaluable, Max, Min, Sum,
};
use crate::eval::expr::pattern_match::like_to_re_pattern;
use crate::eval::expr::{
    EvalBagExpr, EvalBetweenExpr, EvalBinOp, EvalBinOpExpr, EvalDynamicLookup, EvalExpr, EvalFnAbs,
    EvalFnBitLength, EvalFnBtrim, EvalFnCardinality, EvalFnCharLength, EvalFnExists,
    EvalFnExtractDay, EvalFnExtractHour, EvalFnExtractMinute, EvalFnExtractMonth,
    EvalFnExtractSecond, EvalFnExtractTimezoneHour, EvalFnExtractTimezoneMinute, EvalFnExtractYear,
    EvalFnLower, EvalFnLtrim, EvalFnModulus, EvalFnOctetLength, EvalFnOverlay, EvalFnPosition,
    EvalFnRtrim, EvalFnSubstring, EvalFnUpper, EvalIsTypeExpr, EvalLikeMatch,
    EvalLikeNonStringNonLiteralMatch, EvalListExpr, EvalLitExpr, EvalPath, EvalSearchedCaseExpr,
    EvalTupleExpr, EvalUnaryOp, EvalUnaryOpExpr, EvalVarRef,
};
use crate::eval::EvalPlan;
use partiql_value::Value::Null;

#[derive(Default)]
pub struct EvaluatorPlanner;

impl EvaluatorPlanner {
    #[inline]
    pub fn compile(&self, plan: &LogicalPlan<BindingsOp>) -> EvalPlan {
        self.plan_eval(plan)
    }

    #[inline]
    fn plan_eval(&self, lg: &LogicalPlan<BindingsOp>) -> EvalPlan {
        let ops = lg.operators();
        let flows = lg.flows();

        let mut graph: StableGraph<_, _> = Default::default();
        let mut seen = HashMap::new();

        for (s, d, w) in flows {
            let mut add_node = |op_id: &OpId| {
                let logical_op = &ops[op_id.index() - 1];
                *seen
                    .entry(*op_id)
                    .or_insert_with(|| graph.add_node(self.get_eval_node(logical_op)))
            };

            let (s, d) = (add_node(s), add_node(d));
            graph.add_edge(s, d, *w);
        }

        EvalPlan(graph)
    }

    fn get_eval_node(&self, be: &BindingsOp) -> Box<dyn Evaluable> {
        match be {
            BindingsOp::Scan(logical::Scan {
                expr,
                as_key,
                at_key,
            }) => {
                if let Some(at_key) = at_key {
                    Box::new(eval::evaluable::EvalScan::new_with_at_key(
                        self.plan_values(expr),
                        as_key,
                        at_key,
                    ))
                } else {
                    Box::new(eval::evaluable::EvalScan::new(
                        self.plan_values(expr),
                        as_key,
                    ))
                }
            }
            BindingsOp::Project(logical::Project { exprs }) => {
                let exprs: HashMap<_, _> = exprs
                    .iter()
                    .map(|(k, v)| (k.clone(), self.plan_values(v)))
                    .collect();
                Box::new(eval::evaluable::EvalSelect::new(exprs))
            }
            BindingsOp::ProjectAll => Box::new(eval::evaluable::EvalSelectAll::new()),
            BindingsOp::ProjectValue(logical::ProjectValue { expr }) => {
                let expr = self.plan_values(expr);
                Box::new(eval::evaluable::EvalSelectValue::new(expr))
            }
            BindingsOp::Filter(logical::Filter { expr }) => {
                Box::new(eval::evaluable::EvalFilter::new(self.plan_values(expr)))
            }
            BindingsOp::Having(logical::Having { expr }) => {
                Box::new(eval::evaluable::EvalHaving::new(self.plan_values(expr)))
            }
            BindingsOp::Distinct => Box::new(eval::evaluable::EvalDistinct::new()),
            BindingsOp::Sink => Box::new(eval::evaluable::EvalSink { input: None }),
            BindingsOp::Pivot(logical::Pivot { key, value }) => Box::new(
                eval::evaluable::EvalPivot::new(self.plan_values(key), self.plan_values(value)),
            ),
            BindingsOp::Unpivot(logical::Unpivot {
                expr,
                as_key,
                at_key,
            }) => Box::new(eval::evaluable::EvalUnpivot::new(
                self.plan_values(expr),
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
                    .map(|on_condition| self.plan_values(on_condition));
                Box::new(eval::evaluable::EvalJoin::new(
                    kind,
                    self.get_eval_node(left),
                    self.get_eval_node(right),
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
                    .map(|(k, v)| (k.clone(), self.plan_values(v)))
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
                            expr: self.plan_values(&a_e.expr),
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
                let expr = self.plan_values(expr);
                Box::new(eval::evaluable::EvalExprQuery::new(expr))
            }
            BindingsOp::OrderBy(logical::OrderBy { specs }) => {
                let cmp = specs
                    .iter()
                    .map(|spec| {
                        let expr = self.plan_values(&spec.expr);
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
                    limit: limit.as_ref().map(|e| self.plan_values(e)),
                    offset: offset.as_ref().map(|e| self.plan_values(e)),
                    input: None,
                })
            }

            BindingsOp::SetOp => todo!("SetOp"),
        }
    }

    fn plan_values(&self, ve: &ValueExpr) -> Box<dyn EvalExpr> {
        match ve {
            ValueExpr::UnExpr(unary_op, operand) => {
                let operand = self.plan_values(operand);
                let op = match unary_op {
                    UnaryOp::Pos => EvalUnaryOp::Pos,
                    UnaryOp::Neg => EvalUnaryOp::Neg,
                    UnaryOp::Not => EvalUnaryOp::Not,
                };
                Box::new(EvalUnaryOpExpr { op, operand })
            }
            ValueExpr::BinaryExpr(binop, lhs, rhs) => {
                let lhs = self.plan_values(lhs);
                let rhs = self.plan_values(rhs);
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
                expr: self.plan_values(expr),
                components: components
                    .iter()
                    .map(|c| match c {
                        PathComponent::Key(k) => eval::expr::EvalPathComponent::Key(k.clone()),
                        PathComponent::Index(i) => eval::expr::EvalPathComponent::Index(*i),
                        PathComponent::KeyExpr(k) => {
                            eval::expr::EvalPathComponent::KeyExpr(self.plan_values(k))
                        }
                        PathComponent::IndexExpr(i) => {
                            eval::expr::EvalPathComponent::IndexExpr(self.plan_values(i))
                        }
                    })
                    .collect(),
            }),
            ValueExpr::VarRef(name) => Box::new(EvalVarRef { name: name.clone() }),
            ValueExpr::TupleExpr(expr) => {
                let attrs: Vec<Box<dyn EvalExpr>> = expr
                    .attrs
                    .iter()
                    .map(|attr| self.plan_values(attr))
                    .collect();
                let vals: Vec<Box<dyn EvalExpr>> = expr
                    .values
                    .iter()
                    .map(|attr| self.plan_values(attr))
                    .collect();
                Box::new(EvalTupleExpr { attrs, vals })
            }
            ValueExpr::ListExpr(expr) => {
                let elements: Vec<Box<dyn EvalExpr>> = expr
                    .elements
                    .iter()
                    .map(|elem| self.plan_values(elem))
                    .collect();
                Box::new(EvalListExpr { elements })
            }
            ValueExpr::BagExpr(expr) => {
                let elements: Vec<Box<dyn EvalExpr>> = expr
                    .elements
                    .iter()
                    .map(|elem| self.plan_values(elem))
                    .collect();
                Box::new(EvalBagExpr { elements })
            }
            ValueExpr::BetweenExpr(expr) => {
                let value = self.plan_values(expr.value.as_ref());
                let from = self.plan_values(expr.from.as_ref());
                let to = self.plan_values(expr.to.as_ref());
                Box::new(EvalBetweenExpr { value, from, to })
            }
            ValueExpr::PatternMatchExpr(PatternMatchExpr { value, pattern }) => {
                let value = self.plan_values(value);
                match pattern {
                    Pattern::Like(logical::LikeMatch { pattern, escape }) => {
                        // TODO statically assert escape length
                        assert!(escape.chars().count() <= 1);
                        let escape = escape.chars().next();
                        let regex = like_to_re_pattern(pattern, escape);
                        Box::new(EvalLikeMatch::new(value, &regex))
                    }
                    Pattern::LikeNonStringNonLiteral(logical::LikeNonStringNonLiteralMatch {
                        pattern,
                        escape,
                    }) => {
                        let pattern = self.plan_values(pattern);
                        let escape = self.plan_values(escape);
                        Box::new(EvalLikeNonStringNonLiteralMatch::new(
                            value, pattern, escape,
                        ))
                    }
                }
            }
            ValueExpr::SubQueryExpr(expr) => {
                Box::new(EvalSubQueryExpr::new(self.plan_eval(&expr.plan)))
            }
            ValueExpr::SimpleCase(e) => {
                let cases = e
                    .cases
                    .iter()
                    .map(|case| {
                        (
                            self.plan_values(&ValueExpr::BinaryExpr(
                                BinaryOp::Eq,
                                e.expr.clone(),
                                case.0.clone(),
                            )),
                            self.plan_values(case.1.as_ref()),
                        )
                    })
                    .collect();
                let default = match &e.default {
                    // If no `ELSE` clause is specified, use implicit `ELSE NULL` (see section 6.9, pg 142 of SQL-92 spec)
                    None => Box::new(EvalLitExpr {
                        lit: Box::new(Null),
                    }),
                    Some(def) => self.plan_values(def),
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
                            self.plan_values(case.0.as_ref()),
                            self.plan_values(case.1.as_ref()),
                        )
                    })
                    .collect();
                let default = match &e.default {
                    // If no `ELSE` clause is specified, use implicit `ELSE NULL` (see section 6.9, pg 142 of SQL-92 spec)
                    None => Box::new(EvalLitExpr {
                        lit: Box::new(Null),
                    }),
                    Some(def) => self.plan_values(def.as_ref()),
                };
                Box::new(EvalSearchedCaseExpr { cases, default })
            }
            ValueExpr::IsTypeExpr(i) => {
                let expr = self.plan_values(i.expr.as_ref());
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
                self.plan_values(&rewritten_as_case)
            }
            ValueExpr::CoalesceExpr(c) => {
                // COALESCE can be rewritten using CASE WHEN expressions as per section 6.9 pg 142 of SQL-92 spec:
                //     2) COALESCE (V1, V2) is equivalent to the following <case specification>:
                //         CASE WHEN V1 IS NOT NULL THEN V1 ELSE V2 END
                //
                //     3) COALESCE (V1, V2, . . . ,n ), for n >= 3, is equivalent to the following <case specification>:
                //         CASE WHEN V1 IS NOT NULL THEN V1 ELSE COALESCE (V2, . . . ,n )
                //         END
                assert!(!c.elements.is_empty());
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
                self.plan_values(&as_case(c.elements.first().unwrap(), &c.elements[1..]))
            }
            ValueExpr::DynamicLookup(lookups) => {
                let lookups = lookups
                    .iter()
                    .map(|lookup| self.plan_values(lookup))
                    .collect_vec();

                Box::new(EvalDynamicLookup { lookups })
            }
            ValueExpr::Call(logical::CallExpr { name, arguments }) => {
                let mut args = arguments
                    .iter()
                    .map(|arg| self.plan_values(arg))
                    .collect_vec();
                match name {
                    CallName::Lower => {
                        assert_eq!(args.len(), 1);
                        Box::new(EvalFnLower {
                            value: args.pop().unwrap(),
                        })
                    }
                    CallName::Upper => {
                        assert_eq!(args.len(), 1);
                        Box::new(EvalFnUpper {
                            value: args.pop().unwrap(),
                        })
                    }
                    CallName::CharLength => {
                        assert_eq!(args.len(), 1);
                        Box::new(EvalFnCharLength {
                            value: args.pop().unwrap(),
                        })
                    }
                    CallName::OctetLength => {
                        assert_eq!(args.len(), 1);
                        Box::new(EvalFnOctetLength {
                            value: args.pop().unwrap(),
                        })
                    }
                    CallName::BitLength => {
                        assert_eq!(args.len(), 1);
                        Box::new(EvalFnBitLength {
                            value: args.pop().unwrap(),
                        })
                    }
                    CallName::LTrim => {
                        assert_eq!(args.len(), 2);
                        let value = args.pop().unwrap();
                        let to_trim = args.pop().unwrap();
                        Box::new(EvalFnLtrim { value, to_trim })
                    }
                    CallName::BTrim => {
                        assert_eq!(args.len(), 2);
                        let value = args.pop().unwrap();
                        let to_trim = args.pop().unwrap();
                        Box::new(EvalFnBtrim { value, to_trim })
                    }
                    CallName::RTrim => {
                        assert_eq!(args.len(), 2);
                        let value = args.pop().unwrap();
                        let to_trim = args.pop().unwrap();
                        Box::new(EvalFnRtrim { value, to_trim })
                    }
                    CallName::Substring => {
                        assert!((2usize..=3).contains(&args.len()));

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
                        assert_eq!(args.len(), 2);
                        let haystack = args.pop().unwrap();
                        let needle = args.pop().unwrap();
                        Box::new(EvalFnPosition { needle, haystack })
                    }
                    CallName::Overlay => {
                        assert!((3usize..=4).contains(&args.len()));

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
                        assert_eq!(args.len(), 1);
                        Box::new(EvalFnExists {
                            value: args.pop().unwrap(),
                        })
                    }
                    CallName::Abs => {
                        assert_eq!(args.len(), 1);
                        Box::new(EvalFnAbs {
                            value: args.pop().unwrap(),
                        })
                    }
                    CallName::Mod => {
                        assert_eq!(args.len(), 2);
                        let rhs = args.pop().unwrap();
                        let lhs = args.pop().unwrap();
                        Box::new(EvalFnModulus { lhs, rhs })
                    }
                    CallName::Cardinality => {
                        assert_eq!(args.len(), 1);
                        Box::new(EvalFnCardinality {
                            value: args.pop().unwrap(),
                        })
                    }
                    CallName::ExtractYear => {
                        assert_eq!(args.len(), 1);
                        Box::new(EvalFnExtractYear {
                            value: args.pop().unwrap(),
                        })
                    }
                    CallName::ExtractMonth => {
                        assert_eq!(args.len(), 1);
                        Box::new(EvalFnExtractMonth {
                            value: args.pop().unwrap(),
                        })
                    }
                    CallName::ExtractDay => {
                        assert_eq!(args.len(), 1);
                        Box::new(EvalFnExtractDay {
                            value: args.pop().unwrap(),
                        })
                    }
                    CallName::ExtractHour => {
                        assert_eq!(args.len(), 1);
                        Box::new(EvalFnExtractHour {
                            value: args.pop().unwrap(),
                        })
                    }
                    CallName::ExtractMinute => {
                        assert_eq!(args.len(), 1);
                        Box::new(EvalFnExtractMinute {
                            value: args.pop().unwrap(),
                        })
                    }
                    CallName::ExtractSecond => {
                        assert_eq!(args.len(), 1);
                        Box::new(EvalFnExtractSecond {
                            value: args.pop().unwrap(),
                        })
                    }
                    CallName::ExtractTimezoneHour => {
                        assert_eq!(args.len(), 1);
                        Box::new(EvalFnExtractTimezoneHour {
                            value: args.pop().unwrap(),
                        })
                    }
                    CallName::ExtractTimezoneMinute => {
                        assert_eq!(args.len(), 1);
                        Box::new(EvalFnExtractTimezoneMinute {
                            value: args.pop().unwrap(),
                        })
                    }
                }
            }
        }
    }
}
