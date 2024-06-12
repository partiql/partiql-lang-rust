#![deny(rust_2018_idioms)]
#![deny(clippy::all)]

//! The `PartiQL` Abstract Syntax Tree (AST) passes.
//!
//! # Note
//!
//! This API is currently unstable and subject to change.

pub mod error;
pub mod name_resolver;
