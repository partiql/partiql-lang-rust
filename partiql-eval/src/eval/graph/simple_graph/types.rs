use lasso::Spur;

use crate::eval::graph::types::GraphTypes;
use partiql_value::{GEdgeId, GLabelId, GNodeId};

#[derive(Debug, Clone, Eq, PartialEq, Hash)]
pub struct SimpleGraphTypes;

impl GraphTypes for SimpleGraphTypes {
    type Binder = Spur;
    type Label = GLabelId;
    type NodeId = GNodeId;
    type EdgeId = GEdgeId;
}
