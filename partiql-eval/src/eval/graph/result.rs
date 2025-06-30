use crate::eval::graph::plan::{BindSpec, NodeMatch, TripleStepMatch};
use crate::eval::graph::types::GraphTypes;
use indexmap::IndexSet;
use rustc_hash::FxBuildHasher;

type FxIndexSet<T> = IndexSet<T, FxBuildHasher>;

/// A result of matching against a [`NodeMatch`]
#[derive(Debug, Clone)]
pub struct NodeBinding<GT: GraphTypes> {
    pub matcher: NodeMatch<GT>,
    pub binding: Vec<GT::NodeId>,
}

/// A result of matching against a [`TripleStepMatch`]
#[derive(Debug, Clone)]
pub struct PathBinding<GT: GraphTypes> {
    pub matcher: TripleStepMatch<GT>,
    pub bindings: Vec<Triple<GT>>,
}

/// A result that is a node,edge,node.
#[derive(Debug, Clone)]
pub struct Triple<GT: GraphTypes> {
    pub lhs: GT::NodeId,
    pub e: GT::EdgeId,
    pub rhs: GT::NodeId,
}

/// Nodes corresponding to a path pattern
#[derive(Debug, Clone, Hash, Eq, PartialEq)]
pub struct PathPatternNodes<GT: GraphTypes> {
    pub head: GT::NodeId,
    pub tail: Vec<(GT::EdgeId, GT::NodeId)>,
}

/// A graph 'element'; either a node or an edge.
#[derive(Debug, PartialEq, Eq, Hash, Clone)]
pub enum GraphElement<'a, GT: GraphTypes> {
    Node(&'a GT::NodeId),
    Edge(&'a GT::EdgeId),
}

impl<GT: GraphTypes> PathPatternNodes<GT> {
    pub fn len(&self) -> usize {
        1 + (self.tail.len() * 2)
    }

    #[allow(dead_code)] // TODO remove once graph planning is implemented
    pub fn is_empty(&self) -> bool {
        false
    }

    pub fn iter(&self) -> impl Iterator<Item = GraphElement<'_, GT>> {
        let head = std::iter::once(GraphElement::Node(&self.head));
        let tail = self
            .tail
            .iter()
            .flat_map(|(e, n)| [GraphElement::Edge(e), GraphElement::Node(n)].into_iter());
        head.chain(tail)
    }
}

/// A result of matching against a [`PathPatternMatch`]
#[derive(Debug, Clone)]
pub struct PathPatternBinding<GT: GraphTypes> {
    pub binders: Vec<BindSpec<GT>>,
    pub bindings: FxIndexSet<PathPatternNodes<GT>>,
}

impl<GT: GraphTypes> From<PathBinding<GT>> for PathPatternBinding<GT> {
    fn from(path_binding: PathBinding<GT>) -> Self {
        let PathBinding { bindings, matcher } = path_binding;

        let (l, e, r) = matcher.binders;
        let binders = vec![l, e, r];
        let bindings = bindings
            .into_iter()
            .map(|triple| PathPatternNodes {
                head: triple.lhs,
                tail: vec![(triple.e, triple.rhs)],
            })
            .collect();

        PathPatternBinding { binders, bindings }
    }
}

impl<GT: GraphTypes> From<NodeBinding<GT>> for PathPatternBinding<GT> {
    fn from(node_binding: NodeBinding<GT>) -> Self {
        let NodeBinding { binding, matcher } = node_binding;

        let binders = vec![matcher.binder];

        let bindings = binding
            .into_iter()
            .map(|node| PathPatternNodes {
                head: node,
                tail: vec![],
            })
            .collect();

        PathPatternBinding { binders, bindings }
    }
}
