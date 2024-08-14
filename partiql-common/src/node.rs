use dashmap::DashMap;
use indexmap::IndexMap;
use std::hash::Hash;
use std::sync::{Arc, MappedRwLockReadGuard, Mutex, RwLock, RwLockReadGuard};

#[cfg(feature = "serde")]
use serde::{Deserialize, Serialize};

// #[derive(Debug, Clone)]
// pub struct NodeMap<T> {
//     map: Arc<RwLock<IndexMap<NodeId, T>>>,
// }
//
// impl<T> Default for NodeMap<T> {
//     fn default() -> Self {
//         Self::new()
//     }
// }
//
// impl<T> NodeMap<T> {
//     pub fn new() -> Self {
//         NodeMap {
//             map: Arc::new(RwLock::new(IndexMap::new())),
//         }
//     }
//
//     pub fn with_capacity(capacity: usize) -> Self {
//         NodeMap {
//             map: Arc::new(RwLock::new(IndexMap::with_capacity(capacity))),
//         }
//     }
//
//     pub fn insert(&self, node_id: NodeId, value: T) -> Option<T> {
//         let mut map = self.map.write().expect("NodeMap write lock");
//         map.insert(node_id, value)
//     }
//
//     // pub fn get(&self, node_id: &NodeId) -> Option<T>
//     // where
//     //     T: Clone,
//     // {
//     //     let map = self.map.read().expect("NodeMap read lock");
//     //     map.get(node_id).cloned()
//     // }
//
//     pub fn get(&self, node_id: &NodeId) -> Option<MappedRwLockReadGuard<Option<&T>>> {
//         let map = self.map.read().unwrap(); // Acquire a read lock
//         Some(RwLockReadGuard::map(map, |m| m.get(node_id)))
//     }
// }

#[derive(Debug, Clone)]
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
pub struct NodeMap<T> {
    map: Arc<DashMap<NodeId, T>>, // DashMap for thread-safe key-value storage
    order: Arc<Mutex<Vec<NodeId>>>, // Mutex-protected Vec for maintaining insertion order
}

impl<T> Default for NodeMap<T> {
    fn default() -> Self {
        Self::new()
    }
}

impl<T> NodeMap<T> {
    // Constructor to create a new, empty NodeMap
    pub fn new() -> Self {
        NodeMap {
            map: Arc::new(DashMap::new()),
            order: Arc::new(Mutex::new(Vec::new())),
        }
    }

    // Constructor to create a NodeMap with a specified capacity
    pub fn with_capacity(capacity: usize) -> Self {
        NodeMap {
            map: Arc::new(DashMap::with_capacity(capacity)),
            order: Arc::new(Mutex::new(Vec::with_capacity(capacity))),
        }
    }

    // The insert method to add a new key-value pair to the map
    pub fn insert(&self, node_id: NodeId, value: T) -> Option<T> {
        let mut order = self.order.lock().unwrap(); // Acquire a lock to modify the order
        if !self.map.contains_key(&node_id) {
            order.push(node_id); // Only add to order if it's a new key
        }
        self.map.insert(node_id, value)
    }

    // The get method to retrieve a reference to a value by its NodeId
    pub fn get(&self, node_id: &NodeId) -> Option<dashmap::mapref::one::Ref<'_, NodeId, T>> {
        self.map.get(node_id)
    }

    // The unwrap_or method to get a value or return a default if not found
    pub fn unwrap_or(&self, node_id: &NodeId, default: T) -> T
    where
        T: Clone,
    {
        self.map.get(node_id).map(|r| r.clone()).unwrap_or(default)
    }

    // Method to retrieve the keys in insertion order
    pub fn keys_in_order(&self) -> Vec<NodeId> {
        let order = self.order.lock().unwrap(); // Acquire a lock to read the order
        order.clone() // Return a cloned version of the order
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
