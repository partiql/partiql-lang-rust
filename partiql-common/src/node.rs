use indexmap::IndexMap;
use std::hash::Hash;

#[cfg(feature = "serde")]
use serde::{Deserialize, Serialize};

pub type NodeMap<T> = IndexMap<NodeId, T>;

#[derive(Copy, Clone, Debug, Eq, PartialEq, Hash)]
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
pub struct NodeId(pub u32);

/// Auto-incrementing [`NodeIdGenerator`]
pub struct AutoNodeIdGenerator {
    next_id: NodeId,
}

impl Default for AutoNodeIdGenerator {
    fn default() -> Self {
        AutoNodeIdGenerator { next_id: NodeId(1) }
    }
}

/// A provider of 'fresh' [`NodeId`]s.
pub trait NodeIdGenerator {
    /// Provides a 'fresh' [`NodeId`].
    fn id(&mut self) -> NodeId;
}

impl NodeIdGenerator for AutoNodeIdGenerator {
    #[inline]
    fn id(&mut self) -> NodeId {
        let mut next = NodeId(&self.next_id.0 + 1);
        std::mem::swap(&mut self.next_id, &mut next);
        next
    }
}

/// A provider of [`NodeId`]s that are always `0`; Useful for testing
#[derive(Default)]
pub struct NullIdGenerator {}

impl NodeIdGenerator for NullIdGenerator {
    fn id(&mut self) -> NodeId {
        NodeId(0)
    }
}
