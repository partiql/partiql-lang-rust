use crate::eval::graph::types::{BinderTy, EdgeIdTy, GraphLabelTy, GraphTypes, NodeIdTy};
use lasso::Spur;
use partiql_value::{GEdgeId, GLabelId, GNodeId};

#[derive(Debug, Clone, Eq, PartialEq, Hash)]
pub struct SimpleGraphTypes;

impl GraphTypes for SimpleGraphTypes {
    type Binder = Spur;
    type Label = GLabelId;
    type NodeId = GNodeId;
    type EdgeId = GEdgeId;
}

impl GraphLabelTy for GLabelId {}

impl NodeIdTy for GNodeId {}

impl EdgeIdTy for GEdgeId {}

impl BinderTy for Spur {}
