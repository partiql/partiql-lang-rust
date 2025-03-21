use std::fmt::Debug;
use std::hash::Hash;

/// A plan specification for an edge's direction filtering.
#[allow(dead_code)] // TODO remove once graph planning is implemented
#[allow(clippy::upper_case_acronyms)]
#[derive(Debug, Clone, Copy)]
pub enum DirectionFilter {
    L,   // <-
    U,   //  ~
    R,   //  ->
    LU,  // <~
    UR,  //  ~>
    LR,  // <->
    LUR, //  -
}

/// A plan specification for bind names.
#[derive(Debug, Clone, Eq, PartialEq, Hash)]
pub struct BindSpec(pub String);

/// A plan specification for label filtering.
#[derive(Debug, Clone, Default)]
pub enum LabelFilter {
    #[default]
    Always,
    Named(String),
    Never,
}

/// A plan specification for value filtering.
#[derive(Debug, Clone, Copy, Default)]
pub enum ValueFilter {
    #[default]
    Always,
    // TODO other variant for, e.g., `WHERE` filters
}

/// A plan specification for node label & value filtering.
#[derive(Debug, Clone)]
pub struct NodeFilter {
    pub label: LabelFilter,
    pub filter: ValueFilter,
}

/// A plan specification for edge label & value filtering.
#[derive(Debug, Clone)]
pub struct EdgeFilter {
    pub label: LabelFilter,
    pub filter: ValueFilter,
}

/// A plan specification for triple (node, edge, node) matching.
#[derive(Debug, Clone)]
pub struct TripleFilter {
    pub lhs: NodeFilter,
    pub e: EdgeFilter,
    pub rhs: NodeFilter,
}

/// A plan specification for 'step' (triple + edge direction) matching.
#[derive(Debug, Clone)]
pub struct StepFilter {
    pub dir: DirectionFilter,
    pub triple: TripleFilter,
}

/// A plan specification for 'path patterns' (i.e., sequences of 'node edge node's) matching.
#[allow(dead_code)] // TODO remove once graph planning is implemented
#[derive(Debug, Clone)]
pub struct PathPatternFilter {
    pub head: NodeFilter,
    pub tail: Vec<(DirectionFilter, EdgeFilter, NodeFilter)>,
}

/// A plan specification for node matching.
#[derive(Debug, Clone)]
pub struct NodeMatch {
    pub binder: BindSpec,
    pub spec: NodeFilter,
}

/// A plan specification for edge matching.
#[allow(dead_code)] // TODO remove once graph planning is implemented
#[derive(Debug, Clone)]
pub struct EdgeMatch {
    pub binder: BindSpec,
    pub spec: EdgeFilter,
}

/// A plan specification for path (i.e., node, edge, node) matching.
#[derive(Debug, Clone)]
pub struct PathMatch {
    pub binders: (BindSpec, BindSpec, BindSpec),
    pub spec: StepFilter,
}

/// A plan specification for path patterns (i.e., sequences of [`PathMatch`]s) matching.
#[allow(dead_code)] // TODO remove once graph planning is implemented
#[derive(Debug, Clone)]
pub enum PathPatternMatch {
    Node(NodeMatch),
    Match(PathMatch),
    Concat(Vec<PathPatternMatch>),
}

impl From<PathMatch> for PathPatternMatch {
    fn from(value: PathMatch) -> Self {
        Self::Match(value)
    }
}

impl From<NodeMatch> for PathPatternMatch {
    fn from(value: NodeMatch) -> Self {
        Self::Node(value)
    }
}

#[allow(dead_code)] // TODO remove once graph planning is implemented
pub trait ElementFilterBuilder {
    fn any() -> Self;
    fn labeled(label: String) -> Self;
}

impl ElementFilterBuilder for NodeFilter {
    fn any() -> Self {
        Self {
            label: LabelFilter::Always,
            filter: ValueFilter::Always,
        }
    }

    fn labeled(label: String) -> Self {
        Self {
            label: LabelFilter::Named(label),
            filter: ValueFilter::Always,
        }
    }
}

impl ElementFilterBuilder for EdgeFilter {
    fn any() -> Self {
        Self {
            label: LabelFilter::Always,
            filter: ValueFilter::Always,
        }
    }
    fn labeled(label: String) -> Self {
        Self {
            label: LabelFilter::Named(label),
            filter: ValueFilter::Always,
        }
    }
}
