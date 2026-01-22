use thiserror::Error;

#[derive(Debug, Error)]
pub enum EngineError {
    #[error("engine not implemented")]
    NotImplemented,
    #[error("reader does not support projection: {0}")]
    ProjectionNotSupported(&'static str),
}

pub type Result<T> = std::result::Result<T, EngineError>;
