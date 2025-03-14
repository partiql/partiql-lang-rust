use crate::eval::graph::plan::{
    BindSpec, EdgeMatch, EdgeSpec, LabelSpec, NodeMatch, NodeSpec, PathMatch, PathPatternMatch,
    StepSpec, TripleSpec,
};
use std::fmt::Debug;
use std::hash::Hash;

pub trait GraphLabelTy: Debug + Clone + Eq + Hash {}
pub trait BinderTy: Debug + Clone + Eq + Hash {}
pub trait NodeIdTy: Debug + Clone + Eq + Hash {}
pub trait EdgeIdTy: Debug + Clone + Eq + Hash {}

pub trait GraphTypes: 'static + Sized + Debug + Clone + Eq + Hash {
    type Binder: BinderTy;
    type Label: GraphLabelTy;
    type NodeId: NodeIdTy;
    type EdgeId: EdgeIdTy;
}

#[derive(Debug, PartialEq, Eq, Hash, Clone)]
pub enum GraphElement<'a, GT: GraphTypes> {
    Node(&'a GT::NodeId),
    Edge(&'a GT::EdgeId),
}

pub trait GraphTypeMapper<In: GraphTypes, Out: GraphTypes>: Debug {
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
            spec: self.convert_step(&matcher.spec),
        }
    }
    fn convert_step(&self, step: &StepSpec<In>) -> StepSpec<Out> {
        StepSpec {
            dir: step.dir,
            triple: self.convert_triple_spec(&step.triple),
        }
    }
    fn convert_triple_spec(&self, step: &TripleSpec<In>) -> TripleSpec<Out> {
        TripleSpec {
            lhs: self.convert_node_spec(&step.lhs),
            e: self.convert_edge_spec(&step.e),
            rhs: self.convert_node_spec(&step.rhs),
        }
    }
    fn convert_node_spec(&self, node: &NodeSpec<In>) -> NodeSpec<Out> {
        NodeSpec {
            label: self.convert_label(&node.label),
            filter: node.filter,
        }
    }
    fn convert_edge_spec(&self, edge: &EdgeSpec<In>) -> EdgeSpec<Out> {
        EdgeSpec {
            label: self.convert_label(&edge.label),
            filter: edge.filter,
        }
    }
    fn convert_node_match(&self, node: &NodeMatch<In>) -> NodeMatch<Out> {
        NodeMatch {
            binder: self.convert_binder(&node.binder),
            spec: self.convert_node_spec(&node.spec),
        }
    }
    fn convert_edge_match(&self, edge: &EdgeMatch<In>) -> EdgeMatch<Out> {
        EdgeMatch {
            binder: self.convert_binder(&edge.binder),
            spec: self.convert_edge_spec(&edge.spec),
        }
    }
    fn convert_label(&self, node: &LabelSpec<In>) -> LabelSpec<Out>;
    fn convert_binder(&self, binder: &BindSpec<In>) -> BindSpec<Out>;
}
