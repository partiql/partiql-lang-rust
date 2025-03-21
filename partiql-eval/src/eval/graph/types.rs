use std::fmt::Debug;
use std::hash::Hash;

/// The collected types for a graph.
pub trait GraphTypes: 'static + Sized + Debug + Clone + Eq + Hash {
    /// A Binder (name in a graph pattern) for a graph.
    type Binder: Debug + Clone + Eq + Hash;
    /// A Label for a graph.
    type Label: Debug + Clone + Eq + Hash;
    /// A Node for a graph.
    type NodeId: Debug + Clone + Eq + Hash;
    /// An Edge for a graph.
    type EdgeId: Debug + Clone + Eq + Hash;
}
