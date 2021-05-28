// Copyright Amazon.com, Inc. or its affiliates.

//! Provides a simple API to scan PartiQL syntax.
//!
//! The [`Scanner`] trait provides tools the capability to recognize PartiQL lexemes such as
//! keywords, identifiers, etc.  This API is not a full parser, but a lexer in
//! traditional parser terminology.  This API can be useful for tooling or IDEs that wish to
//! do things like syntax highlighting, where full parsing is not required.

use crate::peg::{PairExt, PairsExt, PartiQLParser, Rule};
use crate::prelude::*;
use bigdecimal::BigDecimal;
use num_bigint::BigInt;
use num_traits::Num;
use pest::iterators::Pair;
use pest::{Parser, RuleType};
use std::borrow::Cow;

/// The parsed content associated with a [`Token`] that has been scanned.
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum Content<'val> {
    /// A PartiQL keyword.  Contains the slice for the keyword case folded to upper case.
    Keyword(Cow<'val, str>),

    /// An identifier.  Contains the slice for the text of the identifier.
    Identifier(Cow<'val, str>),

    /// An integer literal.  Stores this as an as a [`BigInt`].
    ///
    /// Users will likely deal with smaller integers and encode this in execution/compilation
    /// as `i64` or the like, but the parser need not deal with that detail.
    IntegerLiteral(BigInt),

    /// A decimal literal.  Contains the parsed [`BigDecimal`] for the literal.
    DecimalLiteral(BigDecimal),

    /// A string literal.  Contains the slice for the content of the literal.
    StringLiteral(Cow<'val, str>),
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
    pub fn content(&self) -> &Content<'val> {
        &self.content
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

/// Removes PartiQL escapes from a string literal and dequotes it.
#[inline]
fn normalize_string_lit(raw_text: &str) -> Cow<str> {
    raw_text[1..(raw_text.len() - 1)].replace("''", "'").into()
}

/// Removes PartiQL escapes from a quoted identifier and dequotes it.
#[inline]
fn normalize_quoted_ident(raw_text: &str) -> Cow<str> {
    raw_text[1..(raw_text.len() - 1)]
        .replace(r#""""#, r#"""#)
        .into()
}

fn parse_num<T, R, E>(pair: Pair<R>) -> ParserResult<T>
where
    T: Num<FromStrRadixErr = E>,
    R: RuleType,
    E: std::fmt::Display,
{
    match T::from_str_radix(pair.as_str(), 10) {
        Ok(value) => Ok(value),
        Err(e) => pair.syntax_error(format!("Could not parse number {}: {}", pair.as_str(), e)),
    }
}

impl<'val> PartiQLScanner<'val> {
    fn do_next_token(&mut self) -> ParserResult<Token<'val>> {
        // the scanner rule is expected to return a single node
        let pair: Pair<'val, Rule> =
            PartiQLParser::parse(Rule::Scanner, self.remainder.input)?.exactly_one()?;
        let start = pair.start()?.position_from(self.remainder.offset);
        let end = pair.end()?.position_from(self.remainder.offset);
        let text = pair.as_str();
        let start_off = pair.as_span().start();
        self.remainder = self.remainder.consume(start_off + text.len(), pair.end()?);

        let content = match pair.as_rule() {
            Rule::Keyword => Content::Keyword(text.to_uppercase().into()),
            Rule::String => Content::StringLiteral(normalize_string_lit(pair.as_str())),
            Rule::Identifier => {
                let ident_pair = pair.into_inner().exactly_one()?;
                match ident_pair.as_rule() {
                    Rule::NonQuotedIdentifier => Content::Identifier(ident_pair.as_str().into()),
                    Rule::QuotedIdentifier => {
                        Content::Identifier(normalize_quoted_ident(ident_pair.as_str()))
                    }
                    _ => return ident_pair.unexpected(),
                }
            }
            Rule::Number => {
                let number_pair = pair.into_inner().exactly_one()?;
                match number_pair.as_rule() {
                    Rule::Integer => Content::IntegerLiteral(parse_num(number_pair)?),
                    Rule::Decimal | Rule::DecimalExp => {
                        Content::DecimalLiteral(parse_num(number_pair)?)
                    }
                    _ => return number_pair.unexpected(),
                }
            }
            _ => return pair.unexpected(),
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
            // other errors that don't have position information just pass up as position is not
            // relevant...
            error => error,
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
    use crate::result::syntax_error;
    use rstest::*;

    #[rstest]
    #[case::single_keyword(
        "  SELECT  ",
        vec![
            Ok(Token {
                content: Content::Keyword("SELECT".into()),
                start: LineAndColumn::at(1, 3),
                end: LineAndColumn::at(1, 9),
                text: "SELECT",
                remainder: Remainder {
                    input: "  ",
                    offset: LineAndColumn::at(1, 9)
                }
            }),
            syntax_error("IGNORED MESSAGE", Position::at(1, 11)),
        ]
    )]
    #[case::some_keywords(
        "  CASE\tFROM\n \x0B\x0CWHERE",
        vec![
            Ok(Token {
                content: Content::Keyword("CASE".into()),
                start: LineAndColumn::at(1, 3),
                end: LineAndColumn::at(1, 7),
                text: "CASE",
                remainder: Remainder {
                    input: "\tFROM\n \x0B\x0CWHERE",
                    offset: LineAndColumn::at(1, 7)
                }
            }),
            Ok(Token {
                content: Content::Keyword("FROM".into()),
                start: LineAndColumn::at(1, 8),
                end: LineAndColumn::at(1, 12),
                text: "FROM",
                remainder: Remainder {
                    input: "\n \x0B\x0CWHERE",
                    offset: LineAndColumn::at(1, 12)
                }
            }),
            Ok(Token {
                content: Content::Keyword("WHERE".into()),
                start: LineAndColumn::at(2, 4),
                end: LineAndColumn::at(2, 9),
                text: "WHERE",
                remainder: Remainder {
                    input: "",
                    offset: LineAndColumn::at(2, 9)
                }
            }),
            syntax_error("IGNORED MESSAGE", Position::at(2, 9)),
        ]
    )]
    #[case::plain_identifiers(
        "moo_cow_1999 _1 $$$$",
        vec![
            Ok(Token {
                content: Content::Identifier("moo_cow_1999".into()),
                start: LineAndColumn::at(1, 1),
                end: LineAndColumn::at(1, 13),
                text: "moo_cow_1999",
                remainder: Remainder {
                    input: " _1 $$$$",
                    offset: LineAndColumn::at(1, 13)
                }
            }),
            Ok(Token {
                content: Content::Identifier("_1".into()),
                start: LineAndColumn::at(1, 14),
                end: LineAndColumn::at(1, 16),
                text: "_1",
                remainder: Remainder {
                    input: " $$$$",
                    offset: LineAndColumn::at(1, 16)
                }
            }),
            Ok(Token {
                content: Content::Identifier("$$$$".into()),
                start: LineAndColumn::at(1, 17),
                end: LineAndColumn::at(1, 21),
                text: "$$$$",
                remainder: Remainder {
                    input: "",
                    offset: LineAndColumn::at(1, 21)
                }
            }),
            syntax_error("IGNORED MESSAGE", Position::at(1, 21)),
        ]
    )]
    #[case::bad_identifier(
        "        💩",
        vec![
            syntax_error("IGNORED MESSAGE", Position::at(1, 9)),
        ]
    )]
    #[case::quoted_identifiers(
        r#"    "moo"   """ʕノ•ᴥ•ʔノ ︵ ┻━┻""#,
        vec![
            Ok(Token {
                content: Content::Identifier("moo".into()),
                start: LineAndColumn::at(1, 5),
                end: LineAndColumn::at(1, 10),
                text: r#""moo""#,
                remainder: Remainder {
                    input: r#"   """ʕノ•ᴥ•ʔノ ︵ ┻━┻""#,
                    offset: LineAndColumn::at(1, 10)
                }
            }),
            Ok(Token {
                content: Content::Identifier("\"ʕノ•ᴥ•ʔノ ︵ ┻━┻".into()),
                start: LineAndColumn::at(1, 13),
                end: LineAndColumn::at(1, 30),
                text: r#""""ʕノ•ᴥ•ʔノ ︵ ┻━┻""#,
                remainder: Remainder {
                    input: "",
                    offset: LineAndColumn::at(1, 30)
                }
            }),
            syntax_error("IGNORED MESSAGE", Position::at(1, 30)),
        ]
    )]
    #[case::string_literals(
        "    'boo'   '''┬─┬''ノ( º _ ºノ)'",
        vec![
            Ok(Token {
                content: Content::StringLiteral("boo".into()),
                start: LineAndColumn::at(1, 5),
                end: LineAndColumn::at(1, 10),
                text: "'boo'",
                remainder: Remainder {
                    input: "   '''┬─┬''ノ( º _ ºノ)'",
                    offset: LineAndColumn::at(1, 10)
                }
            }),
            Ok(Token {
                content: Content::StringLiteral("'┬─┬'ノ( º _ ºノ)".into()),
                start: LineAndColumn::at(1, 13),
                end: LineAndColumn::at(1, 32),
                text: "'''┬─┬''ノ( º _ ºノ)'",
                remainder: Remainder {
                    input: "",
                    offset: LineAndColumn::at(1, 32)
                }
            }),
            syntax_error("IGNORED MESSAGE", Position::at(1, 32)),
        ]
    )]
    #[case::numeric_literals(
        "1 -0099 1.1 +00055.023100 99.1234e0010",
        vec![
            Ok(Token {
                content: Content::IntegerLiteral(1.into()),
                start: LineAndColumn::at(1, 1),
                end: LineAndColumn::at(1, 2),
                text: "1",
                remainder: Remainder {
                    input: " -0099 1.1 +00055.023100 99.1234e0010",
                    offset: LineAndColumn::at(1, 2)
                }
            }),
            Ok(Token {
                content: Content::IntegerLiteral(BigInt::from(-99)),
                start: LineAndColumn::at(1, 3),
                end: LineAndColumn::at(1, 8),
                text: "-0099",
                remainder: Remainder {
                    input: " 1.1 +00055.023100 99.1234e0010",
                    offset: LineAndColumn::at(1, 8)
                }
            }),
            Ok(Token {
                content: Content::DecimalLiteral(BigDecimal::from_str_radix("1.1", 10).unwrap()),
                start: LineAndColumn::at(1, 9),
                end: LineAndColumn::at(1, 12),
                text: "1.1",
                remainder: Remainder {
                    input: " +00055.023100 99.1234e0010",
                    offset: LineAndColumn::at(1, 12)
                }
            }),
            Ok(Token {
                content: Content::DecimalLiteral(BigDecimal::from_str_radix("55.023100", 10).unwrap()),
                start: LineAndColumn::at(1, 13),
                end: LineAndColumn::at(1, 26),
                text: "+00055.023100",
                remainder: Remainder {
                    input: " 99.1234e0010",
                    offset: LineAndColumn::at(1, 26)
                }
            }),
            Ok(Token {
                content: Content::DecimalLiteral(BigDecimal::from_str_radix("99.1234e10", 10).unwrap()),
                start: LineAndColumn::at(1, 27),
                end: LineAndColumn::at(1, 39),
                text: "99.1234e0010",
                remainder: Remainder {
                    input: "",
                    offset: LineAndColumn::at(1, 39)
                }
            }),
            syntax_error("IGNORED MESSAGE", Position::at(1, 39)),
        ]
    )]
    #[case::numeric_literals_with_pads(
        "+0005 .0001 -00.0002 000003.004E+001",
        vec![
            Ok(Token {
                content: Content::IntegerLiteral(5.into()),
                start: LineAndColumn::at(1, 1),
                end: LineAndColumn::at(1, 6),
                text: "+0005",
                remainder: Remainder {
                    input: " .0001 -00.0002 000003.004E+001",
                    offset: LineAndColumn::at(1, 6)
                }
            }),
            Ok(Token {
                content: Content::DecimalLiteral(BigDecimal::from_str_radix("0.0001", 10).unwrap()),
                start: LineAndColumn::at(1, 7),
                end: LineAndColumn::at(1, 12),
                text: ".0001",
                remainder: Remainder {
                    input: " -00.0002 000003.004E+001",
                    offset: LineAndColumn::at(1, 12)
                }
            }),
            Ok(Token {
                content: Content::DecimalLiteral(BigDecimal::from_str_radix("-0.0002", 10).unwrap()),
                start: LineAndColumn::at(1, 13),
                end: LineAndColumn::at(1, 21),
                text: "-00.0002",
                remainder: Remainder {
                    input: " 000003.004E+001",
                    offset: LineAndColumn::at(1, 21)
                }
            }),
            Ok(Token {
                content: Content::DecimalLiteral(BigDecimal::from_str_radix("3.004e1", 10).unwrap()),
                start: LineAndColumn::at(1, 22),
                end: LineAndColumn::at(1, 37),
                text: "000003.004E+001",
                remainder: Remainder {
                    input: "",
                    offset: LineAndColumn::at(1, 37)
                }
            }),
            syntax_error("IGNORED MESSAGE", Position::at(1, 37)),
        ]
    )]
    #[case::zeroes(
        "0 000 .0 000.000 .0e0 0.0e000",
        vec![
            Ok(Token {
                content: Content::IntegerLiteral(0.into()),
                start: LineAndColumn::at(1, 1),
                end: LineAndColumn::at(1, 2),
                text: "0",
                remainder: Remainder {
                    input: " 000 .0 000.000 .0e0 0.0e000",
                    offset: LineAndColumn::at(1, 2)
                }
            }),
            Ok(Token {
                content: Content::IntegerLiteral(0.into()),
                start: LineAndColumn::at(1, 3),
                end: LineAndColumn::at(1, 6),
                text: "000",
                remainder: Remainder {
                    input: " .0 000.000 .0e0 0.0e000",
                    offset: LineAndColumn::at(1, 6)
                }
            }),
            Ok(Token {
                content: Content::DecimalLiteral(BigDecimal::from_str_radix("0.0", 10).unwrap()),
                start: LineAndColumn::at(1, 7),
                end: LineAndColumn::at(1, 9),
                text: ".0",
                remainder: Remainder {
                    input: " 000.000 .0e0 0.0e000",
                    offset: LineAndColumn::at(1, 9)
                }
            }),
            Ok(Token {
                content: Content::DecimalLiteral(BigDecimal::from_str_radix("0.000", 10).unwrap()),
                start: LineAndColumn::at(1, 10),
                end: LineAndColumn::at(1, 17),
                text: "000.000",
                remainder: Remainder {
                    input: " .0e0 0.0e000",
                    offset: LineAndColumn::at(1, 17)
                }
            }),
            Ok(Token {
                content: Content::DecimalLiteral(BigDecimal::from_str_radix("0.0", 10).unwrap()),
                start: LineAndColumn::at(1, 18),
                end: LineAndColumn::at(1, 22),
                text: ".0e0",
                remainder: Remainder {
                    input: " 0.0e000",
                    offset: LineAndColumn::at(1, 22)
                }
            }),
            Ok(Token {
                content: Content::DecimalLiteral(BigDecimal::from_str_radix("0.0", 10).unwrap()),
                start: LineAndColumn::at(1, 23),
                end: LineAndColumn::at(1, 30),
                text: "0.0e000",
                remainder: Remainder {
                    input: "",
                    offset: LineAndColumn::at(1, 30)
                }
            }),
            syntax_error("IGNORED MESSAGE", Position::at(1, 30)),
        ]
    )]
    #[case::select_from(
        r#"SelEct '✨✨✨' fROM "┬─┬" "#,
        vec![
            Ok(Token {
                content: Content::Keyword("SELECT".into()),
                start: LineAndColumn::at(1, 1),
                end: LineAndColumn::at(1, 7),
                text: "SelEct",
                remainder: Remainder {
                    input: r#" '✨✨✨' fROM "┬─┬" "#,
                    offset: LineAndColumn::at(1, 7)
                }
            }),
            Ok(Token {
                content: Content::StringLiteral("✨✨✨".into()),
                start: LineAndColumn::at(1, 8),
                end: LineAndColumn::at(1, 13),
                text: "'✨✨✨'",
                remainder: Remainder {
                    input: r#" fROM "┬─┬" "#,
                    offset: LineAndColumn::at(1, 13)
                }
            }),
            Ok(Token {
                content: Content::Keyword("FROM".into()),
                start: LineAndColumn::at(1, 14),
                end: LineAndColumn::at(1, 18),
                text: "fROM",
                remainder: Remainder {
                    input: r#" "┬─┬" "#,
                    offset: LineAndColumn::at(1, 18)
                }
            }),
            Ok(Token {
                content: Content::Identifier("┬─┬".into()),
                start: LineAndColumn::at(1, 19),
                end: LineAndColumn::at(1, 24),
                text: r#""┬─┬""#,
                remainder: Remainder {
                    input: " ",
                    offset: LineAndColumn::at(1, 24)
                }
            }),
            syntax_error("IGNORED MESSAGE", Position::at(1, 25)),
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
                    assert_eq!(expected_tok.content, *actual_tok.content(), "Content NE");
                    assert_eq!(expected_tok.start, actual_tok.start(), "Start Location NE");
                    assert_eq!(expected_tok.end, actual_tok.end(), "End Location NE");
                    assert_eq!(expected_tok.text, actual_tok.text(), "Text NE");
                    assert_eq!(
                        expected_tok.remainder.input,
                        actual_tok.text_after(),
                        "Remainder NE"
                    );
                }
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
                    assert_eq!(expected_position, actual_position);
                }
                _ => panic!("Did not expect: {:?} and {:?}", expected, actual),
            }
        }
        Ok(())
    }
}
