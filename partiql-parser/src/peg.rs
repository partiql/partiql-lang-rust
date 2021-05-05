// Copyright Amazon.com, Inc. or its affiliates.

//! Contains the [Pest](https://pest.rs) defined parser for PartiQL and a wrapper APIs that
//! can be exported for users to consume.

use crate::prelude::*;
use pest::Parser;
use pest_derive::Parser;

#[derive(Parser)]
#[grammar = "partiql.pest"]
struct PartiQLParser;

/// Recognizer for PartiQL queries.
///
/// Returns `Ok(())` in the case that the input is valid PartiQL.  Returns `Err([ParserError])`
/// in the case that the input is not valid PartiQL.
///
/// This API will be replaced with one that produces an AST in the future.
pub fn recognize_partiql(input: &str) -> ParserResult<()> {
    PartiQLParser::parse(Rule::Keywords, input)?;
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
            Err(ParserError::SyntaxError { position, .. }) => assert_eq!(
                Position::At {
                    line: 1,
                    column: 13
                },
                position
            ),
            _ => panic!("Expected Syntax Error"),
        };
        Ok(())
    }
}
