use crate::node::NodeMap;
use crate::syntax::location::{BytePosition, Location};

/// Map of `T` to a [`Location<BytePosition>>`]
pub type LocationMap = NodeMap<Location<BytePosition>>;
