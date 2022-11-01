use std::collections::HashMap;

use partiql_logical as logical;
use partiql_logical::{BinaryOp, BindingsExpr, LogicalPlan, PathComponent, ValueExpr};

use crate::eval;
use crate::eval::{
    EvalBinOpExpr, EvalBinop, EvalExpr, EvalLitExpr, EvalPath, EvalPlan, EvalVarRef, Evaluable,
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
            }) => Box::new(eval::EvalScan::new(
                self.plan_values(expr.clone()),
                as_key,
                at_key,
            )),
            BindingsExpr::Project(logical::Project { exprs }) => {
                let exprs: HashMap<_, _> = exprs
                    .into_iter()
                    .map(|(k, v)| (k.clone(), self.plan_values(v.clone())))
                    .collect();
                Box::new(eval::EvalProject::new(exprs))
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
            ValueExpr::UnExpr(_, _) => todo!("{:?}", ve),
            ValueExpr::BinaryExpr(binop, lhs, rhs) => {
                let lhs = self.plan_values(*lhs);
                let rhs = self.plan_values(*rhs);
                let op = match binop {
                    BinaryOp::And => EvalBinop::And,
                    BinaryOp::Or => EvalBinop::Or,
                    BinaryOp::Concat => EvalBinop::Concat,
                    BinaryOp::Eq => EvalBinop::Eq,
                    BinaryOp::Neq => EvalBinop::Neq,
                    BinaryOp::Gt => EvalBinop::Gt,
                    BinaryOp::Gteq => EvalBinop::Gteq,
                    BinaryOp::Lt => EvalBinop::Lt,
                    BinaryOp::Lteq => EvalBinop::Gteq,
                    BinaryOp::Add => EvalBinop::Add,
                    BinaryOp::Sub => EvalBinop::Sub,
                    BinaryOp::Mul => EvalBinop::Mul,
                    BinaryOp::Div => EvalBinop::Div,
                    BinaryOp::Mod => EvalBinop::Mod,
                    BinaryOp::Exp => EvalBinop::Exp,
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
        }
    }
}
