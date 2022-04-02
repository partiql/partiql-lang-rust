// Copyright Amazon.com, Inc. or its affiliates.

//! [`Error`] and [`Result`] types for parsing PartiQL.

use std::fmt::Debug;

use crate::lalr::Token;
use crate::LexError;
use partiql_common::srcmap::location::Located;
use thiserror::Error;

/// General [`Result`] type for the PartiQL parser.
pub type ParserResult<'input, T, Loc> = Result<T, ParserError<'input, Loc>>;

/// Errors from the PartiQL parser.
#[derive(Error, Debug, Clone, PartialEq, Eq, Hash)]
pub enum ParserError<'input, Loc>
where
    Loc: Debug, // TODO this should be `Loc: Display`
{
    /// Indicates that there was a problem with syntax.
    #[error("Syntax Error: {} at [{:?}]", _0.inner, _0.location)]
    SyntaxError(Located<String, Loc>),

    /// There was a token that was not expected
    #[error("Unexpected token [{:?}] at [{:?}]", _0.inner.token, _0.location)]
    UnexpectedToken(UnexpectedToken<'input, Loc>),

    /// There was an error lexing the input
    #[error("{} at [{:?}]", _0.inner, _0.location)]
    LexicalError(LexicalError<'input, Loc>),

    /// Indicates that there is an internal error that was not due to user input or API violation.
    #[error("Illegal State: {0}")]
    IllegalState(String),
}

impl<'input, Loc: Debug> ParserError<'input, Loc>
where
    Loc: Debug, // TODO this should be `Loc: Display`
{
    /// Maps an `ParserError<Loc>` to `ParserError<Loc2>` by applying a function to each variant
    pub fn map_loc<F, Loc2>(self, tx: F) -> ParserError<'input, Loc2>
    where
        Loc2: Debug, // TODO this should be `Loc2: Display`
        F: Fn(Loc) -> Loc2,
    {
        match self {
            ParserError::SyntaxError(l) => ParserError::SyntaxError(l.map_loc(tx)),
            ParserError::UnexpectedToken(l) => ParserError::UnexpectedToken(l.map_loc(tx)),
            ParserError::LexicalError(l) => ParserError::LexicalError(l.map_loc(tx)),
            ParserError::IllegalState(s) => ParserError::IllegalState(s),
        }
    }
}

pub type LexicalError<'input, L> = Located<LexError<'input>, L>;

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct UnexpectedTokenData<'input> {
    /// The unexpected token
    pub token: Token<'input>,
    // TODO expected: ...,
}
pub type UnexpectedToken<'input, L> = Located<UnexpectedTokenData<'input>, L>;

#[cfg(test)]
mod tests {
    use super::*;
    use crate::lalr::Token;
    use crate::LexError;
    use partiql_common::srcmap::location::{
        ByteOffset, BytePosition, CharOffset, Located, ToLocated,
    };

    #[test]
    fn syntax_error() {
        let e1 = ParserError::SyntaxError(
            "oops"
                .to_string()
                .to_located(ByteOffset::from(255)..512.into()),
        );

        let e2 = e1.map_loc(BytePosition);
        assert_eq!(
            e2.to_string(),
            "Syntax Error: oops at [BytePosition(ByteOffset(255))..BytePosition(ByteOffset(512))]"
        )
    }

    #[test]
    fn unexpected_token() {
        let e1 = ParserError::UnexpectedToken(
            UnexpectedTokenData {
                token: Token::Slash,
            }
            .to_located(0.into()..ByteOffset::from(1)),
        );

        let e2 = e1.map_loc(BytePosition);
        assert_eq!(
            e2.to_string(),
            "Unexpected token [Slash] at [BytePosition(ByteOffset(0))..BytePosition(ByteOffset(1))]"
        )
    }

    #[test]
    fn lexical_error() {
        let e1 = ParserError::LexicalError(Located {
            inner: LexError::InvalidInput("🤷"),
            location: CharOffset::from(66_000)..CharOffset::from(66_003),
        });

        let e2 = e1.map_loc(|offset| offset.0);
        assert_eq!(
            e2.to_string(),
            "Lexing error: invalid input `🤷` at [66000..66003]"
        )
    }

    #[test]
    fn illegal_state() {
        let e1 = ParserError::IllegalState("uh oh".to_string());

        let e2 = e1.map_loc(BytePosition);
        assert_eq!(e2.to_string(), "Illegal State: uh oh")
    }
}
