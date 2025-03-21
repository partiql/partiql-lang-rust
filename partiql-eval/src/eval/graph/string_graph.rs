use crate::eval::graph::types::GraphTypes;

#[derive(Debug, Clone, Eq, PartialEq, Hash)]
pub struct StringGraphTypes;

impl GraphTypes for StringGraphTypes {
    type Binder = String;
    type Label = String;
    type NodeId = ();
    type EdgeId = ();
}
