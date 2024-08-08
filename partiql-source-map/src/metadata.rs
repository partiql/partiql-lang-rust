use crate::location::{BytePosition, Location};
use partiql_core::node::NodeMap;

/// Map of `T` to a [`Location<BytePosition>>`]
pub type LocationMap = NodeMap<Location<BytePosition>>;
