use crate::eval::graph::plan::{DirSpec, EdgeSpec, NodeSpec, StepSpec, Triple, TripleSpec};
use crate::eval::graph::string_graph::types::StringGraphTypes;
use crate::eval::graph::types::{GraphTypeMapper, GraphTypes};
use partiql_value::Value;

pub trait GraphEngine<GT: GraphTypes>:
    GraphScan<GT>
    + GraphTypeMapper<StringGraphTypes, GT>
    + GraphTypeMapper<GT, StringGraphTypes>
    + GraphAccess<GT>
{
}

pub trait GraphScan<GT: GraphTypes> {
    fn scan(&self, spec: &StepSpec<GT>) -> Vec<Triple<GT>>;
    fn get(&self, spec: &NodeSpec<GT>) -> Vec<GT::NodeId>;
}
pub trait GraphAccess<GT: GraphTypes> {
    fn node(&self, id: &GT::NodeId) -> &Option<Value>;
    fn edge(&self, id: &GT::EdgeId) -> &Option<Value>;
}
pub trait GraphScanImpl<GT: GraphTypes> {
    fn scan_directed_from_to(&self, spec: &TripleSpec<GT>) -> impl Iterator<Item = Triple<GT>>;

    fn scan_directed_to_from(&self, spec: &TripleSpec<GT>) -> impl Iterator<Item = Triple<GT>>;

    fn scan_directed_both(&self, spec: &TripleSpec<GT>) -> impl Iterator<Item = Triple<GT>>;

    fn scan_undirected(&self, spec: &TripleSpec<GT>) -> impl Iterator<Item = Triple<GT>>;
    fn get(&self, spec: &NodeSpec<GT>) -> Vec<GT::NodeId>;
}

impl<T, GT> GraphScan<GT> for T
where
    GT: GraphTypes,
    T: GraphScanImpl<GT>,
{
    fn scan(&self, spec: &StepSpec<GT>) -> Vec<Triple<GT>> {
        let StepSpec { dir, triple } = spec;
        let (to_from, undirected, from_to) = match dir {
            DirSpec::L => (true, false, false),
            DirSpec::U => (false, true, false),
            DirSpec::R => (false, false, true),
            DirSpec::LU => (true, true, false),
            DirSpec::UR => (false, true, true),
            DirSpec::LR => (true, false, true),
            DirSpec::LUR => (true, true, true),
        };

        let mut result = vec![];
        if undirected {
            result.extend(self.scan_undirected(triple));
        }
        match (from_to, to_from) {
            (true, true) => {
                result.extend(self.scan_directed_both(triple));
            }
            (true, false) => {
                result.extend(self.scan_directed_from_to(triple));
            }
            (false, true) => {
                result.extend(self.scan_directed_to_from(triple));
            }
            (false, false) => {}
        }

        result
    }

    fn get(&self, spec: &NodeSpec<GT>) -> Vec<GT::NodeId> {
        GraphScanImpl::get(self, spec)
    }
}

pub(crate) trait TripleMatcher<GT: GraphTypes> {
    fn matches(&self, spec: &TripleSpec<GT>, triple: &Triple<GT>) -> bool;
}

pub(crate) trait NodeMatcher<GT: GraphTypes> {
    fn matches(&self, spec: &NodeSpec<GT>, node: &GT::NodeId) -> bool;
}

pub(crate) trait EdgeMatcher<GT: GraphTypes> {
    fn matches(&self, spec: &EdgeSpec<GT>, edge: &GT::EdgeId) -> bool;
}

#[inline]
pub(crate) fn build_triple<GT: GraphTypes>(
    (l, e, r): &(GT::NodeId, GT::EdgeId, GT::NodeId),
) -> Triple<GT> {
    Triple {
        lhs: l.clone(),
        e: e.clone(),
        rhs: r.clone(),
    }
}

#[inline]
pub(crate) fn reverse_triple<GT: GraphTypes>(
    (l, e, r): &(GT::NodeId, GT::EdgeId, GT::NodeId),
) -> Triple<GT> {
    Triple {
        lhs: r.clone(),
        e: e.clone(),
        rhs: l.clone(),
    }
}
