use crate::eval::graph::types::{BinderTy, EdgeIdTy, GraphLabelTy, GraphTypes, NodeIdTy};

#[derive(Debug, Clone, Eq, PartialEq, Hash)]
pub struct StringGraphTypes;

impl GraphTypes for StringGraphTypes {
    type Binder = String;
    type Label = String;
    type NodeId = ();
    type EdgeId = ();
}

impl GraphLabelTy for String {}

impl NodeIdTy for () {}

impl EdgeIdTy for () {}

impl BinderTy for String {}
