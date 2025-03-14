use crate::eval::graph::engine::GraphEngine;
use crate::eval::graph::plan::{
    BindSpec, NodeBinding, NodeMatch, PathPatternBinding, PathPatternNodes,
};
use crate::eval::graph::string_graph::types::StringGraphTypes;
use crate::eval::graph::types::GraphTypes;
use bind_name::BindNameExt;
use fxhash::FxBuildHasher;
use indexmap::IndexMap;
use itertools::Itertools;
use partiql_value::{bag, Tuple, Value};
use plan::{PathBinding, PathMatch, PathPatternMatch};
use std::marker::PhantomData;

pub mod bind_name;
pub(crate) mod engine;
pub mod plan;
pub(crate) mod simple_graph;
pub mod string_graph;
pub mod types;

pub struct GraphEvaluator<GT: GraphTypes, G: GraphEngine<GT>> {
    graph: G,
    phantom: PhantomData<GT>,
}

impl<GT: GraphTypes, G: GraphEngine<GT>> GraphEvaluator<GT, G> {
    pub fn new(graph: G) -> Self {
        Self {
            graph,
            phantom: PhantomData,
        }
    }

    #[inline]
    pub fn eval_path(&self, matcher: PathMatch<GT>) -> PathBinding<GT> {
        let bindings = self.graph.scan(&matcher.spec);
        PathBinding { matcher, bindings }
    }

    #[inline]
    pub fn eval_node(&self, matcher: NodeMatch<GT>) -> NodeBinding<GT> {
        let binding = self.graph.get(&matcher.spec);
        NodeBinding { matcher, binding }
    }

    #[inline]
    pub fn eval_path_pattern(&self, matcher: PathPatternMatch<GT>) -> PathPatternBinding<GT> {
        match matcher {
            PathPatternMatch::Node(n) => self.eval_node(n).into(),
            PathPatternMatch::Match(m) => self.eval_path(m).into(),
            PathPatternMatch::Concat(ms) => ms
                .into_iter()
                .map(|p| self.eval_path_pattern(p))
                .tree_reduce(|l, r| join_bindings(l, r))
                .unwrap(),
        }
    }

    pub fn eval(&self, matcher: &PathPatternMatch<StringGraphTypes>) -> Value {
        // encode plan
        let matcher = self.graph.convert_pathpattern_match(matcher);
        //
        let bindings = self.eval_path_pattern(matcher);

        let mut results = bag![];

        let keys: Vec<BindSpec<StringGraphTypes>> = bindings
            .binders
            .into_iter()
            .map(|b| self.graph.convert_binder(&b))
            .collect();
        for row in bindings.bindings {
            assert_eq!(keys.len(), row.len());
            let mut keys = keys.iter();

            type FxIndexMap<K, V> = IndexMap<K, V, FxBuildHasher>;
            let mut kvs = FxIndexMap::default();

            let k = keys.next().unwrap();
            if !k.0.is_anon() {
                kvs.entry(&k.0)
                    .or_insert_with(|| self.graph.node(&row.head).to_owned().unwrap());
            }

            for (e, r) in row.tail {
                let k = keys.next().unwrap();
                if !k.0.is_anon() {
                    kvs.entry(&k.0)
                        .or_insert_with(|| self.graph.edge(&e).to_owned().unwrap());
                }

                let k = keys.next().unwrap();
                if !k.0.is_anon() {
                    kvs.entry(&k.0)
                        .or_insert_with(|| self.graph.node(&r).to_owned().unwrap());
                }
            }

            results.push(Value::from(Tuple::from_iter(kvs)));
        }

        Value::from(results)
    }
}

fn join_bindings<GT: GraphTypes>(
    l: PathPatternBinding<GT>,
    r: PathPatternBinding<GT>,
) -> PathPatternBinding<GT> {
    debug_assert!(l.binders.len() >= 3);
    debug_assert!(r.binders.len() >= 3);
    debug_assert_eq!(l.bindings.len(), l.bindings.len());
    debug_assert_eq!(r.bindings.len(), r.bindings.len());
    debug_assert_eq!(l.binders.last().unwrap(), r.binders.first().unwrap());

    let lcount = l.binders.len();
    let lidx = l
        .binders
        .iter()
        .enumerate()
        .dedup_by(|(_, x), (_, y)| x == y);
    let ridx = r
        .binders
        .iter()
        .enumerate()
        .map(|(i, e)| (i + lcount, e))
        .dedup_by(|(_, x), (_, y)| x == y);
    let join_on: Vec<_> = lidx
        .cartesian_product(ridx)
        .filter_map(|((li, le), (ri, re))| (le == re).then_some((li, ri)))
        .collect();

    let binders = l
        .binders
        .into_iter()
        .chain(r.binders.into_iter().skip(1))
        .collect();

    let bindings = l
        .bindings
        .iter()
        .cartesian_product(r.bindings.iter())
        .filter_map(|(l, r)| {
            let elts = l.iter().chain(r.iter()).collect::<Vec<_>>();
            for (i, j) in &join_on {
                if elts[*i] != elts[*j] {
                    return None;
                }
            }

            Some(PathPatternNodes {
                head: l.head.clone(),
                tail: l
                    .tail
                    .iter()
                    .cloned()
                    .chain(r.tail.iter().cloned())
                    .collect(),
            })
        })
        .collect();

    PathPatternBinding { binders, bindings }
}
