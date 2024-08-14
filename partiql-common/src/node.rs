use dashmap::DashMap;
use std::sync::{Arc, RwLock};

#[cfg(feature = "serde")]
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone)]
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
pub struct NodeMap<T> {
    map: DashMap<NodeId, T>,
    #[cfg_attr(feature = "serde", serde(skip))]
    order: Arc<RwLock<Vec<NodeId>>>,
}

impl<T> Default for NodeMap<T> {
    fn default() -> Self {
        Self::new()
    }
}

impl<T> NodeMap<T> {
    pub fn new() -> Self {
        NodeMap {
            map: DashMap::new(),
            order: Arc::new(RwLock::new(Vec::new())),
        }
    }

    pub fn with_capacity(capacity: usize) -> Self {
        NodeMap {
            map: DashMap::with_capacity(capacity),
            order: Arc::new(RwLock::new(Vec::with_capacity(capacity))),
        }
    }

    pub fn insert(&self, node_id: NodeId, value: T) -> Option<T> {
        let mut order = self.order.write().expect("NodeMap order write lock");
        if self.map.contains_key(&node_id) {
            self.map.insert(node_id, value)
        } else {
            order.push(node_id);
            self.map.insert(node_id, value)
        }
    }

    pub fn get(&self, node_id: &NodeId) -> Option<dashmap::mapref::one::Ref<'_, NodeId, T>> {
        self.map.get(node_id)
    }
}

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
