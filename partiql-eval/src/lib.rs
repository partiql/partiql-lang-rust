pub mod env;
pub mod eval;
pub mod plan;

#[cfg(test)]
mod tests {
    use std::collections::HashMap;

    use crate::env::basic::MapBindings;
    use crate::plan;
    use ordered_float::OrderedFloat;
    use rust_decimal::Decimal as RustDecimal;

    use crate::eval::Evaluator;

    use partiql_logical as logical;
    use partiql_logical::BindingsExpr::{Distinct, Project};
    use partiql_logical::{BinaryOp, BindingsExpr, LogicalPlan, PathComponent, ValueExpr};
    use partiql_value as value;
    use partiql_value::{
        partiql_bag, partiql_list, partiql_tuple, Bag, BindingsName, List, Tuple, Value,
    };

    fn evaluate(logical: LogicalPlan<BindingsExpr>, bindings: MapBindings<Value>) -> Value {
        let planner = plan::EvaluatorPlanner;

        let plan = planner.compile(logical);
        let mut evaluator = Evaluator::new(bindings);

        if let Ok(out) = evaluator.execute(plan) {
            out.result
        } else {
            Value::Missing
        }
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

    fn data_arithmetic_tuple() -> MapBindings<Value> {
        fn tuple_a_to_v(v: Value) -> value::Value {
            Tuple(HashMap::from([("a".into(), v)])).into()
        }
        // <<{'a': <int>}, {'a': <decimal>}, {'a': <float>}, {'a': NULL}, {'a': MISSING}>>
        let data = partiql_list![
            tuple_a_to_v(Value::Integer(1)),
            tuple_a_to_v(Value::Decimal(RustDecimal::from(1))),
            tuple_a_to_v(Value::Real(OrderedFloat(1.5))),
            tuple_a_to_v(Value::Null),
            tuple_a_to_v(Value::Missing),
        ];
        let mut bindings = MapBindings::default();
        bindings.insert("data", data.into());
        bindings
    }

    #[test]
    fn arithmetic() {
        // Tests arithmetic ops using int, real, decimal, null, and missing with values defined from
        // `data_arithmetic_tuple`
        fn arithmetic_logical(binary_op: BinaryOp, lit: Value) -> LogicalPlan<BindingsExpr> {
            let mut plan = LogicalPlan::new();
            let scan = plan.add_operator(BindingsExpr::Scan(logical::Scan {
                expr: ValueExpr::VarRef(BindingsName::CaseInsensitive("data".into())),
                as_key: "data".to_string(),
                at_key: "".to_string(),
            }));

            let project = plan.add_operator(BindingsExpr::Project(logical::Project {
                exprs: HashMap::from([(
                    "b".to_string(),
                    ValueExpr::BinaryExpr(
                        binary_op,
                        Box::new(ValueExpr::Path(
                            Box::new(ValueExpr::VarRef(BindingsName::CaseInsensitive(
                                "data".into(),
                            ))),
                            vec![PathComponent::Key("a".to_string())],
                        )),
                        Box::new(ValueExpr::Lit(Box::new(lit))),
                    ),
                )]),
            }));

            let sink = plan.add_operator(BindingsExpr::Sink);
            plan.extend_with_flows(&[(scan, project), (project, sink)]);
            plan
        }
        // Plan for `select a + <lit> as b from data`
        println!("Add+++++++++++++++++++++++++++++");
        let logical = arithmetic_logical(BinaryOp::Add, Value::Integer(1));
        let result = evaluate(logical, data_arithmetic_tuple());
        println!("{:?}", result);
        let logical = arithmetic_logical(BinaryOp::Add, Value::Decimal(RustDecimal::new(11, 1)));
        let result = evaluate(logical, data_arithmetic_tuple());
        println!("{:?}", result);
        let logical = arithmetic_logical(BinaryOp::Add, Value::Real(OrderedFloat(1.5)));
        let result = evaluate(logical, data_arithmetic_tuple());
        println!("{:?}", result);
        let logical = arithmetic_logical(BinaryOp::Add, Value::Null);
        let result = evaluate(logical, data_arithmetic_tuple());
        println!("{:?}", result);
        let logical = arithmetic_logical(BinaryOp::Add, Value::Missing);
        let result = evaluate(logical, data_arithmetic_tuple());
        println!("{:?}", result);

        // Plan for `select a - <lit> as b from data`
        println!("Sub-----------------------------");
        let logical = arithmetic_logical(BinaryOp::Sub, Value::Integer(1));
        let result = evaluate(logical, data_arithmetic_tuple());
        println!("{:?}", result);
        let logical = arithmetic_logical(BinaryOp::Sub, Value::Decimal(RustDecimal::new(11, 1)));
        let result = evaluate(logical, data_arithmetic_tuple());
        println!("{:?}", result);
        let logical = arithmetic_logical(BinaryOp::Sub, Value::Real(OrderedFloat(1.5)));
        let result = evaluate(logical, data_arithmetic_tuple());
        println!("{:?}", result);
        let logical = arithmetic_logical(BinaryOp::Sub, Value::Null);
        let result = evaluate(logical, data_arithmetic_tuple());
        println!("{:?}", result);
        let logical = arithmetic_logical(BinaryOp::Sub, Value::Missing);
        let result = evaluate(logical, data_arithmetic_tuple());
        println!("{:?}", result);

        // Plan for `select a * <lit> as b from data`
        println!("Mul*****************************");
        let logical = arithmetic_logical(BinaryOp::Mul, Value::Integer(1));
        let result = evaluate(logical, data_arithmetic_tuple());
        println!("{:?}", result);
        let logical = arithmetic_logical(BinaryOp::Mul, Value::Decimal(RustDecimal::new(11, 1)));
        let result = evaluate(logical, data_arithmetic_tuple());
        println!("{:?}", result);
        let logical = arithmetic_logical(BinaryOp::Mul, Value::Real(OrderedFloat(1.5)));
        let result = evaluate(logical, data_arithmetic_tuple());
        println!("{:?}", result);
        let logical = arithmetic_logical(BinaryOp::Mul, Value::Null);
        let result = evaluate(logical, data_arithmetic_tuple());
        println!("{:?}", result);
        let logical = arithmetic_logical(BinaryOp::Mul, Value::Missing);
        let result = evaluate(logical, data_arithmetic_tuple());
        println!("{:?}", result);

        // Plan for `select a / <lit> as b from data`
        println!("Div/////////////////////////////");
        let logical = arithmetic_logical(BinaryOp::Div, Value::Integer(1));
        let result = evaluate(logical, data_arithmetic_tuple());
        println!("{:?}", result);
        let logical = arithmetic_logical(BinaryOp::Div, Value::Decimal(RustDecimal::new(11, 1)));
        let result = evaluate(logical, data_arithmetic_tuple());
        println!("{:?}", result);
        let logical = arithmetic_logical(BinaryOp::Div, Value::Real(OrderedFloat(1.5)));
        let result = evaluate(logical, data_arithmetic_tuple());
        println!("{:?}", result);
        let logical = arithmetic_logical(BinaryOp::Div, Value::Null);
        let result = evaluate(logical, data_arithmetic_tuple());
        println!("{:?}", result);
        let logical = arithmetic_logical(BinaryOp::Div, Value::Missing);
        let result = evaluate(logical, data_arithmetic_tuple());
        println!("{:?}", result);

        // Plan for `select a % <lit> as b from data`
        println!("Mod%%%%%%%%%%%%%%%%%%%%%%%%%%%%%");
        let logical = arithmetic_logical(BinaryOp::Mod, Value::Integer(1));
        let result = evaluate(logical, data_arithmetic_tuple());
        println!("{:?}", result);
        let logical = arithmetic_logical(BinaryOp::Mod, Value::Decimal(RustDecimal::new(11, 1)));
        let result = evaluate(logical, data_arithmetic_tuple());
        println!("{:?}", result);
        let logical = arithmetic_logical(BinaryOp::Mod, Value::Real(OrderedFloat(1.5)));
        let result = evaluate(logical, data_arithmetic_tuple());
        println!("{:?}", result);
        let logical = arithmetic_logical(BinaryOp::Mod, Value::Null);
        let result = evaluate(logical, data_arithmetic_tuple());
        println!("{:?}", result);
        let logical = arithmetic_logical(BinaryOp::Mod, Value::Missing);
        let result = evaluate(logical, data_arithmetic_tuple());
        println!("{:?}", result);
    }

    #[test]
    fn select() {
        let mut lg = LogicalPlan::new();

        let from = lg.add_operator(BindingsExpr::Scan(logical::Scan {
            expr: ValueExpr::VarRef(BindingsName::CaseInsensitive("data".into())),
            as_key: "data".to_string(),
            at_key: "".to_string(),
        }));

        let project = lg.add_operator(BindingsExpr::Project(logical::Project {
            exprs: HashMap::from([(
                "b".to_string(),
                ValueExpr::Path(
                    Box::new(ValueExpr::VarRef(BindingsName::CaseInsensitive(
                        "data".into(),
                    ))),
                    vec![PathComponent::Key("a".to_string())],
                ),
            )]),
        }));

        let sink = lg.add_operator(BindingsExpr::Sink);

        lg.add_flow(from, project);
        lg.add_flow(project, sink);

        if let Value::Bag(b) = evaluate(lg, data_3_tuple()) {
            assert_eq!(b.len(), 3);
        } else {
            panic!("Wrong output")
        }
    }

    #[test]
    fn select_value() {
        // Plan for `select value a from data`
        // TODO
    }

    #[test]
    fn select_distinct() {
        // Plan for `SELECT DISTINCT firstName, (firstName || firstName) AS doubleName FROM customer WHERE balance > 0`
        let mut logical = LogicalPlan::new();

        let scan = logical.add_operator(BindingsExpr::Scan(logical::Scan {
            expr: ValueExpr::VarRef(BindingsName::CaseInsensitive("customer".into())),
            as_key: "customer".to_string(),
            at_key: "".to_string(),
        }));

        let filter = logical.add_operator(BindingsExpr::Filter(logical::Filter {
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
        }));

        let project = logical.add_operator(Project(logical::Project {
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
        }));

        let distinct = logical.add_operator(Distinct);
        let sink = logical.add_operator(BindingsExpr::Sink);

        logical.extend_with_flows(&[
            (scan, filter),
            (filter, project),
            (project, distinct),
            (distinct, sink),
        ]);

        if let Value::Bag(b) = evaluate(logical, data_customer()) {
            assert_eq!(b.len(), 2);
        } else {
            panic!("Wrong output")
        }
    }

    mod clause_from {
        use partiql_value::{partiql_bag, partiql_list, BindingsName};

        use crate::eval::{
            BasicContext, EvalPath, EvalPathComponent, EvalScan, EvalVarRef, Evaluable,
        };

        use super::*;

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

            let ctx = BasicContext::new(p0);

            let mut scan = EvalScan::new(
                Box::new(EvalVarRef {
                    name: BindingsName::CaseInsensitive("someOrderedTable".to_string()),
                }),
                "x",
                "y",
            );

            let res = scan.evaluate(&ctx);

            println!("{:?}", &scan.output);

            // <<{ y: 0, x:  { b: 0, a: 0 } },  { x:  { b: 1, a: 1 }, y: 1 }>>
            let expected = partiql_bag![
                partiql_tuple![("x", partiql_tuple![("a", 0), ("b", 0)]), ("y", 0)],
                partiql_tuple![("x", partiql_tuple![("a", 1), ("b", 1)]), ("y", 1)],
            ];
            assert_eq!(Value::Bag(Box::new(expected)), res.unwrap());
        }

        // Spec 5.1.1
        #[test]
        fn mistype_at_on_bag() {
            let mut p0: MapBindings<Value> = MapBindings::default();
            p0.insert("someUnorderedTable", some_unordered_table().into());

            let ctx = BasicContext::new(p0);

            let mut scan = EvalScan::new(
                Box::new(EvalVarRef {
                    name: BindingsName::CaseInsensitive("someUnorderedTable".to_string()),
                }),
                "x",
                "y",
            );

            let res = scan.evaluate(&ctx);

            println!("{:?}", &scan.output);

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
            assert_eq!(Value::Bag(Box::new(expected)), res.unwrap());
        }

        // Spec 5.1.1
        #[test]
        fn mistype_scalar() {
            let mut p0: MapBindings<Value> = MapBindings::default();
            p0.insert("someOrderedTable", some_ordered_table().into());

            let table_ref = EvalVarRef {
                name: BindingsName::CaseInsensitive("someOrderedTable".to_string()),
            };
            let path_to_scalar = EvalPath {
                expr: Box::new(table_ref),
                components: vec![
                    EvalPathComponent::Index(0),
                    EvalPathComponent::Key("a".into()),
                ],
            };
            let mut scan = EvalScan::new(Box::new(path_to_scalar), "x", "");

            let ctx = BasicContext::new(p0);
            let scan_res = scan.evaluate(&ctx);

            println!("{:?}", &scan.output);

            let expected = partiql_bag![partiql_tuple![("x", 0)]];
            assert_eq!(Value::Bag(Box::new(expected)), scan_res.unwrap());
        }

        // Spec 5.1.1
        #[test]
        fn mistype_absent() {
            let mut p0: MapBindings<Value> = MapBindings::default();
            p0.insert("someOrderedTable", some_ordered_table().into());

            let table_ref = EvalVarRef {
                name: BindingsName::CaseInsensitive("someOrderedTable".to_string()),
            };
            let path_to_scalar = EvalPath {
                expr: Box::new(table_ref),
                components: vec![
                    EvalPathComponent::Index(0),
                    EvalPathComponent::Key("c".into()),
                ],
            };
            let mut scan = EvalScan::new(Box::new(path_to_scalar), "x", "");

            let ctx = BasicContext::new(p0);
            let res = scan.evaluate(&ctx);

            println!("{:?}", &scan.output.unwrap());
            let expected = partiql_bag![partiql_tuple![("x", value::Value::Missing)]];
            assert_eq!(Value::Bag(Box::new(expected)), res.unwrap());
        }
    }

    mod clause_unpivot {
        use partiql_value::{partiql_bag, BindingsName, Tuple};

        use crate::eval::{BasicContext, EvalUnpivot, EvalVarRef, Evaluable};

        use super::*;

        fn just_a_tuple() -> Tuple {
            partiql_tuple![("amzn", 840.05), ("tdc", 31.06)]
        }

        // Spec 5.2
        #[test]
        fn basic() {
            let mut p0: MapBindings<Value> = MapBindings::default();
            p0.insert("justATuple", just_a_tuple().into());

            let mut unpivot = EvalUnpivot::new(
                Box::new(EvalVarRef {
                    name: BindingsName::CaseInsensitive("justATuple".to_string()),
                }),
                "price",
                "symbol",
            );

            let ctx = BasicContext::new(p0);
            let res = unpivot.evaluate(&ctx);

            println!("{:?}", &unpivot.output.unwrap());
            let expected = partiql_bag![
                partiql_tuple![("symbol", "tdc"), ("price", 31.06)],
                partiql_tuple![("symbol", "amzn"), ("price", 840.05)],
            ];
            assert_eq!(Value::Bag(Box::new(expected)), res.unwrap());
        }

        // Spec 5.2.1
        #[test]
        fn mistype_non_tuple() {
            let mut p0: MapBindings<Value> = MapBindings::default();
            p0.insert("nonTuple", Value::from(1));

            let mut unpivot = EvalUnpivot::new(
                Box::new(EvalVarRef {
                    name: BindingsName::CaseInsensitive("nonTuple".to_string()),
                }),
                "x",
                "y",
            );

            let ctx = BasicContext::new(p0);
            let res = unpivot.evaluate(&ctx);

            println!("{:?}", &unpivot.output);
            let expected = partiql_bag![partiql_tuple![("x", 1), ("y", "_1")]];
            assert_eq!(Value::Bag(Box::new(expected)), res.unwrap());
        }
    }
}
