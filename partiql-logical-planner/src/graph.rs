use num::Integer;
use partiql_ast::ast;
use partiql_ast::ast::{GraphMatchDirection, GraphMatchPatternPart};
use partiql_logical::graph::bind_name::FreshBinder;
use partiql_logical::graph::{
    BindSpec, DirectionFilter, EdgeFilter, EdgeMatch, LabelFilter, NodeFilter, NodeMatch,
    PathMatch, PathPattern, PathPatternMatch, StepFilter, TripleFilter, ValueFilter,
};
use std::mem::take;

#[macro_export]
macro_rules! not_yet_implemented_result {
    ($msg:expr) => {
        return std::result::Result::Err($msg.to_string());
    };
}

#[derive(Debug, Default)]
pub(crate) struct GraphToLogical {
    graph_id: FreshBinder,
}

#[derive(Debug, Clone)]
enum MatchElement {
    Node(NodeMatch),
    Edge(DirectionFilter, EdgeMatch),
    #[allow(dead_code)] // TODO with sub-graphs in graph MATCH
    Pattern(Vec<MatchElement>),
}

#[derive(Debug)]
struct Normalize<'a> {
    head: Option<NodeMatch>,
    tail: Vec<(DirectionFilter, EdgeMatch, Option<NodeMatch>)>,
    fresh_binder: &'a FreshBinder,
}

impl<'a> Normalize<'a> {
    pub fn new(fresh_binder: &'a FreshBinder) -> Self {
        Self {
            head: Default::default(),
            tail: Default::default(),
            fresh_binder,
        }
    }

    pub fn normalize(mut self, elements: Vec<MatchElement>) -> Result<Vec<PathPattern>, String> {
        self.do_normalize(elements)
    }

    fn fresh_node(&mut self) -> NodeMatch {
        NodeMatch {
            binder: BindSpec(self.fresh_binder.node()),
            spec: NodeFilter {
                label: LabelFilter::Always,
                filter: ValueFilter::Always,
            },
        }
    }

    fn flush(&mut self) -> Option<PathPattern> {
        if self.head.is_none() {
            debug_assert!(self.tail.is_empty());
            return None;
        }

        if !self.tail.is_empty() && self.tail.last().unwrap().2.is_none() {
            let fresh = self.fresh_node();
            self.tail.last_mut().unwrap().2 = Some(fresh);
        }

        let head = self.head.take().unwrap();
        let tail = take(&mut self.tail)
            .into_iter()
            .map(|(d, e, n)| (d, e, n.unwrap()))
            .collect::<Vec<_>>();

        Some(PathPattern { head, tail })
    }

    fn do_normalize(&mut self, elements: Vec<MatchElement>) -> Result<Vec<PathPattern>, String> {
        let mut path_patterns = vec![];

        for elt in elements {
            match elt {
                MatchElement::Node(n) => {
                    if self.head.is_none() {
                        self.head = Some(n);
                    } else {
                        match self.tail.last_mut() {
                            Some((_td, _te, tn)) => {
                                if tn.is_none() {
                                    *tn = Some(n);
                                } else {
                                    todo!("Deal with adjacent nodes")
                                }
                            }
                            None => {
                                todo!("Deal with adjacent nodes")
                            }
                        }
                    }
                }
                MatchElement::Edge(d, e) => {
                    if self.head.is_none() {
                        self.head = Some(self.fresh_node());
                    }
                    self.tail.push((d, e, None));
                }
                MatchElement::Pattern(p) => {
                    path_patterns.extend(self.flush());
                    path_patterns.extend(self.do_normalize(p)?);
                }
            }
        }

        path_patterns.extend(self.flush());

        Ok(path_patterns)
    }
}

impl GraphToLogical {
    fn normalize(&self, elements: Vec<MatchElement>) -> Result<Vec<PathPattern>, String> {
        Normalize::new(&self.graph_id).normalize(elements)
    }

    fn expand(&self, paths: Vec<PathPattern>) -> Result<Vec<PathPattern>, String> {
        // TODO handle expansion as described in 6.3 of https://arxiv.org/pdf/2112.06217
        // TODO   this will enable alternation and quantifiers
        Ok(paths)
    }

    fn plan(&self, paths: Vec<PathPattern>) -> Result<PathPatternMatch, String> {
        debug_assert!(paths.len() == 1);
        let path_pattern = &paths[0];
        // pattern at this point should be a node, or a series of node-[edge-node]+
        debug_assert!((1 + (2 * path_pattern.tail.len())).is_odd());

        if path_pattern.tail.is_empty() {
            Ok(PathPatternMatch::Node(path_pattern.head.clone()))
        } else {
            let mut paths = vec![];
            let mut head = &path_pattern.head;
            for (d, e, n) in &path_pattern.tail {
                let binders = (head.binder.clone(), e.binder.clone(), n.binder.clone());
                let spec = StepFilter {
                    dir: d.clone(),
                    triple: TripleFilter {
                        lhs: head.spec.clone(),
                        e: e.spec.clone(),
                        rhs: n.spec.clone(),
                    },
                };
                paths.push(PathPatternMatch::Match(PathMatch { binders, spec }));
                head = n;
            }
            if paths.len() == 1 {
                let path = paths.into_iter().next().unwrap();
                Ok(path)
            } else {
                Ok(PathPatternMatch::Concat(paths))
            }
        }
    }
    pub(crate) fn plan_graph_match(
        &self,
        match_expr: &ast::GraphMatchExpr,
    ) -> Result<PathPatternMatch, String> {
        if match_expr.selector.is_some() {
            not_yet_implemented_result!("MATCH expression selectors are not yet supported.");
        }

        if match_expr.patterns.len() != 1 {
            not_yet_implemented_result!(
                "MATCH expression with multiple patterns are not yet supported."
            );
        }

        let first_pattern = &match_expr.patterns[0].node;
        let pattern = self.plan_graph_pattern(first_pattern)?;
        let normalized = self.normalize(pattern)?;
        let expanded = self.expand(normalized)?;
        self.plan(expanded)
    }

    fn plan_graph_pattern(
        &self,
        pattern: &ast::GraphMatchPattern,
    ) -> Result<Vec<MatchElement>, String> {
        if pattern.restrictor.is_some() {
            not_yet_implemented_result!("MATCH pattern restrictors are not yet supported.");
        }
        if pattern.quantifier.is_some() {
            not_yet_implemented_result!("MATCH pattern quantifiers are not yet supported.");
        }
        if pattern.prefilter.is_some() {
            not_yet_implemented_result!("MATCH pattern prefilters are not yet supported.");
        }
        if pattern.variable.is_some() {
            not_yet_implemented_result!("MATCH pattern path variables are not yet supported.");
        }

        let parts = pattern
            .parts
            .iter()
            .map(|p| self.plan_graph_pattern_part(p));
        let result: Result<Vec<_>, _> = parts.collect();

        result.map(|r| r.into_iter().flatten().collect())
    }

    fn plan_graph_pattern_part(
        &self,
        part: &ast::GraphMatchPatternPart,
    ) -> Result<Vec<MatchElement>, String> {
        match part {
            GraphMatchPatternPart::Node(n) => self.plan_graph_pattern_part_node(&n.node),
            GraphMatchPatternPart::Edge(e) => self.plan_graph_pattern_part_edge(&e.node),
            GraphMatchPatternPart::Pattern(pattern) => self.plan_graph_pattern(&pattern.node),
        }
    }

    fn plan_graph_pattern_label(
        &self,
        label: &Option<Vec<ast::SymbolPrimitive>>,
    ) -> Result<LabelFilter, String> {
        match label {
            None => Ok(LabelFilter::Always),
            Some(labels) => {
                if labels.len() != 1 {
                    not_yet_implemented_result!(
                        "MATCH expression with multiple patterns are not yet supported."
                    );
                }
                Ok(LabelFilter::Named(labels[0].value.clone()))
            }
        }
    }

    fn plan_graph_pattern_part_node(
        &self,
        node: &ast::GraphMatchNode,
    ) -> Result<Vec<MatchElement>, String> {
        if node.prefilter.is_some() {
            not_yet_implemented_result!("MATCH node prefilters are not yet supported.");
        }
        let binder = match &node.variable {
            None => self.graph_id.node(),
            Some(v) => v.value.clone(),
        };
        let node_match = NodeMatch {
            binder: BindSpec(binder),
            spec: NodeFilter {
                label: self.plan_graph_pattern_label(&node.label)?,
                filter: ValueFilter::Always,
            },
        };
        Ok(vec![MatchElement::Node(node_match)])
    }

    fn plan_graph_pattern_part_edge(
        &self,
        edge: &ast::GraphMatchEdge,
    ) -> Result<Vec<MatchElement>, String> {
        if edge.quantifier.is_some() {
            not_yet_implemented_result!("MATCH edge quantifiers are not yet supported.");
        }
        if edge.prefilter.is_some() {
            not_yet_implemented_result!("MATCH edge prefilters are not yet supported.");
        }
        let direction = match &edge.direction {
            GraphMatchDirection::Left => DirectionFilter::L,
            GraphMatchDirection::Undirected => DirectionFilter::U,
            GraphMatchDirection::Right => DirectionFilter::R,
            GraphMatchDirection::LeftOrUndirected => DirectionFilter::LU,
            GraphMatchDirection::UndirectedOrRight => DirectionFilter::UR,
            GraphMatchDirection::LeftOrRight => DirectionFilter::LR,
            GraphMatchDirection::LeftOrUndirectedOrRight => DirectionFilter::LUR,
        };
        let binder = match &edge.variable {
            None => self.graph_id.node(),
            Some(v) => v.value.clone(),
        };
        let edge_match = EdgeMatch {
            binder: BindSpec(binder),
            spec: EdgeFilter {
                label: self.plan_graph_pattern_label(&edge.label)?,
                filter: ValueFilter::Always,
            },
        };
        Ok(vec![MatchElement::Edge(direction, edge_match)])
    }
}
