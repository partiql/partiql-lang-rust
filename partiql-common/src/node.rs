use indexmap::IndexMap;
use std::hash::Hash;
use std::sync::{Arc, RwLock};

#[cfg(feature = "serde")]
use serde::{Deserialize, Serialize};

pub type NodeMap<T> = IndexMap<NodeId, T>;

#[derive(Copy, Clone, Debug, Eq, PartialEq, Hash)]
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
pub struct NodeId(pub u32);

/// Auto-incrementing [`NodeIdGenerator`]
pub struct AutoNodeIdGenerator {
    next_id: Arc<RwLock<NodeId>>,
}

impl Default for AutoNodeIdGenerator {
    fn default() -> Self {
        AutoNodeIdGenerator {
            next_id: Arc::new(RwLock::from(NodeId(1))),
        }
    }
}

impl AutoNodeIdGenerator {
    pub fn next_id(&self) -> Arc<RwLock<NodeId>> {
        self.id()
    }
}

/// A provider of 'fresh' [`NodeId`]s.
pub trait NodeIdGenerator {
    /// Provides a 'fresh' [`NodeId`].
    fn id(&self) -> Arc<RwLock<NodeId>>;
}

impl NodeIdGenerator for AutoNodeIdGenerator {
    #[inline]
    fn id(&self) -> Arc<RwLock<NodeId>> {
        let id = &self.next_id.read().expect("NodeId read lock");
        let next = NodeId(id.0 + 1);
        let mut w = self.next_id.write().expect("NodeId write lock");
        *w = next;
        Arc::clone(&self.next_id)
    }
}

/// A provider of [`NodeId`]s that are always `0`; Useful for testing
#[derive(Default)]
pub struct NullIdGenerator {}

impl NodeIdGenerator for NullIdGenerator {
    fn id(&self) -> Arc<RwLock<NodeId>> {
        Arc::new(RwLock::new(NodeId(0)))
    }
}
