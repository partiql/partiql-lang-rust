// Copyright Amazon.com, Inc. or its affiliates.

//! Provides a parser for the [PartiQL][partiql] query language.
//!
//! # Usage
//!
//! ```
//! use partiql_parser::{parse_partiql, ParserError, ParserResult};
//!
//! let ast = parse_partiql("SELECT g FROM data GROUP BY a").expect("successful parse");
//!
//! let errs: Vec<ParserError> = parse_partiql("SELECT").expect_err("expected error");
//! assert_eq!(errs[0].to_string(), "Unexpected end of input");
//!
//! let errs_at: Vec<ParserError> =
//!     parse_partiql("SELECT * FROM a AS a CROSS JOIN c AS c AT q").unwrap_err();
//! assert_eq!(errs_at[0].to_string(), "Unexpected token `AT` at `(1:40..1:42)`");
//! ```
//!
//! [partiql]: https://partiql.org

mod lexer;
mod parse;
mod result;

pub use result::LexError;
pub use result::LexicalError;
pub use result::ParseError;
pub use result::ParserError;
pub use result::ParserResult;

pub use parse::parse_partiql;
