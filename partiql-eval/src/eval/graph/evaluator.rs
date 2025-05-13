use crate::eval::graph::engine::GraphEngine;
use crate::eval::graph::result::{
    GraphElement, NodeBinding, PathBinding, PathPatternBinding, PathPatternNodes, Triple,
};
use partiql_logical::graph::bind_name::BindNameExt;

use fxhash::FxBuildHasher;
use indexmap::IndexMap;
use itertools::Itertools;

use crate::eval::graph::plan::{BindSpec, NodeMatch, PathMode, PathPatternMatch, TripleStepMatch};
use crate::eval::graph::string_graph::StringGraphTypes;
use crate::eval::graph::types::GraphTypes;
use crate::eval::EvalContext;
use partiql_value::{Bag, Tuple, Value};
use std::marker::PhantomData;

/// An evaluator for [`PathPatternMatch`]s over a graph.
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

    pub fn eval(
        &self,
        matcher: &PathPatternMatch<StringGraphTypes>,
        ctx: &dyn EvalContext,
    ) -> Value {
        // encode plan
        let matcher = self.graph.convert_pathpattern_match(matcher);
        // query for triple bindings
        let bindings = self.eval_path_pattern(matcher, ctx);

        // decode binders to tuple keys for result tuples
        let keys: Vec<BindSpec<StringGraphTypes>> = bindings
            .binders
            .into_iter()
            .map(|b| self.graph.convert_binder(&b))
            .collect();

        let mut results = Vec::with_capacity(bindings.bindings.len());
        // for all result bindings
        for row in bindings.bindings {
            debug_assert_eq!(keys.len(), row.len());

            type FxIndexMap<K, V> = IndexMap<K, V, FxBuildHasher>;
            let kvs: FxIndexMap<_, _> = keys
                .iter()
                .zip(row.iter())
                // remove anonymous bindings
                .filter(|(k, _)| !k.0.is_anon())
                // keep only the first of each binding name
                .dedup_by(|(k1, _), (k2, _)| k1 == k2)
                // filter out matches that have no or `MISSING` value
                .filter_map(|(k, elt)| {
                    let value = match elt {
                        GraphElement::Node(n) => self.graph.node(n),
                        GraphElement::Edge(e) => self.graph.edge(e),
                    };
                    match value {
                        None => None,
                        Some(Value::Missing) => None,
                        Some(v) => Some((&k.0, v.clone())),
                    }
                })
                // collect into an IndexMap to dedup paths
                .collect();

            // transform pairs of (&String, Value) into a tuple
            results.push(Value::from(Tuple::from_iter(kvs)));
        }

        Value::from(Bag::from(results))
    }

    #[inline]
    fn eval_triple_step(
        &self,
        matcher: TripleStepMatch<GT>,
        ctx: &dyn EvalContext,
    ) -> PathBinding<GT> {
        let bindings = self.graph.scan(&matcher, ctx);
        PathBinding { matcher, bindings }
    }

    #[inline]
    fn eval_node(&self, matcher: NodeMatch<GT>, ctx: &dyn EvalContext) -> NodeBinding<GT> {
        let binding = self.graph.get(&matcher, ctx);
        NodeBinding { matcher, binding }
    }

    #[inline]
    fn eval_path_pattern(
        &self,
        matcher: PathPatternMatch<GT>,
        ctx: &dyn EvalContext,
    ) -> PathPatternBinding<GT> {
        match matcher {
            PathPatternMatch::Node(n) => self.eval_node(n, ctx).into(),
            PathPatternMatch::Match(m) => {
                let PathBinding {
                    matcher,
                    mut bindings,
                } = self.eval_triple_step(m, ctx);

                // if edge is cyclic, assure identity for identical binder names; for longer paths this is handled by `join_bindings`
                let (n1, _, n2) = &matcher.binders;
                if n1 == n2 {
                    bindings.retain(|Triple { lhs, e: _, rhs }| lhs == rhs)
                }

                PathBinding { matcher, bindings }.into()
            }
            PathPatternMatch::Concat(ms, filter, path_mode) => {
                let PathPatternBinding { binders, bindings } = ms
                    .into_iter()
                    .map(|p| self.eval_path_pattern(p, ctx))
                    .tree_reduce(|l, r| join_bindings(l, r, path_mode))
                    .unwrap();

                let bindings = self
                    .graph
                    .filter_path_nodes(&binders, &filter, bindings, ctx);

                PathPatternBinding { binders, bindings }
            }
        }
    }
}

// 'join' adjacent [`PathPatternBindings`] with an equi-join on identical binder names
fn join_bindings<GT: GraphTypes>(
    l: PathPatternBinding<GT>,
    r: PathPatternBinding<GT>,
    path_mode: PathMode,
) -> PathPatternBinding<GT> {
    debug_assert!(l.binders.len() >= 3);
    debug_assert!(r.binders.len() >= 3);
    debug_assert_eq!(l.binders.last().unwrap(), r.binders.first().unwrap());

    let lcount = l.binders.len();
    // map l binders to integers and dedup
    //  e.g., (a) -[b]-> (c) - (a) - (z) will result in something like [(a, 0), (b,1), (c,2), (z, 4)]
    let lidx = l
        .binders
        .iter()
        .enumerate()
        .dedup_by(|(_, x), (_, y)| x == y);

    // map r binders to integers and dedup, starting at 1+ highest lidx
    //  e.g., (q) -[r]-> (s) - (q) - (z) will result in something like [(q, 5), (r,6), (s,7), (z, 9)]
    let ridx = r
        .binders
        .iter()
        .enumerate()
        .map(|(i, e)| (i + lcount, e))
        .dedup_by(|(_, x), (_, y)| x == y);

    // collect the integers corresponding to matching binders on which to join from lidx & ridx.
    // e.g., In the above examples, both l & r mention `z` (at `4` and `9`, respectively): vec![(4, 9)]
    let join_on: Vec<_> = lidx
        .cartesian_product(ridx)
        .filter_map(|((li, le), (ri, re))| (le == re).then_some((li, ri)))
        .collect();

    // cartesian product of all bindings in l with all bindings in r (cross join)
    let lbindings = l.bindings.iter();
    let rbindings = r.bindings.iter();
    let bindings = lbindings.cartesian_product(rbindings);

    let bindings = bindings
        .filter_map(|(l, r)| {
            let elts = l.iter().chain(r.iter()).collect::<Vec<_>>();
            // for all pairs that are expected to match, filter out those that do not
            for (i, j) in &join_on {
                if elts[*i] != elts[*j] {
                    return None;
                }
            }

            // Filter according to path_mode
            let path_elts = l.iter().chain(r.iter().skip(1));
            match path_mode {
                PathMode::Walk => (),
                PathMode::Trail => {
                    // No duplicate edges allowed
                    if path_elts
                        .filter(|elt| matches!(elt, GraphElement::Edge(_)))
                        .duplicates()
                        .any(|_| true)
                    {
                        return None;
                    }
                }
                PathMode::Acyclic => {
                    // No duplicate nodes allowed
                    if path_elts
                        .filter(|elt| matches!(elt, GraphElement::Node(_)))
                        .duplicates()
                        .any(|_| true)
                    {
                        return None;
                    }
                }
                PathMode::Simple => {
                    // No duplicate nodes allowed except that first&last nodes in the pattern may match each other
                    let path_elts = path_elts.collect_vec();
                    // all elements except the first
                    let tail = path_elts
                        .iter()
                        .skip(1)
                        .filter(|elt| matches!(elt, GraphElement::Node(_)));
                    let head = path_elts
                        .iter()
                        .rev()
                        .skip(1)
                        .rev()
                        .filter(|elt| matches!(elt, GraphElement::Node(_)));

                    if tail.duplicates().any(|_| true) || head.duplicates().any(|_| true) {
                        return None;
                    }
                }
            }

            // join condition has succeeded; join l & r at l.tail.last() == r.head()
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

    // l binders followed by r tail binders (l.last() and r.first() are expected to be identical)
    let binders: Vec<_> = l
        .binders
        .into_iter()
        .chain(r.binders.into_iter().skip(1))
        .collect();

    PathPatternBinding { binders, bindings }
}
