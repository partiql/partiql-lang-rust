use crate::graph::bind_name::BindNameExt;
use crate::ValueExpr;
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

/// A plan specification for a path's matching mode.
#[derive(Debug, Copy, Clone, Eq, PartialEq)]
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
pub enum PathMode {
    /// No filtering of edges/nodes.
    Walk,
    /// No repeated edges.
    Trail,
    /// No repeated nodes.
    Acyclic,
    /// No repeated nodes, except that the ﬁrst and last nodes may be the same.
    Simple,
}

/// A plan specification for bind names.
#[derive(Debug, Clone, Eq, PartialEq)]
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
pub struct BindSpec(pub String);

impl BindSpec {
    /// `true` if a bind name is 'anonymous'. Anonymous bind names are stand-ins in places
    /// where the graph match expression doesn't explicitly include a bind name variable.
    pub fn is_anon(&self) -> bool {
        self.0.is_anon()
    }
}

/// A plan specification for label filtering.
#[derive(Default, Debug, Clone, Eq, PartialEq)]
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
pub enum LabelFilter {
    #[default]
    Always,
    Named(String),
    Negated(Box<LabelFilter>),
    Conjunction(Vec<LabelFilter>),
    Disjunction(Vec<LabelFilter>),
    Never,
}

/// A plan specification for value filtering.
#[derive(Default, Debug, Clone, Eq, PartialEq)]
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
pub enum ValueFilter {
    #[default]
    Always,
    Filter(Vec<ValueExpr>),
}

impl ValueFilter {
    pub fn and(mut self, other: ValueFilter) -> ValueFilter {
        self.extend(other);
        self
    }

    pub fn extend(&mut self, other: ValueFilter) {
        match other {
            ValueFilter::Always => {}
            ValueFilter::Filter(rhs) => match self {
                ValueFilter::Always => {
                    *self = ValueFilter::Filter(rhs);
                }
                ValueFilter::Filter(lhs) => {
                    lhs.extend(rhs);
                }
            },
        }
    }
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
    pub filter: ValueFilter,
    pub mode: PathMode,
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
pub struct TripleMatch {
    pub binders: (BindSpec, BindSpec, BindSpec),
    pub spec: StepFilter,
    pub filter: ValueFilter,
    pub path_mode: PathMode,
}

/// A plan specification for path (i.e., node, edge, node) matching.
#[derive(Debug, Clone, Eq, PartialEq)]
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
pub struct TripleSeriesMatch {
    pub triples: Vec<TripleMatch>,
    pub filter: ValueFilter,
    pub path_mode: PathMode,
}

/// A plan specification for path patterns (i.e., sequences of [`TripleMatch`]s) matching.
#[derive(Debug, Clone, Eq, PartialEq)]
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
pub enum PathPatternMatch {
    Node(NodeMatch),
    Match(TripleMatch),
    Concat(Vec<TripleSeriesMatch>, PathMode),
}
