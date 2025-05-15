use crate::eval::graph::plan::{
    BindSpec, DirectionFilter, GraphPlanConvert, NodeFilter, NodeMatch, PathMode, TripleFilter,
    TripleStepFilter, TripleStepMatch, ValueFilter,
};
use crate::eval::graph::result::{PathPatternNodes, Triple};
use crate::eval::graph::string_graph::StringGraphTypes;
use crate::eval::graph::types::GraphTypes;
use crate::eval::EvalContext;
use fxhash::FxBuildHasher;
use indexmap::IndexSet;
use partiql_value::Value;

/// A graph 'engine'; Exposes scanning, node access, and plan conversion to a target graph.
pub trait GraphEngine<GT: GraphTypes>:
    GraphScan<GT>
    + GraphPlanConvert<StringGraphTypes, GT>
    + GraphPlanConvert<GT, StringGraphTypes>
    + GraphAccess<GT>
    + GraphFilter<GT>
{
}

/// A trait to scan paths and nodes for a graph.
pub trait GraphScan<GT: GraphTypes> {
    fn scan(&self, spec: &TripleStepMatch<GT>, ctx: &dyn EvalContext) -> Vec<Triple<GT>>;
    fn get(&self, spec: &NodeMatch<GT>, ctx: &dyn EvalContext) -> Vec<GT::NodeId>;
}

type FxIndexSet<T> = IndexSet<T, FxBuildHasher>;

pub trait GraphFilter<GT: GraphTypes> {
    fn filter_path_nodes(
        &self,
        binders: &[BindSpec<GT>],
        spec: &ValueFilter,
        bindings: FxIndexSet<PathPatternNodes<GT>>,
        ctx: &dyn EvalContext,
    ) -> FxIndexSet<PathPatternNodes<GT>>;
}

/// A trait to retrieve named nodes and edges for a graph.
pub trait GraphAccess<GT: GraphTypes> {
    fn node(&self, id: &GT::NodeId) -> &Option<Value>;
    fn edge(&self, id: &GT::EdgeId) -> &Option<Value>;
}

/// A train to scan paths and nodes for a triple-based graph.
pub trait TripleScan<GT: GraphTypes> {
    fn scan_directed_from_to(
        &self,
        binders: &(BindSpec<GT>, BindSpec<GT>, BindSpec<GT>),
        spec: &TripleFilter<GT>,
        allow_repeated_nodes: bool,
        filter: &ValueFilter,
        ctx: &dyn EvalContext,
    ) -> impl Iterator<Item = Triple<GT>>;

    fn scan_directed_to_from(
        &self,
        binders: &(BindSpec<GT>, BindSpec<GT>, BindSpec<GT>),
        spec: &TripleFilter<GT>,
        allow_repeated_nodes: bool,
        filter: &ValueFilter,
        ctx: &dyn EvalContext,
    ) -> impl Iterator<Item = Triple<GT>>;

    fn scan_directed_both(
        &self,
        binders: &(BindSpec<GT>, BindSpec<GT>, BindSpec<GT>),
        spec: &TripleFilter<GT>,
        allow_repeated_nodes: bool,
        filter: &ValueFilter,
        ctx: &dyn EvalContext,
    ) -> impl Iterator<Item = Triple<GT>>;

    fn scan_undirected(
        &self,
        binders: &(BindSpec<GT>, BindSpec<GT>, BindSpec<GT>),
        spec: &TripleFilter<GT>,
        allow_repeated_nodes: bool,
        filter: &ValueFilter,
        ctx: &dyn EvalContext,
    ) -> impl Iterator<Item = Triple<GT>>;
    fn get(
        &self,
        binder: &BindSpec<GT>,
        spec: &NodeFilter<GT>,
        ctx: &dyn EvalContext,
    ) -> Vec<GT::NodeId>;
}

impl<T, GT> GraphScan<GT> for T
where
    GT: GraphTypes,
    T: TripleScan<GT>,
{
    fn scan(&self, spec: &TripleStepMatch<GT>, ctx: &dyn EvalContext) -> Vec<Triple<GT>> {
        let TripleStepMatch {
            binders,
            spec: TripleStepFilter { dir, triple },
            filter,
            path_mode,
        } = spec;
        let (to_from, undirected, from_to) = match dir {
            DirectionFilter::L => (true, false, false),
            DirectionFilter::U => (false, true, false),
            DirectionFilter::R => (false, false, true),
            DirectionFilter::LU => (true, true, false),
            DirectionFilter::UR => (false, true, true),
            DirectionFilter::LR => (true, false, true),
            DirectionFilter::LUR => (true, true, true),
        };

        let allow_repeated_nodes = !matches!(path_mode, PathMode::Acyclic);
        let mut result = vec![];
        if undirected {
            result.extend(self.scan_undirected(binders, triple, allow_repeated_nodes, filter, ctx));
        }
        match (from_to, to_from) {
            (true, true) => {
                result.extend(self.scan_directed_both(
                    binders,
                    triple,
                    allow_repeated_nodes,
                    filter,
                    ctx,
                ));
            }
            (true, false) => {
                result.extend(self.scan_directed_from_to(
                    binders,
                    triple,
                    allow_repeated_nodes,
                    filter,
                    ctx,
                ));
            }
            (false, true) => {
                result.extend(self.scan_directed_to_from(
                    binders,
                    triple,
                    allow_repeated_nodes,
                    filter,
                    ctx,
                ));
            }
            (false, false) => {}
        }

        result
    }

    fn get(&self, spec: &NodeMatch<GT>, ctx: &dyn EvalContext) -> Vec<GT::NodeId> {
        let NodeMatch { binder, spec } = spec;
        TripleScan::get(self, binder, spec, ctx)
    }
}
