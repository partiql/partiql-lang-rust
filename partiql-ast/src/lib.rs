#![deny(rust_2018_idioms)]
#![deny(clippy::all)]

//! The `PartiQL` Abstract Syntax Tree (AST).
//!
//! # Note
//!
//! This API is currently unstable and subject to change.

pub mod ast;

pub mod pretty;

pub mod builder;

pub mod visit;
