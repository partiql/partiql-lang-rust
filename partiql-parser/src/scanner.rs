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

// TODO turn operator/delimiter into enums of their own (nested or otherwise)

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

    /// The `.` punctuation
    Dot,

    /// The `*` operator and wildcard.
    Star,

    /// The `?` placeholder for a query parameter.
    Parameter,

    /// An operator represented by punctuation (as opposed to a keyword based operator).
    /// Contains the slice for the operator.
    Operator(Cow<'val, str>),

    /// A separator character.  Contains the slice for the delimiter character.
    Delimiter(Cow<'val, str>),
}

/// Convenience constructor for a [`Content::Keyword`].
pub fn keyword<'val, S: Into<Cow<'val, str>>>(text: S) -> Content<'val> {
    Content::Keyword(text.into())
}

/// Convenience constructor for a [`Content::Identifier`].
pub fn identifier<'val, S: Into<Cow<'val, str>>>(text: S) -> Content<'val> {
    Content::Identifier(text.into())
}

/// Convenience constructor for a [`Content::IntegerLiteral`].
pub fn integer_literal<'val, V: Into<BigInt>>(value: V) -> Content<'val> {
    Content::IntegerLiteral(value.into())
}

/// Convenience constructor for a [`Content::DecimalLiteral`].
pub fn decimal_literal<'val, V: Into<BigDecimal>>(value: V) -> Content<'val> {
    Content::DecimalLiteral(value.into())
}

/// Convenience constructor for a [`Content::StringLiteral`].
pub fn string_literal<'val, S: Into<Cow<'val, str>>>(text: S) -> Content<'val> {
    Content::StringLiteral(text.into())
}

/// Convenience constructor for a [`Content::Operator`].
pub fn operator<'val, S: Into<Cow<'val, str>>>(text: S) -> Content<'val> {
    Content::Operator(text.into())
}

/// Convenience constructor for a [`Content::Operator`].
pub fn delimiter<'val, S: Into<Cow<'val, str>>>(text: S) -> Content<'val> {
    Content::Delimiter(text.into())
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

fn normalize_operator(raw_text: &str) -> Cow<str> {
    match raw_text {
        "!=" => "<>",
        _ => raw_text,
    }
    .into()
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
            Rule::Keyword => keyword(text.to_uppercase()),
            Rule::String => string_literal(normalize_string_lit(pair.as_str())),
            Rule::Identifier => {
                let ident_pair = pair.into_inner().exactly_one()?;
                match ident_pair.as_rule() {
                    Rule::NonQuotedIdentifier => identifier(ident_pair.as_str()),
                    Rule::QuotedIdentifier => {
                        identifier(normalize_quoted_ident(ident_pair.as_str()))
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
            Rule::Dot_ => Content::Dot,
            Rule::Star_ => Content::Star,
            Rule::Parameter => Content::Parameter,
            Rule::Operator => operator(normalize_operator(text)),
            Rule::Delimiter => delimiter(text),
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

    /// Convenience for a decimal literal from a string--panics if it cannot parse the text.
    fn decimal_literal_from_str<'val, S: AsRef<str>>(text: S) -> Content<'val> {
        decimal_literal(BigDecimal::from_str_radix(text.as_ref(), 10).unwrap())
    }

    struct ScannerTestCase<'val> {
        /// The input text to scan over.
        input: String,

        /// The expected tokens and their ending offsets.
        ///
        /// Note that the tokens in here will have an incomplete remainder that
        /// can be calculated by the test driver before comparison based on the final
        /// state of the test case.  The ending offset is used to calculate the
        /// appropriate remainder slice at `finalize` time.
        tokens_and_offsets: Vec<(Token<'val>, usize)>,

        /// Position of the end of the input.  Used for building up the test case.
        end: LineAndColumn,
    }

    impl<'val> ScannerTestCase<'val> {
        fn new() -> Self {
            Self {
                input: String::new(),
                tokens_and_offsets: Vec::new(),
                end: LineAndColumn::at(1, 1),
            }
        }

        /// Add text that has no token associated with it.  Typically used for whitespace.
        fn add_text(&mut self, input: &'val str) {
            let mut remainder = input;
            let mut line = 1;
            // count the lines and move the remainder portion
            while let Some(offset) = remainder.find("\n") {
                line += 1;
                remainder = &remainder[offset + 1..];
            }
            let col = remainder.chars().count() + 1;
            let logical_position = LineAndColumn::at(line, col);

            self.input.push_str(input);
            self.end = logical_position.position_from(self.end);
        }

        /// Add a token from a parsed content and its associated input text.
        fn add_token(&mut self, input: &'val str, content: Content<'val>) {
            let start = self.end;
            self.add_text(input);

            let incomplete_token = Token {
                content,
                start,
                end: self.end,
                text: input,

                // this is not complete because we don't have all the input
                // we could do this backwards, but it is probably not worth the complexity
                // for a test case, instead we can just derive the expected token when we go
                // to run the expectations from the then complete information.
                remainder: Remainder {
                    input: "",
                    offset: self.end,
                },
            };

            self.tokens_and_offsets
                .push((incomplete_token, self.input.len()));
        }

        /// Finalize the input for this case and patch up the the tokens, consuming this
        /// case and returning the relevant test components.
        fn expected(&self) -> Vec<ParserResult<Token>> {
            let mut expected = Vec::new();
            for (token, end_offset) in self.tokens_and_offsets.iter() {
                expected.push(Ok(Token {
                    remainder: Remainder {
                        input: &self.input.as_str()[*end_offset..],
                        offset: token.end,
                    },
                    ..token.clone()
                }));
            }
            expected.push(syntax_error("IGNORED MESSAGE", self.end.into()));
            expected
        }
    }

    /// Constructs scanner test cases in a less manual way, providing the code to
    /// keep track of the position information we expect from the scanner.
    ///
    /// The test case is constructed by specifying string literals that are expected to be
    /// whitespace, or string literals that are expected to be tokens, the test case
    /// writer provides the [`Content`] of the token, and the macro/[`ScannerTestCase`]
    /// fills in the expected token positions.
    ///
    /// Since the macro user is delimiting where they expect the tokens to be delimited,
    /// this is just doing the trivial book keeping around those chunks of string and
    /// filling in what otherwise would be very manual line/column counting to generate
    /// the assertions.
    macro_rules! scanner_test_case {
        // entry point -- single string or string => content chunk
        ($lit:literal $(=> $expr:expr)?) => {
            // delegate to the general form
            scanner_test_case!($lit $(=> $expr)* ,)
        };
        // entry point -- multiple string or string => content chunks
        ($lit:literal $(=> $expr:expr)? , $($tail:tt)*) => {{
            let mut test_case = ScannerTestCase::new();
            // delegate to the internal builders
            scanner_test_case!(@inner test_case $lit $(=> $expr)* , $($tail)*);
            test_case
        }};
        // termination case -- no more chunks to process
        (@inner $test_case:ident) => {};
        // final whitespace chunk without a terminating ',' -- just delegate to the general form
        (@inner $test_case:ident $lit:literal) => {
            scanner_test_case!(@inner $test_case $lit ,)
        };
        // final token chunk without a terminating ',' -- just delegate to the general form
        (@inner $test_case:ident $lit:literal => $expr:expr) => {
            scanner_test_case!(@inner $test_case $lit => $expr ,)
        };
        // add whitespace for a chunk of string and continue...
        (@inner $test_case:ident $lit:literal , $($tail:tt)*) => {
            $test_case.add_text($lit);
            scanner_test_case!(@inner $test_case $($tail)*)
        };
        // add a token for a chunk of string associated with some expected content and continue...
        (@inner $test_case:ident $lit:literal => $expr:expr , $($tail:tt)*) => {
            $test_case.add_token($lit, $expr);
            scanner_test_case!(@inner $test_case $($tail)*)
        };
    }

    #[rstest]
    #[case::comment_single_keyword(
        scanner_test_case![
            "--",
            "SELECT",
            " \n ",
        ]
    )]
    #[case::comment_mid_line(
        scanner_test_case![
            "SELECT" => keyword("SELECT"),
            "  ",
            "FROM" => keyword("FROM"),
            " -- ",
            "WHERE",
            " \n ",
        ]
    )]
    #[case::comment_until_eol(
        scanner_test_case![
            " -- ",
            "CASE",
            "  ",
            "IN",
            "  ",
            "WHERE",
            " \n ",
            "SELECT" => keyword("SELECT"),
        ]
    )]
    #[case::comment_block(
        scanner_test_case![
            " /* ",
            "CASE",
            "  ",
            "IN",
            " */ ",
            "SELECT" => keyword("SELECT"),
        ]
    )]
    #[case::comment_block_nested(
        scanner_test_case![
            "employee" => identifier("employee"),
            " /*\n ",
            "CASE",
            " /* ",
            "WHERE",
            " \n */ ",
            "employee",
            " \n\n*/ ",
            "IN" => keyword("IN"),
        ]
    )]
    #[case::single_keyword(
        scanner_test_case![
            "  ",
            "SELECT" => keyword("SELECT"),
            "  "
        ]
    )]
    #[case::some_keywords(
        scanner_test_case![
            "  ",
            "CASE" => keyword("CASE"),
            "\t\r\r\n",
            "FROM" => keyword("FROM"),
            "\n \x0B\x0C",
            "WHERE" => keyword("WHERE")
        ]
    )]
    #[case::some_keywords(
        scanner_test_case![
            "moo_cow_1999" => identifier("moo_cow_1999"),
            " ",
            "_1" => identifier("_1"),
            " ",
            "$$$$" => identifier("$$$$")
        ]
    )]
    #[case::quoted_identifiers(
        scanner_test_case![
            "    ",
            r#""moo""# => identifier("moo"),
            "   ",
            r#""""ʕノ•ᴥ•ʔノ ︵ ┻━┻""# => identifier(r#""ʕノ•ᴥ•ʔノ ︵ ┻━┻"#)
        ]
    )]
    #[case::string_literals(
        scanner_test_case![
            "    ",
            "'boo'" => string_literal("boo"),
            "   ",
            "'''┬─┬''ノ( º _ ºノ)'" => string_literal("'┬─┬'ノ( º _ ºノ)")
        ]
    )]
    #[case::numeric_literals(
        scanner_test_case![
            "1" => integer_literal(1),
            " ",
            "-0099" => integer_literal(-99),
            " ",
            "1.1" => decimal_literal_from_str("1.1"),
            " ",
            "+00055.023100" => decimal_literal_from_str("55.023100"),
            " ",
            "99.1234e0010" => decimal_literal_from_str("99.1234e10")
        ]
    )]
    #[case::numeric_literals_with_pads(
        scanner_test_case![
            "+0005" => integer_literal(5),
            " ",
            ".0001" => decimal_literal_from_str("0.0001"),
            " ",
            "-00.0002" => decimal_literal_from_str("-0.0002"),
            " ",
            "000003.004E+001" => decimal_literal_from_str("3.004e1")
        ]
    )]
    #[case::zeroes(
        scanner_test_case![
            "0" => integer_literal(0),
            " ",
            "000" => integer_literal(0),
            " ",
            ".0" => decimal_literal_from_str("0.0"),
            " ",
            "000.000" => decimal_literal_from_str("0.000"),
            " ",
            ".0e0" => decimal_literal_from_str("0.0"),
            " ",
            "0.0e000" => decimal_literal_from_str("0.0")
        ]
    )]
    #[case::delimiters(
        scanner_test_case![
            "[" => delimiter("["),
            "]" => delimiter("]"),
            "(" => delimiter("("),
            ")" => delimiter(")"),
            "{" => delimiter("{"),
            "}" => delimiter("}"),
            "<<" => delimiter("<<"),
            ">>" => delimiter(">>"),
            "," => delimiter(","),
            ":" => delimiter(":"),
            ";" => delimiter(";"),
        ]
    )]
    #[case::operators(
        scanner_test_case![
            "@" => operator("@"),
            "+" => operator("+"),
            "-" => operator("-"),
            "/" => operator("/"),
            "%" => operator("%"),
            "<" => operator("<"),
            " ",
            "<=" => operator("<="),
            ">" => operator(">"),
            " ",
            ">=" => operator(">="),
            "=" => operator("="),
            "<>" => operator("<>"),
            "!=" => operator("<>"),
        ]
    )]
    #[case::left_angles(
        scanner_test_case![
            "<<" => delimiter("<<"),
            "<<" => delimiter("<<"),
            "<" => operator("<"),
        ]
    )]
    #[case::right_angles(
        scanner_test_case![
            ">>" => delimiter(">>"),
            ">>" => delimiter(">>"),
            ">" => operator(">"),
        ]
    )]
    #[case::balanced_angles(
        scanner_test_case![
            "<<" => delimiter("<<"),
            "<<" => delimiter("<<"),
            "<>" => operator("<>"),
            ">>" => delimiter(">>"),
            ">>" => delimiter(">>"),
            " ",
            "<<" => delimiter("<<"),
            "<=" => operator("<="),
            ">>" => delimiter(">>"),
            ">" => operator(">"),
        ]
    )]
    #[case::comment_no_minus(
        scanner_test_case![
            "-------- a line comment with no minus...\n"
        ]
    )]
    #[case::divide_block_comment(
        scanner_test_case![
            "/" => operator("/"),
            "/" => operator("/"),
            "/**/",
            "/" => operator("/"),
            "/" => operator("/"),
        ]
    )]
    #[case::select_from(
        scanner_test_case![
            "SelEct" => keyword("SELECT"),
            " ",
            "'✨✨✨'" => string_literal("✨✨✨"),
            " ",
            "fROM" => keyword("FROM"),
            " ",
            r#""┬─┬""# => identifier("┬─┬"),
            " "
        ]
    )]
    fn scan(#[case] test_case: ScannerTestCase) -> ParserResult<()> {
        let mut scanner = scanner(&test_case.input);
        for expected in test_case.expected() {
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

    #[rstest]
    #[case::bad_identifier("💩")]
    #[case::unterminated_line_comment("-- DROP")]
    #[case::unbalanced_block_nested("/*\n\n SELECT /* WHERE */")]
    #[case::unbalanced_block_end_dangling("/* CASE do WHEN re THEN mi ELSE fa END /*")]
    #[case::unbalanced_block_nested_open_two_deep("/*SELECT /* FROM /* FULL OUTER JOIN */ */ ")]
    #[case::unbalanced_block_deeply_nested("/*/*/*/*/*/*/*/*[ascii art here]*/*/*/*/*/*/*/ ")]
    fn bad_tokens(#[case] input: &str) -> ParserResult<()> {
        let expecteds = vec![syntax_error("IGNORED MESSAGE", Position::at(1, 1))];
        assert_input(input, expecteds)
    }

    fn assert_input(input: &str, expecteds: Vec<ParserResult<Token>>) -> ParserResult<()> {
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
