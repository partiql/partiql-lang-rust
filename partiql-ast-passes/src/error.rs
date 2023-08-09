use partiql_catalog::call_defs::CallLookupError;
use thiserror::Error;

/// Contains the errors that occur during AST transformations
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct AstTransformationError {
    pub errors: Vec<AstTransformError>,
}

/// Represents an AST transform Error
#[derive(Error, Debug, Clone, PartialEq, Eq, Hash)]
#[non_exhaustive]
pub enum AstTransformError {
    /// Indicates that AST lowering has not yet been implemented for this feature.
    #[error("Not yet implemented: {0}")]
    NotYetImplemented(String),

    /// Indicates that there is an internal error that was not due to user input or API violation.
    #[error("Illegal State: {0}")]
    IllegalState(String),

    /// Indicates that there was an error interpreting a literal value.
    #[error("Error with literal: {literal}: {error}")]
    Literal { literal: String, error: String },

    /// Invalid number of arguments to the function call.
    #[error("Invalid number of arguments: {0}")]
    InvalidNumberOfArguments(String),

    /// Indicates that this function is not supported.
    #[error("Unsupported function: {0}")]
    UnsupportedFunction(String),

    /// Indicates that this aggregation function is not supported.
    #[error("Unsupported aggregation function: {0}")]
    UnsupportedAggregationFunction(String),

    /// Any other lowering error.
    #[error("Lowering error: {0}")]
    Unknown(String),

    /// Indicates that a `HAVING` clause was provided without a `GROUP BY`
    #[error("HAVING clause provided without GROUP BY")]
    HavingWithoutGroupBy,
}

impl From<CallLookupError> for AstTransformError {
    fn from(err: CallLookupError) -> Self {
        match err {
            CallLookupError::InvalidNumberOfArguments(e) => {
                AstTransformError::InvalidNumberOfArguments(e)
            }
            e => AstTransformError::Unknown(e.to_string()),
        }
    }
}
