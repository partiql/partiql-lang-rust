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

/// A provider of 'fresh' [`NodeId`]s.
pub trait NodeIdGenerator {
    fn id(&self) -> Arc<RwLock<NodeId>>;

    /// Provides a 'fresh' [`NodeId`].
    fn next_id(&self) -> NodeId;
}

impl NodeIdGenerator for AutoNodeIdGenerator {
    fn id(&self) -> Arc<RwLock<NodeId>> {
        let id = self.next_id();
        let mut w = self.next_id.write().expect("NodId write lock");
        *w = id;
        Arc::clone(&self.next_id)
    }

    #[inline]
    fn next_id(&self) -> NodeId {
        let id = &self.next_id.read().expect("NodId read lock");
        NodeId(id.0 + 1)
    }
}

/// A provider of [`NodeId`]s that are always `0`; Useful for testing
#[derive(Default)]
pub struct NullIdGenerator {}

impl NodeIdGenerator for NullIdGenerator {
    fn id(&self) -> Arc<RwLock<NodeId>> {
        Arc::new(RwLock::from(self.next_id()))
    }

    fn next_id(&self) -> NodeId {
        NodeId(0)
    }
}
