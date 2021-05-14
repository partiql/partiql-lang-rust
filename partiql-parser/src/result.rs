// Copyright Amazon.com, Inc. or its affiliates.

//! [`Error`] and [`Result`] types for parsing PartiQL.

use pest::error::{ErrorVariant, LineColLocation};
use std::fmt;
use thiserror::Error;

/// Position in the source for an error.
#[derive(Copy, Clone, Debug, Eq, PartialEq)]
pub enum Position {
    Unknown,
    At { line: usize, column: usize },
}

impl fmt::Display for Position {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Position::Unknown => write!(f, "unknown position"),
            Position::At { line, column } => write!(f, "line {}, column {}", *line, *column),
        }
    }
}

/// Errors from the PartiQL parser.
#[derive(Clone, Debug, Eq, PartialEq, Error)]
pub enum ParserError {
    #[error("Syntax Error: {message} ({position})")]
    SyntaxError { message: String, position: Position },
}

impl ParserError {
    /// Convenience function to create a [SyntaxError](ParserError::SyntaxError).
    pub fn syntax_error<S: Into<String>>(message: S, position: Position) -> Self {
        Self::SyntaxError {
            message: message.into(),
            position,
        }
    }
}

impl<R> From<pest::error::Error<R>> for ParserError
where
    R: fmt::Debug,
{
    fn from(error: pest::error::Error<R>) -> Self {
        // obtain the line/column information from the Pest error
        let (line, column) = match error.line_col {
            LineColLocation::Pos((line, column)) => (line, column),
            LineColLocation::Span((line, column), _) => (line, column),
        };
        let message = match error.variant {
            // TODO extract a better error message
            ErrorVariant::ParsingError { positives, .. } => format!("Expected {:?}", positives),
            ErrorVariant::CustomError { message } => message,
        };
        Self::syntax_error(message, Position::At { line, column })
    }
}

/// General [`Result`] type for the PartiQL parser.
pub type ParserResult<T> = Result<T, ParserError>;

#[cfg(test)]
mod tests {
    use super::*;
    use rstest::*;

    #[rstest]
    #[case::syntax_error_with_pos(
        ParserError::syntax_error("Boo", Position::At { line: 12, column: 3 }),
        "Syntax Error: Boo (line 12, column 3)"
    )]
    #[case::syntax_error_no_pos(
        ParserError::syntax_error("Moo", Position::Unknown),
        "Syntax Error: Moo (unknown position)"
    )]
    fn display(#[case] error: ParserError, #[case] expected: &str) {
        let message = format!("{}", error);
        assert_eq!(expected, message);
    }
}
