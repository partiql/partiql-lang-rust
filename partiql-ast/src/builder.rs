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
        let id = self.id_gen.next_id();
        AstNode { id, node }
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

#[cfg(test)]
mod tests {
    use super::AstNodeBuilderWithAutoId;
    use crate::ast;

    use crate::visit::{Traverse, Visit, Visitor};
    use partiql_common::node::NodeId;
    use partiql_common::pretty::ToPretty;

    #[test]
    fn unique_ids() {
        let mut bld = AstNodeBuilderWithAutoId::default();

        let mut i64_to_expr = |n| Box::new(ast::Expr::Lit(bld.node(ast::Lit::Int64Lit(n))));

        let lhs = i64_to_expr(5);
        let v1 = i64_to_expr(42);
        let v2 = i64_to_expr(13);
        let list = bld.node(ast::List {
            values: vec![v1, v2],
        });
        let rhs = Box::new(ast::Expr::List(list));
        let op = bld.node(ast::In { lhs, rhs });

        let pretty_printed = op.to_pretty_string(80).expect("pretty print");
        println!("{pretty_printed}");

        dbg!(&op);

        #[derive(Default)]
        pub struct IdVisitor {
            ids: Vec<NodeId>,
        }

        impl Visitor<'_> for IdVisitor {
            fn enter_ast_node(&mut self, id: NodeId) -> Traverse {
                self.ids.push(id);
                Traverse::Continue
            }
        }

        let mut idv = IdVisitor::default();
        op.visit(&mut idv);
        let IdVisitor { ids } = idv;
        dbg!(&ids);

        for i in 0..ids.len() {
            for j in i + 1..ids.len() {
                assert_ne!(ids[i], ids[j]);
            }
        }
    }
}
