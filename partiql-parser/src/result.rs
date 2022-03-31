// Copyright Amazon.com, Inc. or its affiliates.

//! [`Error`] and [`Result`] types for parsing PartiQL.

use std::fmt::Debug;
use std::ops::Range;

use crate::lalr::Token;
use crate::LexError;
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
    LexicalError(LexicalError<Loc>),

    /// Indicates that there is an internal error that was not due to user input or API violation.
    #[error("Illegal State: {0}")]
    IllegalState(String),
}

impl<'input, Loc> ParserError<'input, Loc>
where
    Loc: Debug, // TODO this should be `Loc: Display`
{
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

/// A wrapper type that holds an `inner` value and a `location` for it
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct Located<T, Loc: Debug> {
    // TODO this should be `Loc: Display`
    /// The item that has a location attached
    pub inner: T,
    /// The location of the error
    pub location: Range<Loc>,
}

pub(crate) trait ToLocated<Loc: Debug>: Sized {
    fn located(self, location: Range<Loc>) -> Located<Self, Loc> {
        Located {
            inner: self,
            location,
        }
    }
}

impl<T, Loc: Debug> ToLocated<Loc> for T {}

impl<T, Loc: Debug> Located<T, Loc> {
    pub fn map_loc<F, Loc2>(self, tx: F) -> Located<T, Loc2>
    where
        Loc2: Debug,
        F: Fn(Loc) -> Loc2,
    {
        let Located {
            inner: cause,
            location,
        } = self;
        let location = Range {
            start: tx(location.start),
            end: tx(location.end),
        };
        Located {
            inner: cause,
            location,
        }
    }
}

pub type LexicalError<L> = Located<LexError, L>;

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct UnexpectedTokenData<'input> {
    /// The unexpected token
    pub token: Token<'input>,
    // TODO expected: ...,
}
pub type UnexpectedToken<'input, L> = Located<UnexpectedTokenData<'input>, L>;

#[cfg(test)]
mod tests {}
