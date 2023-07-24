pub(crate) mod eval_wrapper;

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
mod operators;
pub(crate) use operators::*;

use crate::eval::EvalContext;

use partiql_value::{Tuple, Value};
use std::borrow::Cow;
use std::fmt::Debug;
use thiserror::Error;

/// A trait for expressions that require evaluation, e.g. `a + b` or `c > 2`.
pub trait EvalExpr: Debug {
    fn evaluate<'a>(&'a self, bindings: &'a Tuple, ctx: &'a dyn EvalContext) -> Cow<'a, Value>;
}

#[derive(Error, Debug, Clone, PartialEq)]
#[non_exhaustive]
/// An error in binding an expression for evaluation
pub enum BindError {
    #[error("Argument number mismatch: expected `{expected}`, found `{found}` ")]
    ArgNumMismatch { expected: usize, found: usize },

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
        &self,
        args: Vec<Box<dyn EvalExpr>>,
    ) -> Result<Box<dyn EvalExpr>, BindError>;
}
