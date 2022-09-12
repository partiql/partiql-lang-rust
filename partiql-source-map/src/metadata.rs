use crate::location::{BytePosition, Location};
use std::collections::HashMap;

/// Map of `T` to a [`Location<BytePosition>>`]
pub type LocationMap<T> = HashMap<T, Location<BytePosition>>;
