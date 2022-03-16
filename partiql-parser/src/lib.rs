// Copyright Amazon.com, Inc. or its affiliates.

//! Provides a parser for the [PartiQL][partiql] query language.
//!
//! # Usage
//! TODO
//!
//! [partiql]: https://partiql.org

pub mod prelude;
pub mod result;

mod peg;

mod lalr;

pub use lalr::lex_partiql as logos_lex;
pub use lalr::parse_partiql as lalr_parse;
pub use lalr::ParseResult as LalrParseResult;
pub use peg::parse_partiql as peg_parse;
