#![deny(rust_2018_idioms)]
#![deny(clippy::all)]
// Copyright Amazon.com, Inc. or its affiliates.

//! Provides a parser for the [PartiQL][partiql] query language.
//!
//! # Usage
//!
//! ```
//! use partiql_parser::{Parser, ParserError, ParserResult};
//!
//! let parser = Parser::default();
//!
//! let parsed = parser.parse("SELECT g FROM data GROUP BY a").expect("successful parse");
//!
//! let errs: ParserError = parser.parse("SELECT").expect_err("expected error");
//!
//! let errs_at: ParserError =
//!     parser.parse("SELECT * FROM a AY a CROSS JOIN c AS c AT q").unwrap_err();
//! assert_eq!(errs_at.errors[0].to_string(), "Unexpected token `<a:UNQUOTED_IDENT>` at `(b19..b20)`");
//! ```
//!
//! [partiql]: https://partiql.org

mod error;
mod lexer;
mod parse;
mod preprocessor;
mod token_parser;

use parse::{parse_partiql, AstData, ErrorData};
use partiql_ast::ast;
use partiql_source_map::line_offset_tracker::LineOffsetTracker;
use partiql_source_map::location::BytePosition;
use partiql_source_map::metadata::LocationMap;
#[cfg(feature = "serde")]
use serde::{Deserialize, Serialize};

/// [`std::error::Error`] type for errors in the lexical structure for the `PartiQL` parser.
pub type LexicalError<'input> = error::LexError<'input>;

/// [`std::error::Error`] type for errors in the syntactic structure for the `PartiQL` parser.
pub type ParseError<'input> = error::ParseError<'input, BytePosition>;

/// General [`Result`] type for the `PartiQL` [`Parser`].
pub type ParserResult<'input> = Result<Parsed<'input>, ParserError<'input>>;

/// A `PartiQL` parser from statement strings to AST.
#[non_exhaustive]
#[derive(Debug, Default)]
pub struct Parser {}

impl Parser {
    /// Parse a `PartiQL` statement into an AST.
    pub fn parse<'input>(&self, text: &'input str) -> ParserResult<'input> {
        match parse_partiql(text) {
            Ok(AstData {
                ast,
                locations,
                offsets,
            }) => Ok(Parsed {
                text,
                offsets,
                ast,
                locations,
            }),
            Err(ErrorData { errors, offsets }) => Err(ParserError {
                text,
                offsets,
                errors,
            }),
        }
    }
}

/// The output of parsing `PartiQL` statement strings: an AST and auxiliary data.
#[non_exhaustive]
#[derive(Debug)]
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
#[allow(dead_code)]
pub struct Parsed<'input> {
    pub text: &'input str,
    pub offsets: LineOffsetTracker,
    pub ast: ast::AstNode<ast::TopLevelQuery>,
    pub locations: LocationMap,
}

/// The output of errors when parsing `PartiQL` statement strings: an errors and auxiliary data.
#[non_exhaustive]
#[allow(dead_code)]
#[derive(Debug)]
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
pub struct ParserError<'input> {
    pub text: &'input str,
    pub offsets: LineOffsetTracker,
    pub errors: Vec<ParseError<'input>>,
}
