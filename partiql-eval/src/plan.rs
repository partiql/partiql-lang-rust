use std::collections::HashMap;

use partiql_logical as logical;
use partiql_logical::{BinaryOp, BindingsExpr, LogicalPlan, PathComponent, UnaryOp, ValueExpr};

use crate::eval;
use crate::eval::{
    EvalBagExpr, EvalBinOp, EvalBinOpExpr, EvalExpr, EvalListExpr, EvalLitExpr, EvalPath, EvalPlan,
    EvalTupleExpr, EvalUnaryOp, EvalUnaryOpExpr, EvalVarRef, Evaluable,
};

pub struct EvaluatorPlanner;

impl EvaluatorPlanner {
    pub fn compile(&self, plan: LogicalPlan<BindingsExpr>) -> EvalPlan {
        self.plan_eval(plan)
    }

    #[inline]
    fn plan_eval(&self, lg: LogicalPlan<BindingsExpr>) -> EvalPlan {
        let mut eval_plan = EvalPlan::default();
        let ops = lg.operators();
        let flows = lg.flows();

        let mut seen = HashMap::new();

        flows.into_iter().for_each(|r| {
            let (s, d) = r;

            let mut nodes = vec![];
            for op_id in vec![s, d] {
                let logical_op = &ops[op_id.index() - 1];
                let eval_op = if let Some(op) = seen.get(op_id) {
                    *op
                } else {
                    let id = eval_plan.0.add_node(self.get_eval_node(logical_op));
                    seen.insert(op_id, id);
                    id
                };
                nodes.push(eval_op)
            }

            eval_plan.0.add_edge(nodes[0], nodes[1], ());
        });

        eval_plan
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
                    .into_iter()
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
                    BinaryOp::Lteq => EvalBinOp::Gteq,
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
        }
    }
}
