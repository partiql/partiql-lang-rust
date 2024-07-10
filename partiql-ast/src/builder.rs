use crate::ast;
use crate::ast::{AstNode, NodeId};

/// A provider of 'fresh' [`NodeId`]s.
pub trait IdGenerator {
    /// Provides a 'fresh' [`NodeId`].
    fn id(&mut self) -> NodeId;
}

/// Auto-incrementing [`IdGenerator`]
pub struct AutoNodeIdGenerator {
    next_id: ast::NodeId,
}

impl Default for AutoNodeIdGenerator {
    fn default() -> Self {
        AutoNodeIdGenerator { next_id: NodeId(1) }
    }
}

impl IdGenerator for AutoNodeIdGenerator {
    #[inline]
    fn id(&mut self) -> NodeId {
        let mut next = NodeId(&self.next_id.0 + 1);
        std::mem::swap(&mut self.next_id, &mut next);
        next
    }
}

/// A provider of [`NodeId`]s that are always `0`; Useful for testing
#[derive(Default)]
pub struct NullIdGenerator {}

impl IdGenerator for NullIdGenerator {
    fn id(&mut self) -> NodeId {
        NodeId(0)
    }
}

pub struct NodeBuilder<Id: IdGenerator> {
    /// Generator for 'fresh' [`NodeId`]s
    pub id_gen: Id,
}

impl<Id> NodeBuilder<Id>
where
    Id: IdGenerator,
{
    pub fn new(id_gen: Id) -> Self {
        Self { id_gen }
    }

    pub fn node<T>(&mut self, node: T) -> AstNode<T> {
        let id = self.id_gen.id();
        AstNode { id, node }
    }
}

impl<T> Default for NodeBuilder<T>
where
    T: IdGenerator + Default,
{
    fn default() -> Self {
        Self::new(T::default())
    }
}

pub type NodeBuilderWithAutoId = NodeBuilder<AutoNodeIdGenerator>;
pub type NodeBuilderWithNullId = NodeBuilder<NullIdGenerator>;
