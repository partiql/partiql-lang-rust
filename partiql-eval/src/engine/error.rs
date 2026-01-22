use thiserror::Error;

#[derive(Debug, Error)]
pub enum EngineError {
    #[error("engine not implemented")]
    NotImplemented,
}

pub type Result<T> = std::result::Result<T, EngineError>;
