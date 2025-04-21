use itertools::Itertools;
use std::any::Any;
use std::borrow::Cow;
use std::cell::RefCell;
use std::collections::HashMap;

use delegate::delegate;
use petgraph::algo::toposort;
use petgraph::dot::Dot;
use petgraph::prelude::StableGraph;
use petgraph::{Directed, Incoming, Outgoing};
use std::fmt::Debug;

use partiql_value::{BindingsName, Value};

use crate::env::basic::MapBindings;

use petgraph::graph::NodeIndex;

use crate::error::{EvalErr, EvaluationError};
use partiql_catalog::context::{Bindings, SessionContext, SystemContext};
use petgraph::visit::EdgeRef;
use unicase::UniCase;

use crate::eval::evaluable::{EvalType, Evaluable};
use crate::plan::EvaluationMode;

#[cfg(feature = "serde")]
use serde::{Deserialize, Serialize};

pub(crate) mod eval_expr_wrapper;
pub mod evaluable;
pub mod expr;
pub mod graph;

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
    fn plan_graph(&self) -> &StableGraph<Box<dyn Evaluable>, u8> {
        &self.plan_graph
    }

    #[inline]
    fn get_node(&self, idx: NodeIndex) -> Result<&Box<dyn Evaluable>, EvalErr> {
        self.plan_graph()
            .node_weight(idx)
            .ok_or_else(|| err_illegal_state("Error in retrieving node"))
    }

    /// Executes the plan while mutating its state by changing the inputs and outputs of plan
    /// operators.
    pub fn execute<'c>(&self, ctx: &'c dyn EvalContext<'c>) -> Result<Evaluated, EvalErr> {
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
        let mut inputs: HashMap<NodeIndex, [Option<Value>; 2]> = HashMap::new();

        // Set source node inputs to empty
        for idx in ops.clone() {
            let source_node = self.plan_graph.edges_directed(idx, Incoming).count() == 0;
            let managed = self
                .get_node(idx)
                .map(|d| d.eval_type() != EvalType::GraphManaged)
                .unwrap_or(false);
            if source_node || managed {
                inputs.insert(idx, [None, None]);
            }
        }

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
                let input = inputs
                    .remove(&idx)
                    .ok_or_else(|| err_illegal_state("Error in retrieving node input"))?;
                result = Some(src.evaluate(input, ctx));

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
                    let inputs = inputs.entry(dst_id).or_insert_with(|| [None, None]);
                    inputs[branch_num as usize] = Some(res);
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
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
pub struct Evaluated {
    pub result: Value,
}

/// Represents an evaluation context that is used during evaluation of a plan.
pub trait EvalContext<'a>: Bindings<'a, Value> + SessionContext<'a> + Debug {
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

impl BasicContext<'_> {
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
    fn system_context(&self) -> &SystemContext {
        &self.sys
    }

    fn user_context(&self, name: &str) -> Option<&(dyn Any)> {
        let key = name.into();
        self.user.get(&key).copied()
    }
}

impl<'a> Bindings<'a, Value> for BasicContext<'a> {
    fn get(&'a self, name: &BindingsName<'_>) -> Option<Cow<'a, Value>> {
        self.bindings.get(name)
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
pub struct NestedContext<'c> {
    pub bindings: MapBindings<Value>,
    pub parent: &'c dyn EvalContext<'c>,
}

impl<'c> NestedContext<'c> {
    pub fn new(bindings: MapBindings<Value>, parent: &'c dyn EvalContext<'c>) -> Self {
        NestedContext { bindings, parent }
    }
}

impl<'c> SessionContext<'c> for NestedContext<'c> {
    delegate! {
        to self.parent {
            fn system_context(&self) -> &SystemContext;
            fn user_context(&self, name: &str) -> Option<& (dyn Any )>;
        }
    }
}

impl<'a> Bindings<'a, Value> for NestedContext<'_> {
    fn get(&'a self, name: &BindingsName<'_>) -> Option<Cow<'a, Value>> {
        match self.bindings.get(name) {
            Some(v) => Some(v),
            None => self.parent.get(name),
        }
    }
}

impl<'c> EvalContext<'c> for NestedContext<'c> {
    fn as_session(&'c self) -> &'c dyn SessionContext<'c> {
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
