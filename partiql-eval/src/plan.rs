use std::cell::RefCell;
use std::collections::HashMap;
use std::rc::Rc;

use partiql_logical as logical;
use partiql_logical::{BinaryOp, BindingsExpr, LogicalPlan, PathComponent, ValueExpr};

use crate::eval;
use crate::eval::{
    DagEvaluable, EvalBinop, EvalBinopExpr, EvalExpr, EvalLitExpr, EvalPath, EvalPlan, EvalVarRef,
    Evaluable, TupleSink,
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

    pub fn compile_dag(&self, plan: LogicalPlan) -> EvalPlan {
        self.plan_eval_dag(plan)
    }

    #[inline]
    fn plan_eval_dag(&self, lg: LogicalPlan) -> EvalPlan {
        let plan = lg.0;
        let mut eval_plan = EvalPlan::default();
        eval_plan.0 = plan.map(|_, n| self.get_eval_node(n), |_, e| e.clone());

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
                Box::new(EvalBinopExpr { op, lhs, rhs })
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
