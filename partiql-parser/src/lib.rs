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
pub use peg::parse_partiql as peg_parse;
