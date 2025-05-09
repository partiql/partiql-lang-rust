use crate::eval::expr::EvalExpr;
use crate::eval::graph::types::GraphTypes;
use std::fmt::Debug;
use std::hash::Hash;
use std::rc::Rc;

/// A plan specification for an edge's direction filtering.
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
#[derive(Debug, Clone, Copy, Eq, PartialEq, Hash)]
pub struct BindSpec<GT: GraphTypes>(pub GT::Binder);

/// A plan specification for label filtering.
#[derive(Debug, Clone, Default)]
pub enum LabelFilter<GT: GraphTypes> {
    #[default]
    Always,
    Named(GT::Label),
    Negated(Box<LabelFilter<GT>>),
    Disjunction(Vec<LabelFilter<GT>>),
    Conjunction(Vec<LabelFilter<GT>>),
    Never,
}

/// A plan specification for value filtering.
#[derive(Debug, Clone, Default)]
pub enum ValueFilter {
    #[default]
    Always,
    Filter(Rc<dyn EvalExpr>),
}

/// A plan specification for node label & value filtering.
#[derive(Debug, Clone)]
pub struct NodeFilter<GT: GraphTypes> {
    pub label: LabelFilter<GT>,
    pub filter: ValueFilter,
}

/// A plan specification for edge label & value filtering.
#[derive(Debug, Clone)]
pub struct EdgeFilter<GT: GraphTypes> {
    pub label: LabelFilter<GT>,
    pub filter: ValueFilter,
}

/// A plan specification for triple (node, edge, node) matching.
#[derive(Debug, Clone)]
pub struct TripleFilter<GT: GraphTypes> {
    pub lhs: NodeFilter<GT>,
    pub e: EdgeFilter<GT>,
    pub rhs: NodeFilter<GT>,
}

/// A plan specification for 'step' (triple + edge direction) matching.
#[derive(Debug, Clone)]
pub struct StepFilter<GT: GraphTypes> {
    pub dir: DirectionFilter,
    pub triple: TripleFilter<GT>,
}

/// A plan specification for node matching.
#[derive(Debug, Clone)]
pub struct NodeMatch<GT: GraphTypes> {
    pub binder: BindSpec<GT>,
    pub spec: NodeFilter<GT>,
}

/// A plan specification for path (i.e., node, edge, node) matching.
#[derive(Debug, Clone)]
pub struct PathMatch<GT: GraphTypes> {
    pub binders: (BindSpec<GT>, BindSpec<GT>, BindSpec<GT>),
    pub spec: StepFilter<GT>,
}

/// A plan specification for path patterns (i.e., sequences of [`PathMatch`]s) matching.
#[derive(Debug, Clone)]
pub enum PathPatternMatch<GT: GraphTypes> {
    Node(NodeMatch<GT>),
    Match(PathMatch<GT>),
    Concat(Vec<PathPatternMatch<GT>>),
}

/// A trait for converting between plans parameterized by different [`GraphTypes`]
pub trait GraphPlanConvert<In: GraphTypes, Out: GraphTypes>: Debug {
    fn convert_pathpattern_match(&self, matcher: &PathPatternMatch<In>) -> PathPatternMatch<Out> {
        match matcher {
            PathPatternMatch::Node(n) => PathPatternMatch::Node(self.convert_node_match(n)),
            PathPatternMatch::Match(m) => PathPatternMatch::Match(self.convert_path_match(m)),
            PathPatternMatch::Concat(ms) => PathPatternMatch::Concat(
                ms.iter()
                    .map(|m| self.convert_pathpattern_match(m))
                    .collect(),
            ),
        }
    }
    fn convert_path_match(&self, matcher: &PathMatch<In>) -> PathMatch<Out> {
        let (x, y, z) = &matcher.binders;
        let binders = (
            self.convert_binder(x),
            self.convert_binder(y),
            self.convert_binder(z),
        );
        PathMatch {
            binders,
            spec: self.convert_step_filter(&matcher.spec),
        }
    }
    fn convert_step_filter(&self, step: &StepFilter<In>) -> StepFilter<Out> {
        StepFilter {
            dir: step.dir,
            triple: self.convert_triple_filter(&step.triple),
        }
    }
    fn convert_triple_filter(&self, step: &TripleFilter<In>) -> TripleFilter<Out> {
        TripleFilter {
            lhs: self.convert_node_filter(&step.lhs),
            e: self.convert_edge_filter(&step.e),
            rhs: self.convert_node_filter(&step.rhs),
        }
    }
    fn convert_node_filter(&self, node: &NodeFilter<In>) -> NodeFilter<Out> {
        NodeFilter {
            label: self.convert_label_filter(&node.label),
            filter: node.filter.clone(),
        }
    }
    fn convert_edge_filter(&self, edge: &EdgeFilter<In>) -> EdgeFilter<Out> {
        EdgeFilter {
            label: self.convert_label_filter(&edge.label),
            filter: edge.filter.clone(),
        }
    }
    fn convert_node_match(&self, node: &NodeMatch<In>) -> NodeMatch<Out> {
        NodeMatch {
            binder: self.convert_binder(&node.binder),
            spec: self.convert_node_filter(&node.spec),
        }
    }

    fn convert_label_filter(&self, node: &LabelFilter<In>) -> LabelFilter<Out>;
    fn convert_binder(&self, binder: &BindSpec<In>) -> BindSpec<Out>;
}
