use crate::eval::graph::engine::{GraphAccess, GraphEngine, GraphFilter, TripleScan};
use crate::eval::graph::result::{GraphElement, PathPatternNodes, Triple};
use std::borrow::Cow;

use crate::eval::graph::plan::{
    BindSpec, EdgeFilter, GraphPlanConvert, LabelFilter, NodeFilter, TripleFilter, ValueFilter,
};
use crate::eval::graph::simple_graph::types::SimpleGraphTypes;
use crate::eval::graph::string_graph::StringGraphTypes;
use crate::eval::graph::types::GraphTypes;
use crate::eval::EvalContext;
use delegate::delegate;
use indexmap::IndexSet;
use lasso::Rodeo;
use partiql_value::datum::DatumTupleRef;
use partiql_value::{GEdgeId, GLabelId, GNodeId, SimpleGraph, Value};
use rustc_hash::FxBuildHasher;
use std::cell::RefCell;
use std::collections::HashMap;
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
            LabelFilter::Negated(inner) => {
                LabelFilter::Negated(Box::new(self.convert_label_filter(inner)))
            }
            LabelFilter::Conjunction(inner) => {
                let inner = inner.iter().map(|l| self.convert_label_filter(l));
                LabelFilter::Conjunction(inner.collect())
            }
            LabelFilter::Disjunction(inner) => {
                let inner = inner.iter().map(|l| self.convert_label_filter(l));
                LabelFilter::Disjunction(inner.collect())
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
            LabelFilter::Negated(inner) => {
                LabelFilter::Negated(Box::new(self.convert_label_filter(inner)))
            }
            LabelFilter::Conjunction(inner) => {
                let inner = inner.iter().map(|l| self.convert_label_filter(l));
                LabelFilter::Conjunction(inner.collect())
            }
            LabelFilter::Disjunction(inner) => {
                let inner = inner.iter().map(|l| self.convert_label_filter(l));
                LabelFilter::Disjunction(inner.collect())
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

impl TripleScan<SimpleGraphTypes> for SimpleGraphEngine {
    fn scan_directed_from_to(
        &self,
        binders: &(
            BindSpec<SimpleGraphTypes>,
            BindSpec<SimpleGraphTypes>,
            BindSpec<SimpleGraphTypes>,
        ),
        spec: &TripleFilter<SimpleGraphTypes>,
        allow_repeated_nodes: bool,
        filter: &ValueFilter,
        ctx: &dyn EvalContext,
    ) -> impl Iterator<Item = Triple<SimpleGraphTypes>> {
        // scan directed triples left to right
        self.graph
            .g_dir
            .iter()
            .filter(move |(n1, _, n2)| allow_repeated_nodes || n1 != n2)
            .map(build_triple)
            .filter(move |t| self.triple_matches(binders, spec, t, filter, ctx))
    }

    fn scan_directed_to_from(
        &self,
        binders: &(
            BindSpec<SimpleGraphTypes>,
            BindSpec<SimpleGraphTypes>,
            BindSpec<SimpleGraphTypes>,
        ),
        spec: &TripleFilter<SimpleGraphTypes>,
        allow_repeated_nodes: bool,
        filter: &ValueFilter,
        ctx: &dyn EvalContext,
    ) -> impl Iterator<Item = Triple<SimpleGraphTypes>> {
        // scan directed triples right to left
        self.graph
            .g_dir
            .iter()
            .filter(move |(n1, _, n2)| allow_repeated_nodes || n1 != n2)
            .map(reverse_triple)
            .filter(move |t| self.triple_matches(binders, spec, t, filter, ctx))
    }

    fn scan_directed_both(
        &self,
        binders: &(
            BindSpec<SimpleGraphTypes>,
            BindSpec<SimpleGraphTypes>,
            BindSpec<SimpleGraphTypes>,
        ),
        spec: &TripleFilter<SimpleGraphTypes>,
        allow_repeated_nodes: bool,
        filter: &ValueFilter,
        ctx: &dyn EvalContext,
    ) -> impl Iterator<Item = Triple<SimpleGraphTypes>> {
        let (bl, bm, br) = binders;
        // scan directed triples left to right and right to left
        self.graph
            .g_dir
            .iter()
            .filter(move |(n1, _, n2)| allow_repeated_nodes || n1 != n2)
            .filter(|(lhs, e, rhs)| {
                let triple = Triple {
                    lhs: *lhs,
                    e: *e,
                    rhs: *rhs,
                };
                let edge = self.edge_matches(bm, &spec.e, e, ctx);
                let triple = self.triple_value_matches(binders, filter, &triple, ctx);
                edge && triple
            })
            .flat_map(|(l, e, r)| {
                let mut res = Vec::with_capacity(2);
                if self.node_matches(bl, &spec.lhs, l, ctx)
                    && self.node_matches(br, &spec.rhs, r, ctx)
                {
                    res.push(build_triple(&(*l, *e, *r)))
                }
                if self.node_matches(br, &spec.rhs, l, ctx)
                    && self.node_matches(bl, &spec.lhs, r, ctx)
                {
                    res.push(reverse_triple(&(*l, *e, *r)))
                }

                res.into_iter()
            })
    }

    fn scan_undirected(
        &self,
        binders: &(
            BindSpec<SimpleGraphTypes>,
            BindSpec<SimpleGraphTypes>,
            BindSpec<SimpleGraphTypes>,
        ),
        spec: &TripleFilter<SimpleGraphTypes>,
        allow_repeated_nodes: bool,
        filter: &ValueFilter,
        ctx: &dyn EvalContext,
    ) -> impl Iterator<Item = Triple<SimpleGraphTypes>> {
        let (bl, bm, br) = binders;
        // scan undirected triples
        self.graph
            .g_undir
            .iter()
            .filter(move |(n1, _, n2)| allow_repeated_nodes || n1 != n2)
            .filter(|(lhs, e, rhs)| {
                let triple = Triple {
                    lhs: *lhs,
                    e: *e,
                    rhs: *rhs,
                };
                let edge = self.edge_matches(bm, &spec.e, e, ctx);
                let triple = self.triple_value_matches(binders, filter, &triple, ctx);
                edge && triple
            })
            .flat_map(|(l, e, r)| {
                let mut res = Vec::with_capacity(2);
                if self.node_matches(bl, &spec.lhs, l, ctx)
                    && self.node_matches(br, &spec.rhs, r, ctx)
                {
                    res.push(build_triple(&(*l, *e, *r)))
                }
                if self.node_matches(br, &spec.rhs, l, ctx)
                    && self.node_matches(bl, &spec.lhs, r, ctx)
                {
                    res.push(reverse_triple(&(*l, *e, *r)))
                }

                res.into_iter()
            })
    }

    fn get(
        &self,
        binder: &BindSpec<SimpleGraphTypes>,
        spec: &NodeFilter<SimpleGraphTypes>,
        ctx: &dyn EvalContext,
    ) -> Vec<GNodeId> {
        (0..self.graph.nodes.len())
            .map(GNodeId)
            .filter(|node| self.node_matches(binder, spec, node, ctx))
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

type FxIndexSet<T> = IndexSet<T, FxBuildHasher>;

impl GraphFilter<SimpleGraphTypes> for SimpleGraphEngine {
    fn filter_path_nodes(
        &self,
        binders: &[BindSpec<SimpleGraphTypes>],
        spec: &ValueFilter,
        mut bindings: FxIndexSet<PathPatternNodes<SimpleGraphTypes>>,
        ctx: &dyn EvalContext,
    ) -> FxIndexSet<PathPatternNodes<SimpleGraphTypes>> {
        match spec {
            ValueFilter::Always => (),
            ValueFilter::Filter(exprs) => {
                let resolver = self.binder.borrow();

                bindings.retain(|path_nodes| {
                    let binders = binders
                        .iter()
                        .map(|b| Cow::Borrowed(resolver.resolve(&b.0)));
                    let values = path_nodes.iter().map(|elt| match elt {
                        GraphElement::Node(node_id) => &self.graph.nodes[node_id.0].value,
                        GraphElement::Edge(edge_id) => &self.graph.edges[edge_id.0].value,
                    });

                    let map: HashMap<_, _> = binders
                        .zip(values)
                        .filter_map(|(k, v)| v.as_ref().map(|v| (k, v)))
                        .collect();

                    let bindings = match map.len() {
                        0 => DatumTupleRef::Empty,
                        1 => {
                            let (key, payload) = map.into_iter().next().unwrap();
                            DatumTupleRef::SingleKey(key, payload)
                        }
                        _ => DatumTupleRef::Bindings(&map),
                    };

                    exprs.iter().all(|expr| {
                        matches!(expr.evaluate(&bindings, ctx).as_ref(), Value::Boolean(true))
                    })
                });
            }
        }
        bindings
    }
}

impl SimpleGraphEngine {
    #[inline]
    fn triple_matches(
        &self,
        binders: &(
            BindSpec<SimpleGraphTypes>,
            BindSpec<SimpleGraphTypes>,
            BindSpec<SimpleGraphTypes>,
        ),
        spec: &TripleFilter<SimpleGraphTypes>,
        triple: &Triple<SimpleGraphTypes>,
        filter: &ValueFilter,
        ctx: &dyn EvalContext,
    ) -> bool {
        let (bl, bm, br) = binders;
        self.node_matches(bl, &spec.lhs, &triple.lhs, ctx)
            && self.edge_matches(bm, &spec.e, &triple.e, ctx)
            && self.node_matches(br, &spec.rhs, &triple.rhs, ctx)
            && self.triple_value_matches(binders, filter, triple, ctx)
    }

    #[inline]
    fn triple_value_matches(
        &self,
        binders: &(
            BindSpec<SimpleGraphTypes>,
            BindSpec<SimpleGraphTypes>,
            BindSpec<SimpleGraphTypes>,
        ),
        spec: &ValueFilter,
        triple: &Triple<SimpleGraphTypes>,
        ctx: &dyn EvalContext,
    ) -> bool {
        match spec {
            ValueFilter::Always => true,
            ValueFilter::Filter(exprs) => {
                let resolver = self.binder.borrow();
                let (bl, be, br) = binders;
                let Triple { lhs, e, rhs } = triple;
                let (lv, ev, rv) = (
                    &self.graph.nodes[lhs.0].value,
                    &self.graph.edges[e.0].value,
                    &self.graph.nodes[rhs.0].value,
                );

                let map: HashMap<_, _> = [(bl, lv), (be, ev), (br, rv)]
                    .into_iter()
                    .filter_map(|(k, v)| {
                        v.as_ref()
                            .map(|v| (Cow::Borrowed(resolver.resolve(&k.0)), v))
                    })
                    .collect();

                let bindings = match map.len() {
                    0 => DatumTupleRef::Empty,
                    1 => {
                        let (key, payload) = map.into_iter().next().unwrap();
                        DatumTupleRef::SingleKey(key, payload)
                    }
                    _ => DatumTupleRef::Bindings(&map),
                };

                exprs.iter().all(|expr| {
                    matches!(expr.evaluate(&bindings, ctx).as_ref(), Value::Boolean(true))
                })
            }
        }
    }

    #[inline]
    fn node_matches(
        &self,
        binder: &BindSpec<SimpleGraphTypes>,
        spec: &NodeFilter<SimpleGraphTypes>,
        node: &GNodeId,
        ctx: &dyn EvalContext,
    ) -> bool {
        let NodeFilter { label, filter } = spec;
        match (label, filter) {
            (LabelFilter::Never, _) => false,
            (LabelFilter::Always, ValueFilter::Always) => true,
            (LabelFilter::Always, v) => self.node_value_matches(binder, v, node, ctx),
            (l, ValueFilter::Always) => self.node_label_matches(binder, l, node, ctx),
            (l, v) => {
                self.node_label_matches(binder, l, node, ctx)
                    && self.node_value_matches(binder, v, node, ctx)
            }
        }
    }

    fn node_label_matches(
        &self,
        _binder: &BindSpec<SimpleGraphTypes>,
        spec: &LabelFilter<SimpleGraphTypes>,
        node: &GNodeId,
        _ctx: &dyn EvalContext,
    ) -> bool {
        match spec {
            LabelFilter::Never => false,
            LabelFilter::Always => true,
            LabelFilter::Named(l) => self.graph.nodes[node.0].labels.0.contains(l),
            LabelFilter::Negated(inner) => {
                !self.node_label_matches(_binder, inner.as_ref(), node, _ctx)
            }
            LabelFilter::Disjunction(inner) => inner
                .iter()
                .any(|l| self.node_label_matches(_binder, l, node, _ctx)),
            LabelFilter::Conjunction(inner) => inner
                .iter()
                .all(|l| self.node_label_matches(_binder, l, node, _ctx)),
        }
    }

    fn node_value_matches(
        &self,
        binder: &BindSpec<SimpleGraphTypes>,
        spec: &ValueFilter,
        node: &GNodeId,
        ctx: &dyn EvalContext,
    ) -> bool {
        match spec {
            ValueFilter::Always => true,
            ValueFilter::Filter(exprs) => {
                let resolver = self.binder.borrow();
                let bindings = match &self.graph.nodes[node.0].value {
                    None => DatumTupleRef::Empty,
                    Some(payload) => {
                        let key = resolver.resolve(&binder.0);
                        DatumTupleRef::SingleKey(key.into(), payload)
                    }
                };
                exprs.iter().all(|expr| {
                    matches!(expr.evaluate(&bindings, ctx).as_ref(), Value::Boolean(true))
                })
            }
        }
    }

    #[inline]
    fn edge_matches(
        &self,
        binder: &BindSpec<SimpleGraphTypes>,
        spec: &EdgeFilter<SimpleGraphTypes>,
        edge: &GEdgeId,
        ctx: &dyn EvalContext,
    ) -> bool {
        let EdgeFilter { label, filter } = spec;
        match (label, filter) {
            (LabelFilter::Never, _) => false,
            (LabelFilter::Always, ValueFilter::Always) => true,
            (LabelFilter::Always, v) => self.edge_value_matches(binder, v, edge, ctx),
            (l, ValueFilter::Always) => self.edge_label_matches(binder, l, edge, ctx),
            (l, v) => {
                self.edge_label_matches(binder, l, edge, ctx)
                    && self.edge_value_matches(binder, v, edge, ctx)
            }
        }
    }

    fn edge_label_matches(
        &self,
        _binder: &BindSpec<SimpleGraphTypes>,
        spec: &LabelFilter<SimpleGraphTypes>,
        edge: &GEdgeId,
        _ctx: &dyn EvalContext,
    ) -> bool {
        match spec {
            LabelFilter::Never => false,
            LabelFilter::Always => true,
            LabelFilter::Named(l) => self.graph.edges[edge.0].labels.0.contains(l),
            LabelFilter::Negated(inner) => {
                !self.edge_label_matches(_binder, inner.as_ref(), edge, _ctx)
            }
            LabelFilter::Disjunction(inner) => inner
                .iter()
                .any(|l| self.edge_label_matches(_binder, l, edge, _ctx)),
            LabelFilter::Conjunction(inner) => inner
                .iter()
                .all(|l| self.edge_label_matches(_binder, l, edge, _ctx)),
        }
    }

    fn edge_value_matches(
        &self,
        binder: &BindSpec<SimpleGraphTypes>,
        spec: &ValueFilter,
        edge: &GEdgeId,
        ctx: &dyn EvalContext,
    ) -> bool {
        match spec {
            ValueFilter::Always => true,
            ValueFilter::Filter(exprs) => {
                let resolver = self.binder.borrow();
                let bindings = match &self.graph.edges[edge.0].value {
                    None => DatumTupleRef::Empty,
                    Some(payload) => {
                        let key = resolver.resolve(&binder.0);
                        DatumTupleRef::SingleKey(key.into(), payload)
                    }
                };
                exprs.iter().all(|expr| {
                    matches!(expr.evaluate(&bindings, ctx).as_ref(), Value::Boolean(true))
                })
            }
        }
    }
}
