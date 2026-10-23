use thiserror::Error;

#[derive(Debug, Error)]
pub enum EngineError {
    #[error("engine not implemented")]
    NotImplemented,
    #[error("illegal state: {0}")]
    IllegalState(String),
    #[error("reader does not support projection: {0}")]
    ProjectionNotSupported(&'static str),
    #[error("type error: {0}")]
    TypeError(String),
    #[error("unsupported expression: {0}")]
    UnsupportedExpr(String),
    #[error("udf not found: {0}")]
    UdfNotFound(String),
    #[error("invalid plan: {0}")]
    InvalidPlan(String),
    #[error("reader error: {0}")]
    ReaderError(String),
    #[error("slot index out of bounds: {0}")]
    SlotOutOfBounds(u16),
    #[error("strict mode violation: {0}")]
    StrictModeViolation(String),
    #[error("invalid Ion literal: {0}")]
    InvalidIon(String),
    #[error("arithmetic error: {0}")]
    Arithmetic(String),
    /// The query needs more of something (registers, constants, instructions, ...) than
    /// the bytecode can address.
    #[error("query too large: too many {0}")]
    TooLarge(&'static str),
}

pub type Result<T> = std::result::Result<T, EngineError>;

/// `n` as a `u16` index or count of `what`, or [`EngineError::TooLarge`].
pub(crate) fn count_u16(n: usize, what: &'static str) -> Result<u16> {
    u16::try_from(n).map_err(|_| EngineError::TooLarge(what))
}
