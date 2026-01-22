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
}

pub type Result<T> = std::result::Result<T, EngineError>;
