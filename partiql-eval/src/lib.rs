pub mod env;
pub mod eval;
pub mod plan;

#[cfg(test)]
mod tests {
    use crate::eval::{EvalFromAt, EvalOutputAccumulator, Evaluable, Evaluator, Output};
    use partiql_logical as logical;
    use partiql_value as value;

    use crate::env::basic::MapBindings;
    use crate::plan;
    use partiql_logical::{BindingsExpr, PathComponent, ValueExpr};
    use partiql_value::{
        partiql_bag, partiql_list, partiql_tuple, Bag, BindingsName, List, Tuple, Value,
    };
    use std::cell::RefCell;
    use std::collections::HashMap;
    use std::rc::Rc;

    fn evaluate(logical: BindingsExpr, bindings: MapBindings<Value>) -> Bag {
        let output = Rc::new(RefCell::new(EvalOutputAccumulator::default()));
        let planner = plan::EvaluatorPlanner {
            output: output.clone(),
        };

        let evaluable = planner.compile(logical);

        let mut evaluator = Evaluator::new(bindings, evaluable);
        evaluator.execute();

        println!("{:?}", &output);

        let result = &output.borrow_mut().output;
        result.clone()
    }

    fn data_customer() -> MapBindings<Value> {
        fn customer_tuple(id: i64, first_name: &str, balance: i64) -> value::Value {
            Tuple(HashMap::from([
                ("id".into(), id.into()),
                ("firstName".into(), first_name.into()),
                ("balance".into(), balance.into()),
            ]))
            .into()
        }

        let customer_val = partiql_bag![
            customer_tuple(5, "jason", 100),
            customer_tuple(4, "sisko", 0),
            customer_tuple(3, "jason", -30),
            customer_tuple(2, "miriam", 20),
            customer_tuple(1, "miriam", 10),
        ];

        let mut bindings = MapBindings::default();
        bindings.insert("customer", customer_val.into());
        bindings
    }

    fn data_3_tuple() -> MapBindings<Value> {
        fn a_tuple(n: i64) -> value::Value {
            Tuple(HashMap::from([("a".into(), n.into())])).into()
        }
        //let data = List(vec![a_tuple(1), a_tuple(2), a_tuple(3)]);
        let data = partiql_list![a_tuple(1), a_tuple(2), a_tuple(3)];

        let mut bindings = MapBindings::default();
        bindings.insert("data", data.into());
        bindings
    }

    #[test]
    fn select() {
        // Plan for `select a as b from data`
        let logical = BindingsExpr::From(logical::From {
            expr: ValueExpr::VarRef(BindingsName::CaseInsensitive("data".into())),
            as_key: "data".to_string(),
            at_key: None,
            out: Box::new(BindingsExpr::Select(logical::Select {
                exprs: HashMap::from([(
                    "b".to_string(),
                    ValueExpr::Path(
                        Box::new(ValueExpr::VarRef(BindingsName::CaseInsensitive(
                            "data".into(),
                        ))),
                        vec![PathComponent::Key("a".to_string())],
                    ),
                )]),
                out: Box::new(BindingsExpr::Output),
            })),
        });

        let result = evaluate(logical, data_3_tuple());
        assert_eq!(result.len(), 3);
    }

    #[test]
    fn select_value() {
        // Plan for `select value a from data`
        // TODO
    }

    #[test]
    fn select_distinct() {
        // Plan for `SELECT DISTINCT firstName, (firstName || firstName) AS doubleName FROM customer WHERE balance > 0`
        let logical = BindingsExpr::From(logical::From {
            expr: ValueExpr::VarRef(BindingsName::CaseInsensitive("customer".into())),
            as_key: "customer".to_string(),
            at_key: None,
            out: Box::new(BindingsExpr::Where(logical::Where {
                expr: ValueExpr::BinaryExpr(
                    logical::BinaryOp::Gt,
                    Box::new(ValueExpr::Path(
                        Box::new(ValueExpr::VarRef(BindingsName::CaseInsensitive(
                            "customer".into(),
                        ))),
                        vec![PathComponent::Key("balance".to_string())],
                    )),
                    Box::new(ValueExpr::Lit(Box::new(Value::Integer(0)))),
                ),
                out: Box::new(BindingsExpr::Select(logical::Select {
                    exprs: HashMap::from([
                        (
                            "firstName".to_string(),
                            ValueExpr::Path(
                                Box::new(ValueExpr::VarRef(BindingsName::CaseInsensitive(
                                    "customer".into(),
                                ))),
                                vec![PathComponent::Key("firstName".to_string())],
                            ),
                        ),
                        (
                            "doubleName".to_string(),
                            ValueExpr::BinaryExpr(
                                logical::BinaryOp::Concat,
                                Box::new(ValueExpr::Path(
                                    Box::new(ValueExpr::VarRef(BindingsName::CaseInsensitive(
                                        "customer".into(),
                                    ))),
                                    vec![PathComponent::Key("firstName".to_string())],
                                )),
                                Box::new(ValueExpr::Path(
                                    Box::new(ValueExpr::VarRef(BindingsName::CaseInsensitive(
                                        "customer".into(),
                                    ))),
                                    vec![PathComponent::Key("firstName".to_string())],
                                )),
                            ),
                        ),
                    ]),
                    out: Box::new(BindingsExpr::Distinct(logical::Distinct {
                        out: Box::new(BindingsExpr::Output),
                    })),
                })),
            })),
        });

        let result = evaluate(logical, data_customer());
        assert_eq!(result.len(), 2);
    }

    mod clause_from {
        use super::*;
        use crate::eval::{BasicContext, EvalFrom, EvalPath, EvalVarRef, PathComponent};
        use partiql_value::{partiql_bag, partiql_list, BindingsName};

        fn some_ordered_table() -> List {
            partiql_list![
                partiql_tuple![("a", 0), ("b", 0)],
                partiql_tuple![("a", 1), ("b", 1)],
            ]
        }

        fn some_unordered_table() -> Bag {
            Bag::from(some_ordered_table())
        }

        // Spec 5.1
        #[test]
        fn basic() {
            let mut p0: MapBindings<Value> = MapBindings::default();
            p0.insert("someOrderedTable", some_ordered_table().into());

            let output = Rc::new(RefCell::new(EvalOutputAccumulator::default()));
            let eout = Box::new(Output {
                output: output.clone(),
            });
            let mut from = EvalFrom::new(
                Box::new(EvalVarRef {
                    name: BindingsName::CaseInsensitive("someOrderedTable".to_string()),
                }),
                "x",
                eout,
            );

            let ctx = BasicContext::new(p0);
            from.evaluate(&ctx);

            println!("{:?}", &output.borrow().output);
            // <<{ x:  { b: 0, a: 0 } },  { x:  { b: 1, a: 1 } }>>
            let expected = partiql_bag![
                partiql_tuple![("x", partiql_tuple![("a", 0), ("b", 0)]),],
                partiql_tuple![("x", partiql_tuple![("a", 1), ("b", 1)]),],
            ];
            assert_eq!(&expected, &output.borrow().output);
        }

        // Spec 5.1
        #[test]
        fn at() {
            let mut p0: MapBindings<Value> = MapBindings::default();
            p0.insert("someOrderedTable", some_ordered_table().into());

            let output = Rc::new(RefCell::new(EvalOutputAccumulator::default()));
            let eout = Box::new(Output {
                output: output.clone(),
            });
            let mut from = EvalFromAt::new(
                Box::new(EvalVarRef {
                    name: BindingsName::CaseInsensitive("someOrderedTable".to_string()),
                }),
                "x",
                "y",
                eout,
            );

            let ctx = BasicContext::new(p0);
            from.evaluate(&ctx);

            println!("{:?}", &output.borrow().output);
            // <<{ y: 0, x:  { b: 0, a: 0 } },  { x:  { b: 1, a: 1 }, y: 1 }>>
            let expected = partiql_bag![
                partiql_tuple![("x", partiql_tuple![("a", 0), ("b", 0)]), ("y", 0)],
                partiql_tuple![("x", partiql_tuple![("a", 1), ("b", 1)]), ("y", 1)],
            ];
            assert_eq!(&expected, &output.borrow().output);
        }

        // Spec 5.1.1
        #[test]
        fn mistype_at_on_bag() {
            let mut p0: MapBindings<Value> = MapBindings::default();
            p0.insert("someUnorderedTable", some_unordered_table().into());

            let output = Rc::new(RefCell::new(EvalOutputAccumulator::default()));
            let eout = Box::new(Output {
                output: output.clone(),
            });
            let mut from = EvalFromAt::new(
                Box::new(EvalVarRef {
                    name: BindingsName::CaseInsensitive("someUnorderedTable".to_string()),
                }),
                "x",
                "y",
                eout,
            );

            let ctx = BasicContext::new(p0);
            from.evaluate(&ctx);

            println!("{:?}", &output.borrow().output);
            // <<{ y: MISSING, x:  { b: 0, a: 0 } },  { x:  { b: 1, a: 1 }, y: MISSING }>>
            let expected = partiql_bag![
                partiql_tuple![
                    ("x", partiql_tuple![("a", 0), ("b", 0)]),
                    ("y", value::Value::Missing)
                ],
                partiql_tuple![
                    ("x", partiql_tuple![("a", 1), ("b", 1)]),
                    ("y", value::Value::Missing)
                ],
            ];
            assert_eq!(&expected, &output.borrow().output);
        }

        // Spec 5.1.1
        #[test]
        fn mistype_scalar() {
            let mut p0: MapBindings<Value> = MapBindings::default();
            p0.insert("someOrderedTable", some_ordered_table().into());

            let output = Rc::new(RefCell::new(EvalOutputAccumulator::default()));
            let eout = Box::new(Output {
                output: output.clone(),
            });

            let table_ref = EvalVarRef {
                name: BindingsName::CaseInsensitive("someOrderedTable".to_string()),
            };
            let path_to_scalar = EvalPath {
                expr: Box::new(table_ref),
                components: vec![PathComponent::Index(0), PathComponent::Key("a".into())],
            };
            let mut from = EvalFrom::new(Box::new(path_to_scalar), "x", eout);

            let ctx = BasicContext::new(p0);
            from.evaluate(&ctx);

            println!("{:?}", &output.borrow().output);
            let expected = partiql_bag![partiql_tuple![("x", 0)]];
            assert_eq!(&expected, &output.borrow().output);
        }

        // Spec 5.1.1
        #[test]
        fn mistype_absent() {
            let mut p0: MapBindings<Value> = MapBindings::default();
            p0.insert("someOrderedTable", some_ordered_table().into());

            let output = Rc::new(RefCell::new(EvalOutputAccumulator::default()));
            let eout = Box::new(Output {
                output: output.clone(),
            });

            let table_ref = EvalVarRef {
                name: BindingsName::CaseInsensitive("someOrderedTable".to_string()),
            };
            let path_to_scalar = EvalPath {
                expr: Box::new(table_ref),
                components: vec![PathComponent::Index(0), PathComponent::Key("c".into())],
            };
            let mut from = EvalFrom::new(Box::new(path_to_scalar), "x", eout);

            let ctx = BasicContext::new(p0);
            from.evaluate(&ctx);

            println!("{:?}", &output.borrow().output);
            let expected = partiql_bag![partiql_tuple![("x", value::Value::Missing)]];
            assert_eq!(&expected, &output.borrow().output);
        }
    }

    mod clause_unpivot {
        use super::*;
        use crate::eval::{BasicContext, EvalUnpivot, EvalVarRef, Output};
        use partiql_value::{partiql_bag, BindingsName, Tuple};

        fn just_a_tuple() -> Tuple {
            partiql_tuple![("amzn", 840.05), ("tdc", 31.06)]
        }

        // Spec 5.2
        #[test]
        fn basic() {
            let mut p0: MapBindings<Value> = MapBindings::default();
            p0.insert("justATuple", just_a_tuple().into());

            let output = Rc::new(RefCell::new(EvalOutputAccumulator::default()));
            let eout = Box::new(Output {
                output: output.clone(),
            });
            let mut unpivot = EvalUnpivot::new(
                Box::new(EvalVarRef {
                    name: BindingsName::CaseInsensitive("justATuple".to_string()),
                }),
                "price",
                "symbol",
                eout,
            );

            let ctx = BasicContext::new(p0);
            unpivot.evaluate(&ctx);

            println!("{:?}", &output.borrow().output);
            let expected = partiql_bag![
                partiql_tuple![("symbol", "tdc"), ("price", 31.06)],
                partiql_tuple![("symbol", "amzn"), ("price", 840.05)],
            ];
            assert_eq!(&expected, &output.borrow().output);
        }

        // Spec 5.2.1
        #[test]
        fn mistype_non_tuple() {
            let mut p0: MapBindings<Value> = MapBindings::default();
            p0.insert("nonTuple", Value::from(1));

            let output = Rc::new(RefCell::new(EvalOutputAccumulator::default()));
            let eout = Box::new(Output {
                output: output.clone(),
            });
            let mut unpivot = EvalUnpivot::new(
                Box::new(EvalVarRef {
                    name: BindingsName::CaseInsensitive("nonTuple".to_string()),
                }),
                "x",
                "y",
                eout,
            );

            let ctx = BasicContext::new(p0);
            unpivot.evaluate(&ctx);

            println!("{:?}", &output.borrow().output);
            let expected = partiql_bag![partiql_tuple![("x", 1), ("y", "_1")]];
            assert_eq!(&expected, &output.borrow().output);
        }
    }
}
