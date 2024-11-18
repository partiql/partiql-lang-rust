mod base_table;
pub(crate) use base_table::*;
mod coll;
pub(crate) use coll::*;
mod control_flow;
pub(crate) use control_flow::*;
mod data_types;
pub(crate) use data_types::*;
mod datetime;
pub(crate) use datetime::*;
mod strings;
pub(crate) use strings::*;
mod path;
pub(crate) use path::*;
mod pattern_match;
pub(crate) use pattern_match::*;
mod functions;
mod operators;

pub(crate) use operators::*;

use crate::eval::EvalContext;

use partiql_value::{Tuple, Value};
use std::borrow::Cow;
use std::fmt::Debug;
use thiserror::Error;

/// A trait for expressions that require evaluation, e.g. `a + b` or `c > 2`.
pub trait EvalExpr: Debug {
    fn evaluate<'a, 'c>(
        &'a self,
        bindings: &'a Tuple,
        ctx: &'c dyn EvalContext<'c>,
    ) -> Cow<'a, Value>
    where
        'c: 'a;

    fn evaluate_owned<'a, 'c>(&'a self, bindings: Tuple, ctx: &'c dyn EvalContext<'c>) -> Value
    where
        'c: 'a,
    {
        self.evaluate(&bindings, ctx).into_owned()
    }
}

#[derive(Error, Debug, Clone, PartialEq)]
#[non_exhaustive]
/// An error in binding an expression for evaluation
pub enum BindError {
    #[error("Argument number mismatch: expected one of `{expected:?}`, found `{found}` ")]
    ArgNumMismatch { expected: Vec<usize>, found: usize },

    /// Feature has not yet been implemented.
    #[error("Argument constraint not satisfied: `{0}`")]
    ArgumentConstraint(String),

    /// Feature has not yet been implemented.
    #[error("Not yet implemented: {0}")]
    NotYetImplemented(String),

    /// Any other error.
    #[error("Bind error: unknown error")]
    Unknown,
}

/// A trait for binding an expression to its arguments into an `EvalExpr`
pub trait BindEvalExpr: Debug {
    fn bind<const STRICT: bool>(
        self,
        args: Vec<Box<dyn EvalExpr>>,
    ) -> Result<Box<dyn EvalExpr>, BindError>;
}
