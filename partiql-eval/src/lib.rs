pub mod env;
pub mod eval;
pub mod plan;

#[cfg(test)]
mod tests {
    use std::collections::HashMap;

    use crate::env::basic::MapBindings;
    use crate::plan;
    use rust_decimal_macros::dec;

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

    // Creates the plan: `SELECT <lhs> <op> <rhs> AS result FROM data` where <lhs> comes from data
    // Evaluates the plan and asserts the result is a bag of the tuple mapping to `expected_first_elem`
    // (i.e. <<{'result': <expected_first_elem>}>>)
    // TODO: once eval conformance tests added and/or modified evaluation API (to support other values
    //  in evaluator output), change or delete tests using this function
    fn eval_bin_op(op: BinaryOp, lhs: Value, rhs: Value, expected_first_elem: Value) {
        let mut plan = LogicalPlan::new();
        let scan = plan.add_operator(BindingsExpr::Scan(logical::Scan {
            expr: ValueExpr::VarRef(BindingsName::CaseInsensitive("data".into())),
            as_key: "data".to_string(),
            at_key: None,
        }));

        let project = plan.add_operator(BindingsExpr::Project(logical::Project {
            exprs: HashMap::from([(
                "result".to_string(),
                ValueExpr::BinaryExpr(
                    op,
                    Box::new(ValueExpr::Path(
                        Box::new(ValueExpr::VarRef(BindingsName::CaseInsensitive(
                            "data".into(),
                        ))),
                        vec![PathComponent::Key("lhs".to_string())],
                    )),
                    Box::new(ValueExpr::Lit(Box::new(rhs))),
                ),
            )]),
        }));

        let sink = plan.add_operator(BindingsExpr::Sink);
        plan.extend_with_flows(&[(scan, project), (project, sink)]);

        let mut bindings = MapBindings::default();
        bindings.insert(
            "data",
            partiql_list![Tuple(HashMap::from([("lhs".into(), lhs)]))].into(),
        );

        let result = evaluate(plan, bindings).coerce_to_bag();
        assert!(!&result.is_empty());
        let expected_result = partiql_bag!(Tuple(HashMap::from([(
            "result".into(),
            expected_first_elem
        )])));
        assert_eq!(expected_result, result);
    }

    #[test]
    fn arithmetic_ops() {
        // Addition
        // Plan for `select lhs + rhs as result from data`
        eval_bin_op(
            BinaryOp::Add,
            Value::from(1),
            Value::from(2),
            Value::from(3),
        );
        eval_bin_op(
            BinaryOp::Add,
            Value::from(1),
            Value::from(2.),
            Value::from(3.),
        );
        eval_bin_op(
            BinaryOp::Add,
            Value::from(1.),
            Value::from(2),
            Value::from(3.),
        );
        eval_bin_op(
            BinaryOp::Add,
            Value::from(1.),
            Value::from(2.),
            Value::from(3.),
        );
        eval_bin_op(
            BinaryOp::Add,
            Value::from(1),
            Value::from(dec!(2.)),
            Value::from(dec!(3.)),
        );
        eval_bin_op(
            BinaryOp::Add,
            Value::from(1.),
            Value::from(dec!(2.)),
            Value::from(dec!(3.)),
        );
        eval_bin_op(
            BinaryOp::Add,
            Value::from(dec!(1.)),
            Value::from(2),
            Value::from(dec!(3.)),
        );
        eval_bin_op(
            BinaryOp::Add,
            Value::from(dec!(1.)),
            Value::from(2.),
            Value::from(dec!(3.)),
        );
        eval_bin_op(
            BinaryOp::Add,
            Value::from(dec!(1.)),
            Value::from(dec!(2.)),
            Value::from(dec!(3.)),
        );
        eval_bin_op(BinaryOp::Add, Value::Null, Value::Null, Value::Null);
        eval_bin_op(
            BinaryOp::Add,
            Value::Missing,
            Value::Missing,
            Value::Missing,
        );

        // Subtraction
        // Plan for `select lhs - rhs as result from data`
        eval_bin_op(
            BinaryOp::Sub,
            Value::from(1),
            Value::from(2),
            Value::from(-1),
        );
        eval_bin_op(
            BinaryOp::Sub,
            Value::from(1),
            Value::from(2.),
            Value::from(-1.),
        );
        eval_bin_op(
            BinaryOp::Sub,
            Value::from(1.),
            Value::from(2),
            Value::from(-1.),
        );
        eval_bin_op(
            BinaryOp::Sub,
            Value::from(1.),
            Value::from(2.),
            Value::from(-1.),
        );
        eval_bin_op(
            BinaryOp::Sub,
            Value::from(1),
            Value::from(dec!(2.)),
            Value::from(dec!(-1.)),
        );
        eval_bin_op(
            BinaryOp::Sub,
            Value::from(1.),
            Value::from(dec!(2.)),
            Value::from(dec!(-1.)),
        );
        eval_bin_op(
            BinaryOp::Sub,
            Value::from(dec!(1.)),
            Value::from(2),
            Value::from(dec!(-1.)),
        );
        eval_bin_op(
            BinaryOp::Sub,
            Value::from(dec!(1.)),
            Value::from(2.),
            Value::from(dec!(-1.)),
        );
        eval_bin_op(
            BinaryOp::Sub,
            Value::from(dec!(1.)),
            Value::from(dec!(2.)),
            Value::from(dec!(-1.)),
        );
        eval_bin_op(BinaryOp::Sub, Value::Null, Value::Null, Value::Null);
        eval_bin_op(
            BinaryOp::Sub,
            Value::Missing,
            Value::Missing,
            Value::Missing,
        );

        // Multiplication
        // Plan for `select lhs * rhs as result from data`
        eval_bin_op(
            BinaryOp::Mul,
            Value::from(1),
            Value::from(2),
            Value::from(2),
        );
        eval_bin_op(
            BinaryOp::Mul,
            Value::from(1),
            Value::from(2.),
            Value::from(2.),
        );
        eval_bin_op(
            BinaryOp::Mul,
            Value::from(1.),
            Value::from(2),
            Value::from(2.),
        );
        eval_bin_op(
            BinaryOp::Mul,
            Value::from(1.),
            Value::from(2.),
            Value::from(2.),
        );
        eval_bin_op(
            BinaryOp::Mul,
            Value::from(1),
            Value::from(dec!(2.)),
            Value::from(dec!(2.)),
        );
        eval_bin_op(
            BinaryOp::Mul,
            Value::from(1.),
            Value::from(dec!(2.)),
            Value::from(dec!(2.)),
        );
        eval_bin_op(
            BinaryOp::Mul,
            Value::from(dec!(1.)),
            Value::from(2),
            Value::from(dec!(2.)),
        );
        eval_bin_op(
            BinaryOp::Mul,
            Value::from(dec!(1.)),
            Value::from(2.),
            Value::from(dec!(2.)),
        );
        eval_bin_op(
            BinaryOp::Mul,
            Value::from(dec!(1.)),
            Value::from(dec!(2.)),
            Value::from(dec!(2.)),
        );
        eval_bin_op(BinaryOp::Mul, Value::Null, Value::Null, Value::Null);
        eval_bin_op(
            BinaryOp::Mul,
            Value::Missing,
            Value::Missing,
            Value::Missing,
        );

        // Division
        // Plan for `select lhs / rhs as result from data`
        eval_bin_op(
            BinaryOp::Div,
            Value::from(1),
            Value::from(2),
            Value::from(0),
        );
        eval_bin_op(
            BinaryOp::Div,
            Value::from(1),
            Value::from(2.),
            Value::from(0.5),
        );
        eval_bin_op(
            BinaryOp::Div,
            Value::from(1.),
            Value::from(2),
            Value::from(0.5),
        );
        eval_bin_op(
            BinaryOp::Div,
            Value::from(1.),
            Value::from(2.),
            Value::from(0.5),
        );
        eval_bin_op(
            BinaryOp::Div,
            Value::from(1),
            Value::from(dec!(2.)),
            Value::from(dec!(0.5)),
        );
        eval_bin_op(
            BinaryOp::Div,
            Value::from(1.),
            Value::from(dec!(2.)),
            Value::from(dec!(0.5)),
        );
        eval_bin_op(
            BinaryOp::Div,
            Value::from(dec!(1.)),
            Value::from(2),
            Value::from(dec!(0.5)),
        );
        eval_bin_op(
            BinaryOp::Div,
            Value::from(dec!(1.)),
            Value::from(2.),
            Value::from(dec!(0.5)),
        );
        eval_bin_op(
            BinaryOp::Div,
            Value::from(dec!(1.)),
            Value::from(dec!(2.)),
            Value::from(dec!(0.5)),
        );
        eval_bin_op(BinaryOp::Div, Value::Null, Value::Null, Value::Null);
        eval_bin_op(
            BinaryOp::Div,
            Value::Missing,
            Value::Missing,
            Value::Missing,
        );

        // Modulo
        // Plan for `select lhs % rhs as result from data`
        eval_bin_op(
            BinaryOp::Mod,
            Value::from(1),
            Value::from(2),
            Value::from(1),
        );
        eval_bin_op(
            BinaryOp::Mod,
            Value::from(1),
            Value::from(2.),
            Value::from(1.),
        );
        eval_bin_op(
            BinaryOp::Mod,
            Value::from(1.),
            Value::from(2),
            Value::from(1.),
        );
        eval_bin_op(
            BinaryOp::Mod,
            Value::from(1.),
            Value::from(2.),
            Value::from(1.),
        );
        eval_bin_op(
            BinaryOp::Mod,
            Value::from(1),
            Value::from(dec!(2.)),
            Value::from(dec!(1.)),
        );
        eval_bin_op(
            BinaryOp::Mod,
            Value::from(1.),
            Value::from(dec!(2.)),
            Value::from(dec!(1.)),
        );
        eval_bin_op(
            BinaryOp::Mod,
            Value::from(dec!(1.)),
            Value::from(2),
            Value::from(dec!(1.)),
        );
        eval_bin_op(
            BinaryOp::Mod,
            Value::from(dec!(1.)),
            Value::from(2.),
            Value::from(dec!(1.)),
        );
        eval_bin_op(
            BinaryOp::Mod,
            Value::from(dec!(1.)),
            Value::from(dec!(2.)),
            Value::from(dec!(1.)),
        );
        eval_bin_op(BinaryOp::Mod, Value::Null, Value::Null, Value::Null);
        eval_bin_op(
            BinaryOp::Mod,
            Value::Missing,
            Value::Missing,
            Value::Missing,
        );
    }

    #[test]
    fn select() {
        let mut lg = LogicalPlan::new();

        let from = lg.add_operator(BindingsExpr::Scan(logical::Scan {
            expr: ValueExpr::VarRef(BindingsName::CaseInsensitive("data".into())),
            as_key: "data".to_string(),
            at_key: None,
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
            at_key: None,
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

            let mut scan = EvalScan::new_with_at_key(
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

            let mut scan = EvalScan::new_with_at_key(
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
            let mut scan = EvalScan::new(Box::new(path_to_scalar), "x");

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
            let mut scan = EvalScan::new(Box::new(path_to_scalar), "x");

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
