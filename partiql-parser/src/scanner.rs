// Copyright Amazon.com, Inc. or its affiliates.

//! Provides a simple API to scan PartiQL syntax.
//!
//! The [`Scanner`] trait provides tools the capability to recognize PartiQL lexemes such as
//! keywords, identifiers, etc.  This API is not a full parser, but a lexer in
//! traditional parser terminology.  This API can be useful for tooling or IDEs that wish to
//! do things like syntax highlighting, where full parsing is not required.

use crate::peg::{PairExt, PairsExt, PartiQLParser, Rule};
use crate::prelude::*;
use crate::result::syntax_error;
use pest::iterators::Pair;
use pest::Parser;

/// The parsed content associated with a [`Token`] that has been scanned.
#[derive(Copy, Clone, Debug, Eq, PartialEq)]
pub enum Content<'val> {
    /// A PartiQL keyword. Contains a `str` reference to the UTF8 input bytes that comprise the keyword.
    Keyword(&'val str),
    // TODO things like literals, punctuation, etc.
}

/// Internal type to keep track of remaining input and relative line/column information.
///
/// This is used to leverage the PEG to do continuation parsing and calculating the line/offset
/// information correctly.  Line/offset information does not correspond to UTF-8 code unit (octet)
/// positions so this has to be tracked separately.
#[derive(Copy, Clone, Debug, Eq, PartialEq)]
struct Remainder<'val> {
    /// The remainder of text to scan.
    input: &'val str,
    /// The offset in the input to translate locations from
    offset: LineAndColumn,
}

impl<'val> Remainder<'val> {
    /// Produces a new [`Remainder`] by slicing the input by the given amount and providing
    /// a new line/column baseline.
    ///
    /// The offset given is the logical one as if the input was the start of the sequence.
    /// This method will calculate the new [`Remainder`] offset based on that.
    fn consume(&self, amount: usize, offset: LineAndColumn) -> Self {
        Self {
            input: &self.input[amount..],
            offset: offset.position_from(self.offset),
        }
    }
}

/// A lexeme of PartiQL derived from a slice of input text.
///
/// A token can be thought of as a sort of continuation, it knows where it is in the input list
/// and allows scanning to resume from it.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct Token<'val> {
    /// The underlying value of the token.
    content: Content<'val>,
    /// Start location for this token.
    start: LineAndColumn,
    /// End location for this token.
    end: LineAndColumn,
    /// Slice from the input containing the whole token.
    text: &'val str,
    /// The remaining unscanned value after this token.
    remainder: Remainder<'val>,
}

impl<'val> Token<'val> {
    /// Returns the parsed content of this token.
    pub fn content(&self) -> Content<'val> {
        self.content
    }

    /// Returns the location of where this token starts in the input.
    pub fn start(&self) -> LineAndColumn {
        self.start
    }

    /// Returns the location of where this token ends in the input.
    pub fn end(&self) -> LineAndColumn {
        self.end
    }

    /// Returns the slice of the input encompassing the entire token.
    pub fn text(&self) -> &str {
        self.text
    }

    /// Returns the remainder of the input after this token.
    pub fn text_after(&self) -> &str {
        self.remainder.input
    }
}

/// Returns tokens from a given slice of input text.
pub trait Scanner<'val> {
    /// Returns the next token from the input.
    fn next_token(&mut self) -> ParserResult<Token<'val>>;
}

/// Root scanner, leverages the underlying PEG to provide the scanner.
pub struct PartiQLScanner<'val> {
    /// The remaining input to scan.
    remainder: Remainder<'val>,
}

impl<'val> PartiQLScanner<'val> {
    fn do_next_token(&mut self) -> ParserResult<Token<'val>> {
        // the scanner rule is expected to return a single node
        let pair: Pair<'val, Rule> =
            PartiQLParser::parse(Rule::Scanner, self.remainder.input)?.exactly_one()?;
        let start = pair.start().position_from(self.remainder.offset);
        let end = pair.end().position_from(self.remainder.offset);
        let text = pair.as_str();
        let start_off = pair.as_span().start();
        self.remainder = self.remainder.consume(start_off + text.len(), pair.end());

        let content = match pair.as_rule() {
            Rule::Keyword => Content::Keyword(text),
            _ => return syntax_error(format!("Unexpected rule: {:?}", pair), pair.start().into()),
        };

        Ok(Token {
            content,
            start,
            end,
            text,
            remainder: self.remainder,
        })
    }
}

impl<'val> Scanner<'val> for PartiQLScanner<'val> {
    fn next_token(&mut self) -> ParserResult<Token<'val>> {
        let start_loc = self.remainder.offset;
        self.do_next_token().map_err(|e| match e {
            ParserError::SyntaxError { message, position } => {
                let position = match position {
                    Position::Unknown => Position::Unknown,
                    // make sure to translate line/column position from where we started
                    Position::At(location) => Position::At(location.position_from(start_loc)),
                };
                ParserError::syntax_error(message, position)
            }
        })
    }
}

impl<'val> From<Token<'val>> for PartiQLScanner<'val> {
    fn from(token: Token<'val>) -> Self {
        PartiQLScanner {
            remainder: token.remainder,
        }
    }
}

/// Returns a [`Scanner`] over an slice containing PartiQL.
pub fn scanner(input: &str) -> PartiQLScanner {
    let remainder = Remainder {
        input,
        offset: LineAndColumn::at(1, 1),
    };
    PartiQLScanner { remainder }
}

#[cfg(test)]
mod test {
    use super::*;
    use rstest::*;

    #[rstest]
    #[case::single_keyword(
        "  SELECT  ",
        vec![
            Ok(Token {
                content: Content::Keyword("SELECT"),
                start: LineAndColumn::at(1, 3),
                end: LineAndColumn::at(1, 9),
                text: "SELECT",
                remainder: Remainder {
                    input: "  ",
                    offset: LineAndColumn::at(1, 9)
                }
            }),
            syntax_error("Expected [Keyword]", Position::at(1, 11)),
        ]
    )]
    #[case::some_keywords(
        "  CASE\tFROM\n \x0B\x0CWHERE",
        vec![
            Ok(Token {
                content: Content::Keyword("CASE"),
                start: LineAndColumn::at(1, 3),
                end: LineAndColumn::at(1, 7),
                text: "CASE",
                remainder: Remainder {
                    input: "\tFROM\n \x0B\x0CWHERE",
                    offset: LineAndColumn::at(1, 7)
                }
            }),
            Ok(Token {
                content: Content::Keyword("FROM"),
                start: LineAndColumn::at(1, 8),
                end: LineAndColumn::at(1, 12),
                text: "FROM",
                remainder: Remainder {
                    input: "\n \x0B\x0CWHERE",
                    offset: LineAndColumn::at(1, 12)
                }
            }),
            Ok(Token {
                content: Content::Keyword("WHERE"),
                start: LineAndColumn::at(2, 4),
                end: LineAndColumn::at(2, 9),
                text: "WHERE",
                remainder: Remainder {
                    input: "",
                    offset: LineAndColumn::at(2, 9)
                }
            }),
            syntax_error("Expected [Keyword]", Position::at(2, 9)),
        ]
    )]
    fn tokenize(
        #[case] input: &str,
        #[case] expecteds: Vec<ParserResult<Token>>,
    ) -> ParserResult<()> {
        let mut scanner = scanner(input);
        for expected in expecteds {
            let actual = scanner.next_token();
            match (&expected, &actual) {
                (Ok(expected_tok), Ok(actual_tok)) => {
                    assert_eq!(expected_tok, actual_tok);
                    // make sure accessors do what we expect
                    assert_eq!(expected_tok.content, actual_tok.content(), "Content NE");
                    assert_eq!(expected_tok.start, actual_tok.start(), "Start Location NE");
                    assert_eq!(expected_tok.end, actual_tok.end(), "End Location NE");
                    assert_eq!(expected_tok.text, actual_tok.text(), "Text NE");
                    assert_eq!(
                        expected_tok.remainder.input,
                        actual_tok.text_after(),
                        "Remainder NE"
                    );
                }
                (Err(expected_err), Err(actual_err)) => {
                    // TODO make this less strict with respect to error message
                    assert_eq!(expected_err, actual_err);
                }
                _ => panic!("Did not expect: {:?} and {:?}", expected, actual),
            }
        }
        Ok(())
    }
}
