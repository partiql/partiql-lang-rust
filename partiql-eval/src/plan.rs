use petgraph::prelude::StableGraph;
use std::collections::HashMap;

use partiql_logical as logical;
use partiql_logical::{
    BinaryOp, BindingsExpr, JoinKind, LogicalPlan, OpId, PathComponent, UnaryOp, ValueExpr,
};

use crate::eval;
use crate::eval::{
    EvalBagExpr, EvalBetweenExpr, EvalBinOp, EvalBinOpExpr, EvalExpr, EvalIsTypeExpr, EvalJoinKind,
    EvalListExpr, EvalLitExpr, EvalPath, EvalPlan, EvalSearchedCaseExpr, EvalTupleExpr,
    EvalUnaryOp, EvalUnaryOpExpr, EvalVarRef, Evaluable,
};
use partiql_value::Value::Null;

pub struct EvaluatorPlanner;

impl EvaluatorPlanner {
    pub fn compile(&self, plan: &LogicalPlan<BindingsExpr>) -> EvalPlan {
        self.plan_eval(plan)
    }

    #[inline]
    fn plan_eval(&self, lg: &LogicalPlan<BindingsExpr>) -> EvalPlan {
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

    fn get_eval_node(&self, be: &BindingsExpr) -> Box<dyn Evaluable> {
        match be {
            BindingsExpr::Scan(logical::Scan {
                expr,
                as_key,
                at_key,
            }) => {
                if let Some(at_key) = at_key {
                    Box::new(eval::EvalScan::new_with_at_key(
                        self.plan_values(expr.clone()),
                        as_key,
                        at_key,
                    ))
                } else {
                    Box::new(eval::EvalScan::new(self.plan_values(expr.clone()), as_key))
                }
            }
            BindingsExpr::Project(logical::Project { exprs }) => {
                let exprs: HashMap<_, _> = exprs
                    .iter()
                    .map(|(k, v)| (k.clone(), self.plan_values(v.clone())))
                    .collect();
                Box::new(eval::EvalProject::new(exprs))
            }
            BindingsExpr::ProjectValue(logical::ProjectValue { expr }) => {
                let expr = self.plan_values(expr.clone());
                Box::new(eval::EvalProjectValue::new(expr))
            }
            BindingsExpr::Filter(logical::Filter { expr }) => Box::new(eval::EvalFilter {
                expr: self.plan_values(expr.clone()),
                input: None,
                output: None,
            }),
            BindingsExpr::Distinct => Box::new(eval::EvalDistinct::new()),
            BindingsExpr::Sink => Box::new(eval::EvalSink {
                input: None,
                output: None,
            }),
            BindingsExpr::Unpivot(logical::Unpivot {
                expr,
                as_key,
                at_key,
            }) => Box::new(eval::EvalUnpivot::new(
                self.plan_values(expr.clone()),
                as_key,
                at_key.as_ref().unwrap(),
            )),
            BindingsExpr::Join(logical::Join { kind, on }) => {
                let kind = match kind {
                    JoinKind::Inner => EvalJoinKind::Inner,
                    JoinKind::Left => EvalJoinKind::Left,
                    JoinKind::Right => EvalJoinKind::Right,
                    JoinKind::Full => EvalJoinKind::Full,
                    JoinKind::Cross => EvalJoinKind::Cross,
                };
                let on = on
                    .as_ref()
                    .map(|on_condition| self.plan_values(on_condition.clone()));
                Box::new(eval::EvalJoin::new(kind, on))
            }
            _ => panic!("Unevaluable bexpr"),
        }
    }

    fn plan_values(&self, ve: ValueExpr) -> Box<dyn EvalExpr> {
        match ve {
            ValueExpr::UnExpr(unary_op, operand) => {
                let operand = self.plan_values(*operand);
                let op = match unary_op {
                    UnaryOp::Pos => EvalUnaryOp::Pos,
                    UnaryOp::Neg => EvalUnaryOp::Neg,
                    UnaryOp::Not => EvalUnaryOp::Not,
                };
                Box::new(EvalUnaryOpExpr { op, operand })
            }
            ValueExpr::BinaryExpr(binop, lhs, rhs) => {
                let lhs = self.plan_values(*lhs);
                let rhs = self.plan_values(*rhs);
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
                };
                Box::new(EvalBinOpExpr { op, lhs, rhs })
            }
            ValueExpr::Lit(lit) => Box::new(EvalLitExpr { lit }),
            ValueExpr::Path(expr, components) => Box::new(EvalPath {
                expr: self.plan_values(*expr),
                components: components
                    .iter()
                    .map(|c| match c {
                        PathComponent::Key(k) => eval::EvalPathComponent::Key(k.clone()),
                        PathComponent::Index(i) => eval::EvalPathComponent::Index(*i),
                    })
                    .collect(),
            }),
            ValueExpr::VarRef(name) => Box::new(EvalVarRef { name }),
            ValueExpr::TupleExpr(expr) => {
                let attrs: Vec<Box<dyn EvalExpr>> = expr
                    .attrs
                    .into_iter()
                    .map(|attr| self.plan_values(attr))
                    .collect();
                let vals: Vec<Box<dyn EvalExpr>> = expr
                    .values
                    .into_iter()
                    .map(|attr| self.plan_values(attr))
                    .collect();
                Box::new(EvalTupleExpr { attrs, vals })
            }
            ValueExpr::ListExpr(expr) => {
                let elements: Vec<Box<dyn EvalExpr>> = expr
                    .elements
                    .into_iter()
                    .map(|elem| self.plan_values(elem))
                    .collect();
                Box::new(EvalListExpr { elements })
            }
            ValueExpr::BagExpr(expr) => {
                let elements: Vec<Box<dyn EvalExpr>> = expr
                    .elements
                    .into_iter()
                    .map(|elem| self.plan_values(elem))
                    .collect();
                Box::new(EvalBagExpr { elements })
            }
            ValueExpr::BetweenExpr(expr) => {
                let value = self.plan_values(*expr.value);
                let from = self.plan_values(*expr.from);
                let to = self.plan_values(*expr.to);
                Box::new(EvalBetweenExpr { value, from, to })
            }
            ValueExpr::SimpleCase(e) => {
                let cases = e
                    .cases
                    .into_iter()
                    .map(|case| {
                        (
                            self.plan_values(ValueExpr::BinaryExpr(
                                BinaryOp::Eq,
                                e.expr.clone(),
                                case.0,
                            )),
                            self.plan_values(*case.1),
                        )
                    })
                    .collect();
                let default = match e.default {
                    // If no `ELSE` clause is specified, use implicit `ELSE NULL` (see section 6.9, pg 142 of SQL-92 spec)
                    None => Box::new(EvalLitExpr {
                        lit: Box::new(Null),
                    }),
                    Some(def) => self.plan_values(*def),
                };
                // Here, rewrite `SimpleCaseExpr`s as `SearchedCaseExpr`s
                Box::new(EvalSearchedCaseExpr { cases, default })
            }
            ValueExpr::SearchedCase(e) => {
                let cases = e
                    .cases
                    .into_iter()
                    .map(|case| (self.plan_values(*case.0), self.plan_values(*case.1)))
                    .collect();
                let default = match e.default {
                    // If no `ELSE` clause is specified, use implicit `ELSE NULL` (see section 6.9, pg 142 of SQL-92 spec)
                    None => Box::new(EvalLitExpr {
                        lit: Box::new(Null),
                    }),
                    Some(def) => self.plan_values(*def),
                };
                Box::new(EvalSearchedCaseExpr { cases, default })
            }
            ValueExpr::IsTypeExpr(i) => {
                let expr = self.plan_values(*i.expr);
                match i.not {
                    true => Box::new(EvalUnaryOpExpr {
                        op: EvalUnaryOp::Not,
                        operand: Box::new(EvalIsTypeExpr {
                            expr,
                            is_type: i.is_type,
                        }),
                    }),
                    false => Box::new(EvalIsTypeExpr {
                        expr,
                        is_type: i.is_type,
                    }),
                }
            }
        }
    }
}
