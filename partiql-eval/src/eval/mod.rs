use itertools::Itertools;
use std::any::Any;
use std::cell::RefCell;
use std::collections::HashMap;

use delegate::delegate;
use std::fmt::Debug;

use petgraph::algo::toposort;
use petgraph::dot::Dot;
use petgraph::prelude::StableGraph;
use petgraph::{Directed, Outgoing};

use partiql_value::Value;

use crate::env::basic::{MapBindings, NestedBindings};

use petgraph::graph::NodeIndex;

use crate::error::{EvalErr, EvaluationError};
use partiql_catalog::context::{Bindings, SessionContext, SystemContext};
use petgraph::visit::EdgeRef;
use unicase::UniCase;

use crate::eval::evaluable::{EvalType, Evaluable};
use crate::plan::EvaluationMode;

pub(crate) mod eval_expr_wrapper;
pub mod evaluable;
pub mod expr;

/// Represents a `PartiQL` evaluation query plan which is a plan that can be evaluated to produce
/// a result. The plan uses a directed `petgraph::StableGraph`.
#[derive(Debug)]
pub struct EvalPlan {
    mode: EvaluationMode,
    plan_graph: StableGraph<Box<dyn Evaluable>, u8, Directed>,
}

impl Default for EvalPlan {
    fn default() -> Self {
        Self::new(EvaluationMode::Permissive, Default::default())
    }
}

#[inline]
fn err_illegal_state(msg: impl AsRef<str>) -> EvalErr {
    EvalErr {
        errors: vec![EvaluationError::IllegalState(msg.as_ref().to_string())],
    }
}

impl EvalPlan {
    /// Creates a new evaluation plan.
    #[must_use]
    pub fn new(
        mode: EvaluationMode,
        plan_graph: StableGraph<Box<dyn Evaluable>, u8, Directed>,
    ) -> Self {
        EvalPlan { mode, plan_graph }
    }

    #[inline]
    fn plan_graph(&mut self) -> &mut StableGraph<Box<dyn Evaluable>, u8> {
        &mut self.plan_graph
    }

    #[inline]
    fn get_node(&mut self, idx: NodeIndex) -> Result<&mut Box<dyn Evaluable>, EvalErr> {
        self.plan_graph()
            .node_weight_mut(idx)
            .ok_or_else(|| err_illegal_state("Error in retrieving node"))
    }

    /// Executes the plan while mutating its state by changing the inputs and outputs of plan
    /// operators.
    pub fn execute_mut<'c>(&mut self, ctx: &'c dyn EvalContext<'c>) -> Result<Evaluated, EvalErr> {
        // We are only interested in DAGs that can be used as execution plans, which leads to the
        // following definition.
        // A DAG is a directed, cycle-free graph G = (V, E) with a denoted root node v0 ∈ V such
        // that all v ∈ V \{v0} are reachable from v0. Note that this is the definition of trees
        // without the condition |E| = |V | − 1. Hence, all trees are DAGs.
        // Reference: https://link.springer.com/article/10.1007/s00450-009-0061-0
        let ops = toposort(&self.plan_graph, None).map_err(|e| EvalErr {
            errors: vec![EvaluationError::InvalidEvaluationPlan(format!(
                "Malformed evaluation plan detected: {e:?}"
            ))],
        })?;

        let mut result = None;
        for idx in ops {
            let destinations: Vec<(usize, (u8, NodeIndex))> = self
                .plan_graph()
                .edges_directed(idx, Outgoing)
                .map(|e| (*e.weight(), e.target()))
                .enumerate()
                .collect_vec();

            // Some evaluables (i.e., `JOIN`) manage their own inputs
            let graph_managed = destinations.is_empty()
                || destinations.iter().any(|(_, (_, dest_idx))| {
                    matches!(
                        self.get_node(*dest_idx).map(|d| d.eval_type()),
                        Ok(EvalType::GraphManaged)
                    )
                });
            if graph_managed {
                let src = self.get_node(idx)?;
                result = Some(src.evaluate(ctx));

                // return on first evaluation error
                if ctx.has_errors() && self.mode == EvaluationMode::Strict {
                    return Err(EvalErr {
                        errors: ctx.errors(),
                    });
                }

                let num_destinations = destinations.len();
                for (i, (branch_num, dst_id)) in destinations {
                    let res = if i == num_destinations - 1 {
                        result.take()
                    } else {
                        result.clone()
                    };

                    let res =
                        res.ok_or_else(|| err_illegal_state("Error in retrieving source value"))?;
                    self.get_node(dst_id)?.update_input(res, branch_num, ctx);
                }
            }
        }

        let result = result.ok_or_else(|| err_illegal_state("Error in retrieving eval output"))?;
        Ok(Evaluated { result })
    }

    #[must_use]
    pub fn to_dot_graph(&self) -> String {
        format!("{:?}", Dot::with_config(&self.plan_graph, &[]))
    }
}

/// Represents an evaluation result that contains evaluated result or the error.
pub type EvalResult = Result<Evaluated, EvalErr>;

/// Represents result of evaluation as an evaluated entity.
#[non_exhaustive]
#[derive(Debug)]
pub struct Evaluated {
    pub result: Value,
}

/// Represents an evaluation context that is used during evaluation of a plan.
pub trait EvalContext<'a>: SessionContext<'a> + Debug {
    fn as_session(&'a self) -> &'a dyn SessionContext<'a>;

    fn add_error(&self, error: EvaluationError);
    fn has_errors(&self) -> bool;
    fn errors(&self) -> Vec<EvaluationError>;
}

#[derive(Debug)]
pub struct BasicContext<'a> {
    pub bindings: MapBindings<Value>,

    pub sys: SystemContext,
    pub user: HashMap<UniCase<String>, &'a (dyn Any)>,

    pub errors: RefCell<Vec<EvaluationError>>,
}

impl<'a> BasicContext<'a> {
    #[must_use]
    pub fn new(bindings: MapBindings<Value>, sys: SystemContext) -> Self {
        BasicContext {
            bindings,
            sys,
            user: Default::default(),
            errors: RefCell::new(vec![]),
        }
    }
}

impl<'a> SessionContext<'a> for BasicContext<'a> {
    fn bindings(&self) -> &dyn Bindings<Value> {
        &self.bindings
    }

    fn system_context(&self) -> &SystemContext {
        &self.sys
    }

    fn user_context(&self, name: &str) -> Option<&(dyn Any)> {
        let key = name.into();
        self.user.get(&key).copied()
    }
}

impl<'a> EvalContext<'a> for BasicContext<'a> {
    fn as_session(&'a self) -> &'a dyn SessionContext<'a> {
        self
    }

    fn add_error(&self, error: EvaluationError) {
        self.errors.borrow_mut().push(error);
    }

    fn has_errors(&self) -> bool {
        !self.errors.borrow().is_empty()
    }

    fn errors(&self) -> Vec<EvaluationError> {
        self.errors.take()
    }
}

#[derive(Debug)]
pub struct NestedContext<'a, 'c> {
    pub bindings: NestedBindings<'a, Value>,
    pub parent: &'a dyn EvalContext<'c>,
}

impl<'a, 'c> NestedContext<'a, 'c> {
    pub fn new(bindings: MapBindings<Value>, parent: &'a dyn EvalContext<'c>) -> Self {
        let bindings = NestedBindings::new(bindings, parent.bindings());
        NestedContext { bindings, parent }
    }
}

impl<'a, 'c> SessionContext<'a> for NestedContext<'a, 'c> {
    fn bindings(&self) -> &dyn Bindings<Value> {
        &self.bindings
    }

    delegate! {
        to self.parent {
            fn system_context(&self) -> &SystemContext;
            fn user_context(&self, name: &str) -> Option<& (dyn Any )>;
        }
    }
}

impl<'a, 'c> EvalContext<'a> for NestedContext<'a, 'c> {
    fn as_session(&'a self) -> &'a dyn SessionContext<'a> {
        self
    }

    delegate! {
        to self.parent {
            fn add_error(&self, error: EvaluationError);
            fn has_errors(&self) -> bool;
            fn errors(&self) -> Vec<EvaluationError>;
        }
    }
}
