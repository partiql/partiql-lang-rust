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
            None => illegal_state("Expected at least one token pair, got nothing!"),
        }
    }
}

/// Extension methods for working with [`Pair`].
pub(crate) trait PairExt<'val, R: RuleType> {
    /// Translates the start position of the [`Pair`] into a [`LineAndColumn`].
    fn start(&self) -> ParserResult<LineAndColumn>;

    /// Translates the end position of the [`Pair`] into a [`LineAndColumn`].
    fn end(&self) -> ParserResult<LineAndColumn>;

    /// Returns an `Err` with a syntax error from the unexpected pair.
    fn unexpected<T>(&self) -> ParserResult<T>;

    /// Returns an `Err` with a syntax error from this pair with a message.
    fn syntax_error<T, S: Into<String>>(&self, message: S) -> ParserResult<T>;
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

    fn unexpected<T>(&self) -> ParserResult<T> {
        self.syntax_error(format!("Unexpected rule: {:?}", self))
    }

    fn syntax_error<T, S: Into<String>>(&self, message: S) -> ParserResult<T> {
        let position = self
            .start()
            .map_or(Position::Unknown, |location| location.into());
        syntax_error(message, position)
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
    use crate::result::syntax_error;
    use rstest::*;

    #[rstest]
    #[case::null_literal(
        r#"NULL"#,
        Ok(())
    )]
    #[case::missing_literal(
        r#"MISSING"#,
        Ok(())
    )]
    #[case::true_literal(
        r#"NULL"#,
        Ok(())
    )]
    #[case::false_literal(
        r#"FALSE"#,
        Ok(())
    )]
    #[case::string_literal(
        r#"'foobar'"#,
        Ok(())
    )]
    #[case::quoted_identifier(
        r#""foobar""#,
        Ok(())
    )]
    #[case::string_literal(
        r#"'moobar'"#,
        Ok(())
    )]
    #[case::integer(
        r#"42"#,
        Ok(())
    )]
    #[case::decimal(
        r#"3141.59265e-03"#,
        Ok(())
    )]
    #[case::array(
        r#"[1, 'moo', "some variable", [], 'a', MISSING]"#,
        Ok(())
    )]
    #[case::bag(
        r#"<<1, <<>>, 'boo', some_variable, 'a'>>"#,
        Ok(())
    )]
    #[case::empty_tuple(
        r#"{}"#,
        Ok(())
    )]
    #[case::tuple(
        r#"{a_variable: 1, 'cow': 'moo', 'a': NULL}"#,
        Ok(())
    )]
    #[case::select_value(
        r#"SELECT VALUE 5"#,
        Ok(())
    )]
    #[case::select_value_from(
        r#"SELECT VALUE 5 FROM some_table"#,
        Ok(())
    )]
    #[case::select_value_from_where(
        r#"SELECT VALUE 5 FROM some_table WHERE TRUE"#,
        Ok(())
    )]
    #[case::select_value_from_where_containers(
        r#"select Value {'age': 6, 'ice_cream': "🍦"} fRoM <<'🚽'>> WHERE is_amazing"#,
        Ok(())
    )]
    #[case::bad_identifier(
        r#"SELECT value aWeSoMe FROM 💩"#,
        syntax_error("IGNORED MESSAGE", Position::at(1, 27))
    )]
    #[case::missing_from_with_where(
        r#"SELECT value aWeSoMe WHERE FALSE"#,
        syntax_error("IGNORED MESSAGE", Position::at(1, 22))
    )]
    fn recognize(#[case] input: &str, #[case] expected: ParserResult<()>) -> ParserResult<()> {
        let actual = recognize_partiql(input);
        match (expected, actual) {
            (
                Err(ParserError::SyntaxError {
                    position: expected_position,
                    ..
                }),
                Err(ParserError::SyntaxError {
                    position: actual_position,
                    ..
                }),
            ) => {
                // just compare the positions for syntax errors...
                assert_eq!(expected_position, actual_position)
            }
            (expected, actual) => {
                assert_eq!(expected, actual);
            }
        }
        Ok(())
    }
}
