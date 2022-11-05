use std::cell::RefCell;
use std::collections::HashMap;
use std::rc::Rc;

use partiql_logical as logical;
use partiql_logical::{BinaryOp, BindingsExpr, LogicalPlan, PathComponent, UnaryOp, ValueExpr};

use crate::eval;
use crate::eval::{
    DagEvaluable, EvalBinOp, EvalBinOpExpr, EvalExpr, EvalLitExpr, EvalPath, EvalPlan, EvalUnaryOp,
    EvalUnaryOpExpr, EvalVarRef, Evaluable, TupleSink,
};

pub struct EvaluatorPlanner {
    // TODO remove once we agree on using evaluate output in the following PR:
    // https://github.com/partiql/partiql-lang-rust/pull/202
    pub output: Rc<RefCell<dyn TupleSink>>,
}

impl EvaluatorPlanner {
    pub fn compile(&self, be: BindingsExpr) -> Box<dyn Evaluable> {
        self.plan_eval(be)
    }

    #[inline]
    fn plan_eval(&self, be: BindingsExpr) -> Box<dyn Evaluable> {
        match be {
            BindingsExpr::From(logical::From {
                expr,
                as_key,
                at_key: _,
                out,
            }) => Box::new(eval::EvalFrom::new(
                self.plan_values(expr),
                &as_key,
                self.plan_bindings(*out),
            )),
            _ => panic!("Unevaluable bexpr"),
        }
    }

    pub fn compile_dag(&self, plan: LogicalPlan<BindingsExpr>) -> EvalPlan {
        self.plan_eval_dag(plan)
    }

    #[inline]
    fn plan_eval_dag(&self, lg: LogicalPlan<BindingsExpr>) -> EvalPlan {
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

    fn get_eval_node(&self, be: &BindingsExpr) -> Box<dyn DagEvaluable> {
        match be {
            BindingsExpr::Scan(logical::Scan {
                expr,
                as_key,
                at_key: _,
            }) => Box::new(eval::Scan::new(self.plan_values(expr.clone()), as_key)),
            BindingsExpr::Project(logical::Project { exprs }) => {
                let exprs: HashMap<_, _> = exprs
                    .into_iter()
                    .map(|(k, v)| (k.clone(), self.plan_values(v.clone())))
                    .collect();
                Box::new(eval::Project::new(exprs))
            }
            BindingsExpr::Output => Box::new(eval::Sink {
                input: None,
                output: None,
            }),
            _ => panic!("Unevaluable bexpr"),
        }
    }

    fn plan_bindings(&self, be: BindingsExpr) -> Box<dyn TupleSink> {
        match be {
            BindingsExpr::From(logical::From {
                expr,
                as_key,
                at_key: _,
                out,
            }) => Box::new(eval::EvalFrom::new(
                self.plan_values(expr),
                &as_key,
                self.plan_bindings(*out),
            )),
            BindingsExpr::Limit => todo!(),
            BindingsExpr::Offset => todo!(),
            BindingsExpr::OrderBy => todo!(),
            BindingsExpr::SetOp => todo!(),
            BindingsExpr::Select(logical::Select { exprs, out }) => {
                let exprs: HashMap<_, _> = exprs
                    .into_iter()
                    .map(|(k, v)| (k, self.plan_values(v)))
                    .collect();
                Box::new(eval::EvalSelect::new(exprs, self.plan_bindings(*out)))
            }
            BindingsExpr::Where(logical::Where { expr, out }) => Box::new(eval::EvalWhere::new(
                self.plan_values(expr),
                self.plan_bindings(*out),
            )),
            BindingsExpr::GroupBy => todo!(),
            BindingsExpr::Distinct(logical::Distinct { out }) => {
                Box::new(eval::EvalDistinct::new(self.plan_bindings(*out)))
            }
            BindingsExpr::Output => Box::new(eval::Output {
                output: self.output.clone(),
            }),
            BindingsExpr::SelectValue(_) => todo!(),
            BindingsExpr::Unpivot => todo!(),
            BindingsExpr::Join => todo!(),
            BindingsExpr::Project(_) => todo!(),
            BindingsExpr::Scan(_) => todo!(),
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
                };
                Box::new(EvalBinOpExpr { op, lhs, rhs })
            }
            ValueExpr::Lit(lit) => Box::new(EvalLitExpr { lit }),
            ValueExpr::Path(expr, components) => Box::new(EvalPath {
                expr: self.plan_values(*expr),
                components: components
                    .iter()
                    .map(|c| match c {
                        PathComponent::Key(k) => eval::PathComponent::Key(k.clone()),
                        PathComponent::Index(i) => eval::PathComponent::Index(*i),
                    })
                    .collect(),
            }),
            ValueExpr::VarRef(name) => Box::new(EvalVarRef { name }),
        }
    }
}
