use std::fmt::Debug;
use std::hash::Hash;

use crate::eval::graph::types::{GraphElement, GraphTypes};
use fxhash::FxBuildHasher;
use indexmap::IndexSet;

type FxIndexSet<T> = IndexSet<T, FxBuildHasher>;

#[derive(Debug, Clone, Copy)]
pub enum DirSpec {
    L,   // <-
    U,   //  ~
    R,   //  ->
    LU,  // <~
    UR,  //  ~>
    LR,  // <->
    LUR, //  -
}

#[derive(Debug, Clone, Copy, Eq, PartialEq, Hash)]
pub struct BindSpec<GT: GraphTypes>(pub GT::Binder);

#[derive(Debug, Clone, Default)]
pub enum LabelSpec<GT: GraphTypes> {
    #[default]
    Always,
    Named(GT::Label),
    Never,
}

#[derive(Debug, Clone, Copy, Default)]
pub enum FilterSpec {
    #[default]
    Always,
}

#[derive(Debug, Clone)]
pub struct NodeSpec<GT: GraphTypes> {
    pub label: LabelSpec<GT>,
    pub filter: FilterSpec,
}

pub trait ElemSpecBuilder<GT: GraphTypes> {
    fn any() -> Self;
    fn any_labeled(label: GT::Label) -> Self;
}

impl<GT: GraphTypes> ElemSpecBuilder<GT> for NodeSpec<GT> {
    fn any() -> Self {
        Self {
            label: LabelSpec::Always,
            filter: FilterSpec::Always,
        }
    }

    fn any_labeled(label: GT::Label) -> Self {
        Self {
            label: LabelSpec::Named(label),
            filter: FilterSpec::Always,
        }
    }
}

impl<GT: GraphTypes> ElemSpecBuilder<GT> for EdgeSpec<GT> {
    fn any() -> Self {
        Self {
            label: LabelSpec::Always,
            filter: FilterSpec::Always,
        }
    }
    fn any_labeled(label: GT::Label) -> Self {
        Self {
            label: LabelSpec::Named(label),
            filter: FilterSpec::Always,
        }
    }
}

#[derive(Debug, Clone)]
pub struct EdgeSpec<GT: GraphTypes> {
    pub label: LabelSpec<GT>,
    pub filter: FilterSpec,
}

#[derive(Debug, Clone)]
pub struct TripleSpec<GT: GraphTypes> {
    pub lhs: NodeSpec<GT>,
    pub e: EdgeSpec<GT>,
    pub rhs: NodeSpec<GT>,
}

#[derive(Debug, Clone)]
pub struct StepSpec<GT: GraphTypes> {
    pub dir: DirSpec,
    pub triple: TripleSpec<GT>,
}

#[derive(Debug, Clone)]
pub struct NodeMatch<GT: GraphTypes> {
    pub binder: BindSpec<GT>,
    pub spec: NodeSpec<GT>,
}

#[derive(Debug, Clone)]
pub struct EdgeMatch<GT: GraphTypes> {
    pub binder: BindSpec<GT>,
    pub spec: EdgeSpec<GT>,
}

#[derive(Debug, Clone)]
pub struct PathMatch<GT: GraphTypes> {
    pub binders: (BindSpec<GT>, BindSpec<GT>, BindSpec<GT>),
    pub spec: StepSpec<GT>,
}

#[derive(Debug, Clone)]
pub struct PathBinding<GT: GraphTypes> {
    pub matcher: PathMatch<GT>,
    pub bindings: Vec<Triple<GT>>,
}

#[derive(Debug, Clone)]
pub struct NodeBinding<GT: GraphTypes> {
    pub matcher: NodeMatch<GT>,
    pub binding: Vec<GT::NodeId>,
}

#[derive(Debug, Clone)]
pub struct Triple<GT: GraphTypes> {
    pub lhs: GT::NodeId,
    pub e: GT::EdgeId,
    pub rhs: GT::NodeId,
}

#[derive(Debug, Clone)]
pub enum PathPatternMatch<GT: GraphTypes> {
    Node(NodeMatch<GT>),
    Match(PathMatch<GT>),
    Concat(Vec<PathPatternMatch<GT>>),
    // Alternative(Vec<PathPatternMatch<GT>>), ?
}

impl<GT: GraphTypes> From<PathMatch<GT>> for PathPatternMatch<GT> {
    fn from(value: PathMatch<GT>) -> Self {
        Self::Match(value)
    }
}

impl<GT: GraphTypes> From<NodeMatch<GT>> for PathPatternMatch<GT> {
    fn from(value: NodeMatch<GT>) -> Self {
        Self::Node(value)
    }
}

#[derive(Debug, Clone)]
pub struct PathPatternSpecs<GT: GraphTypes> {
    pub head: NodeSpec<GT>,
    pub tail: Vec<(DirSpec, EdgeSpec<GT>, NodeSpec<GT>)>,
}

#[derive(Debug, Clone, Hash, Eq, PartialEq)]
pub struct PathPatternNodes<GT: GraphTypes> {
    pub head: GT::NodeId,
    pub tail: Vec<(GT::EdgeId, GT::NodeId)>,
}

impl<GT: GraphTypes> PathPatternNodes<GT> {
    pub fn len(&self) -> usize {
        1 + (self.tail.len() * 2)
    }

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

#[derive(Debug, Clone)]
pub struct PathPatternBinding<GT: GraphTypes> {
    pub binders: Vec<BindSpec<GT>>,
    // pub specs: Vec<PathPatternSpecs<GT>>,
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
