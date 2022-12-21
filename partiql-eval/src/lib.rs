pub mod env;
pub mod eval;
pub mod plan;

#[cfg(test)]
mod tests {
    use assert_matches::assert_matches;
    use std::collections::HashMap;

    use crate::env::basic::MapBindings;
    use crate::plan;
    use rust_decimal_macros::dec;

    use partiql_logical as logical;
    use partiql_logical::BindingsOp::{Distinct, Project, ProjectAll, ProjectValue};

    use partiql_logical::{
        BagExpr, BetweenExpr, BinaryOp, BindingsOp, CoalesceExpr, ExprQuery, IsTypeExpr, JoinKind,
        ListExpr, LogicalPlan, NullIfExpr, PathComponent, TupleExpr, Type, ValueExpr,
    };
    use partiql_value as value;
    use partiql_value::Value::{Missing, Null};
    use partiql_value::{
        partiql_bag, partiql_list, partiql_tuple, Bag, BindingsName, List, Tuple, Value,
    };

    fn evaluate(logical: LogicalPlan<BindingsOp>, bindings: MapBindings<Value>) -> Value {
        let planner = plan::EvaluatorPlanner;
        let mut plan = planner.compile(&logical);

        if let Ok(out) = plan.execute_mut(bindings) {
            out.result
        } else {
            Missing
        }
    }

    fn data_customer() -> MapBindings<Value> {
        fn customer_tuple(id: i64, first_name: &str, balance: i64) -> Value {
            partiql_tuple![("id", id), ("firstName", first_name), ("balance", balance),].into()
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
        fn a_tuple(n: i64) -> Value {
            partiql_tuple![("a", n)].into()
        }

        let data = partiql_list![a_tuple(1), a_tuple(2), a_tuple(3)];

        let mut bindings = MapBindings::default();
        bindings.insert("data", data.into());
        bindings
    }

    fn scan(name: &str, as_key: &str) -> BindingsOp {
        BindingsOp::Scan(logical::Scan {
            expr: ValueExpr::VarRef(BindingsName::CaseInsensitive(name.into())),
            as_key: as_key.to_string(),
            at_key: None,
        })
    }

    fn path_var(name: &str, component: &str) -> ValueExpr {
        ValueExpr::Path(
            Box::new(ValueExpr::VarRef(BindingsName::CaseInsensitive(
                name.into(),
            ))),
            vec![PathComponent::Key(BindingsName::CaseInsensitive(
                component.to_string(),
            ))],
        )
    }

    fn join_data() -> MapBindings<Value> {
        let customers = partiql_list![
            partiql_tuple![("id", 5), ("name", "Joe")],
            partiql_tuple![("id", 7), ("name", "Mary")],
        ];

        let orders = partiql_list![
            partiql_tuple![("custId", 7), ("productId", 101)],
            partiql_tuple![("custId", 7), ("productId", 523)],
        ];

        let mut bindings = MapBindings::default();
        bindings.insert("customers", customers.into());
        bindings.insert("orders", orders.into());
        bindings
    }

    fn join_data_sensors() -> MapBindings<Value> {
        let sensors = partiql_list![
            partiql_tuple![(
                "readings",
                partiql_list![partiql_tuple![("v", 1.3)], partiql_tuple![("v", 2)],]
            )],
            partiql_tuple![(
                "readings",
                partiql_list![
                    partiql_tuple![("v", 0.7)],
                    partiql_tuple![("v", 0.8)],
                    partiql_tuple![("v", 0.9)],
                ]
            )],
        ];
        let mut bindings = MapBindings::default();
        bindings.insert("sensors", sensors.into());
        bindings
    }

    fn join_data_sensors_with_empty_table() -> MapBindings<Value> {
        let sensors = partiql_list![
            partiql_tuple![(
                "readings",
                partiql_list![partiql_tuple![("v", 1.3)], partiql_tuple![("v", 2)],]
            )],
            partiql_tuple![(
                "readings",
                partiql_list![
                    partiql_tuple![("v", 0.7)],
                    partiql_tuple![("v", 0.8)],
                    partiql_tuple![("v", 0.9)],
                ]
            )],
            partiql_tuple![("readings", partiql_list![])],
        ];
        let mut bindings = MapBindings::default();
        bindings.insert("sensors", sensors.into());
        bindings
    }

    fn case_when_data() -> MapBindings<Value> {
        let nums = partiql_list![
            partiql_tuple![("a", 1)],
            partiql_tuple![("a", 2)],
            partiql_tuple![("a", 3)],
            partiql_tuple![("a", Null)],
            partiql_tuple![("a", Missing)],
            partiql_tuple![("a", "foo")],
        ];

        let mut bindings = MapBindings::default();
        bindings.insert("nums", nums.into());
        bindings
    }

    // Creates the plan: `SELECT <lhs> <op> <rhs> AS result FROM data` where <lhs> comes from data
    // Evaluates the plan and asserts the result is a bag of the tuple mapping to `expected_first_elem`
    // (i.e. <<{'result': <expected_first_elem>}>>)
    // TODO: once eval conformance tests added and/or modified evaluation API (to support other values
    //  in evaluator output), change or delete tests using this function
    fn eval_bin_op(op: BinaryOp, lhs: Value, rhs: Value, expected_first_elem: Value) {
        let mut plan = LogicalPlan::new();
        let scan = plan.add_operator(BindingsOp::Scan(logical::Scan {
            expr: ValueExpr::VarRef(BindingsName::CaseInsensitive("data".into())),
            as_key: "data".to_string(),
            at_key: None,
        }));

        let project = plan.add_operator(Project(logical::Project {
            exprs: HashMap::from([(
                "result".to_string(),
                ValueExpr::BinaryExpr(
                    op,
                    Box::new(ValueExpr::Path(
                        Box::new(ValueExpr::VarRef(BindingsName::CaseInsensitive(
                            "data".into(),
                        ))),
                        vec![PathComponent::Key(BindingsName::CaseInsensitive(
                            "lhs".to_string(),
                        ))],
                    )),
                    Box::new(ValueExpr::Lit(Box::new(rhs))),
                ),
            )]),
        }));

        let sink = plan.add_operator(BindingsOp::Sink);
        plan.extend_with_flows(&[(scan, project), (project, sink)]);

        let mut bindings = MapBindings::default();
        bindings.insert("data", partiql_list![Tuple::from([("lhs", lhs)])].into());

        let result = evaluate(plan, bindings).coerce_to_bag();
        assert!(!&result.is_empty());
        let expected_result = if expected_first_elem != Missing {
            partiql_bag!(Tuple::from([("result", expected_first_elem)]))
        } else {
            // Filter tuples with `MISSING` vals
            partiql_bag!(Tuple::new())
        };
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
        eval_bin_op(BinaryOp::Add, Null, Null, Null);
        eval_bin_op(BinaryOp::Add, Missing, Missing, Missing);

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
        eval_bin_op(BinaryOp::Sub, Null, Null, Null);
        eval_bin_op(BinaryOp::Sub, Missing, Missing, Missing);

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
        eval_bin_op(BinaryOp::Mul, Null, Null, Null);
        eval_bin_op(BinaryOp::Mul, Missing, Missing, Missing);

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
        eval_bin_op(BinaryOp::Div, Null, Null, Null);
        eval_bin_op(BinaryOp::Div, Missing, Missing, Missing);

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
        eval_bin_op(BinaryOp::Mod, Null, Null, Null);
        eval_bin_op(BinaryOp::Mod, Missing, Missing, Missing);
    }

    #[test]
    fn in_expr() {
        eval_bin_op(
            BinaryOp::In,
            Value::from(1),
            Value::from(partiql_list![1, 2, 3]),
            Value::from(true),
        );
        // We still need to define the rules of coercion for `IN` RHS.
        // See also:
        // - https://github.com/partiql/partiql-docs/pull/13
        // - https://github.com/partiql/partiql-lang-kotlin/issues/524
        // - https://github.com/partiql/partiql-lang-kotlin/pull/621#issuecomment-1147754213
        eval_bin_op(
            BinaryOp::In,
            Value::from(partiql_tuple![("a", 2)]),
            Value::from(partiql_list![
                partiql_tuple![("a", 6)],
                partiql_tuple![("b", 12)],
                partiql_tuple![("a", 2)]
            ]),
            Value::from(true),
        );
        eval_bin_op(
            BinaryOp::In,
            Value::from(10),
            Value::from(partiql_bag!["a", "b", 11]),
            Value::from(false),
        );
        eval_bin_op(BinaryOp::In, Value::from(1), Value::from(1), Null);
        eval_bin_op(
            BinaryOp::In,
            Value::from(1),
            Value::from(partiql_list![10, Missing, "b"]),
            Null,
        );
        eval_bin_op(
            BinaryOp::In,
            Missing,
            Value::from(partiql_list![1, Missing, "b"]),
            Null,
        );
        eval_bin_op(
            BinaryOp::In,
            Null,
            Value::from(partiql_list![1, Missing, "b"]),
            Null,
        );
        eval_bin_op(
            BinaryOp::In,
            Value::from(1),
            Value::from(partiql_list![1, Null, "b"]),
            Value::from(true),
        );
        eval_bin_op(
            BinaryOp::In,
            Value::from(1),
            Value::from(partiql_list![3, Null]),
            Null,
        );
    }

    #[test]
    fn comparison_ops() {
        // Lt
        // Plan for `select lhs < rhs as result from data`
        eval_bin_op(
            BinaryOp::Lt,
            Value::from(1),
            Value::from(2.),
            Value::from(true),
        );
        eval_bin_op(
            BinaryOp::Lt,
            Value::from("abc"),
            Value::from("def"),
            Value::from(true),
        );
        eval_bin_op(BinaryOp::Lt, Missing, Value::from(2.), Missing);
        eval_bin_op(BinaryOp::Lt, Null, Value::from(2.), Null);
        eval_bin_op(BinaryOp::Lt, Value::from(1), Value::from("foo"), Missing);

        // Gt
        // Plan for `select lhs > rhs as result from data`
        eval_bin_op(
            BinaryOp::Gt,
            Value::from(1),
            Value::from(2.),
            Value::from(false),
        );
        eval_bin_op(
            BinaryOp::Gt,
            Value::from("abc"),
            Value::from("def"),
            Value::from(false),
        );
        eval_bin_op(BinaryOp::Gt, Missing, Value::from(2.), Missing);
        eval_bin_op(BinaryOp::Gt, Null, Value::from(2.), Null);
        eval_bin_op(BinaryOp::Gt, Value::from(1), Value::from("foo"), Missing);

        // Lteq
        // Plan for `select lhs <= rhs as result from data`
        eval_bin_op(
            BinaryOp::Lteq,
            Value::from(1),
            Value::from(2.),
            Value::from(true),
        );
        eval_bin_op(
            BinaryOp::Lteq,
            Value::from("abc"),
            Value::from("def"),
            Value::from(true),
        );
        eval_bin_op(BinaryOp::Lteq, Missing, Value::from(2.), Missing);
        eval_bin_op(BinaryOp::Lt, Null, Value::from(2.), Null);
        eval_bin_op(BinaryOp::Lteq, Value::from(1), Value::from("foo"), Missing);

        // Gteq
        // Plan for `select lhs >= rhs as result from data`
        eval_bin_op(
            BinaryOp::Gteq,
            Value::from(1),
            Value::from(2.),
            Value::from(false),
        );
        eval_bin_op(
            BinaryOp::Gteq,
            Value::from("abc"),
            Value::from("def"),
            Value::from(false),
        );
        eval_bin_op(BinaryOp::Gteq, Missing, Value::from(2.), Missing);
        eval_bin_op(BinaryOp::Gteq, Null, Value::from(2.), Null);
        eval_bin_op(BinaryOp::Gteq, Value::from(1), Value::from("foo"), Missing);
    }

    #[test]
    fn and_or_null() {
        fn eval_to_null(op: BinaryOp, lhs: Value, rhs: Value) {
            let mut plan = LogicalPlan::new();
            let expq = plan.add_operator(BindingsOp::ExprQuery(ExprQuery {
                expr: ValueExpr::BinaryExpr(
                    op,
                    Box::new(ValueExpr::Lit(Box::new(lhs))),
                    Box::new(ValueExpr::Lit(Box::new(rhs))),
                ),
            }));

            let sink = plan.add_operator(BindingsOp::Sink);
            plan.add_flow(expq, sink);

            let result = evaluate(plan, MapBindings::default());
            assert_eq!(result, Value::Null);
        }

        eval_to_null(BinaryOp::And, Value::Null, Value::Boolean(true));
        eval_to_null(BinaryOp::And, Value::Missing, Value::Boolean(true));
        eval_to_null(BinaryOp::And, Value::Boolean(true), Value::Null);
        eval_to_null(BinaryOp::And, Value::Boolean(true), Value::Missing);
        eval_to_null(BinaryOp::Or, Value::Null, Value::Boolean(false));
        eval_to_null(BinaryOp::Or, Value::Missing, Value::Boolean(false));
        eval_to_null(BinaryOp::Or, Value::Boolean(false), Value::Null);
        eval_to_null(BinaryOp::Or, Value::Boolean(false), Value::Missing);
    }

    #[test]
    fn between_op() {
        fn eval_between_op(value: Value, from: Value, to: Value, expected_first_elem: Value) {
            let mut plan = LogicalPlan::new();
            let scan = plan.add_operator(BindingsOp::Scan(logical::Scan {
                expr: ValueExpr::VarRef(BindingsName::CaseInsensitive("data".into())),
                as_key: "data".to_string(),
                at_key: None,
            }));

            let project = plan.add_operator(Project(logical::Project {
                exprs: HashMap::from([(
                    "result".to_string(),
                    ValueExpr::BetweenExpr(BetweenExpr {
                        value: Box::new(ValueExpr::Path(
                            Box::new(ValueExpr::VarRef(BindingsName::CaseInsensitive(
                                "data".into(),
                            ))),
                            vec![PathComponent::Key(BindingsName::CaseInsensitive(
                                "value".to_string(),
                            ))],
                        )),
                        from: Box::new(ValueExpr::Lit(Box::new(from))),
                        to: Box::new(ValueExpr::Lit(Box::new(to))),
                    }),
                )]),
            }));

            let sink = plan.add_operator(BindingsOp::Sink);
            plan.extend_with_flows(&[(scan, project), (project, sink)]);

            let mut bindings = MapBindings::default();
            bindings.insert(
                "data",
                partiql_list![Tuple::from([("value", value)])].into(),
            );

            let result = evaluate(plan, bindings).coerce_to_bag();
            assert!(!&result.is_empty());
            let expected_result = partiql_bag!(Tuple::from([("result", expected_first_elem)]));
            assert_eq!(expected_result, result);
        }
        eval_between_op(
            Value::from(2),
            Value::from(1),
            Value::from(3),
            Value::from(true),
        );
        eval_between_op(
            Value::from(2),
            Value::from(1.),
            Value::from(dec!(3.)),
            Value::from(true),
        );
        eval_between_op(
            Value::from(1),
            Value::from(2),
            Value::from(3),
            Value::from(false),
        );
        eval_between_op(Null, Value::from(1), Value::from(3), Null);
        eval_between_op(Value::from(2), Null, Value::from(3), Null);
        eval_between_op(Value::from(2), Value::from(1), Null, Null);
        eval_between_op(Missing, Value::from(1), Value::from(3), Null);
        eval_between_op(Value::from(2), Missing, Value::from(3), Null);
        eval_between_op(Value::from(2), Value::from(1), Missing, Null);
        // left part of AND evaluates to false
        eval_between_op(Value::from(1), Value::from(2), Null, Value::from(false));
        eval_between_op(Value::from(1), Value::from(2), Missing, Value::from(false));
    }

    #[test]
    fn select_with_join_and_on() {
        // Similar to ex 9 from spec with projected columns from different tables with an inner JOIN and ON condition
        // SELECT c.id, c.name, o.custId, o.productId FROM customers AS c, orders AS o ON c.id = o.custId
        let from_lhs = scan("customers", "c");
        let from_rhs = scan("orders", "o");

        let mut lg = LogicalPlan::new();
        let project = lg.add_operator(Project(logical::Project {
            exprs: HashMap::from([
                ("id".to_string(), path_var("c", "id")),
                ("name".to_string(), path_var("c", "name")),
                ("custId".to_string(), path_var("o", "custId")),
                ("productId".to_string(), path_var("o", "productId")),
            ]),
        }));

        let join = lg.add_operator(BindingsOp::Join(logical::Join {
            kind: JoinKind::Cross,
            left: Box::new(from_lhs),
            right: Box::new(from_rhs),
            on: Some(ValueExpr::BinaryExpr(
                BinaryOp::Eq,
                Box::new(path_var("c", "id")),
                Box::new(path_var("o", "custId")),
            )),
        }));

        let sink = lg.add_operator(BindingsOp::Sink);
        lg.add_flow_with_branch_num(join, project, 0);
        lg.add_flow_with_branch_num(project, sink, 0);

        let out = evaluate(lg, join_data());
        println!("{:?}", &out);

        assert_matches!(out, Value::Bag(bag) => {
            let expected = partiql_bag![
                partiql_tuple![("custId", 7), ("name", "Mary"), ("id", 7), ("productId", 101)],
                partiql_tuple![("custId", 7), ("name", "Mary"), ("id", 7), ("productId", 523)],
            ];
            assert_eq!(*bag, expected);
        });
    }

    #[test]
    fn select_with_cross_join_sensors() {
        // Similar to example 10 from PartiQL spec. Equivalent to query:
        //  SELECT r.v AS v FROM sensors AS s, s.readings AS r
        // Above demonstrates LATERAL JOINs since the RHS of the JOIN uses bindings defined from the
        // LHS scan
        let mut lg = LogicalPlan::new();

        let from_lhs = scan("sensors", "s");
        let from_rhs = BindingsOp::Scan(logical::Scan {
            expr: path_var("s", "readings"),
            as_key: "r".to_string(),
            at_key: None,
        });

        let project = lg.add_operator(Project(logical::Project {
            exprs: HashMap::from([("v".to_string(), path_var("r", "v"))]),
        }));

        let join = lg.add_operator(BindingsOp::Join(logical::Join {
            kind: JoinKind::Cross,
            left: Box::new(from_lhs),
            right: Box::new(from_rhs),
            on: None,
        }));

        let sink = lg.add_operator(BindingsOp::Sink);
        lg.add_flow_with_branch_num(join, project, 0);
        lg.add_flow_with_branch_num(project, sink, 0);

        let out = evaluate(lg, join_data_sensors());
        println!("{:?}", &out);

        assert_matches!(out, Value::Bag(bag) => {
            let expected = partiql_bag![
                partiql_tuple![("v", 1.3)],
                partiql_tuple![("v", 2)],
                partiql_tuple![("v", 0.7)],
                partiql_tuple![("v", 0.8)],
                partiql_tuple![("v", 0.9)],
            ];
            assert_eq!(*bag, expected);
        });
    }

    #[test]
    fn select_with_cross_join_sensors_with_empty_table() {
        // Similar to example 11 from PartiQL spec. Equivalent to query:
        //  SELECT r.v AS v FROM sensors AS s, s.readings AS r
        // Above uses a different `sensors` table which includes an empty list for `readings` than
        // example 10. This demonstrates that binding tuple is excluded for CROSS JOINs
        let mut lg = LogicalPlan::new();

        let from_lhs = scan("sensors", "s");
        let from_rhs = BindingsOp::Scan(logical::Scan {
            expr: path_var("s", "readings"),
            as_key: "r".to_string(),
            at_key: None,
        });

        let project = lg.add_operator(Project(logical::Project {
            exprs: HashMap::from([("v".to_string(), path_var("r", "v"))]),
        }));

        let join = lg.add_operator(BindingsOp::Join(logical::Join {
            kind: JoinKind::Cross,
            left: Box::new(from_lhs),
            right: Box::new(from_rhs),
            on: None,
        }));

        let sink = lg.add_operator(BindingsOp::Sink);
        lg.add_flow_with_branch_num(join, project, 0);
        lg.add_flow_with_branch_num(project, sink, 0);

        let out = evaluate(lg, join_data_sensors_with_empty_table());
        println!("{:?}", &out);

        assert_matches!(out, Value::Bag(bag) => {
            let expected = partiql_bag![
                partiql_tuple![("v", 1.3)],
                partiql_tuple![("v", 2)],
                partiql_tuple![("v", 0.7)],
                partiql_tuple![("v", 0.8)],
                partiql_tuple![("v", 0.9)],
            ];
            assert_eq!(*bag, expected);
        });
    }

    #[test]
    fn select_with_left_join_sensors_with_empty_table() {
        // Similar to example 11 from PartiQL spec. Equivalent to query:
        //  SELECT r AS r FROM sensors AS s LEFT CROSS JOIN s.readings AS r
        // Above uses a different `sensors` table which includes an empty list for `readings` than
        // example 10. This demonstrates that empty binding tuples are included for LEFT (CROSS)
        // JOINs and defined by a binding tuple with each variable mapping to NULL (see the last
        // tuple in the result).
        let mut lg = LogicalPlan::new();

        let from_lhs = scan("sensors", "s");
        let from_rhs = BindingsOp::Scan(logical::Scan {
            expr: path_var("s", "readings"),
            as_key: "r".to_string(),
            at_key: None,
        });

        let project = lg.add_operator(Project(logical::Project {
            exprs: HashMap::from([(
                "r".to_string(),
                ValueExpr::VarRef(BindingsName::CaseInsensitive("r".into())),
            )]),
        }));

        let join = lg.add_operator(BindingsOp::Join(logical::Join {
            kind: JoinKind::Left,
            left: Box::new(from_lhs),
            right: Box::new(from_rhs),
            on: Some(ValueExpr::Lit(Box::new(Value::from(true)))),
        }));

        let sink = lg.add_operator(BindingsOp::Sink);
        lg.add_flow_with_branch_num(join, project, 0);
        lg.add_flow_with_branch_num(project, sink, 0);

        let out = evaluate(lg, join_data_sensors_with_empty_table());
        println!("{:?}", &out);

        assert_matches!(out, Value::Bag(bag) => {
            let expected = partiql_bag![
                partiql_tuple![("r", partiql_tuple![("v", 1.3)])],
                partiql_tuple![("r", partiql_tuple![("v", 2)])],
                partiql_tuple![("r", partiql_tuple![("v", 0.7)])],
                partiql_tuple![("r", partiql_tuple![("v", 0.8)])],
                partiql_tuple![("r", partiql_tuple![("v", 0.9)])],
                partiql_tuple![("r", Null)],
            ];
            assert_eq!(*bag, expected);
        });
    }

    fn simple_case_expr_with_default() -> logical::SimpleCase {
        logical::SimpleCase {
            expr: Box::new(path_var("n", "a")),
            cases: vec![
                (
                    Box::new(ValueExpr::Lit(Box::new(Value::Integer(1)))),
                    Box::new(ValueExpr::Lit(Box::new(Value::from("one".to_string())))),
                ),
                (
                    Box::new(ValueExpr::Lit(Box::new(Value::Integer(2)))),
                    Box::new(ValueExpr::Lit(Box::new(Value::from("two".to_string())))),
                ),
            ],
            default: Some(Box::new(ValueExpr::Lit(Box::new(Value::from(
                "other".to_string(),
            ))))),
        }
    }

    fn searched_case_expr_with_default() -> logical::SearchedCase {
        logical::SearchedCase {
            cases: vec![
                (
                    Box::new(ValueExpr::BinaryExpr(
                        BinaryOp::Eq,
                        Box::new(path_var("n", "a")),
                        Box::new(ValueExpr::Lit(Box::new(Value::Integer(1)))),
                    )),
                    Box::new(ValueExpr::Lit(Box::new(Value::from("one".to_string())))),
                ),
                (
                    Box::new(ValueExpr::BinaryExpr(
                        BinaryOp::Eq,
                        Box::new(path_var("n", "a")),
                        Box::new(ValueExpr::Lit(Box::new(Value::Integer(2)))),
                    )),
                    Box::new(ValueExpr::Lit(Box::new(Value::from("two".to_string())))),
                ),
            ],
            default: Some(Box::new(ValueExpr::Lit(Box::new(Value::from(
                "other".to_string(),
            ))))),
        }
    }

    #[test]
    fn simple_case_when_expr_with_default() {
        let mut lg = LogicalPlan::new();
        // SELECT n.a,
        //        CASE n.a WHEN 1 THEN 'one'
        //               WHEN 2 THEN 'two'
        //               ELSE 'other'
        //        END AS b
        // FROM nums AS n
        let scan = lg.add_operator(scan("nums", "n"));

        let project_logical = Project(logical::Project {
            exprs: HashMap::from([
                ("a".to_string(), path_var("n", "a")),
                (
                    "b".to_string(),
                    ValueExpr::SimpleCase(simple_case_expr_with_default()),
                ),
            ]),
        });
        let project = lg.add_operator(project_logical);
        let sink = lg.add_operator(BindingsOp::Sink);
        lg.add_flow(scan, project);
        lg.add_flow(project, sink);

        let out = evaluate(lg, case_when_data());
        println!("{:?}", &out);

        assert_matches!(out, Value::Bag(bag) => {
            let expected = partiql_bag![
                partiql_tuple![("a", 1), ("b", "one")],
                partiql_tuple![("a", 2), ("b", "two")],
                partiql_tuple![("a", 3), ("b", "other")],
                partiql_tuple![("a", Null), ("b", "other")],
                partiql_tuple![("b", "other")],
                partiql_tuple![("a", "foo"), ("b", "other")],
            ];
            assert_eq!(*bag, expected);
        });
    }

    #[test]
    fn simple_case_when_expr_without_default() {
        let mut lg = LogicalPlan::new();
        // SELECT n.a,
        //        CASE n.a WHEN 1 THEN 'one'
        //               WHEN 2 THEN 'two'
        //        END AS b
        // FROM nums AS n
        let scan = lg.add_operator(scan("nums", "n"));
        let project_logical_no_default = Project(logical::Project {
            exprs: HashMap::from([
                ("a".to_string(), path_var("n", "a")),
                (
                    "b".to_string(),
                    ValueExpr::SimpleCase(logical::SimpleCase {
                        default: None,
                        ..simple_case_expr_with_default()
                    }),
                ),
            ]),
        });
        let project = lg.add_operator(project_logical_no_default);
        let sink = lg.add_operator(BindingsOp::Sink);
        lg.add_flow(scan, project);
        lg.add_flow(project, sink);

        let out = evaluate(lg, case_when_data());
        println!("{:?}", &out);

        assert_matches!(out, Value::Bag(bag) => {
            let expected = partiql_bag![
                partiql_tuple![("a", 1), ("b", "one")],
                partiql_tuple![("a", 2), ("b", "two")],
                partiql_tuple![("a", 3), ("b", Null)],
                partiql_tuple![("a", Null), ("b", Null)],
                partiql_tuple![("b", Null)],
                partiql_tuple![("a", "foo"), ("b", Null)],
            ];
            assert_eq!(*bag, expected);
        });
    }

    #[test]
    fn searched_case_when_expr_with_default() {
        let mut lg = LogicalPlan::new();
        // SELECT n.a,
        //        CASE WHEN n.a = 1 THEN 'one'
        //             WHEN n.a = 2 THEN 'two'
        //             ELSE 'other'
        //        END AS b
        // FROM nums AS n
        let scan = lg.add_operator(scan("nums", "n"));

        let project_logical = Project(logical::Project {
            exprs: HashMap::from([
                ("a".to_string(), path_var("n", "a")),
                (
                    "b".to_string(),
                    ValueExpr::SearchedCase(searched_case_expr_with_default()),
                ),
            ]),
        });
        let project = lg.add_operator(project_logical);
        let sink = lg.add_operator(BindingsOp::Sink);
        lg.add_flow(scan, project);
        lg.add_flow(project, sink);

        let out = evaluate(lg, case_when_data());
        println!("{:?}", &out);

        assert_matches!(out, Value::Bag(bag) => {
            let expected = partiql_bag![
                partiql_tuple![("a", 1), ("b", "one")],
                partiql_tuple![("a", 2), ("b", "two")],
                partiql_tuple![("a", 3), ("b", "other")],
                partiql_tuple![("a", Null), ("b", "other")],
                partiql_tuple![("b", "other")],
                partiql_tuple![("a", "foo"), ("b", "other")],
            ];
            assert_eq!(*bag, expected);
        });
    }

    #[test]
    fn searched_case_when_expr_without_default() {
        let mut lg = LogicalPlan::new();
        // SELECT n.a,
        //        CASE WHEN n.a = 1 THEN 'one'
        //             WHEN n.a = 2 THEN 'two'
        //        END AS b
        // FROM nums AS n
        let scan = lg.add_operator(scan("nums", "n"));
        let project_logical_no_default = Project(logical::Project {
            exprs: HashMap::from([
                ("a".to_string(), path_var("n", "a")),
                (
                    "b".to_string(),
                    ValueExpr::SearchedCase(logical::SearchedCase {
                        default: None,
                        ..searched_case_expr_with_default()
                    }),
                ),
            ]),
        });
        let project = lg.add_operator(project_logical_no_default);
        let sink = lg.add_operator(BindingsOp::Sink);
        lg.add_flow(scan, project);
        lg.add_flow(project, sink);

        let out = evaluate(lg, case_when_data());
        println!("{:?}", &out);

        assert_matches!(out, Value::Bag(bag) => {
            let expected = partiql_bag![
                partiql_tuple![("a", 1), ("b", "one")],
                partiql_tuple![("a", 2), ("b", "two")],
                partiql_tuple![("a", 3), ("b", Null)],
                partiql_tuple![("a", Null), ("b", Null)],
                partiql_tuple![("b", Null)],
                partiql_tuple![("a", "foo"), ("b", Null)],
            ];
            assert_eq!(*bag, expected);
        });
    }

    // Creates the plan: `SELECT <expr> IS [<not>] <is_type> AS result FROM data` where <expr> comes from data
    // Evaluates the plan and asserts the result is a bag of the tuple mapping to `expected_first_elem`
    // (i.e. <<{'result': <expected_first_elem>}>>)
    fn eval_is_op(not: bool, expr: Value, is_type: Type, expected_first_elem: Value) {
        let mut plan = LogicalPlan::new();
        let scan = plan.add_operator(BindingsOp::Scan(logical::Scan {
            expr: ValueExpr::VarRef(BindingsName::CaseInsensitive("data".into())),
            as_key: "data".to_string(),
            at_key: None,
        }));

        let project = plan.add_operator(Project(logical::Project {
            exprs: HashMap::from([(
                "result".to_string(),
                ValueExpr::IsTypeExpr(IsTypeExpr {
                    not,
                    expr: Box::new(ValueExpr::Path(
                        Box::new(ValueExpr::VarRef(BindingsName::CaseInsensitive(
                            "data".into(),
                        ))),
                        vec![PathComponent::Key(BindingsName::CaseInsensitive(
                            "expr".to_string(),
                        ))],
                    )),
                    is_type,
                }),
            )]),
        }));

        let sink = plan.add_operator(BindingsOp::Sink);
        plan.extend_with_flows(&[(scan, project), (project, sink)]);

        let mut bindings = MapBindings::default();
        bindings.insert("data", partiql_list![Tuple::from([("expr", expr)])].into());

        let result = evaluate(plan, bindings).coerce_to_bag();
        assert!(!&result.is_empty());
        assert_eq!(
            partiql_bag!(Tuple::from([("result", expected_first_elem)])),
            result
        );
    }

    #[test]
    fn is_type_null_missing() {
        // IS MISSING
        eval_is_op(false, Value::from(1), Type::MissingType, Value::from(false));
        eval_is_op(false, Value::Missing, Type::MissingType, Value::from(true));
        eval_is_op(false, Value::Null, Type::MissingType, Value::from(false));

        // IS NOT MISSING
        eval_is_op(true, Value::from(1), Type::MissingType, Value::from(true));
        eval_is_op(true, Value::Missing, Type::MissingType, Value::from(false));
        eval_is_op(true, Value::Null, Type::MissingType, Value::from(true));

        // IS NULL
        eval_is_op(false, Value::from(1), Type::NullType, Value::from(false));
        eval_is_op(false, Value::Missing, Type::NullType, Value::from(true));
        eval_is_op(false, Value::Null, Type::NullType, Value::from(true));

        // IS NOT NULL
        eval_is_op(true, Value::from(1), Type::NullType, Value::from(true));
        eval_is_op(true, Value::Missing, Type::NullType, Value::from(false));
        eval_is_op(true, Value::Null, Type::NullType, Value::from(false));
    }

    // Creates the plan: `SELECT NULLIF(<lhs>, <rhs>) AS result FROM data` where <lhs> comes from data
    // Evaluates the plan and asserts the result is a bag of the tuple mapping to `expected_first_elem`
    // (i.e. <<{'result': <expected_first_elem>}>>)
    fn eval_null_if_op(lhs: Value, rhs: Value, expected_first_elem: Value) {
        let mut plan = LogicalPlan::new();
        let scan = plan.add_operator(BindingsOp::Scan(logical::Scan {
            expr: ValueExpr::VarRef(BindingsName::CaseInsensitive("data".into())),
            as_key: "data".to_string(),
            at_key: None,
        }));

        let project = plan.add_operator(Project(logical::Project {
            exprs: HashMap::from([(
                "result".to_string(),
                ValueExpr::NullIfExpr(NullIfExpr {
                    lhs: Box::new(ValueExpr::Path(
                        Box::new(ValueExpr::VarRef(BindingsName::CaseInsensitive(
                            "data".into(),
                        ))),
                        vec![PathComponent::Key(BindingsName::CaseInsensitive(
                            "lhs".to_string(),
                        ))],
                    )),
                    rhs: Box::new(ValueExpr::Lit(Box::new(rhs))),
                }),
            )]),
        }));

        let sink = plan.add_operator(BindingsOp::Sink);
        plan.extend_with_flows(&[(scan, project), (project, sink)]);

        let mut bindings = MapBindings::default();
        bindings.insert("data", partiql_list![Tuple::from([("lhs", lhs)])].into());

        let result = evaluate(plan, bindings).coerce_to_bag();
        assert!(!&result.is_empty());
        let expected_result = if expected_first_elem != Missing {
            partiql_bag!(Tuple::from([("result", expected_first_elem)]))
        } else {
            // Filter tuples with `MISSING` vals
            partiql_bag!(Tuple::new())
        };
        assert_eq!(expected_result, result);
    }

    #[test]
    fn test_null_if_op() {
        eval_null_if_op(Value::from(1), Value::from(1), Value::Null);
        eval_null_if_op(Value::from(1), Value::from("foo"), Value::from(1));
        eval_null_if_op(Value::from("foo"), Value::from(1), Value::from("foo"));
        eval_null_if_op(Null, Null, Value::Null);
        eval_null_if_op(Missing, Null, Value::Missing);
        eval_null_if_op(Null, Missing, Value::Null);
    }

    // Creates the plan: `SELECT COALESCE(data.arg1, data.arg2, ..., argN) AS result FROM data` where arg1...argN comes from data
    // Evaluates the plan and asserts the result is a bag of the tuple mapping to `expected_first_elem`
    // (i.e. <<{'result': <expected_first_elem>}>>)
    fn eval_coalesce_op(elements: Vec<Value>, expected_first_elem: Value) {
        let mut plan = LogicalPlan::new();
        let scan = plan.add_operator(BindingsOp::Scan(logical::Scan {
            expr: ValueExpr::VarRef(BindingsName::CaseInsensitive("data".into())),
            as_key: "data".to_string(),
            at_key: None,
        }));

        fn index_to_valueexpr(i: usize) -> ValueExpr {
            ValueExpr::Path(
                Box::new(ValueExpr::VarRef(BindingsName::CaseInsensitive(
                    "data".into(),
                ))),
                vec![PathComponent::Key(BindingsName::CaseInsensitive(format!(
                    "arg{}",
                    i
                )))],
            )
        }

        let project = plan.add_operator(Project(logical::Project {
            exprs: HashMap::from([(
                "result".to_string(),
                ValueExpr::CoalesceExpr(CoalesceExpr {
                    elements: (0..elements.len()).map(index_to_valueexpr).collect(),
                }),
            )]),
        }));

        let sink = plan.add_operator(BindingsOp::Sink);
        plan.extend_with_flows(&[(scan, project), (project, sink)]);

        let mut bindings = MapBindings::default();
        let mut data = Tuple::new();
        // Bindings created from `elements`. For each `e` in elements with index `i`, adds tuple pair ('argi', e)
        elements
            .into_iter()
            .enumerate()
            .for_each(|(i, e)| data.insert(&format!("arg{}", i), e));
        bindings.insert("data", partiql_list![data].into());

        let result = evaluate(plan, bindings).coerce_to_bag();
        assert!(!&result.is_empty());
        assert_eq!(
            partiql_bag!(Tuple::from([("result", expected_first_elem)])),
            result
        );
    }

    #[test]
    fn test_coalesce_op() {
        // 1 elem
        eval_coalesce_op(vec![Value::from(1)], Value::from(1));
        eval_coalesce_op(vec![Null], Null);
        eval_coalesce_op(vec![Missing], Null);

        // Multiple elems
        eval_coalesce_op(vec![Missing, Null, Value::from(1)], Value::from(1));
        eval_coalesce_op(vec![Missing, Null, Value::from(1)], Value::from(1));
        eval_coalesce_op(vec![Missing, Null, Null], Null);
        eval_coalesce_op(
            vec![
                Missing,
                Null,
                Missing,
                Null,
                Missing,
                Null,
                Value::from(1),
                Value::from(2),
                Missing,
                Null,
            ],
            Value::from(1),
        );
        eval_coalesce_op(
            vec![
                Missing, Null, Missing, Null, Missing, Null, Missing, Null, Missing, Null,
            ],
            Null,
        );
    }

    #[test]
    fn expr_query() {
        let mut lg = LogicalPlan::new();
        let expq = lg.add_operator(BindingsOp::ExprQuery(ExprQuery {
            expr: ValueExpr::BinaryExpr(
                BinaryOp::Add,
                Box::new(ValueExpr::Lit(Box::new(40.into()))),
                Box::new(ValueExpr::Lit(Box::new(2.into()))),
            ),
        }));

        let sink = lg.add_operator(BindingsOp::Sink);

        lg.add_flow(expq, sink);

        let out = evaluate(lg, MapBindings::default());
        println!("{:?}", &out);
        assert_matches!(out, Value::Integer(42));
    }

    #[test]
    fn paths() {
        fn test(expr: ValueExpr, expected: Value) {
            let mut lg = LogicalPlan::new();
            let expq = lg.add_operator(BindingsOp::ExprQuery(ExprQuery { expr }));

            let sink = lg.add_operator(BindingsOp::Sink);
            lg.add_flow(expq, sink);

            let out = evaluate(lg, MapBindings::default());
            println!("{:?}", &out);
            assert_eq!(out, expected);
        }
        let list = ValueExpr::Lit(Box::new(Value::List(Box::new(partiql_list![1, 2, 3]))));

        // `[1,2,3][0]` -> `1`
        let index = ValueExpr::Path(Box::new(list.clone()), vec![PathComponent::Index(0)]);
        test(index, Value::Integer(1));

        // `[1,2,3][1+1]` -> `3`
        let index_expr = ValueExpr::BinaryExpr(
            BinaryOp::Add,
            Box::new(ValueExpr::Lit(Box::new(1.into()))),
            Box::new(ValueExpr::Lit(Box::new(1.into()))),
        );
        let index = ValueExpr::Path(
            Box::new(list),
            vec![PathComponent::IndexExpr(Box::new(index_expr))],
        );
        test(index, Value::Integer(3));

        // `{'a':10}[''||'a']` -> `10`
        let tuple = ValueExpr::Lit(Box::new(Value::Tuple(Box::new(partiql_tuple![("a", 10)]))));
        let index_expr = ValueExpr::BinaryExpr(
            BinaryOp::Concat,
            Box::new(ValueExpr::Lit(Box::new("".into()))),
            Box::new(ValueExpr::Lit(Box::new("a".into()))),
        );
        let index = ValueExpr::Path(
            Box::new(tuple),
            vec![PathComponent::KeyExpr(Box::new(index_expr))],
        );
        test(index, Value::Integer(10));
    }

    #[test]
    fn select() {
        let mut lg = LogicalPlan::new();

        let from = lg.add_operator(scan("data", "data"));

        let project = lg.add_operator(Project(logical::Project {
            exprs: HashMap::from([(
                "b".to_string(),
                ValueExpr::Path(
                    Box::new(ValueExpr::VarRef(BindingsName::CaseInsensitive(
                        "data".into(),
                    ))),
                    vec![PathComponent::Key(BindingsName::CaseInsensitive(
                        "a".to_string(),
                    ))],
                ),
            )]),
        }));

        let sink = lg.add_operator(BindingsOp::Sink);

        lg.add_flow(from, project);
        lg.add_flow(project, sink);

        let out = evaluate(lg, data_3_tuple());
        println!("{:?}", &out);
        assert_matches!(out, Value::Bag(bag) => {
            let expected = partiql_bag![
                partiql_tuple![("b", 1)],
                partiql_tuple![("b", 2)],
                partiql_tuple![("b", 3)],
            ];
            assert_eq!(*bag, expected);
        });
    }

    #[test]
    fn select_star() {
        // TODO `SELECT *` is underspecified w.r.t. nested data
        let mut lg = LogicalPlan::new();

        let from = lg.add_operator(scan("data", "data"));

        let project = lg.add_operator(ProjectAll);

        let sink = lg.add_operator(BindingsOp::Sink);

        lg.add_flow(from, project);
        lg.add_flow(project, sink);

        let out = evaluate(lg, data_3_tuple());
        println!("{:?}", &out);
        assert_matches!(out, Value::Bag(bag) => {
            let expected = partiql_bag![
                partiql_tuple![("a", 1)],
                partiql_tuple![("a", 2)],
                partiql_tuple![("a", 3)],
            ];
            assert_eq!(*bag, expected);
        });
    }

    // Spec 6.1:
    //  SELECT VALUE 2*v.a
    //  FROM [{'a': 1}, {'a': 2}, {'a': 3}] AS v;
    //  Expected: <<2, 4, 6>>
    #[test]
    fn select_value() {
        // Plan for `select value a as b from data`
        let mut lg = LogicalPlan::new();

        let from = lg.add_operator(scan("data", "v"));

        let va = path_var("v", "a");
        let select_value = lg.add_operator(ProjectValue(logical::ProjectValue {
            expr: ValueExpr::BinaryExpr(
                BinaryOp::Mul,
                Box::new(va),
                Box::new(ValueExpr::Lit(Box::new(Value::Integer(2)))),
            ),
        }));

        let sink = lg.add_operator(BindingsOp::Sink);

        lg.add_flow(from, select_value);
        lg.add_flow(select_value, sink);

        let out = evaluate(lg, data_3_tuple());
        println!("{:?}", &out);
        assert_matches!(out, Value::Bag(bag) => {
            let expected = partiql_bag![2, 4, 6];
            assert_eq!(*bag, expected);
        });
    }

    // Spec 6.1.1 — Tuple constructors
    //  SELECT VALUE {'a': v.a, 'b': v.b}
    //  FROM [{'a': 1, 'b': 1}, {'a': 2, 'b': 2}] AS v;
    //  Expected: <<{'a': 1, 'b': 1}, {'a': 2, 'b': 2}>>
    #[test]
    fn select_value_tuple_constructor_1() {
        let mut lg = LogicalPlan::new();

        let from = lg.add_operator(scan("data", "v"));

        let va = path_var("v", "a");
        let vb = path_var("v", "b");

        let mut tuple_expr = TupleExpr::new();
        tuple_expr.attrs.push(ValueExpr::Lit(Box::new("a".into())));
        tuple_expr.attrs.push(ValueExpr::Lit(Box::new("b".into())));
        tuple_expr.values.push(va);
        tuple_expr.values.push(vb);

        let select_value = lg.add_operator(ProjectValue(logical::ProjectValue {
            expr: ValueExpr::TupleExpr(tuple_expr),
        }));

        let sink = lg.add_operator(BindingsOp::Sink);

        lg.add_flow(from, select_value);
        lg.add_flow(select_value, sink);

        let data = partiql_bag![
            partiql_tuple![("a", 1), ("b", 1)],
            partiql_tuple![("a", 2), ("b", 2)],
        ];

        let mut bindings: MapBindings<Value> = MapBindings::default();
        bindings.insert("data", data.into());

        let out = evaluate(lg, bindings);
        println!("{:?}", &out);
        assert_matches!(out, Value::Bag(bag) => {
            let expected = partiql_bag![
                partiql_tuple![("a", 1), ("b", 1)],
                partiql_tuple![("a", 2), ("b", 2)],
            ];
            assert_eq!(*bag, expected);
        });
    }

    // Spec 6.1.1 — Tuple constructors
    //  SELECT VALUE {'test': 2*v.a }
    //  FROM [{'a': 1}, {'a': 2}, {'a': 3}] AS v;
    //  Expected: <<{'test': 2}, {'test': 4}, {'test': 6}>>
    #[test]
    fn select_value_tuple_constructor_2() {
        let mut lg = LogicalPlan::new();

        let from = lg.add_operator(scan("data", "v"));

        let va = path_var("v", "a");
        let mut tuple_expr = TupleExpr::new();
        tuple_expr
            .attrs
            .push(ValueExpr::Lit(Box::new("test".into())));
        tuple_expr.values.push(ValueExpr::BinaryExpr(
            BinaryOp::Mul,
            Box::new(va),
            Box::new(ValueExpr::Lit(Box::new(Value::Integer(2)))),
        ));

        let project = lg.add_operator(ProjectValue(logical::ProjectValue {
            expr: ValueExpr::TupleExpr(tuple_expr),
        }));

        let sink = lg.add_operator(BindingsOp::Sink);

        lg.add_flow(from, project);
        lg.add_flow(project, sink);

        let out = evaluate(lg, data_3_tuple());
        println!("{:?}", &out);
        assert_matches!(out, Value::Bag(bag) => {
            let expected = partiql_bag![
                partiql_tuple![("test", 2)],
                partiql_tuple![("test", 4)],
                partiql_tuple![("test", 6)],
            ];
            assert_eq!(*bag, expected);
        });
    }

    // Spec 6.1.1 — Treatment of mistyped attribute names in permissive mode
    //  SELECT VALUE {v.a: v.b}
    //  FROM [{'a': 'legit', 'b': 1}, {'a': 400, 'b': 2}] AS v;
    //  Expected: <<{'legit': 1}, {}>>
    #[test]
    fn select_value_with_tuple_mistype_attr() {
        let mut lg = LogicalPlan::new();

        let from = lg.add_operator(scan("data", "v"));

        let va = path_var("v", "a");
        let vb = path_var("v", "b");

        let mut tuple_expr = TupleExpr::new();
        tuple_expr.attrs.push(va);
        tuple_expr.values.push(vb);

        let select_value = lg.add_operator(ProjectValue(logical::ProjectValue {
            expr: ValueExpr::TupleExpr(tuple_expr),
        }));

        let sink = lg.add_operator(BindingsOp::Sink);

        lg.add_flow(from, select_value);
        lg.add_flow(select_value, sink);

        let data = partiql_list![
            partiql_tuple![("a", "legit"), ("b", 1)],
            partiql_tuple![("a", 400), ("b", 2)],
        ];

        let mut bindings: MapBindings<Value> = MapBindings::default();
        bindings.insert("data", data.into());

        let out = evaluate(lg, bindings);
        println!("{:?}", &out);
        assert_matches!(out, Value::Bag(bag) => {
            let expected = partiql_bag![partiql_tuple![("legit", 1)], partiql_tuple![]];
            assert_eq!(*bag, expected);
        });
    }

    // Spec 6.1.1 — Treatment of duplicate attribute names
    //  SELECT VALUE {v.a: v.b, v.c: v.d}
    //  FROM [{'a': 'same', 'b': 1, 'c': 'same', 'd': 2}] AS v;
    //  Expected: <<{'same': 1, 'same': 2}>>
    #[test]
    fn select_value_with_duplicate_attrs() {
        let mut lg = LogicalPlan::new();
        let from = lg.add_operator(scan("data", "v"));

        let va = path_var("v", "a");
        let vb = path_var("v", "b");
        let vc = path_var("v", "c");
        let vd = path_var("v", "d");

        let mut tuple_expr = TupleExpr::new();
        tuple_expr.attrs.push(va);
        tuple_expr.values.push(vb);
        tuple_expr.attrs.push(vc);
        tuple_expr.values.push(vd);

        let select_value = lg.add_operator(ProjectValue(logical::ProjectValue {
            expr: ValueExpr::TupleExpr(tuple_expr),
        }));

        let sink = lg.add_operator(BindingsOp::Sink);

        lg.add_flow(from, select_value);
        lg.add_flow(select_value, sink);

        let data = partiql_list![partiql_tuple![
            ("a", "same"),
            ("b", 1),
            ("c", "same"),
            ("d", 2)
        ]];

        let mut bindings: MapBindings<Value> = MapBindings::default();
        bindings.insert("data", data.into());

        let out = evaluate(lg, bindings);
        println!("{:?}", &out);
        assert_matches!(out, Value::Bag(bag) => {
            let expected = partiql_bag![partiql_tuple![("same", 1), ("same", 2)]];
            assert_eq!(*bag, expected);
        });
    }

    // Spec 6.1.2 — Array Constructors
    //  SELECT VALUE [v.a, v.b]
    //  FROM [{'a': 1, 'b': 1}, {'a': 2, 'b': 2}] AS v;
    //  Expected: <<[1, 1], [2, 2]>>
    #[test]
    fn select_value_array_constructor_1() {
        let mut lg = LogicalPlan::new();

        let from = lg.add_operator(scan("data", "v"));

        let va = path_var("v", "a");
        let vb = path_var("v", "b");

        let mut list_expr = ListExpr::new();
        list_expr.elements.push(va);
        list_expr.elements.push(vb);

        let select_value = lg.add_operator(ProjectValue(logical::ProjectValue {
            expr: ValueExpr::ListExpr(list_expr),
        }));

        let sink = lg.add_operator(BindingsOp::Sink);

        lg.add_flow(from, select_value);
        lg.add_flow(select_value, sink);

        let data = partiql_list![
            partiql_tuple![("a", 1), ("b", 1)],
            partiql_tuple![("a", 2), ("b", 2)],
        ];

        let mut bindings: MapBindings<Value> = MapBindings::default();
        bindings.insert("data", data.into());

        let out = evaluate(lg, bindings);
        println!("{:?}", &out);
        assert_matches!(out, Value::Bag(bag) => {
            let expected = partiql_bag![partiql_list![1, 1], partiql_list![2, 2]];
            assert_eq!(*bag, expected);
        });
    }

    // Spec 6.1.2 — Array Constructors
    //  SELECT VALUE [2*v.a]
    //  FROM [{'a': 1, 'b': 1}, {'a': 2, 'b': 2}] AS v;
    //  Expected: <<[2], [4], [6]>>
    #[test]
    fn select_value_with_array_constructor_2() {
        let mut lg = LogicalPlan::new();

        let from = lg.add_operator(scan("data", "v"));

        let va = path_var("v", "a");
        let mut list_expr = ListExpr::new();
        list_expr.elements.push(ValueExpr::BinaryExpr(
            BinaryOp::Mul,
            Box::new(va),
            Box::new(ValueExpr::Lit(Box::new(Value::Integer(2)))),
        ));

        let select_value = lg.add_operator(ProjectValue(logical::ProjectValue {
            expr: ValueExpr::ListExpr(list_expr),
        }));

        let sink = lg.add_operator(BindingsOp::Sink);

        lg.add_flow(from, select_value);
        lg.add_flow(select_value, sink);

        let out = evaluate(lg, data_3_tuple());
        println!("{:?}", &out);
        assert_matches!(out, Value::Bag(bag) => {
            let expected = partiql_bag![partiql_list![2], partiql_list![4], partiql_list![6]];
            assert_eq!(*bag, expected);
        });
    }

    // Spec 6.1.3 — Bag Constructors
    //  SELECT VALUE <<v.a, v.b>>
    //  FROM [{'a': 1, 'b': 1}, {'a': 2, 'b': 2}] AS v;
    //  Expected: << <<1, 1>>, <<2, 2>> >>
    #[test]
    fn select_value_bag_constructor() {
        let mut lg = LogicalPlan::new();

        let from = lg.add_operator(scan("data", "v"));

        let va = path_var("v", "a");
        let vb = path_var("v", "b");

        let mut bag_expr = BagExpr::new();
        bag_expr.elements.push(va);
        bag_expr.elements.push(vb);

        let select_value = lg.add_operator(ProjectValue(logical::ProjectValue {
            expr: ValueExpr::BagExpr(bag_expr),
        }));

        let sink = lg.add_operator(BindingsOp::Sink);

        lg.add_flow(from, select_value);
        lg.add_flow(select_value, sink);

        let data = partiql_list![
            partiql_tuple![("a", 1), ("b", 1)],
            partiql_tuple![("a", 2), ("b", 2)],
        ];

        let mut bindings: MapBindings<Value> = MapBindings::default();
        bindings.insert("data", data.into());

        let out = evaluate(lg, bindings);
        println!("{:?}", &out);
        assert_matches!(out, Value::Bag(bag) => {
            let expected = partiql_bag![partiql_bag![1, 1], partiql_bag![2, 2]];
            assert_eq!(*bag, expected);
        });
    }

    // Spec 6.1.4 — Treatment of MISSING in SELECT VALUE
    //  SELECT VALUE {'a': v.a, 'b': v.b}
    //  FROM [{'a': 1, 'b': 1}, {'a': 2}] AS v;
    //  Expected: <<{'a':1, 'b':1}, {'a':2}>>
    #[test]
    fn missing_in_select_value_for_tuple() {
        let mut lg = LogicalPlan::new();
        let from = lg.add_operator(scan("data", "v"));

        let va = path_var("v", "a");
        let vb = path_var("v", "b");

        let mut tuple_expr = TupleExpr::new();
        tuple_expr.attrs.push(ValueExpr::Lit(Box::new("a".into())));
        tuple_expr.values.push(va);
        tuple_expr.attrs.push(ValueExpr::Lit(Box::new("b".into())));
        tuple_expr.values.push(vb);

        let select_value = lg.add_operator(ProjectValue(logical::ProjectValue {
            expr: ValueExpr::TupleExpr(tuple_expr),
        }));

        let sink = lg.add_operator(BindingsOp::Sink);

        lg.add_flow(from, select_value);
        lg.add_flow(select_value, sink);

        let data = partiql_list![partiql_tuple![("a", 1), ("b", 1)], partiql_tuple![("a", 2)]];

        let mut bindings: MapBindings<Value> = MapBindings::default();
        bindings.insert("data", data.into());

        let out = evaluate(lg, bindings);
        println!("{:?}", &out);
        assert_matches!(out, Value::Bag(bag) => {
            let expected =
                partiql_bag![partiql_tuple![("a", 1), ("b", 1)], partiql_tuple![("a", 2)],];
            assert_eq!(*bag, expected);
        });
    }

    // Spec 6.1.4 — Treatment of MISSING in SELECT VALUE
    //  SELECT VALUE [v.a, v.b]
    //  FROM [{'a': 1, 'b': 1}, {'a': 2}] AS v;
    // Expected: <<[1, 1], [2, MISSING]>>
    #[test]
    fn missing_in_select_value_for_list() {
        let mut lg = LogicalPlan::new();
        let from = lg.add_operator(scan("data", "v"));

        let va = path_var("v", "a");
        let vb = path_var("v", "b");

        let mut list_expr = ListExpr::new();
        list_expr.elements.push(va);
        list_expr.elements.push(vb);

        let select_value = lg.add_operator(ProjectValue(logical::ProjectValue {
            expr: ValueExpr::ListExpr(list_expr),
        }));

        let sink = lg.add_operator(BindingsOp::Sink);

        lg.add_flow(from, select_value);
        lg.add_flow(select_value, sink);

        let data = partiql_list![partiql_tuple![("a", 1), ("b", 1)], partiql_tuple![("a", 2)]];

        let mut bindings: MapBindings<Value> = MapBindings::default();
        bindings.insert("data", data.into());

        let out = evaluate(lg, bindings);
        println!("{:?}", &out);
        assert_matches!(out, Value::Bag(bag) => {
            let expected = partiql_bag![partiql_list![1, 1], partiql_list![2, Value::Missing]];
            assert_eq!(*bag, expected);
        });
    }

    // Spec 6.1.4 — Treatment of MISSING in SELECT VALUE
    //  SELECT VALUE v.b
    //  FROM [{'a':1, 'b':1}, {'a':2}] AS v;
    //  Expected: <<1, MISSING>>
    #[test]
    fn missing_in_select_value_for_bag_1() {
        let mut lg = LogicalPlan::new();
        let from = lg.add_operator(scan("data", "v"));

        let vb = path_var("v", "b");

        let select_value = lg.add_operator(ProjectValue(logical::ProjectValue { expr: vb }));

        let sink = lg.add_operator(BindingsOp::Sink);

        lg.add_flow(from, select_value);
        lg.add_flow(select_value, sink);

        let data = partiql_list![partiql_tuple![("a", 1), ("b", 1)], partiql_tuple![("a", 2)]];

        let mut bindings: MapBindings<Value> = MapBindings::default();
        bindings.insert("data", data.into());

        let out = evaluate(lg, bindings);
        println!("{:?}", &out);
        assert_matches!(out, Value::Bag(bag) => {
            let expected = partiql_bag![1, Value::Missing];
            assert_eq!(*bag, expected);
        });
    }

    // Spec 6.1.4 — Treatment of MISSING in SELECT VALUE
    //  SELECT VALUE <<v.a, v.b>>
    //  FROM [{'a': 1, 'b': 1}, {'a': 2}] AS v;
    // Expected: << <<1, 1>>, <<2, MISSING>> >>
    #[test]
    fn missing_in_select_value_for_bag_2() {
        let mut lg = LogicalPlan::new();
        let from = lg.add_operator(scan("data", "v"));

        let va = path_var("v", "a");
        let vb = path_var("v", "b");

        let mut bag_expr = BagExpr::new();
        bag_expr.elements.push(va);
        bag_expr.elements.push(vb);

        let select_value = lg.add_operator(ProjectValue(logical::ProjectValue {
            expr: ValueExpr::BagExpr(bag_expr),
        }));

        let sink = lg.add_operator(BindingsOp::Sink);

        lg.add_flow(from, select_value);
        lg.add_flow(select_value, sink);

        let data = partiql_list![partiql_tuple![("a", 1), ("b", 1)], partiql_tuple![("a", 2)]];

        let mut bindings: MapBindings<Value> = MapBindings::default();
        bindings.insert("data", data.into());

        let out = evaluate(lg, bindings);
        println!("{:?}", &out);
        assert_matches!(out, Value::Bag(bag) => {
            let expected = partiql_bag![partiql_bag![1, 1], partiql_bag![2, Value::Missing]];
            assert_eq!(*bag, expected);
        });
    }

    #[test]
    fn select_distinct() {
        // Plan for `SELECT DISTINCT firstName, (firstName || firstName) AS doubleName FROM customer WHERE balance > 0`
        let mut logical = LogicalPlan::new();

        let scan = logical.add_operator(scan("customer", "customer"));

        let filter = logical.add_operator(BindingsOp::Filter(logical::Filter {
            expr: ValueExpr::BinaryExpr(
                BinaryOp::Gt,
                Box::new(ValueExpr::Path(
                    Box::new(ValueExpr::VarRef(BindingsName::CaseInsensitive(
                        "customer".into(),
                    ))),
                    vec![PathComponent::Key(BindingsName::CaseInsensitive(
                        "balance".to_string(),
                    ))],
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
                        vec![PathComponent::Key(BindingsName::CaseInsensitive(
                            "firstName".to_string(),
                        ))],
                    ),
                ),
                (
                    "doubleName".to_string(),
                    ValueExpr::BinaryExpr(
                        BinaryOp::Concat,
                        Box::new(ValueExpr::Path(
                            Box::new(ValueExpr::VarRef(BindingsName::CaseInsensitive(
                                "customer".into(),
                            ))),
                            vec![PathComponent::Key(BindingsName::CaseInsensitive(
                                "firstName".to_string(),
                            ))],
                        )),
                        Box::new(ValueExpr::Path(
                            Box::new(ValueExpr::VarRef(BindingsName::CaseInsensitive(
                                "customer".into(),
                            ))),
                            vec![PathComponent::Key(BindingsName::CaseInsensitive(
                                "firstName".to_string(),
                            ))],
                        )),
                    ),
                ),
            ]),
        }));

        let distinct = logical.add_operator(Distinct);
        let sink = logical.add_operator(BindingsOp::Sink);

        logical.extend_with_flows(&[
            (scan, filter),
            (filter, project),
            (project, distinct),
            (distinct, sink),
        ]);

        let out = evaluate(logical, data_customer());
        println!("{:?}", &out);
        assert_matches!(out, Value::Bag(bag) => {
            let expected = partiql_bag![
                partiql_tuple![("firstName", "jason"), ("doubleName", "jasonjason")],
                partiql_tuple![("firstName", "miriam"), ("doubleName", "miriammiriam")],
            ];
            assert_eq!(*bag, expected);
        });
    }

    #[test]
    fn select_with_in_as_predicate() {
        // Plan for `SELECT a AS b FROM data WHERE a IN [1]`
        let mut logical = LogicalPlan::new();

        let scan = logical.add_operator(scan("data", "data"));

        let filter = logical.add_operator(BindingsOp::Filter(logical::Filter {
            expr: ValueExpr::BinaryExpr(
                BinaryOp::In,
                Box::new(ValueExpr::Path(
                    Box::new(ValueExpr::VarRef(BindingsName::CaseInsensitive(
                        "data".into(),
                    ))),
                    vec![PathComponent::Key(BindingsName::CaseInsensitive(
                        "a".to_string(),
                    ))],
                )),
                Box::new(ValueExpr::Lit(Box::new(partiql_list![1].into()))),
            ),
        }));

        let project = logical.add_operator(Project(logical::Project {
            exprs: HashMap::from([(
                "b".to_string(),
                ValueExpr::Path(
                    Box::new(ValueExpr::VarRef(BindingsName::CaseInsensitive(
                        "data".into(),
                    ))),
                    vec![PathComponent::Key(BindingsName::CaseInsensitive(
                        "a".to_string(),
                    ))],
                ),
            )]),
        }));

        let sink = logical.add_operator(BindingsOp::Sink);

        logical.extend_with_flows(&[(scan, filter), (filter, project), (project, sink)]);

        let out = evaluate(logical, data_3_tuple());
        println!("{:?}", &out);
        assert_matches!(out, Value::Bag(bag) => {
            let expected = partiql_bag![
                partiql_tuple![("b", 1)],
            ];
            assert_eq!(*bag, expected);
        });
    }

    #[test]
    fn subquery_in_from() {
        // SELECT t.a, s FROM data AS t, (SELECT v.a*2 AS u FROM t AS v) AS s;
        let mut subq_plan = LogicalPlan::new();
        let subq_scan = subq_plan.add_operator(scan("t", "v"));
        let va = path_var("v", "a");
        let subq_project = subq_plan.add_operator(Project(logical::Project {
            exprs: HashMap::from([(
                "u".to_string(),
                ValueExpr::BinaryExpr(
                    BinaryOp::Mul,
                    Box::new(va),
                    Box::new(ValueExpr::Lit(Box::new(Value::Integer(2)))),
                ),
            )]),
        }));
        let subq_sink = subq_plan.add_operator(BindingsOp::Sink);

        subq_plan.add_flow(subq_scan, subq_project);
        subq_plan.add_flow(subq_project, subq_sink);

        let mut lg = LogicalPlan::new();

        let from_lhs = scan("data", "t");
        let from_rhs = BindingsOp::Scan(logical::Scan {
            expr: ValueExpr::SubQueryExpr(logical::SubQueryExpr { plan: subq_plan }),
            as_key: "s".to_string(),
            at_key: None,
        });

        let join = lg.add_operator(BindingsOp::Join(logical::Join {
            kind: JoinKind::Cross,
            left: Box::new(from_lhs),
            right: Box::new(from_rhs),
            on: None,
        }));

        let ta = path_var("t", "a");
        let su = path_var("s", "u");
        let project = lg.add_operator(Project(logical::Project {
            exprs: HashMap::from([("ta".to_string(), ta), ("su".to_string(), su)]),
        }));

        let sink = lg.add_operator(BindingsOp::Sink);

        lg.add_flow_with_branch_num(join, project, 0);
        lg.add_flow_with_branch_num(project, sink, 0);

        let data = partiql_list![
            partiql_tuple![("a", 1)],
            partiql_tuple![("a", 2)],
            partiql_tuple![("a", 3)],
        ];

        let mut bindings: MapBindings<Value> = MapBindings::default();
        bindings.insert("data", data.into());

        let out = evaluate(lg, bindings);
        println!("{:?}", &out);
        assert_matches!(out, Value::Bag(bag) => {
            let expected = partiql_bag![
                partiql_tuple![
                    ("ta", 1),
                    ("su", 2),
                ],
                partiql_tuple![
                    ("ta", 2),
                    ("su", 4),
                ],
                partiql_tuple![
                    ("ta", 3),
                    ("su", 6),
                ],
            ];
            assert_eq!(*bag, expected);
        });
    }

    #[test]
    fn subquery_in_project() {
        // SELECT t.a, (SELECT v.a*2 AS u FROM t AS v) AS s FROM data AS t;
        let mut subq_plan = LogicalPlan::new();
        let subq_scan = subq_plan.add_operator(scan("t", "v"));
        let va = path_var("v", "a");
        let subq_project = subq_plan.add_operator(Project(logical::Project {
            exprs: HashMap::from([(
                "u".to_string(),
                ValueExpr::BinaryExpr(
                    BinaryOp::Mul,
                    Box::new(va),
                    Box::new(ValueExpr::Lit(Box::new(Value::Integer(2)))),
                ),
            )]),
        }));
        let subq_sink = subq_plan.add_operator(BindingsOp::Sink);

        subq_plan.add_flow(subq_scan, subq_project);
        subq_plan.add_flow(subq_project, subq_sink);

        let mut lg = LogicalPlan::new();
        let from = lg.add_operator(scan("data", "t"));
        let ta = path_var("t", "a");
        let project = lg.add_operator(Project(logical::Project {
            exprs: HashMap::from([
                ("ta".to_string(), ta),
                (
                    "s".to_string(),
                    ValueExpr::SubQueryExpr(logical::SubQueryExpr { plan: subq_plan }),
                ),
            ]),
        }));

        let sink = lg.add_operator(BindingsOp::Sink);

        lg.add_flow(from, project);
        lg.add_flow(project, sink);

        let data = partiql_list![
            partiql_tuple![("a", 1), ("b", 1)],
            partiql_tuple![("a", 2), ("b", 2)]
        ];

        let mut bindings: MapBindings<Value> = MapBindings::default();
        bindings.insert("data", data.into());

        let out = evaluate(lg, bindings);
        println!("{:?}", &out);
        assert_matches!(out, Value::Bag(bag) => {
            let expected = partiql_bag![
                partiql_tuple![
                    ("ta", 1),
                    ("s", partiql_bag![partiql_tuple![("u", 2)]]),
                ],
                partiql_tuple![
                    ("ta", 2),
                    ("s", partiql_bag![partiql_tuple![("u", 4)]]),
                ],
            ];
            assert_eq!(*bag, expected);
        });
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
                    EvalPathComponent::Key(BindingsName::CaseInsensitive("a".into())),
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
                    EvalPathComponent::Key(BindingsName::CaseInsensitive("c".into())),
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
