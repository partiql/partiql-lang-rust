// Copyright Amazon.com, Inc. or its affiliates.

//! Provides a parser for the [PartiQL][partiql] query language.
//!
//! # Usage
//! TODO
//!
//! [partiql]: https://partiql.org

mod lalr;
mod result;

use partiql_source_map::location::LineAndColumn;

pub type LexicalError<'input> = crate::result::LexicalError<'input>;
pub type ParserError<'input> = crate::result::ParserError<'input, LineAndColumn>;
pub use lalr::ParserResult;

pub use lalr::parse_partiql;
