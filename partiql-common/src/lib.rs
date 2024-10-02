#![deny(rust_2018_idioms)]
#![deny(clippy::all)]

pub static FN_VAR_ARG_MAX: usize = 10;
pub mod metadata;
pub mod node;
pub mod syntax;

pub mod catalog;
