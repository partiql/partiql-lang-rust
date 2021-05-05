// Copyright Amazon.com, Inc. or its affiliates.

//! Provides a parser for the [PartiQL][partiql] query language.
//!
//! [partiql]: https://partiql.org

mod peg;
pub mod prelude;
pub mod result;

pub use peg::recognize_partiql;
