use std::fmt::Debug;
use std::hash::Hash;

#[allow(dead_code)] // TODO remove once graph planning is implemented
/// A Label for a graph.
pub trait GraphLabelTy: Debug + Clone + Eq + Hash {}

#[allow(dead_code)] // TODO remove once graph planning is implemented
/// A Binder (name in a graph pattern) for a graph.
pub trait BinderTy: Debug + Clone + Eq + Hash {}

#[allow(dead_code)] // TODO remove once graph planning is implemented
/// A Node for a graph.
pub trait NodeIdTy: Debug + Clone + Eq + Hash {}

#[allow(dead_code)] // TODO remove once graph planning is implemented
/// An Edge for a graph.
pub trait EdgeIdTy: Debug + Clone + Eq + Hash {}

/// The collected types for a graph.
pub trait GraphTypes: 'static + Sized + Debug + Clone + Eq + Hash {
    type Binder: BinderTy;
    type Label: GraphLabelTy;
    type NodeId: NodeIdTy;
    type EdgeId: EdgeIdTy;
}
