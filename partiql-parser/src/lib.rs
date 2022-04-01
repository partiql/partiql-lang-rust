// Copyright Amazon.com, Inc. or its affiliates.

//! Provides a parser for the [PartiQL][partiql] query language.
//!
//! # Usage
//! TODO
//!
//! [partiql]: https://partiql.org

pub mod location;
pub mod result;

mod lalr;

pub use lalr::lex_partiql as logos_lex;
pub use lalr::parse_partiql as lalr_parse;
pub use lalr::LexError;
pub use lalr::LineOffsetTracker;
pub use lalr::ParserResult as LalrParserResult;
