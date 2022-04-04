// Copyright Amazon.com, Inc. or its affiliates.

//! [`Error`] and [`Result`] types for parsing PartiQL.

use std::borrow::Cow;
use std::fmt::{Debug, Display};

use partiql_source_map::location::Located;
use thiserror::Error;

/// Errors in the lexical structure of a PartiQL query.
///
/// ### Notes
/// This is marked `#[non_exhaustive]`, to reserve the right to add more variants in the future.
#[derive(Error, Debug, Clone, PartialEq, Eq, Hash)]
#[non_exhaustive]
pub enum LexicalError<'input> {
    /// Generic invalid input; likely an unrecognizable token.
    #[error("Lexing error: invalid input `{}`", .0)]
    InvalidInput(Cow<'input, str>),
    /// Embedded Ion value is not properly terminated.
    #[error("Lexing error: unterminated ion literal")]
    UnterminatedIonLiteral,
    /// Comment is not properly terminated.
    #[error("Lexing error: unterminated comment")]
    UnterminatedComment,
    /// Any other lexing error.
    #[error("Lexing error: unknown error")]
    Unknown,
}

/// Errors in the syntactic structer of a PartiQL query.
///
/// ### Notes
/// This is marked `#[non_exhaustive]`, to reserve the right to add more variants in the future.
#[derive(Error, Debug, Clone, PartialEq, Eq, Hash)]
#[non_exhaustive]
pub enum ParserError<'input, Loc>
where
    Loc: Display,
{
    /// Indicates that there was a problem with syntax.
    #[error("Syntax Error: {} at `{}`", _0.inner, _0.location)]
    SyntaxError(Located<String, Loc>),

    /// There were not enough tokens to complete a parse
    #[error("Unexpected end of input at `{}`", _0)]
    UnexpectedEndOfInput(Loc),

    /// An otherwise un-categorized error occurred
    #[error("Unknown parse error at `{}`", _0)]
    UnknownParseError(Loc),

    /// There was a token that was not expected
    #[error("Unexpected token `{}` at `{}`", _0.inner.token, _0.location)]
    UnexpectedToken(UnexpectedToken<'input, Loc>),

    /// There was an error lexing the input
    #[error("{} at `{}`", _0.inner, _0.location)]
    LexicalError(Located<LexicalError<'input>, Loc>),

    /// Indicates that there is an internal error that was not due to user input or API violation.
    #[error("Illegal State: {0}")]
    IllegalState(String),
}

impl<'input, Loc: Debug> ParserError<'input, Loc>
where
    Loc: Display,
{
    /// Maps an `ParserError<Loc>` to `ParserError<Loc2>` by applying a function to each variant
    pub fn map_loc<F, Loc2>(self, mut tx: F) -> ParserError<'input, Loc2>
    where
        Loc2: Display,
        F: FnMut(Loc) -> Loc2,
    {
        match self {
            ParserError::SyntaxError(l) => ParserError::SyntaxError(l.map_loc(tx)),
            ParserError::UnexpectedEndOfInput(loc) => ParserError::UnexpectedEndOfInput(tx(loc)),
            ParserError::UnexpectedToken(l) => ParserError::UnexpectedToken(l.map_loc(tx)),
            ParserError::LexicalError(l) => ParserError::LexicalError(l.map_loc(tx)),
            ParserError::IllegalState(s) => ParserError::IllegalState(s),
            _ => ParserError::IllegalState("Unhandled internal error".to_string()),
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct UnexpectedTokenData<'input> {
    /// The unexpected token
    pub token: Cow<'input, str>,
    // TODO expected: ...,
}
pub type UnexpectedToken<'input, L> = Located<UnexpectedTokenData<'input>, L>;

#[cfg(test)]
mod tests {
    use super::*;
    use partiql_source_map::location::{ByteOffset, BytePosition, LineAndColumn, ToLocated};
    use std::num::NonZeroUsize;

    #[test]
    fn syntax_error() {
        let e1 = ParserError::SyntaxError("oops".to_string().to_located(
            BytePosition::from(ByteOffset::from(255))..BytePosition::from(ByteOffset::from(512)),
        ));

        let e2 = e1.map_loc(|BytePosition(x)| BytePosition(x - 2));
        assert_eq!(e2.to_string(), "Syntax Error: oops at `(b253..b510)`")
    }

    #[test]
    fn unexpected_token() {
        let e1 = ParserError::UnexpectedToken(
            UnexpectedTokenData { token: "/".into() }
                .to_located(BytePosition(0.into())..ByteOffset::from(1).into()),
        );

        let e2 = e1.map_loc(|x| BytePosition(x.0 + 1));
        assert_eq!(e2.to_string(), "Unexpected token `/` at `(b1..b2)`")
    }

    #[test]
    fn lexical_error() {
        let lex = LexicalError::InvalidInput("🤷".into())
            .to_located(LineAndColumn::new(1, 1).unwrap()..LineAndColumn::new(5, 5).unwrap());
        let e1 = ParserError::LexicalError(lex);

        let e2 = e1.map_loc(|LineAndColumn { line, column }| LineAndColumn {
            line,
            column: NonZeroUsize::new(column.get() + 10).unwrap(),
        });
        assert_eq!(
            e2.to_string(),
            "Lexing error: invalid input `🤷` at `(1:11..5:15)`"
        )
    }

    #[test]
    fn illegal_state() {
        let e1: ParserError<BytePosition> = ParserError::IllegalState("uh oh".to_string());

        let e2 = e1.map_loc(|x| x);
        assert_eq!(e2.to_string(), "Illegal State: uh oh")
    }
}
