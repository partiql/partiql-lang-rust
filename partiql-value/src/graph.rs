use crate::graph::Graph::Simple;
use crate::{Tuple, Value};
use lasso::{Key, Rodeo, RodeoReader, Spur};
use petgraph::{Directed, Undirected};
#[cfg(feature = "serde")]
use serde::{Deserialize, Serialize};
use std::cmp::Ordering;
use std::collections::{HashMap, HashSet};
use std::fmt::Debug;
use std::hash::{Hash, Hasher};
use std::rc::Rc;

#[derive(Clone, Debug)]
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
pub enum Graph {
    Simple(Rc<SimpleGraph>),
}

impl Hash for Graph {
    fn hash<H: Hasher>(&self, _state: &mut H) {
        todo!("Hash for Graph")
    }
}

impl Eq for Graph {}
impl PartialEq for Graph {
    fn eq(&self, _other: &Self) -> bool {
        todo!("PartialEq for Graph")
    }
}

impl PartialOrd for Graph {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}

impl Ord for Graph {
    fn cmp(&self, _other: &Self) -> Ordering {
        todo!("Ord for Graph")
    }
}

#[derive(Clone, Hash, PartialEq, Eq)]
pub struct GNodeId(pub usize);

#[derive(Clone, Hash, PartialEq, Eq)]
pub struct GEdgeId(pub usize);

#[derive(Clone, Hash, PartialEq, Eq)]
pub struct GLabelId(pub Spur);

#[derive(Clone, PartialEq, Eq)]
pub struct GLabels(pub HashSet<GLabelId>);

#[derive(Clone, PartialEq, Eq)]
pub struct GElem(pub Option<Value>, pub GLabels);

#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
pub struct SimpleGraph {
    directed: petgraph::stable_graph::StableGraph<GNodeId, GEdgeId, Directed>,
    undirected: petgraph::stable_graph::StableGraph<GNodeId, GEdgeId, Undirected>,
    nodes: Vec<GElem>,
    edges: Vec<GElem>,
    labels: RodeoReader,
}

impl Debug for SimpleGraph {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        todo!("Debutg for SimpleGraph")
    }
}

pub enum EdgeSpec {
    Directed(String, String),   // from node, to node
    Undirected(String, String), // node, node
}

impl SimpleGraph {
    pub fn from_spec(
        node_specs: (Vec<String>, Vec<HashSet<String>>, Vec<Option<Value>>),
        edge_specs: (
            Vec<String>,
            Vec<HashSet<String>>,
            Vec<EdgeSpec>,
            Vec<Option<Value>>,
        ),
    ) -> Self {
        let mut node_ids = Rodeo::default();
        let mut label_ids = Rodeo::default();
        let mut nodes: Vec<GElem> = vec![];

        // Process all nodes
        let (ids, labels, values) = node_specs;
        assert_eq!(ids.len(), labels.len());
        assert_eq!(ids.len(), values.len());
        for ((id, labels), value) in ids
            .into_iter()
            .zip(labels.into_iter())
            .zip(values.into_iter())
        {
            let nid = node_ids.get_or_intern(id);
            let labels: HashSet<_> = labels
                .into_iter()
                .map(|l| GLabelId(label_ids.get_or_intern(l)))
                .collect();
            let nidx = nid.into_usize();
            assert_eq!(nodes.len(), nidx);
            nodes.push(GElem(value, GLabels(labels)));
        }

        let mut edge_ids = Rodeo::default();
        let mut edges: Vec<GElem> = vec![];

        let mut directed: petgraph::stable_graph::StableGraph<GNodeId, GEdgeId, Directed> =
            Default::default();
        let mut directed_contains = HashMap::new();
        let mut undirected: petgraph::stable_graph::StableGraph<GNodeId, GEdgeId, Undirected> =
            Default::default();
        let mut undirected_contains = HashMap::new();

        // Process all edges
        let (ids, labels, ends, values) = edge_specs;
        assert_eq!(ids.len(), labels.len());
        assert_eq!(ids.len(), ends.len());
        assert_eq!(ids.len(), values.len());
        for ((((id, labels), edge_spec), value)) in ids
            .into_iter()
            .zip(labels.into_iter())
            .zip(ends.into_iter())
            .zip(values.into_iter())
        {
            let eid = edge_ids.get_or_intern(id);
            let labels: HashSet<_> = labels
                .into_iter()
                .map(|l| GLabelId(label_ids.get_or_intern(l)))
                .collect();

            let eidx = eid.into_usize();
            assert_eq!(edges.len(), eidx);
            edges.push(GElem(value, GLabels(labels)));

            let e = match edge_spec {
                EdgeSpec::Directed(l, r) => {
                    let mut get_or_insert = |idx: GNodeId| {
                        directed_contains
                            .entry(idx)
                            .or_insert_with_key(|idx| directed.add_node(idx.clone()))
                            .clone()
                    };
                    let lidx = GNodeId(node_ids.get(l).expect("expected node").into_usize());
                    let ridx = GNodeId(node_ids.get(r).expect("expected node").into_usize());
                    let l = get_or_insert(lidx);
                    let r = get_or_insert(ridx);
                    directed.add_edge(l, r, GEdgeId(eidx))
                }
                EdgeSpec::Undirected(l, r) => {
                    let mut get_or_insert = |idx: GNodeId| {
                        undirected_contains
                            .entry(idx)
                            .or_insert_with_key(|idx| undirected.add_node(idx.clone()))
                            .clone()
                    };
                    let lidx = GNodeId(node_ids.get(l).expect("expected node").into_usize());
                    let ridx = GNodeId(node_ids.get(r).expect("expected node").into_usize());
                    let l = get_or_insert(lidx);
                    let r = get_or_insert(ridx);
                    undirected.add_edge(l, r, GEdgeId(eidx))
                }
            };
        }

        let labels = label_ids.into_reader();
        SimpleGraph {
            directed,
            undirected,
            nodes,
            edges,
            labels,
        }
    }
}
