use crate::eval::evaluable::Evaluable;
use crate::eval::expr::EvalExpr;
use crate::eval::EvalContext;
use partiql_catalog::extension::ExtensionResultError;
use partiql_value::datum::RefTupleView;
use partiql_value::Value;
use std::borrow::Cow;
use thiserror::Error;

/// All errors that occurred during [`partiql_logical::LogicalPlan`] to [`eval::EvalPlan`] creation.
#[derive(Debug)]
pub struct PlanErr {
    pub errors: Vec<PlanningError>,
}

/// An error that can happen during [`partiql_logical::LogicalPlan`] to [`eval::EvalPlan`] creation.
#[derive(Error, Debug, Clone, PartialEq, Eq, Hash)]
#[non_exhaustive]
pub enum PlanningError {
    /// Feature has not yet been implemented.
    #[error("Not yet implemented: {0}")]
    NotYetImplemented(String),
    /// Internal error that was not due to user input or API violation.
    #[error("Illegal State: {0}")]
    IllegalState(String),
}

/// All errors that occurred during evaluation.
#[derive(Debug)]
pub struct EvalErr {
    pub errors: Vec<EvaluationError>,
}

/// An error that can happen during evaluation.
#[derive(Error, Debug)]
#[non_exhaustive]
pub enum EvaluationError {
    /// Internal error that was not due to user input or API violation.
    #[error("Illegal State: {0}")]
    IllegalState(String),
    /// Malformed evaluation plan with graph containing cycle.
    #[error("Evaluation Error: invalid evaluation plan detected `{0}`")]
    InvalidEvaluationPlan(String),
    /// Feature has not yet been implemented.
    #[error("Not yet implemented: {0}")]
    NotYetImplemented(String),

    /// Error originating in an extension
    #[error("Extension Result Expression Error")]
    ExtensionResultError(#[from] ExtensionResultError),
}

/// Used when an error occurs during the logical to eval plan conversion. Allows the conversion
/// to continue in order to report multiple errors.
#[derive(Debug)]
pub(crate) struct ErrorNode {}

impl ErrorNode {
    pub(crate) fn new() -> ErrorNode {
        ErrorNode {}
    }
}

impl Evaluable for ErrorNode {
    fn evaluate(&self, _: [Option<Value>; 2], _ctx: &dyn EvalContext) -> Value {
        panic!("ErrorNode will not be evaluated")
    }
}

impl EvalExpr for ErrorNode {
    fn evaluate<'a, 'c, 'o>(
        &'a self,
        _bindings: &'a dyn RefTupleView<'a, Value>,
        _ctx: &'c dyn EvalContext,
    ) -> Cow<'o, Value>
    where
        'c: 'a,
        'a: 'o,
    {
        panic!("ErrorNode will not be evaluated")
    }
}
