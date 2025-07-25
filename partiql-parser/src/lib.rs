#![deny(rust_2018_idioms)]
#![deny(clippy::all)]
// Copyright Amazon.com, Inc. or its affiliates.

//! Provides a parser for the [PartiQL][partiql] query language.
//!
//! # Usage
//!
//! ```
//! use std::fmt;
//! use std::fmt::Formatter;
//! use itertools::Itertools;
//! use partiql_common::syntax::location::LineAndColumn;
//! use partiql_parser::{Parser, ParserError, ParserResult};
//!
//! let parser = Parser::default();
//!
//! let parsed = parser.parse("SELECT g FROM data GROUP BY a").expect("successful parse");
//!
//! let errs: ParserError = parser.parse("SELECT").expect_err("expected error");
//!
//! // Print out messages with byte offsets
//! let errs_at: ParserError =
//!     parser.parse("SELECT * FROM a AY a CROSS JOIN c AS c AT q").unwrap_err();
//! assert_eq!(errs_at.errors[0].to_string(), "Unexpected token `<a:UNQUOTED_IDENT>` at `(b19..b20)`");
//!
//! // Print out messages with line:column offsets
//! let errs_at_nice: ParserError =
//!     parser.parse("SELECT * FROM a AY a CROSS JOIN c AS c AT q").unwrap_err();
//! let offsets = &errs_at_nice.offsets;
//! let source = &errs_at_nice.text;
//! let err_msg = errs_at_nice.errors.iter().map(|e|
//!     e.clone().map_loc(|loc| LineAndColumn::from(offsets.at(source, loc).unwrap()).to_string())).join("\n");
//! assert_eq!(err_msg, "Unexpected token `<a:UNQUOTED_IDENT>` at `(1:20..1:21)`");
//!
//!
//!
//! // Print out messages with custom line:column offsets
//! #[derive(Debug, Copy, Clone, PartialEq, Eq, Ord, PartialOrd, Hash)]
//! pub struct VerboseLineAndColumn(LineAndColumn);
//!
//! impl fmt::Display for VerboseLineAndColumn {
//!     fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
//!         write!(f, "Line {}, Offset {}", self.0.line, self.0.column)
//!     }
//! }
//!
//! let err_msg = errs_at_nice.errors.iter().map(|e|
//!     e.clone().map_loc(|loc| VerboseLineAndColumn(LineAndColumn::from(offsets.at(source, loc).unwrap())).to_string())).join("\n");
//! assert_eq!(err_msg, "Unexpected token `<a:UNQUOTED_IDENT>` at `(Line 1, Offset 20..Line 1, Offset 21)`");
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
use partiql_common::syntax::line_offset_tracker::LineOffsetTracker;
use partiql_common::syntax::location::BytePosition;
use partiql_common::syntax::metadata::LocationMap;
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
