#[cfg(feature = "serde")]
use serde::{Deserialize, Serialize};
use std::fmt::Debug;

pub mod bind_name;

/// A plan specification for an edge's direction filtering.
#[allow(clippy::upper_case_acronyms)]
#[derive(Debug, Clone, Eq, PartialEq)]
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
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
#[derive(Debug, Clone, Eq, PartialEq)]
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
pub struct BindSpec(pub String);

/// A plan specification for label filtering.
#[derive(Default, Debug, Clone, Eq, PartialEq)]
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
pub enum LabelFilter {
    #[default]
    Always,
    Named(String),
    Never,
}

/// A plan specification for value filtering.
#[derive(Default, Debug, Clone, Eq, PartialEq)]
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
pub enum ValueFilter {
    #[default]
    Always,
    // TODO other variant for, e.g., `WHERE` filters
}

/// A plan specification for node label & value filtering.
#[derive(Debug, Clone, Eq, PartialEq)]
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
pub struct NodeFilter {
    pub label: LabelFilter,
    pub filter: ValueFilter,
}

/// A plan specification for edge label & value filtering.
#[derive(Debug, Clone, Eq, PartialEq)]
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
pub struct EdgeFilter {
    pub label: LabelFilter,
    pub filter: ValueFilter,
}

/// A plan specification for triple (node, edge, node) matching.
#[derive(Debug, Clone, Eq, PartialEq)]
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
pub struct TripleFilter {
    pub lhs: NodeFilter,
    pub e: EdgeFilter,
    pub rhs: NodeFilter,
}

/// A plan specification for 'step' (triple + edge direction) matching.
#[derive(Debug, Clone, Eq, PartialEq)]
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
pub struct StepFilter {
    pub dir: DirectionFilter,
    pub triple: TripleFilter,
}

/// A plan specification for 'path patterns' (i.e., sequences of 'node edge node's) matching.
#[derive(Debug, Clone, Eq, PartialEq)]
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
pub struct PathPattern {
    pub head: NodeMatch,
    pub tail: Vec<(DirectionFilter, EdgeMatch, NodeMatch)>,
}

/// A plan specification for node matching.
#[derive(Debug, Clone, Eq, PartialEq)]
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
pub struct NodeMatch {
    pub binder: BindSpec,
    pub spec: NodeFilter,
}

/// A plan specification for edge matching.
#[derive(Debug, Clone, Eq, PartialEq)]
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
pub struct EdgeMatch {
    pub binder: BindSpec,
    pub spec: EdgeFilter,
}

/// A plan specification for path (i.e., node, edge, node) matching.
#[derive(Debug, Clone, Eq, PartialEq)]
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
pub struct PathMatch {
    pub binders: (BindSpec, BindSpec, BindSpec),
    pub spec: StepFilter,
}

/// A plan specification for path patterns (i.e., sequences of [`PathMatch`]s) matching.
#[derive(Debug, Clone, Eq, PartialEq)]
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
pub enum PathPatternMatch {
    Node(NodeMatch),
    Match(PathMatch),
    Concat(Vec<PathPatternMatch>),
}
