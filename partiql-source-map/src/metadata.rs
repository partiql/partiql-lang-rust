use crate::location::{BytePosition, Location};
use partiql_ast::ast::AstTypeMap;

/// Map of `T` to a [`Location<BytePosition>>`]
pub type LocationMap = AstTypeMap<Location<BytePosition>>;
