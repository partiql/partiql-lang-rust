use thiserror::Error;

/// Contains the errors that occur during AST to logical plan conversion
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct LoweringError {
    pub errors: Vec<LowerError>,
}

/// An error that can happen during the AST to logical plan conversion
#[derive(Error, Debug, Clone, PartialEq, Eq, Hash)]
#[non_exhaustive]
pub enum LowerError {
    /// Indicates that AST lowering has not yet been implemented for this feature.
    #[error("Not yet implemented: {0}")]
    NotYetImplemented(String),

    /// Indicates that there is an internal error that was not due to user input or API violation.
    #[error("Illegal State: {0}")]
    IllegalState(String),

    /// Indicates that there was an error interpreting a literal value.
    #[error("Error with literal: {literal}: {error}")]
    Literal { literal: String, error: String },

    /// Indicates that this function is not supported.
    #[error("Unsupported function: {0}")]
    UnsupportedFunction(String),

    /// Indicates that this aggregation function is not supported.
    #[error("Unsupported aggregation function: {0}")]
    UnsupportedAggregationFunction(String),
}
