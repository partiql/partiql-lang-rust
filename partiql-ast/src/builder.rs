use crate::ast::AstNode;
use partiql_common::node::{AutoNodeIdGenerator, NodeIdGenerator, NullIdGenerator};

/// A Builder for [`AstNode`]s that uses a [`NodeIdGenerator`] to assign [`NodeId`]s
pub struct AstNodeBuilder<IdGen: NodeIdGenerator> {
    /// Generator for 'fresh' [`NodeId`]s
    pub id_gen: IdGen,
}

impl<IdGen> AstNodeBuilder<IdGen>
where
    IdGen: NodeIdGenerator,
{
    pub fn new(id_gen: IdGen) -> Self {
        Self { id_gen }
    }

    pub fn node<T>(&mut self, node: T) -> AstNode<T> {
        let id = self.id_gen.id();
        let id = id.read().expect("NodId read lock");
        AstNode { id: *id, node }
    }
}

impl<T> Default for AstNodeBuilder<T>
where
    T: NodeIdGenerator + Default,
{
    fn default() -> Self {
        Self::new(T::default())
    }
}

/// A [`AstNodeBuilder`] whose 'fresh' [`NodeId`]s are Auto-incrementing.
pub type AstNodeBuilderWithAutoId = AstNodeBuilder<AutoNodeIdGenerator>;

/// A [`AstNodeBuilder`] whose 'fresh' [`NodeId`]s are always `0`; Useful for testing
pub type AstNodeBuilderWithNullId = AstNodeBuilder<NullIdGenerator>;
