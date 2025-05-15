use crate::eval::eval_expr_wrapper::UnaryValueExpr;
use crate::eval::expr::{BindError, BindEvalExpr, EvalExpr};
use crate::eval::graph::evaluator::GraphEvaluator;
use crate::eval::graph::simple_graph::engine::SimpleGraphEngine;

use crate::eval::graph::plan::PathPatternMatch;
use crate::eval::graph::string_graph::StringGraphTypes;
use partiql_types::{type_graph, PartiqlNoIdShapeBuilder};
use partiql_value::Value::Missing;
use partiql_value::{Graph, Value};

/// Represents an evaluation `MATCH` operator, e.g. in `graph MATCH () -> ()'`.
#[derive(Debug)]
pub(crate) struct EvalGraphMatch {
    pub(crate) pattern: PathPatternMatch<StringGraphTypes>,
}

impl EvalGraphMatch {
    pub(crate) fn new(pattern: PathPatternMatch<StringGraphTypes>) -> Self {
        EvalGraphMatch { pattern }
    }
}

impl BindEvalExpr for EvalGraphMatch {
    fn bind<const STRICT: bool>(
        self,
        args: Vec<Box<dyn EvalExpr>>,
    ) -> Result<Box<dyn EvalExpr>, BindError> {
        // use DummyShapeBuilder, as we don't care about shape Ids for evaluation dispatch
        let mut bld = PartiqlNoIdShapeBuilder::default();
        UnaryValueExpr::create_typed_with_ctx::<{ STRICT }, _>(
            [type_graph!(bld)],
            args,
            move |value, ctx| match value {
                Value::Graph(graph) => match graph.as_ref() {
                    Graph::Simple(g) => {
                        let engine = SimpleGraphEngine::new(g.clone());
                        let ge = GraphEvaluator::new(engine);
                        ge.eval(&self.pattern, ctx)
                    }
                },
                _ => Missing,
            },
        )
    }
}

#[cfg(test)]
mod tests {
    use crate::eval::expr::{BindEvalExpr, EvalGlobalVarRef, EvalGraphMatch};
    use crate::eval::graph::plan::{
        BindSpec, DirectionFilter, EdgeFilter, LabelFilter, NodeFilter, NodeMatch, PathMode,
        PathPatternMatch, TripleFilter, TripleStepFilter, TripleStepMatch, ValueFilter,
    };
    use crate::eval::graph::string_graph::StringGraphTypes;
    use crate::eval::graph::types::GraphTypes;
    use crate::eval::{BasicContext, MapBindings};
    use crate::test_value::TestValue;
    use partiql_catalog::context::SystemContext;
    use partiql_common::pretty::ToPretty;
    use partiql_logical::graph::bind_name::FreshBinder;
    use partiql_value::datum::DatumTupleRef;
    use partiql_value::{tuple, BindingsName, DateTime, Value};

    impl<GT: GraphTypes> From<TripleStepMatch<GT>> for PathPatternMatch<GT> {
        fn from(value: TripleStepMatch<GT>) -> Self {
            Self::Match(value)
        }
    }

    impl<GT: GraphTypes> From<NodeMatch<GT>> for PathPatternMatch<GT> {
        fn from(value: NodeMatch<GT>) -> Self {
            Self::Node(value)
        }
    }

    pub trait ElementFilterBuilder<GT: GraphTypes> {
        fn any() -> Self;
        fn labeled(label: GT::Label) -> Self;
    }

    impl<GT: GraphTypes> ElementFilterBuilder<GT> for NodeFilter<GT> {
        fn any() -> Self {
            Self {
                label: LabelFilter::Always,
                filter: ValueFilter::Always,
            }
        }

        fn labeled(label: GT::Label) -> Self {
            Self {
                label: LabelFilter::Named(label),
                filter: ValueFilter::Always,
            }
        }
    }

    impl<GT: GraphTypes> ElementFilterBuilder<GT> for EdgeFilter<GT> {
        fn any() -> Self {
            Self {
                label: LabelFilter::Always,
                filter: ValueFilter::Always,
            }
        }
        fn labeled(label: GT::Label) -> Self {
            Self {
                label: LabelFilter::Named(label),
                filter: ValueFilter::Always,
            }
        }
    }

    /*
        A simple 3-node, 3-edge graph which is intended to be able to be exactly matched by:
       ```(graph MATCH
            (n1:a WHERE n1 == 1) -[e12:e WHERE e12 == 1.2]-> (n2),
            (n2:b WHERE n2 == 2) -[e23:d WHERE e23 == 2.3]-> (n3),
            (n3:a WHERE n3 == 3) ~[e_u:self WHERE e_u == <<>>]~ (n3)
        )```
    */
    fn graph() -> Value {
        let graph = r##"
            $graph::{
                nodes: [ {id: n1, labels: ["a"], payload: 1},
                         {id: n2, labels: ["b"], payload: 2},
                         {id: n3, labels: ["a"], payload: 3} ],
                edges: [ {id: e12, labels: ["e"], payload: 1.2, ends: (n1 -> n2) },
                         {id: e23, labels: ["d"], payload: 2.3, ends: (n2 -> n3) },
                         {id: e_u, labels: ["self"], payload: $bag::[] , ends: (n3 -- n3) } ]
            }
            "##;
        TestValue::from(graph).value
    }

    fn bindings() -> MapBindings<Value> {
        let mut bindings: MapBindings<Value> = MapBindings::default();
        bindings.insert("graph", graph());
        bindings
    }

    fn context() -> BasicContext<'static> {
        let sys = SystemContext {
            now: DateTime::from_system_now_utc(),
        };
        let ctx = BasicContext::new(bindings(), sys);
        ctx
    }

    fn graph_reference() -> Box<EvalGlobalVarRef> {
        Box::new(EvalGlobalVarRef {
            name: BindingsName::CaseInsensitive("graph".to_string().into()),
        })
    }

    #[track_caller]
    fn test_graph(matcher: PathPatternMatch<StringGraphTypes>, expected: &'static str) {
        let eval = EvalGraphMatch::new(matcher)
            .bind::<false>(vec![graph_reference()])
            .expect("graph match bind");

        let bindings = tuple![("graph", graph())];
        let bindings = DatumTupleRef::Tuple(&bindings);
        let ctx = context();
        let res = eval.evaluate(&bindings, &ctx);
        let expected = crate::test_value::parse_partiql_value_str(expected);

        let pretty = |v: &Value| v.to_pretty_string(80).unwrap();

        assert_eq!(pretty(&expected), pretty(res.as_ref()));
    }

    #[test]
    fn node() {
        // Query: (graph MATCH (x:a))
        let binder = BindSpec("x".to_string());
        let spec = NodeFilter::labeled("a".to_string());
        let matcher = NodeMatch { binder, spec };

        test_graph(matcher.into(), "<< { 'x': 1 }, { 'x': 3 } >>")
    }

    #[test]
    fn no_edge_matches() {
        let fresh = FreshBinder::default();

        // Query: (graph MATCH () -[e:foo]- ())
        let binders = (
            BindSpec(fresh.node()),
            BindSpec("e".to_string()),
            BindSpec(fresh.node()),
        );
        let spec = TripleStepFilter {
            dir: DirectionFilter::LUR,
            triple: TripleFilter {
                lhs: NodeFilter::any(),
                e: EdgeFilter::labeled("foo".to_string()),
                rhs: NodeFilter::any(),
            },
        };

        let matcher: TripleStepMatch<StringGraphTypes> = TripleStepMatch {
            binders,
            spec,
            filter: ValueFilter::Always,
            path_mode: PathMode::Walk,
        };

        test_graph(matcher.into(), "<<  >>")
    }

    #[test]
    fn no_node_matches() {
        let fresh = FreshBinder::default();

        // Query: (graph MATCH (:foo) -[]- ())
        let binders = (
            BindSpec(fresh.node()),
            BindSpec(fresh.edge()),
            BindSpec(fresh.node()),
        );
        let spec = TripleStepFilter {
            dir: DirectionFilter::LUR,
            triple: TripleFilter {
                lhs: NodeFilter::labeled("foo".to_string()),
                e: EdgeFilter::any(),
                rhs: NodeFilter::any(),
            },
        };

        let matcher: TripleStepMatch<StringGraphTypes> = TripleStepMatch {
            binders,
            spec,
            filter: ValueFilter::Always,
            path_mode: PathMode::Walk,
        };

        test_graph(matcher.into(), "<<  >>")
    }

    #[test]
    fn node_edge_node() {
        // Query: (graph MATCH (x)<-[z:e]-(y))
        let binders = (
            BindSpec("x".to_string()),
            BindSpec("z".to_string()),
            BindSpec("y".to_string()),
        );
        let spec = TripleStepFilter {
            dir: DirectionFilter::L,
            triple: TripleFilter {
                lhs: NodeFilter::any(),
                e: EdgeFilter::labeled("e".to_string()),
                rhs: NodeFilter::any(),
            },
        };

        let matcher: TripleStepMatch<StringGraphTypes> = TripleStepMatch {
            binders,
            spec,
            filter: ValueFilter::Always,
            path_mode: PathMode::Walk,
        };

        test_graph(matcher.into(), "<< {'x': 2, 'z': 1.2, 'y': 1} >>")
    }

    #[test]
    fn edge() {
        let fresh = FreshBinder::default();

        // Query: (graph MATCH -> )
        let binders = (
            BindSpec(fresh.node()),
            BindSpec(fresh.edge()),
            BindSpec(fresh.node()),
        );
        let spec = TripleStepFilter {
            dir: DirectionFilter::R,
            triple: TripleFilter {
                lhs: NodeFilter::any(),
                e: EdgeFilter::any(),
                rhs: NodeFilter::any(),
            },
        };

        let matcher: TripleStepMatch<StringGraphTypes> = TripleStepMatch {
            binders,
            spec,
            filter: ValueFilter::Always,
            path_mode: PathMode::Walk,
        };

        test_graph(matcher.into(), "<< {  }, {  } >>")
    }

    #[test]
    fn edge_outgoing() {
        let fresh = FreshBinder::default();

        // Query: (graph MATCH <-[z]-> )
        let binders = (
            BindSpec(fresh.node()),
            BindSpec("z".to_string()),
            BindSpec(fresh.node()),
        );
        let spec = TripleStepFilter {
            dir: DirectionFilter::LR,
            triple: TripleFilter {
                lhs: NodeFilter::any(),
                e: EdgeFilter::any(),
                rhs: NodeFilter::any(),
            },
        };

        let matcher: TripleStepMatch<StringGraphTypes> = TripleStepMatch {
            binders,
            spec,
            filter: ValueFilter::Always,
            path_mode: PathMode::Walk,
        };

        test_graph(
            matcher.into(),
            "<< { 'z': 1.2 }, { 'z': 1.2 }, { 'z': 2.3 }, { 'z': 2.3 } >>",
        )
    }

    #[test]
    fn n_e_n_e_n() {
        // Query: (graph MATCH (x:b)-[z1]-(y1:a)-[z2]-(y2:b) )
        let binders = (
            BindSpec("x".to_string()),
            BindSpec("z1".to_string()),
            BindSpec("y1".to_string()),
        );
        let spec = TripleStepFilter {
            dir: DirectionFilter::LUR,
            triple: TripleFilter {
                lhs: NodeFilter::labeled("b".to_string()),
                e: EdgeFilter::any(),
                rhs: NodeFilter::labeled("a".to_string()),
            },
        };
        let matcher1: TripleStepMatch<StringGraphTypes> = TripleStepMatch {
            binders,
            spec,
            filter: Default::default(),
            path_mode: PathMode::Walk,
        };

        let binders = (
            BindSpec("y1".to_string()),
            BindSpec("z2".to_string()),
            BindSpec("y2".to_string()),
        );
        let spec = TripleStepFilter {
            dir: DirectionFilter::LUR,
            triple: TripleFilter {
                lhs: NodeFilter::labeled("a".to_string()),
                e: EdgeFilter::any(),
                rhs: NodeFilter::labeled("b".to_string()),
            },
        };
        let matcher2: TripleStepMatch<StringGraphTypes> = TripleStepMatch {
            binders,
            spec,
            filter: Default::default(),
            path_mode: PathMode::Walk,
        };

        let pattern_match = PathPatternMatch::Concat(
            vec![
                PathPatternMatch::Match(matcher1),
                PathPatternMatch::Match(matcher2),
            ],
            Default::default(),
            PathMode::Walk,
        );

        test_graph(
            pattern_match,
            "<< { 'x': 2, 'z1': 1.2, 'y1': 1, 'z2': 1.2, 'y2': 2 }, \
                             { 'x': 2, 'z1': 2.3, 'y1': 3, 'z2': 2.3, 'y2': 2 } >>",
        )
    }
    #[test]
    fn cycle() {
        let fresh = FreshBinder::default();

        // Query: (graph MATCH (x1) - (x2) - (x1))
        let binders = (
            BindSpec("x1".to_string()),
            BindSpec(fresh.edge()),
            BindSpec("x2".to_string()),
        );
        let spec = TripleStepFilter {
            dir: DirectionFilter::LUR,
            triple: TripleFilter {
                lhs: NodeFilter::any(),
                e: EdgeFilter::any(),
                rhs: NodeFilter::any(),
            },
        };
        let matcher1: TripleStepMatch<StringGraphTypes> = TripleStepMatch {
            binders,
            spec,
            filter: Default::default(),
            path_mode: PathMode::Walk,
        };

        let binders = (
            BindSpec("x2".to_string()),
            BindSpec(fresh.edge()),
            BindSpec("x1".to_string()),
        );
        let spec = TripleStepFilter {
            dir: DirectionFilter::LUR,
            triple: TripleFilter {
                lhs: NodeFilter::any(),
                e: EdgeFilter::any(),
                rhs: NodeFilter::any(),
            },
        };
        let matcher2: TripleStepMatch<StringGraphTypes> = TripleStepMatch {
            binders,
            spec,
            filter: Default::default(),
            path_mode: PathMode::Walk,
        };

        let pattern_match = PathPatternMatch::Concat(
            vec![
                PathPatternMatch::Match(matcher1),
                PathPatternMatch::Match(matcher2),
            ],
            Default::default(),
            PathMode::Walk,
        );
        test_graph(
            pattern_match,
            "<< { 'x1': 3, 'x2': 3 }, \
                             { 'x1': 1, 'x2': 2 }, \
                             { 'x1': 2, 'x2': 1 }, \
                             { 'x1': 2, 'x2': 3 }, \
                             { 'x1': 3, 'x2': 2 } >>",
        )
    }
}
