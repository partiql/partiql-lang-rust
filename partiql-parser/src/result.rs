// Copyright Amazon.com, Inc. or its affiliates.

//! [`Error`] and [`Result`] types for parsing PartiQL.

use crate::location::{ByteOffset, Position};

use crate::lalr::{Spanned, Token};
use crate::LexError;
use thiserror::Error;

/// General [`Result`] type for the PartiQL parser.
pub type ParserResult<'input, T> = Result<T, ParserError<'input>>;

/// Errors from the PartiQL parser.
#[derive(Error, Debug, Clone, PartialEq, Eq, Hash)]
pub enum ParserError<'input> {
    /// Indicates that there was a problem with syntax.
    #[error("Syntax Error: {message} ({position})")]
    SyntaxError { message: String, position: Position },

    /// There was a token that was ot expected
    // TODO how to report location in errors
    #[error("Unexpected token [{:?}] at [TODO]", token.1)]
    UnexpectedToken {
        /// The unexpected token of type `T` with a span given by the two `L` values.
        token: Spanned<Token<'input>, ByteOffset>,
        // TODO expected: ...,
    },

    #[error("{cause} ({position})")]
    LexicalError { cause: LexError, position: Position },

    /// Indicates that there is an internal error that was not due to user input or API violation.
    #[error("Illegal State: {message}")]
    IllegalState { message: String },
}

#[cfg(test)]
mod tests {}
