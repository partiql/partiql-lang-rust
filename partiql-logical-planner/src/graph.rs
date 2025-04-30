use num::Integer;
use partiql_ast::ast;

use crate::lower::extract_vexpr_by_id;
use partiql_common::node::{IdAnnotated, NodeId};
use partiql_logical::graph::bind_name::FreshBinder;
use partiql_logical::graph::{
    BindSpec, DirectionFilter, EdgeFilter, EdgeMatch, LabelFilter, NodeFilter, NodeMatch,
    PathMatch, PathPattern, PathPatternMatch, StepFilter, TripleFilter, ValueFilter,
};
use partiql_logical::ValueExpr;
use std::mem::take;

#[macro_export]
macro_rules! not_yet_implemented_result {
    ($msg:expr) => {
        return std::result::Result::Err($msg.to_string());
    };
}

#[derive(Debug)]
pub(crate) struct GraphToLogical {
    graph_id: FreshBinder,
    env: Vec<(NodeId, ValueExpr)>,
}

impl GraphToLogical {
    pub fn new(env: Vec<(NodeId, ValueExpr)>) -> Self {
        Self {
            graph_id: Default::default(),
            env,
        }
    }
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
    fn normalize(&mut self, elements: Vec<MatchElement>) -> Result<Vec<PathPattern>, String> {
        Normalize::new(&self.graph_id).normalize(elements)
    }

    fn expand(&mut self, paths: Vec<PathPattern>) -> Result<Vec<PathPattern>, String> {
        // TODO handle expansion as described in 6.3 of https://arxiv.org/pdf/2112.06217
        // TODO   this will enable alternation and quantifiers
        Ok(paths)
    }

    fn plan(&mut self, paths: Vec<PathPattern>) -> Result<PathPatternMatch, String> {
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
        mut self,
        graph_match: &ast::GraphMatch,
    ) -> Result<PathPatternMatch, String> {
        if graph_match.shape.cols.is_some() {
            not_yet_implemented_result!("MATCH expression COLUMNS are not yet supported.");
        }
        if graph_match.shape.export.is_some() {
            not_yet_implemented_result!("MATCH expression EXPORT are not yet supported.");
        }
        if graph_match.shape.rows.is_some() {
            not_yet_implemented_result!("MATCH expression ROWS are not yet supported.");
        }

        let pattern = self.plan_graph_pattern(&graph_match.pattern)?;
        let normalized = self.normalize(pattern)?;
        let expanded = self.expand(normalized)?;
        let result = self.plan(expanded);
        debug_assert!(self.env.is_empty());
        result
    }

    fn plan_graph_pattern(
        &mut self,
        pattern: &ast::GraphPattern,
    ) -> Result<Vec<MatchElement>, String> {
        if pattern.mode.is_some() {
            not_yet_implemented_result!("MATCH expression MATCH MODE is not yet supported.");
        }
        if pattern.keep.is_some() {
            not_yet_implemented_result!("MATCH expression KEEP is not yet supported.");
        }
        if pattern.where_clause.is_some() {
            not_yet_implemented_result!("MATCH expression pattern WHERE is not yet supported.");
        }

        if pattern.patterns.len() != 1 {
            not_yet_implemented_result!(
                "MATCH expression with multiple patterns are not yet supported."
            );
        }

        let first_pattern = &pattern.patterns[0];
        self.plan_graph_path_pattern(first_pattern)
    }

    fn plan_graph_path_pattern(
        &mut self,
        pattern: &ast::GraphPathPattern,
    ) -> Result<Vec<MatchElement>, String> {
        if pattern.prefix.is_some() {
            not_yet_implemented_result!("MATCH pattern SEARCH/MODE prefix are not yet supported.");
        }
        if pattern.variable.is_some() {
            not_yet_implemented_result!("MATCH pattern path variables are not yet supported.");
        }

        self.plan_graph_match_path_pattern(&pattern.path)
    }

    fn plan_graph_subpath_pattern(
        &mut self,
        pattern: &ast::GraphPathSubPattern,
    ) -> Result<Vec<MatchElement>, String> {
        if pattern.mode.is_some() {
            not_yet_implemented_result!("MATCH pattern MODE prefix are not yet supported.");
        }
        if pattern.variable.is_some() {
            not_yet_implemented_result!("MATCH pattern path variables are not yet supported.");
        }
        if pattern.where_clause.is_some() {
            not_yet_implemented_result!("MATCH expression subpath WHERE is not yet supported.");
        }

        self.plan_graph_match_path_pattern(&pattern.path)
    }

    fn plan_graph_match_path_pattern(
        &mut self,
        pattern: &ast::GraphMatchPathPattern,
    ) -> Result<Vec<MatchElement>, String> {
        match pattern {
            ast::GraphMatchPathPattern::Path(path) => {
                let path: Result<Vec<Vec<_>>, _> = path
                    .iter()
                    .map(|elt| self.plan_graph_match_path_pattern(elt))
                    .collect();
                Ok(path?.into_iter().flatten().collect())
            }
            ast::GraphMatchPathPattern::Union(_) => {
                not_yet_implemented_result!("MATCH expression UNION is not yet supported.");
            }
            ast::GraphMatchPathPattern::Multiset(_) => {
                not_yet_implemented_result!("MATCH expression MULTISET is not yet supported.");
            }
            ast::GraphMatchPathPattern::Questioned(_) => {
                not_yet_implemented_result!("MATCH expression QUESTIONED is not yet supported.");
            }
            ast::GraphMatchPathPattern::Quantified(_) => {
                not_yet_implemented_result!("MATCH expression QUANTIFIED is not yet supported.");
            }
            ast::GraphMatchPathPattern::Sub(subpath) => self.plan_graph_subpath_pattern(subpath),
            ast::GraphMatchPathPattern::Node(n) => self.plan_graph_pattern_part_node(n),
            ast::GraphMatchPathPattern::Edge(e) => self.plan_graph_pattern_part_edge(e),
            ast::GraphMatchPathPattern::Simplified(_) => {
                not_yet_implemented_result!(
                    "MATCH expression Simplified Edge Expressions are not yet supported."
                );
            }
        }
    }

    fn plan_graph_pattern_part_node(
        &mut self,
        node: &ast::GraphMatchNode,
    ) -> Result<Vec<MatchElement>, String> {
        let binder = match &node.variable {
            None => self.graph_id.node(),
            Some(v) => v.value.clone(),
        };
        let label = plan_graph_pattern_label(node.label.as_deref())?;
        let filter = if let Some(expr) = &node.where_clause {
            let expr = extract_vexpr_by_id(&mut self.env, expr.id());
            if expr.is_none() {
                not_yet_implemented_result!("Internal error: expression not found.");
            }
            ValueFilter::Filter(Box::new(expr.unwrap()))
        } else {
            ValueFilter::Always
        };
        let node_match = NodeMatch {
            binder: BindSpec(binder),
            spec: NodeFilter { label, filter },
        };
        Ok(vec![MatchElement::Node(node_match)])
    }

    fn plan_graph_pattern_part_edge(
        &mut self,
        edge: &ast::GraphMatchEdge,
    ) -> Result<Vec<MatchElement>, String> {
        let direction = match &edge.direction {
            ast::GraphMatchDirection::Left => DirectionFilter::L,
            ast::GraphMatchDirection::Undirected => DirectionFilter::U,
            ast::GraphMatchDirection::Right => DirectionFilter::R,
            ast::GraphMatchDirection::LeftOrUndirected => DirectionFilter::LU,
            ast::GraphMatchDirection::UndirectedOrRight => DirectionFilter::UR,
            ast::GraphMatchDirection::LeftOrRight => DirectionFilter::LR,
            ast::GraphMatchDirection::LeftOrUndirectedOrRight => DirectionFilter::LUR,
        };
        let binder = match &edge.variable {
            None => self.graph_id.node(),
            Some(v) => v.value.clone(),
        };
        let label = plan_graph_pattern_label(edge.label.as_deref())?;
        let filter = if let Some(expr) = &edge.where_clause {
            let expr = extract_vexpr_by_id(&mut self.env, expr.id());
            if expr.is_none() {
                not_yet_implemented_result!("Internal error: expression not found.");
            }
            ValueFilter::Filter(Box::new(expr.unwrap()))
        } else {
            ValueFilter::Always
        };
        let edge_match = EdgeMatch {
            binder: BindSpec(binder),
            spec: EdgeFilter { label, filter },
        };
        Ok(vec![MatchElement::Edge(direction, edge_match)])
    }
}

fn plan_graph_pattern_label(label: Option<&ast::GraphMatchLabel>) -> Result<LabelFilter, String> {
    if let Some(label) = label {
        match label {
            ast::GraphMatchLabel::Name(n) => Ok(LabelFilter::Named(n.value.clone())),
            ast::GraphMatchLabel::Wildcard => Ok(LabelFilter::Always),
            ast::GraphMatchLabel::Negated(l) => {
                let inner = plan_graph_pattern_label(Some(l.as_ref()))?;
                Ok(LabelFilter::Negated(Box::new(inner)))
            }
            ast::GraphMatchLabel::Conjunction(inner) => {
                let inner: Result<Vec<_>, _> = inner
                    .iter()
                    .map(|l| plan_graph_pattern_label(Some(l)))
                    .collect();
                Ok(LabelFilter::Conjunction(inner?))
            }
            ast::GraphMatchLabel::Disjunction(inner) => {
                let inner: Result<Vec<_>, _> = inner
                    .iter()
                    .map(|l| plan_graph_pattern_label(Some(l)))
                    .collect();
                Ok(LabelFilter::Disjunction(inner?))
            }
        }
    } else {
        Ok(LabelFilter::Always)
    }
}
