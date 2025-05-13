use num::Integer;
use partiql_ast::ast;

use crate::lower::extract_vexpr_by_id;
use partiql_ast::ast::GraphPathPrefix;
use partiql_common::node::{IdAnnotated, NodeId};
use partiql_logical::graph::bind_name::FreshBinder;
use partiql_logical::graph::{
    BindSpec, DirectionFilter, EdgeFilter, EdgeMatch, LabelFilter, NodeFilter, NodeMatch, PathMode,
    PathPattern, PathPatternMatch, StepFilter, TripleFilter, TripleMatch, TripleSeriesMatch,
    ValueFilter,
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
    id_generator: FreshBinder,
    env: Vec<(NodeId, ValueExpr)>,
}

impl GraphToLogical {
    pub fn new(env: Vec<(NodeId, ValueExpr)>) -> Self {
        Self {
            id_generator: Default::default(),
            env,
        }
    }
}

#[derive(Debug, Clone)]
enum MatchElement {
    Node(NodeMatch),
    Edge(DirectionFilter, EdgeMatch),
    Pattern(Vec<MatchElement>, ValueFilter, PathMode),
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

    /// 'Normalize' a series of node/edge/subpath match elements into a rigid pattern of
    ///    (<node> ( <edge> <node>)*)
    pub fn normalize(mut self, elements: Vec<MatchElement>) -> Result<Vec<PathPattern>, String> {
        let path_mode = PathMode::Walk; // TODO replace with input to function?
        self.do_normalize(elements, path_mode)
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

    fn flush(&mut self, mode: PathMode) -> Option<PathPattern> {
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

        Some(PathPattern {
            head,
            tail,
            filter: ValueFilter::Always,
            mode,
        })
    }

    fn do_normalize(
        &mut self,
        elements: Vec<MatchElement>,
        normalize_mode: PathMode,
    ) -> Result<Vec<PathPattern>, String> {
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
                MatchElement::Pattern(p, filter, pattern_mode) => {
                    path_patterns.extend(self.flush(normalize_mode));
                    let mut normalized_pattern = self.do_normalize(p, pattern_mode)?;
                    match filter {
                        ValueFilter::Always => {
                            path_patterns.extend(normalized_pattern);
                        }
                        filter @ ValueFilter::Filter(_) => {
                            debug_assert!(normalized_pattern.len() == 1);
                            let mut normalized_pattern = normalized_pattern.remove(0);
                            normalized_pattern.filter = filter;
                            path_patterns.push(normalized_pattern);
                        }
                    }
                }
            }
        }

        path_patterns.extend(self.flush(normalize_mode));

        Ok(path_patterns)
    }
}

impl GraphToLogical {
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
        if pattern.match_mode.is_some() {
            // The current graph engine is basically doing RepeatableElements currently
            not_yet_implemented_result!("MATCH expression MATCH MODE is not yet supported.");
        }
        if pattern.keep.is_some() {
            // TODO... this overrides mode and search prefixes
            not_yet_implemented_result!("MATCH expression KEEP is not yet supported.");
        }

        let filter = if let Some(expr) = &pattern.where_clause {
            let expr = extract_vexpr_by_id(&mut self.env, expr.id());
            if expr.is_none() {
                not_yet_implemented_result!("Internal error: expression not found.");
            }
            ValueFilter::Filter(vec![expr.unwrap()])
        } else {
            ValueFilter::Always
        };

        if pattern.patterns.len() != 1 {
            not_yet_implemented_result!(
                "MATCH expression with multiple patterns are not yet supported."
            );
        }

        let first_pattern = &pattern.patterns[0];
        let planned_first_pattern = self.plan_graph_path_pattern(first_pattern)?;

        let mode = plan_path_mode(None)?; // TODO replace with processing of `keep`
        Ok(vec![MatchElement::Pattern(
            planned_first_pattern,
            filter,
            mode,
        )])
    }

    fn plan_graph_path_pattern(
        &mut self,
        pattern: &ast::GraphPathPattern,
    ) -> Result<Vec<MatchElement>, String> {
        let (mode, prefix) = match &pattern.prefix {
            None => (None, None),
            Some(GraphPathPrefix::Mode(mode)) => (Some(mode), None),
            Some(GraphPathPrefix::Search(prefix, mode)) => (mode.as_ref(), Some(prefix)),
        };
        if prefix.is_some() {
            not_yet_implemented_result!("MATCH pattern SEARCH prefix are not yet supported.");
        }
        if pattern.variable.is_some() {
            not_yet_implemented_result!("MATCH pattern path variables are not yet supported.");
        }

        let elts = self.plan_graph_match_path_pattern(&pattern.path, mode)?;
        let mode = plan_path_mode(mode)?;
        Ok(vec![MatchElement::Pattern(elts, ValueFilter::Always, mode)])
    }

    fn plan_graph_subpath_pattern(
        &mut self,
        pattern: &ast::GraphPathSubPattern,
        mode: Option<&ast::GraphPathMode>,
    ) -> Result<Vec<MatchElement>, String> {
        // override mode if supplied in sub-path
        let mode = pattern.mode.as_ref().or(mode);

        if pattern.variable.is_some() {
            not_yet_implemented_result!("MATCH pattern path variables are not yet supported.");
        }

        let filter = if let Some(expr) = &pattern.where_clause {
            let expr = extract_vexpr_by_id(&mut self.env, expr.id());
            if expr.is_none() {
                not_yet_implemented_result!("Internal error: expression not found.");
            }
            ValueFilter::Filter(vec![expr.unwrap()])
        } else {
            ValueFilter::Always
        };

        let elts = self.plan_graph_match_path_pattern(&pattern.path, mode)?;
        let mode = plan_path_mode(mode)?;
        Ok(vec![MatchElement::Pattern(elts, filter, mode)])
    }

    fn plan_graph_match_path_pattern(
        &mut self,
        pattern: &ast::GraphMatchPathPattern,
        mode: Option<&ast::GraphPathMode>,
    ) -> Result<Vec<MatchElement>, String> {
        match pattern {
            ast::GraphMatchPathPattern::Path(path) => {
                let path: Result<Vec<Vec<_>>, _> = path
                    .iter()
                    .map(|elt| self.plan_graph_match_path_pattern(elt, mode))
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
            ast::GraphMatchPathPattern::Sub(subpath) => {
                self.plan_graph_subpath_pattern(subpath, mode)
            }
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
            None => self.id_generator.node(),
            Some(v) => v.value.clone(),
        };
        let label = plan_graph_pattern_label(node.label.as_deref())?;
        let filter = if let Some(expr) = &node.where_clause {
            let expr = extract_vexpr_by_id(&mut self.env, expr.id());
            if expr.is_none() {
                not_yet_implemented_result!("Internal error: expression not found.");
            }
            ValueFilter::Filter(vec![expr.unwrap()])
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
            None => self.id_generator.edge(),
            Some(v) => v.value.clone(),
        };
        let label = plan_graph_pattern_label(edge.label.as_deref())?;
        let filter = if let Some(expr) = &edge.where_clause {
            let expr = extract_vexpr_by_id(&mut self.env, expr.id());
            if expr.is_none() {
                not_yet_implemented_result!("Internal error: expression not found.");
            }
            ValueFilter::Filter(vec![expr.unwrap()])
        } else {
            ValueFilter::Always
        };
        let edge_match = EdgeMatch {
            binder: BindSpec(binder),
            spec: EdgeFilter { label, filter },
        };
        Ok(vec![MatchElement::Edge(direction, edge_match)])
    }

    fn normalize(&mut self, elements: Vec<MatchElement>) -> Result<Vec<PathPattern>, String> {
        Normalize::new(&self.id_generator).normalize(elements)
    }

    fn expand(&mut self, paths: Vec<PathPattern>) -> Result<Vec<PathPattern>, String> {
        debug_assert!(!paths.is_empty());
        // TODO handle expansion as described in 6.3 of https://arxiv.org/pdf/2112.06217
        // TODO   this will enable alternation and quantifiers

        // first, add disjuncts for alternation/union
        // TODO

        // second, fix iterations for quantifiers
        // TODO

        // third, clean-up; delete duplicated anonymous edges and merge
        let mut paths = paths.into_iter();
        let mut path_patterns = vec![paths.next().unwrap()];
        for path in paths {
            if path.head.binder.is_anon() {
                // if the head of this path has an anonymous binder, it was synthetically inserted;
                //   Drop the head and merge its tail into the previous path.
                let current_path = path_patterns.last_mut().unwrap();
                current_path.tail.extend(path.tail);
                current_path.filter.extend(path.filter);
            } else {
                path_patterns.push(path);
            }
        }

        Ok(path_patterns)
    }

    fn plan(&mut self, mut paths: Vec<PathPattern>) -> Result<PathPatternMatch, String> {
        debug_assert!(paths.len() == 1);
        let path_pattern = paths.remove(0);
        // pattern at this point should be a node, or a series of node-[edge-node]+
        debug_assert!((1 + (2 * path_pattern.tail.len())).is_odd());

        let path_mode = path_pattern.mode;

        if path_pattern.tail.is_empty() {
            Ok(PathPatternMatch::Node(path_pattern.head.clone()))
        } else {
            let mut triples = vec![];
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
                triples.push(TripleMatch {
                    binders,
                    spec,
                    filter: ValueFilter::Always,
                    path_mode,
                });
                head = n;
            }
            if triples.len() == 1 {
                let mut triple = triples.remove(0);
                triple.filter.extend(path_pattern.filter);
                Ok(PathPatternMatch::Match(triple))
            } else {
                Ok(PathPatternMatch::Concat(
                    vec![TripleSeriesMatch {
                        triples,
                        filter: path_pattern.filter,
                        path_mode,
                    }],
                    path_mode,
                ))
            }
        }
    }
}

fn plan_path_mode(mode: Option<&ast::GraphPathMode>) -> Result<PathMode, String> {
    Ok(match mode {
        None => PathMode::Walk,
        Some(ast::GraphPathMode::Walk) => PathMode::Walk,
        Some(ast::GraphPathMode::Trail) => PathMode::Trail,
        Some(ast::GraphPathMode::Acyclic) => PathMode::Acyclic,
        Some(ast::GraphPathMode::Simple) => PathMode::Simple,
    })
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
