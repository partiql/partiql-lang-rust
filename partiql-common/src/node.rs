use indexmap::IndexMap;
use std::hash::Hash;

pub type NodeMap<T> = IndexMap<NodeId, T>;

#[derive(Copy, Clone, Debug, Eq, PartialEq, Hash)]
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
    fn next_id(&mut self) -> NodeId;
}

impl NodeIdGenerator for AutoNodeIdGenerator {
    #[inline]
    fn next_id(&mut self) -> NodeId {
        let mut next = NodeId(&self.next_id.0 + 1);
        std::mem::swap(&mut self.next_id, &mut next);
        next
    }
}

/// A provider of [`NodeId`]s that are always `0`; Useful for testing
#[derive(Default)]
pub struct NullIdGenerator {}

impl NodeIdGenerator for NullIdGenerator {
    fn next_id(&mut self) -> NodeId {
        NodeId(0)
    }
}

#[cfg(test)]
mod tests {
    use crate::node::{AutoNodeIdGenerator, NodeIdGenerator};

    #[test]
    fn unique_ids() {
        let mut gen = AutoNodeIdGenerator::default();

        let ids: Vec<_> = std::iter::repeat_with(|| gen.next_id()).take(15).collect();
        dbg!(&ids);
        for i in 0..ids.len() {
            for j in i + 1..ids.len() {
                assert_ne!(ids[i], ids[j]);
            }
        }
    }
}
