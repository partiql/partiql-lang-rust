use crate::Value;
use lasso::{Key, Rodeo, RodeoReader, Spur};
#[cfg(feature = "serde")]
use serde::{Deserialize, Deserializer, Serialize, Serializer};
use std::cmp::Ordering;
use std::collections::HashSet;
use std::fmt::{Debug, Formatter};
use std::hash::{Hash, Hasher};
use std::rc::Rc;

#[derive(Clone, Debug)]
pub enum Graph {
    Simple(Rc<SimpleGraph>),
}

#[cfg(feature = "serde")]
impl Serialize for Graph {
    fn serialize<S>(&self, _: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        todo!("Serialize for Graph")
    }
}

#[cfg(feature = "serde")]
impl<'de> Deserialize<'de> for Graph {
    fn deserialize<D>(_: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        todo!("Deserialize for Graph")
    }
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

#[derive(Debug, Copy, Clone, Hash, PartialEq, Eq)]
pub struct GNodeId(pub usize);

#[derive(Debug, Copy, Clone, Hash, PartialEq, Eq)]
pub struct GEdgeId(pub usize);

#[derive(Debug, Copy, Clone, Hash, PartialEq, Eq)]
pub struct GLabelId(pub Spur);

#[derive(Clone, PartialEq, Eq)]
pub struct GLabels(pub HashSet<GLabelId>);

#[derive(Clone, PartialEq, Eq)]
pub struct GElem {
    pub value: Option<Value>,
    pub labels: GLabels,
}

impl GElem {
    pub fn new(value: Option<Value>, labels: GLabels) -> Self {
        GElem { value, labels }
    }
}

pub struct SimpleGraph {
    pub nodes: Vec<GElem>,
    pub edges: Vec<GElem>,
    pub g_dir: Vec<(GNodeId, GEdgeId, GNodeId)>,
    pub g_undir: Vec<(GNodeId, GEdgeId, GNodeId)>,
    pub labels: RodeoReader,
}

impl Debug for SimpleGraph {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("SimpleGraph")
            .field("nodes", &DebugGElems("node", &self.nodes, &self.labels))
            .field("edges", &DebugGElems("edge", &self.edges, &self.labels))
            .field("directed", &self.g_dir)
            .field("undirected", &self.g_undir)
            .finish()
    }
}

pub struct DebugGElems<'a>(&'a str, &'a Vec<GElem>, &'a RodeoReader);
pub struct DebugGElem<'a>(usize, &'a str, &'a GElem, &'a RodeoReader);

impl Debug for DebugGElems<'_> {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        let mut l = f.debug_list();

        for (id, gelem) in self.1.iter().enumerate() {
            l.entry(&DebugGElem(id, self.0, gelem, self.2));
        }

        l.finish()
    }
}

impl Debug for DebugGElem<'_> {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        let DebugGElem(id, name, GElem { value, labels, .. }, reader) = self;
        let labels = labels
            .0
            .iter()
            .map(|id| reader.resolve(&id.0))
            .collect::<Vec<_>>();
        f.debug_struct(name)
            .field("id", id)
            .field("value", value)
            .field("labels", &labels)
            .finish()
    }
}

pub enum EdgeSpec {
    Directed(String, String),   // from node, to node
    Undirected(String, String), // node, node
}

type NodeSpec = (Vec<String>, Vec<HashSet<String>>, Vec<Option<Value>>);
#[allow(clippy::type_complexity)]
type EdgesSpec = (
    Vec<String>,
    Vec<HashSet<String>>,
    Vec<EdgeSpec>,
    Vec<Option<Value>>,
);
impl SimpleGraph {
    pub fn from_spec(node_specs: NodeSpec, edge_specs: EdgesSpec) -> Self {
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
            nodes.push(GElem::new(value, GLabels(labels)));
        }

        let mut edge_ids = Rodeo::default();
        let mut edges: Vec<GElem> = vec![];

        let mut g_dir = vec![];
        let mut g_undir = vec![];

        // Process all edges
        let (ids, labels, ends, values) = edge_specs;
        assert_eq!(ids.len(), labels.len());
        assert_eq!(ids.len(), ends.len());
        assert_eq!(ids.len(), values.len());
        for (((id, labels), edge_spec), value) in ids
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
            edges.push(GElem::new(value, GLabels(labels)));

            match edge_spec {
                EdgeSpec::Directed(l, r) => {
                    let eidx = GEdgeId(eidx);
                    let lidx = GNodeId(node_ids.get(l).expect("expected node").into_usize());
                    let ridx = GNodeId(node_ids.get(r).expect("expected node").into_usize());
                    g_dir.push((lidx, eidx, ridx));
                }
                EdgeSpec::Undirected(l, r) => {
                    let eidx = GEdgeId(eidx);
                    let lidx = GNodeId(node_ids.get(l).expect("expected node").into_usize());
                    let ridx = GNodeId(node_ids.get(r).expect("expected node").into_usize());
                    g_undir.push((lidx, eidx, ridx));
                }
            }
        }

        let labels = label_ids.into_reader();
        SimpleGraph {
            nodes,
            edges,
            labels,
            g_dir,
            g_undir,
        }
    }
}
