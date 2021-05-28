// Copyright Amazon.com, Inc. or its affiliates.

//! Contains the [Pest](https://pest.rs) defined parser for PartiQL and a wrapper APIs that
//! can be exported for users to consume.

use crate::prelude::*;
use crate::result::{illegal_state, syntax_error};
use pest::iterators::{Pair, Pairs};
use pest::{Parser, RuleType};
use pest_derive::Parser;

#[derive(Parser)]
#[grammar = "partiql.pest"]
pub(crate) struct PartiQLParser;

/// Extension methods for working with [`Pairs`].
pub(crate) trait PairsExt<'val, R: RuleType> {
    /// Consumes a [`Pairs`] as a singleton, returning an error if there are less or more than
    /// one [`Pair`].
    fn exactly_one(self) -> ParserResult<Pair<'val, R>>;
}

impl<'val, R: RuleType> PairsExt<'val, R> for Pairs<'val, R> {
    fn exactly_one(mut self) -> ParserResult<Pair<'val, R>> {
        match self.next() {
            Some(pair) => {
                // make sure there isn't something more...
                if let Some(other_pair) = self.next() {
                    syntax_error(
                        format!("Expected one token pair, got: {:?}, {:?}", pair, other_pair),
                        pair.start()?.into(),
                    )?;
                }
                Ok(pair)
            }
            None => illegal_state(),
        }
    }
}

/// Extension methods for working with [`Pair`].
pub(crate) trait PairExt<'val, R: RuleType> {
    /// Translates the start position of the [`Pair`] into a [`LineAndColumn`].
    fn start(&self) -> ParserResult<LineAndColumn>;

    /// Translates the end position of the [`Pair`] into a [`LineAndColumn`].
    fn end(&self) -> ParserResult<LineAndColumn>;
}

impl<'val, R: RuleType> PairExt<'val, R> for Pair<'val, R> {
    #[inline]
    fn start(&self) -> ParserResult<LineAndColumn> {
        self.as_span().start_pos().line_col().try_into()
    }

    #[inline]
    fn end(&self) -> ParserResult<LineAndColumn> {
        self.as_span().end_pos().line_col().try_into()
    }
}

/// Recognizer for PartiQL queries.
///
/// Returns `Ok(())` in the case that the input is valid PartiQL.  Returns `Err([ParserError])`
/// in the case that the input is not valid PartiQL.
///
/// This API will be replaced with one that produces an AST in the future.
pub fn recognize_partiql(input: &str) -> ParserResult<()> {
    PartiQLParser::parse(Rule::Query, input)?;
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn simple() -> ParserResult<()> {
        recognize_partiql("SELECT FROM WHERE")
    }

    #[test]
    fn error() -> ParserResult<()> {
        match recognize_partiql("SELECT FROM MOO") {
            Err(ParserError::SyntaxError { position, .. }) => {
                assert_eq!(Position::at(1, 13), position)
            }
            _ => panic!("Expected Syntax Error"),
        };
        Ok(())
    }
}
