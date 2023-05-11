use thiserror::Error;

#[derive(Error, Debug, Clone, PartialEq, Eq, Hash)]
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
#[non_exhaustive]
pub enum LowerError {
    /// Indicates that AST lowering has not yet been implemented for this feature.
    #[error("Not yet implemented: {0}")]
    NotYetImplemented(String),
    /// Indicates that there is an internal error that was not due to user input or API violation.
    #[error("Illegal State: {0}")]
    IllegalState(String),
    /// Indicates that this function is not supported.
    #[error("Unsupported function: {0}")]
    UnsupportedFunction(String),
    /// Indicates that this aggregation function is not supported.
    #[error("Unsupported aggregation function: {0}")]
    UnsupportedAggregationFunction(String),
}
