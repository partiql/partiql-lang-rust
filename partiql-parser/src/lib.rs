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
//! let errs: Vec<ParserError> = parser.parse("SELECT").expect_err("expected error");
//!
//! let errs_at: Vec<ParserError> =
//!     parser.parse("SELECT * FROM a AY a CROSS JOIN c AS c AT q").unwrap_err();
//! assert_eq!(errs_at[0].to_string(), "Unexpected token `<a:UNQUOTED_IDENT>` at `(1:20..1:21)`");
//! ```
//!
//! [partiql]: https://partiql.org

mod error;
mod lexer;
mod parse;
mod preprocessor;
mod token_parser;

use parse::parse_partiql;
use partiql_ast::ast;
use partiql_source_map::line_offset_tracker::LineOffsetTracker;

pub use error::LexError;
pub use error::LexicalError;
pub use error::ParseError;
pub use error::ParserError;

/// General [`Result`] type for the PartiQL [`Parser`].
pub type ParserResult<'input> = Result<Parsed<'input>, Vec<ParserError<'input>>>;

/// A PartiQL parser from statement strings to AST.
#[non_exhaustive]
#[derive(Debug)]
pub struct Parser {}

impl Default for Parser {
    fn default() -> Self {
        Parser {}
    }
}

impl Parser {
    /// Parse a PartiQL statement into an AST.
    pub fn parse<'input>(&self, text: &'input str) -> ParserResult<'input> {
        let mut offsets = LineOffsetTracker::default();
        let ast = parse_partiql(text, &mut offsets)?;
        Ok(Parsed { text, offsets, ast })
    }
}

/// The output of parsing PartiQL statement strings: an AST and auxiliary data.
#[non_exhaustive]
#[derive(Debug)]
pub struct Parsed<'input> {
    text: &'input str,
    offsets: LineOffsetTracker,
    ast: Box<ast::Expr>,
}
