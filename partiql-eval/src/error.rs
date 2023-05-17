use crate::eval::evaluable::Evaluable;
use crate::eval::expr::EvalExpr;
use crate::eval::EvalContext;
use partiql_value::{Tuple, Value};
use std::borrow::Cow;
use thiserror::Error;

/// All errors that occur during evaluation.
#[derive(Debug)]
pub struct EvalErr {
    pub errors: Vec<EvaluationError>,
}

/// An error that can happen during evaluation.
/// TODO: decide on splitting out logical to eval plan to separate error enum
#[derive(Error, Debug, Clone, PartialEq, Eq, Hash)]
#[non_exhaustive]
pub enum EvaluationError {
    /// Malformed evaluation plan with graph containing cycle.
    #[error("Evaluation Error: invalid evaluation plan detected `{0}`")]
    InvalidEvaluationPlan(String),
    /// Feature has not yet been implemented.
    #[error("Not yet implemented: {0}")]
    NotYetImplemented(String),
    /// Internal error that was not due to user input or API violation.
    #[error("Illegal State: {0}")]
    IllegalState(String),
    /// Invalid number of arguments to the function call.
    #[error("Invalid number of arguments: {0}")]
    InvalidNumberOfArguments(String),
    /// Invalid escape pattern.
    #[error("Invalid LIKE expression pattern: {0}")]
    InvalidLikeEscape(String),
}

/// Used when an error occurs during the the logical to eval plan conversion. Allows the conversion
/// to continue in order to report multiple errors.
#[derive(Debug)]
pub(crate) struct ErrorNode {}

impl ErrorNode {
    pub(crate) fn new() -> ErrorNode {
        ErrorNode {}
    }
}

impl Evaluable for ErrorNode {
    fn evaluate(&mut self, _ctx: &dyn EvalContext) -> Option<Value> {
        panic!("ErrorNode will not be evaluated")
    }

    fn update_input(&mut self, _input: Value, _branch_num: u8) {
        panic!("ErrorNode will not be evaluated")
    }
}

impl EvalExpr for ErrorNode {
    fn evaluate<'a>(&'a self, _bindings: &'a Tuple, _ctx: &'a dyn EvalContext) -> Cow<'a, Value> {
        panic!("ErrorNode will not be evaluated")
    }
}
