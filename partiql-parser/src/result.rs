// Copyright Amazon.com, Inc. or its affiliates.

//! [`Error`] and [`Result`] types for parsing PartiQL.

use pest::error::{ErrorVariant, LineColLocation};
use std::fmt;
use std::fmt::Formatter;
use thiserror::Error;

/// A line and column location.
///
/// This value is one-based, as that is how most people think of lines and columns.
///
/// ## Example
/// ```
/// # use partiql_parser::prelude::*;
/// println!("Beginning of a document: {}", LineAndColumn::at(1, 1));
/// ```
#[derive(Copy, Clone, Debug, Eq, PartialEq)]
pub struct LineAndColumn(pub usize, pub usize);

impl LineAndColumn {
    /// Constructs a [`LineAndColumn`].
    ///
    /// Note that this function will panic if `line` or `column` is zero.
    #[inline]
    pub fn at(line: usize, column: usize) -> Self {
        assert_ne!(0, line);
        assert_ne!(0, column);
        Self(line, column)
    }

    /// Returns a [`LineAndColumn`] that repositions this position relative
    /// to the given one one as a sort of "origin."
    ///
    /// Note that this positioning is 1-based, so repositioning `(1, 1)` from `(1, 1)` is a no-op.
    ///
    /// ## Examples
    /// ```
    /// # use partiql_parser::prelude::*;
    /// // we're not repositioning anything!
    /// assert_eq!(
    ///     LineAndColumn::at(1, 1),
    ///     LineAndColumn::at(1, 1).position_from(LineAndColumn::at(1, 1))
    /// );
    /// ```
    ///
    /// ```
    /// # use partiql_parser::prelude::*;
    /// // same here, we're really at the origin
    /// assert_eq!(
    ///     LineAndColumn::at(1, 2),
    ///     LineAndColumn::at(1, 2).position_from(LineAndColumn::at(1, 1))
    /// );
    /// ```
    ///
    /// ```
    /// # use partiql_parser::prelude::*;
    /// // same line from origin, adjust only the column
    /// assert_eq!(
    ///     LineAndColumn::at(5, 10),
    ///     LineAndColumn::at(1, 4).position_from(LineAndColumn::at(5, 7))
    /// );
    /// ```
    ///
    /// ```
    /// # use partiql_parser::prelude::*;
    /// // we're moving lines, adjust the line and take the target column as-is
    /// assert_eq!(
    ///     LineAndColumn::at(21, 2),
    ///     LineAndColumn::at(20, 2).position_from(LineAndColumn::at(2, 15))
    /// );
    /// ```
    pub fn position_from(self, location: LineAndColumn) -> Self {
        match (location, self) {
            (LineAndColumn(base_line, base_column), LineAndColumn(dest_line, dest_column)) => {
                let diff_line = dest_line - 1;
                if diff_line > 0 {
                    // we're moving lines, adjust the line and take the target column as-is
                    LineAndColumn::at(base_line + diff_line, dest_column)
                } else {
                    // same line from base, adjust only the column
                    let diff_column = dest_column - 1;
                    LineAndColumn::at(base_line, base_column + diff_column)
                }
            }
        }
    }
}

impl From<(usize, usize)> for LineAndColumn {
    /// Constructs a [`LineAndColumn`] from a pair.
    ///
    /// This function will panic if the `line` or `column` is zero.
    fn from(line_and_column: (usize, usize)) -> Self {
        let (line, column) = line_and_column;
        Self::at(line, column)
    }
}

impl fmt::Display for LineAndColumn {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        write!(f, "line {}, column {}", self.0, self.1)
    }
}

/// A possible position in the source.
#[derive(Copy, Clone, Debug, Eq, PartialEq)]
pub enum Position {
    /// Variant indicating that there *is no* known location in source for some context.
    Unknown,
    /// Variant indicating that there *is* a known location in source for some context.
    At(LineAndColumn),
}

impl Position {
    /// Shorthand for creating a [`Position::At`] variant.
    ///
    /// Note that this will panic if `line` or `column` is zero.
    #[inline]
    pub fn at(line: usize, column: usize) -> Self {
        Self::At(LineAndColumn::at(line, column))
    }
}

impl From<LineAndColumn> for Position {
    fn from(line_column: LineAndColumn) -> Self {
        Self::At(line_column)
    }
}

impl fmt::Display for Position {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Position::Unknown => write!(f, "unknown position"),
            Position::At(location) => {
                write!(f, "{}", location)
            }
        }
    }
}

/// Errors from the PartiQL parser.
#[derive(Clone, Debug, Eq, PartialEq, Error)]
pub enum ParserError {
    /// Indicates that there was a problem with syntax.
    #[error("Syntax Error: {message} ({position})")]
    SyntaxError { message: String, position: Position },
}

impl ParserError {
    /// Convenience function to create a [`SyntaxError`](ParserError::SyntaxError).
    #[inline]
    pub fn syntax_error<S: Into<String>>(message: S, position: Position) -> Self {
        Self::SyntaxError {
            message: message.into(),
            position,
        }
    }
}

/// Convenience function to create a `Err([SyntaxError](ParserError::SyntaxError))`.
#[inline]
pub fn syntax_error<T, S: Into<String>>(message: S, position: Position) -> ParserResult<T> {
    Err(ParserError::syntax_error(message, position))
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
        Self::syntax_error(message, Position::at(line, column))
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
        ParserError::syntax_error("Boo", Position::at(12, 3)),
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

    #[test]
    #[should_panic]
    fn bad_position() {
        Position::at(0, 0);
    }

    #[test]
    #[should_panic]
    fn bad_line_and_column() {
        LineAndColumn::at(0, 0);
    }

    #[test]
    #[should_panic]
    fn bad_line_and_column_from_pair() {
        LineAndColumn::from((0, 0));
    }
}
