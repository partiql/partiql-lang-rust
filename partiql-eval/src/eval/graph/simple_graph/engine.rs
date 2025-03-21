use crate::eval::graph::engine::{GraphAccess, GraphEngine, TripleScan};
use crate::eval::graph::result::Triple;

use crate::eval::graph::plan::{
    BindSpec, EdgeFilter, GraphPlanConvert, LabelFilter, NodeFilter, TripleFilter, ValueFilter,
};
use crate::eval::graph::simple_graph::types::SimpleGraphTypes;
use crate::eval::graph::string_graph::StringGraphTypes;
use crate::eval::graph::types::GraphTypes;
use delegate::delegate;
use lasso::Rodeo;
use partiql_value::{GEdgeId, GLabelId, GNodeId, SimpleGraph, Value};
use std::cell::RefCell;
use std::rc::Rc;

/// [`GraphEngine`] for [`SimpleGraph`]
#[derive(Debug, Clone)]
pub struct SimpleGraphEngine {
    /// The graph.
    pub graph: Rc<SimpleGraph>,
    /// A string interner for turning string labels into interned labels.
    pub binder: RefCell<Rodeo>,
}

impl SimpleGraphEngine {
    pub fn new(g: Rc<SimpleGraph>) -> Self {
        Self {
            graph: g,
            binder: Rodeo::default().into(),
        }
    }
}
impl GraphEngine<SimpleGraphTypes> for SimpleGraphEngine {}

impl TripleScan<SimpleGraphTypes> for SimpleGraphEngine {
    delegate! {
        to self.graph {
            fn scan_directed_to_from(&self, spec: &TripleFilter<SimpleGraphTypes>) -> impl Iterator<Item = Triple<SimpleGraphTypes>>;
            fn scan_directed_from_to(
                &self,
                spec: &TripleFilter<SimpleGraphTypes>,
            ) -> impl Iterator<Item = Triple<SimpleGraphTypes>>;
            fn scan_directed_both(
                &self,
                spec: &TripleFilter<SimpleGraphTypes>,
            ) -> impl Iterator<Item = Triple<SimpleGraphTypes>> ;

            fn scan_undirected(
                &self,
                spec: &TripleFilter<SimpleGraphTypes>,
            ) -> impl Iterator<Item = Triple<SimpleGraphTypes>> ;

            fn get(&self, spec: &NodeFilter<SimpleGraphTypes>) -> Vec<GNodeId>;
        }
    }
}

impl GraphAccess<SimpleGraphTypes> for SimpleGraphEngine {
    delegate! {
        to self.graph {
            fn node(&self, id: &GNodeId) -> &Option<Value>;
            fn edge(&self, id: &GEdgeId) -> &Option<Value>;
        }
    }
}

impl GraphPlanConvert<StringGraphTypes, SimpleGraphTypes> for SimpleGraphEngine {
    fn convert_label_filter(
        &self,
        label: &LabelFilter<StringGraphTypes>,
    ) -> LabelFilter<SimpleGraphTypes> {
        match label {
            LabelFilter::Always => LabelFilter::Always,
            LabelFilter::Never => LabelFilter::Never,
            LabelFilter::Named(l) => {
                if let Some(l) = self.graph.labels.get(l) {
                    // If the label exists in the graph, filter by it
                    LabelFilter::Named(GLabelId(l))
                } else {
                    // If the label doesn't exist in the graph, it can never match
                    LabelFilter::Never
                }
            }
        }
    }

    fn convert_binder(&self, binder: &BindSpec<StringGraphTypes>) -> BindSpec<SimpleGraphTypes> {
        BindSpec(self.binder.borrow_mut().get_or_intern(&binder.0))
    }
}

impl GraphPlanConvert<SimpleGraphTypes, StringGraphTypes> for SimpleGraphEngine {
    fn convert_label_filter(
        &self,
        label: &LabelFilter<SimpleGraphTypes>,
    ) -> LabelFilter<StringGraphTypes> {
        match label {
            LabelFilter::Always => LabelFilter::Always,
            LabelFilter::Never => LabelFilter::Never,
            LabelFilter::Named(l) => {
                LabelFilter::Named(self.graph.labels.resolve(&l.0).to_string())
            }
        }
    }

    fn convert_binder(&self, binder: &BindSpec<SimpleGraphTypes>) -> BindSpec<StringGraphTypes> {
        BindSpec(self.binder.borrow_mut().resolve(&binder.0).to_string())
    }
}

impl GraphAccess<SimpleGraphTypes> for SimpleGraph {
    fn node(&self, id: &GNodeId) -> &Option<Value> {
        &self.nodes[id.0].value
    }

    fn edge(&self, id: &GEdgeId) -> &Option<Value> {
        &self.edges[id.0].value
    }
}

impl TripleScan<SimpleGraphTypes> for SimpleGraph {
    fn scan_directed_from_to(
        &self,
        spec: &TripleFilter<SimpleGraphTypes>,
    ) -> impl Iterator<Item = Triple<SimpleGraphTypes>> {
        // scan directed triples left to right
        self.g_dir
            .iter()
            .map(build_triple)
            .filter(move |t| TripleMatcher::matches(self, spec, t))
    }

    fn scan_directed_to_from(
        &self,
        spec: &TripleFilter<SimpleGraphTypes>,
    ) -> impl Iterator<Item = Triple<SimpleGraphTypes>> {
        // scan directed triples right to left
        self.g_dir
            .iter()
            .map(reverse_triple)
            .filter(move |t| TripleMatcher::matches(self, spec, t))
    }

    fn scan_directed_both(
        &self,
        spec: &TripleFilter<SimpleGraphTypes>,
    ) -> impl Iterator<Item = Triple<SimpleGraphTypes>> {
        // scan directed triples left to right and right to left
        self.g_dir
            .iter()
            .filter(move |(_, e, _)| EdgeMatcher::matches(self, &spec.e, e))
            .flat_map(move |(l, e, r)| {
                let mut res = Vec::with_capacity(2);
                if NodeMatcher::matches(self, &spec.lhs, l)
                    && NodeMatcher::matches(self, &spec.rhs, r)
                {
                    res.push(build_triple(&(*l, *e, *r)))
                }
                if NodeMatcher::matches(self, &spec.rhs, l)
                    && NodeMatcher::matches(self, &spec.lhs, r)
                {
                    res.push(reverse_triple(&(*l, *e, *r)))
                }

                res.into_iter()
            })
    }

    fn scan_undirected(
        &self,
        spec: &TripleFilter<SimpleGraphTypes>,
    ) -> impl Iterator<Item = Triple<SimpleGraphTypes>> {
        // scan undirected triples
        self.g_undir
            .iter()
            .filter(move |(_, e, _)| EdgeMatcher::matches(self, &spec.e, e))
            .flat_map(move |(l, e, r)| {
                let mut res = Vec::with_capacity(2);
                if NodeMatcher::matches(self, &spec.lhs, l)
                    && NodeMatcher::matches(self, &spec.rhs, r)
                {
                    res.push(build_triple(&(*l, *e, *r)))
                }
                if NodeMatcher::matches(self, &spec.rhs, l)
                    && NodeMatcher::matches(self, &spec.lhs, r)
                {
                    res.push(reverse_triple(&(*l, *e, *r)))
                }

                res.into_iter()
            })
    }

    fn get(&self, spec: &NodeFilter<SimpleGraphTypes>) -> Vec<GNodeId> {
        (0..self.nodes.len())
            .map(GNodeId)
            .filter(|node| NodeMatcher::matches(self, spec, node))
            .collect()
    }
}

#[inline]
fn build_triple<GT: GraphTypes>((l, e, r): &(GT::NodeId, GT::EdgeId, GT::NodeId)) -> Triple<GT> {
    Triple {
        lhs: l.clone(),
        e: e.clone(),
        rhs: r.clone(),
    }
}

#[inline]
fn reverse_triple<GT: GraphTypes>((l, e, r): &(GT::NodeId, GT::EdgeId, GT::NodeId)) -> Triple<GT> {
    Triple {
        lhs: r.clone(),
        e: e.clone(),
        rhs: l.clone(),
    }
}

trait TripleMatcher<GT: GraphTypes> {
    fn matches(&self, spec: &TripleFilter<GT>, triple: &Triple<GT>) -> bool;
}

trait NodeMatcher<GT: GraphTypes> {
    fn matches(&self, spec: &NodeFilter<GT>, node: &GT::NodeId) -> bool;
}

trait EdgeMatcher<GT: GraphTypes> {
    fn matches(&self, spec: &EdgeFilter<GT>, edge: &GT::EdgeId) -> bool;
}

impl TripleMatcher<SimpleGraphTypes> for SimpleGraph {
    #[inline]
    fn matches(
        &self,
        spec: &TripleFilter<SimpleGraphTypes>,
        triple: &Triple<SimpleGraphTypes>,
    ) -> bool {
        NodeMatcher::matches(self, &spec.lhs, &triple.lhs)
            && EdgeMatcher::matches(self, &spec.e, &triple.e)
            && NodeMatcher::matches(self, &spec.rhs, &triple.rhs)
    }
}

impl NodeMatcher<SimpleGraphTypes> for SimpleGraph {
    #[inline]
    fn matches(&self, spec: &NodeFilter<SimpleGraphTypes>, node: &GNodeId) -> bool {
        let NodeFilter { label, filter } = spec;
        match (label, filter) {
            (LabelFilter::Never, _) => false,
            (LabelFilter::Always, ValueFilter::Always) => true,
            (LabelFilter::Named(l), ValueFilter::Always) => self.nodes[node.0].labels.0.contains(l),
        }
    }
}

impl EdgeMatcher<SimpleGraphTypes> for SimpleGraph {
    #[inline]
    fn matches(&self, spec: &EdgeFilter<SimpleGraphTypes>, edge: &GEdgeId) -> bool {
        let EdgeFilter { label, filter } = spec;
        match (label, filter) {
            (LabelFilter::Never, _) => false,
            (LabelFilter::Always, ValueFilter::Always) => true,
            (LabelFilter::Named(l), ValueFilter::Always) => self.edges[edge.0].labels.0.contains(l),
        }
    }
}
