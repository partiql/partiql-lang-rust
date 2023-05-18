use itertools::Itertools;

use std::fmt::Debug;

use petgraph::algo::toposort;
use petgraph::dot::{Config, Dot};
use petgraph::prelude::StableGraph;
use petgraph::{Directed, Outgoing};

use partiql_value::Value;

use crate::env::basic::MapBindings;
use crate::env::Bindings;

use petgraph::graph::NodeIndex;

use crate::error::{EvalErr, EvaluationError};
use petgraph::visit::EdgeRef;

use crate::eval::evaluable::Evaluable;

pub mod evaluable;
pub mod expr;

/// Represents a PartiQL evaluation query plan which is a plan that can be evaluated to produce
/// a result. The plan uses a directed `petgraph::StableGraph`.
#[derive(Debug)]
pub struct EvalPlan(pub StableGraph<Box<dyn Evaluable>, u8, Directed>);

impl Default for EvalPlan {
    fn default() -> Self {
        Self::new()
    }
}

impl EvalPlan {
    /// Creates a new evaluation plan.
    fn new() -> Self {
        EvalPlan(StableGraph::<Box<dyn Evaluable>, u8, Directed>::new())
    }

    /// Executes the plan while mutating its state by changing the inputs and outputs of plan
    /// operators.
    pub fn execute_mut(&mut self, bindings: MapBindings<Value>) -> Result<Evaluated, EvalErr> {
        let ctx: Box<dyn EvalContext> = Box::new(BasicContext { bindings });
        // We are only interested in DAGs that can be used as execution plans, which leads to the
        // following definition.
        // A DAG is a directed, cycle-free graph G = (V, E) with a denoted root node v0 ∈ V such
        // that all v ∈ V \{v0} are reachable from v0. Note that this is the definition of trees
        // without the condition |E| = |V | − 1. Hence, all trees are DAGs.
        // Reference: https://link.springer.com/article/10.1007/s00450-009-0061-0
        let sorted_ops = toposort(&self.0, None);
        match sorted_ops {
            Ok(ops) => {
                let plan_graph = &mut self.0;
                let mut result = None;
                for idx in ops.into_iter() {
                    let src = plan_graph
                        .node_weight_mut(idx)
                        .expect("Error in retrieving node");
                    result = src.evaluate(&*ctx);

                    let destinations: Vec<(usize, (u8, NodeIndex))> = plan_graph
                        .edges_directed(idx, Outgoing)
                        .map(|e| (*e.weight(), e.target()))
                        .enumerate()
                        .collect_vec();
                    let branches = destinations.len();
                    for (i, (branch_num, dst_id)) in destinations {
                        let res = if i == branches - 1 {
                            result.take()
                        } else {
                            result.clone()
                        }
                        .expect("Error in retrieving source value");

                        let dst = plan_graph
                            .node_weight_mut(dst_id)
                            .expect("Error in retrieving node");
                        dst.update_input(res, branch_num);
                    }
                }

                let result = result.expect("Error in retrieving eval output");
                Ok(Evaluated { result })
            }
            Err(e) => Err(EvalErr {
                errors: vec![EvaluationError::InvalidEvaluationPlan(format!(
                    "Malformed evaluation plan detected: {e:?}"
                ))],
            }),
        }
    }

    pub fn to_dot_graph(&self) -> String {
        format!("{:?}", Dot::with_config(&self.0, &[Config::EdgeNoLabel]))
    }
}

/// Represents an evaluation result that contains evaluated result or the error.
pub type EvalResult = Result<Evaluated, EvalErr>;

/// Represents result of evaluation as an evaluated entity.
pub struct Evaluated {
    pub result: Value,
}

/// Represents an evaluation context that is used during evaluation of a plan.
pub trait EvalContext {
    fn bindings(&self) -> &dyn Bindings<Value>;
}

#[derive(Default, Debug)]
pub struct BasicContext {
    bindings: MapBindings<Value>,
}

impl BasicContext {
    pub fn new(bindings: MapBindings<Value>) -> Self {
        BasicContext { bindings }
    }
}

impl EvalContext for BasicContext {
    fn bindings(&self) -> &dyn Bindings<Value> {
        &self.bindings
    }
}
