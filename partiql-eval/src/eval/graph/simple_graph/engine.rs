use crate::eval::graph::engine::{
    build_triple, reverse_triple, EdgeMatcher, GraphAccess, GraphEngine, GraphScanImpl,
    NodeMatcher, TripleMatcher,
};
use crate::eval::graph::plan::{
    BindSpec, EdgeSpec, FilterSpec, LabelSpec, NodeSpec, Triple, TripleSpec,
};
use crate::eval::graph::simple_graph::types::SimpleGraphTypes;
use crate::eval::graph::string_graph::types::StringGraphTypes;
use crate::eval::graph::types::GraphTypeMapper;
use delegate::delegate;
use lasso::Rodeo;
use partiql_value::{GEdgeId, GLabelId, GNodeId, SimpleGraph, Value};
use std::cell::RefCell;
use std::rc::Rc;

#[derive(Debug, Clone)]
pub struct SimpleGraphEngine {
    pub graph: Rc<SimpleGraph>,
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

impl GraphScanImpl<SimpleGraphTypes> for SimpleGraphEngine {
    delegate! {
        to self.graph {
            fn scan_directed_to_from(&self, spec: &TripleSpec<SimpleGraphTypes>) -> impl Iterator<Item = Triple<SimpleGraphTypes>>;
            fn scan_directed_from_to(
                &self,
                spec: &TripleSpec<SimpleGraphTypes>,
            ) -> impl Iterator<Item = Triple<SimpleGraphTypes>>;
            fn scan_directed_both(
                &self,
                spec: &TripleSpec<SimpleGraphTypes>,
            ) -> impl Iterator<Item = Triple<SimpleGraphTypes>> ;

            fn scan_undirected(
                &self,
                spec: &TripleSpec<SimpleGraphTypes>,
            ) -> impl Iterator<Item = Triple<SimpleGraphTypes>> ;

            fn get(&self, spec: &NodeSpec<SimpleGraphTypes>) -> Vec<GNodeId>;
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

impl GraphTypeMapper<StringGraphTypes, SimpleGraphTypes> for SimpleGraphEngine {
    fn convert_label(&self, label: &LabelSpec<StringGraphTypes>) -> LabelSpec<SimpleGraphTypes> {
        match label {
            LabelSpec::Always => LabelSpec::Always,
            LabelSpec::Named(l) => {
                if let Some(l) = self.graph.labels.get(l) {
                    LabelSpec::Named(GLabelId(l))
                } else {
                    LabelSpec::Never
                }
            }
            LabelSpec::Never => LabelSpec::Never,
        }
    }

    fn convert_binder(&self, binder: &BindSpec<StringGraphTypes>) -> BindSpec<SimpleGraphTypes> {
        BindSpec(self.binder.borrow_mut().get_or_intern(&binder.0))
    }
}

impl GraphTypeMapper<SimpleGraphTypes, StringGraphTypes> for SimpleGraphEngine {
    fn convert_label(&self, label: &LabelSpec<SimpleGraphTypes>) -> LabelSpec<StringGraphTypes> {
        match label {
            LabelSpec::Always => LabelSpec::Always,
            LabelSpec::Named(l) => LabelSpec::Named(self.graph.labels.resolve(&l.0).to_string()),
            LabelSpec::Never => LabelSpec::Never,
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

impl GraphScanImpl<SimpleGraphTypes> for SimpleGraph {
    fn scan_directed_from_to(
        &self,
        spec: &TripleSpec<SimpleGraphTypes>,
    ) -> impl Iterator<Item = Triple<SimpleGraphTypes>> {
        self.g_dir
            .iter()
            .map(build_triple)
            .filter(move |t| TripleMatcher::matches(self, spec, t))
    }

    fn scan_directed_to_from(
        &self,
        spec: &TripleSpec<SimpleGraphTypes>,
    ) -> impl Iterator<Item = Triple<SimpleGraphTypes>> {
        self.g_dir
            .iter()
            .map(reverse_triple)
            .filter(move |t| TripleMatcher::matches(self, spec, t))
    }

    fn scan_directed_both(
        &self,
        spec: &TripleSpec<SimpleGraphTypes>,
    ) -> impl Iterator<Item = Triple<SimpleGraphTypes>> {
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
        spec: &TripleSpec<SimpleGraphTypes>,
    ) -> impl Iterator<Item = Triple<SimpleGraphTypes>> {
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

    fn get(&self, spec: &NodeSpec<SimpleGraphTypes>) -> Vec<GNodeId> {
        (0..self.nodes.len())
            .map(GNodeId)
            .filter(|node| NodeMatcher::matches(self, spec, node))
            .collect()
    }
}

impl TripleMatcher<SimpleGraphTypes> for SimpleGraph {
    fn matches(
        &self,
        spec: &TripleSpec<SimpleGraphTypes>,
        triple: &Triple<SimpleGraphTypes>,
    ) -> bool {
        NodeMatcher::matches(self, &spec.lhs, &triple.lhs)
            && EdgeMatcher::matches(self, &spec.e, &triple.e)
            && NodeMatcher::matches(self, &spec.rhs, &triple.rhs)
    }
}

impl NodeMatcher<SimpleGraphTypes> for SimpleGraph {
    fn matches(&self, spec: &NodeSpec<SimpleGraphTypes>, node: &GNodeId) -> bool {
        let NodeSpec { label, filter } = spec;
        match (label, filter) {
            (LabelSpec::Never, _) => false,
            (LabelSpec::Always, FilterSpec::Always) => true,
            (LabelSpec::Named(l), FilterSpec::Always) => self.nodes[node.0].labels.0.contains(l),
        }
    }
}

impl EdgeMatcher<SimpleGraphTypes> for SimpleGraph {
    fn matches(&self, spec: &EdgeSpec<SimpleGraphTypes>, edge: &GEdgeId) -> bool {
        let EdgeSpec { label, filter } = spec;
        match (label, filter) {
            (LabelSpec::Never, _) => false,
            (LabelSpec::Always, FilterSpec::Always) => true,
            (LabelSpec::Named(l), FilterSpec::Always) => self.edges[edge.0].labels.0.contains(l),
        }
    }
}
