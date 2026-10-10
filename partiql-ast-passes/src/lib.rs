#![deny(rust_2018_idioms)]
#![deny(clippy::all)]

//! The `PartiQL` Abstract Syntax Tree (AST) passes.
//!
//! # Note
//!
//! This API is currently unstable and subject to change.

pub mod error;
// TODO delete `name_resolver`: the logical planner resolves names while lowering
//  (`partiql-logical-planner/src/lower/scope.rs`) and no longer uses it.
pub mod name_resolver;
