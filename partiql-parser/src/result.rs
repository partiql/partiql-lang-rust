// Copyright Amazon.com, Inc. or its affiliates.

//! [`Error`] and [`Result`] types for parsing PartiQL.

use crate::location::Position;

use crate::LexicalError;
use thiserror::Error;

/// General [`Result`] type for the PartiQL parser.
pub type ParserResult<T> = Result<T, ParserError>;

/// Errors from the PartiQL parser.
#[derive(Error, Debug, Clone, PartialEq, Eq, Hash)]
pub enum ParserError {
    /// Indicates that there was a problem with syntax.
    #[error("Syntax Error: {message} ({position})")]
    SyntaxError { message: String, position: Position },

    #[error("{cause} ({position})")]
    LexicalError {
        cause: LexicalError,
        position: Position,
    },

    /// Indicates that there is an internal error that was not due to user input or API violation.
    #[error("Illegal State: {message}")]
    IllegalState { message: String },
}

#[cfg(test)]
mod tests {}
