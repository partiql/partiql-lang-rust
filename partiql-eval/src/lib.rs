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

    use crate::eval::Evaluator;

    use partiql_logical as logical;
    use partiql_logical::BindingsExpr::{Distinct, Project, ProjectValue};
    use partiql_logical::{
        BagExpr, BetweenExpr, BinaryOp, BindingsExpr, IsTypeExpr, JoinKind, ListExpr, LogicalPlan,
        PathComponent, TupleExpr, Type, ValueExpr,
    };
    use partiql_value as value;
    use partiql_value::Value::{Missing, Null};
    use partiql_value::{
        partiql_bag, partiql_list, partiql_tuple, Bag, BindingsName, List, Tuple, Value,
    };

    fn evaluate(logical: LogicalPlan<BindingsExpr>, bindings: MapBindings<Value>) -> Value {
        let planner = plan::EvaluatorPlanner;

        let plan = planner.compile(&logical);
        let mut evaluator = Evaluator::new(bindings);

        if let Ok(out) = evaluator.execute(plan) {
            out.result
        } else {
            Value::Missing
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

    fn scan(name: &str, as_key: &str) -> BindingsExpr {
        BindingsExpr::Scan(logical::Scan {
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
            vec![PathComponent::Key(component.to_string())],
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
        let scan = plan.add_operator(BindingsExpr::Scan(logical::Scan {
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
                        vec![PathComponent::Key("lhs".to_string())],
                    )),
                    Box::new(ValueExpr::Lit(Box::new(rhs))),
                ),
            )]),
        }));

        let sink = plan.add_operator(BindingsExpr::Sink);
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
        eval_bin_op(
            BinaryOp::Lt,
            Value::Missing,
            Value::from(2.),
            Value::Missing,
        );
        eval_bin_op(BinaryOp::Lt, Value::Null, Value::from(2.), Value::Null);
        eval_bin_op(
            BinaryOp::Lt,
            Value::from(1),
            Value::from("foo"),
            Value::Missing,
        );

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
        eval_bin_op(
            BinaryOp::Gt,
            Value::Missing,
            Value::from(2.),
            Value::Missing,
        );
        eval_bin_op(BinaryOp::Gt, Value::Null, Value::from(2.), Value::Null);
        eval_bin_op(
            BinaryOp::Gt,
            Value::from(1),
            Value::from("foo"),
            Value::Missing,
        );

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
        eval_bin_op(
            BinaryOp::Lteq,
            Value::Missing,
            Value::from(2.),
            Value::Missing,
        );
        eval_bin_op(BinaryOp::Lt, Value::Null, Value::from(2.), Value::Null);
        eval_bin_op(
            BinaryOp::Lteq,
            Value::from(1),
            Value::from("foo"),
            Value::Missing,
        );

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
        eval_bin_op(
            BinaryOp::Gteq,
            Value::Missing,
            Value::from(2.),
            Value::Missing,
        );
        eval_bin_op(BinaryOp::Gteq, Value::Null, Value::from(2.), Value::Null);
        eval_bin_op(
            BinaryOp::Gteq,
            Value::from(1),
            Value::from("foo"),
            Value::Missing,
        );
    }

    #[test]
    fn between_op() {
        fn eval_between_op(value: Value, from: Value, to: Value, expected_first_elem: Value) {
            let mut plan = LogicalPlan::new();
            let scan = plan.add_operator(BindingsExpr::Scan(logical::Scan {
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
                            vec![PathComponent::Key("value".to_string())],
                        )),
                        from: Box::new(ValueExpr::Lit(Box::new(from))),
                        to: Box::new(ValueExpr::Lit(Box::new(to))),
                    }),
                )]),
            }));

            let sink = plan.add_operator(BindingsExpr::Sink);
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
        eval_between_op(Value::Null, Value::from(1), Value::from(3), Value::Null);
        eval_between_op(Value::from(2), Value::Null, Value::from(3), Value::Null);
        eval_between_op(Value::from(2), Value::from(1), Value::Null, Value::Null);
        eval_between_op(Value::Missing, Value::from(1), Value::from(3), Value::Null);
        eval_between_op(Value::from(2), Value::Missing, Value::from(3), Value::Null);
        eval_between_op(Value::from(2), Value::from(1), Value::Missing, Value::Null);
        // left part of AND evaluates to false
        eval_between_op(
            Value::from(1),
            Value::from(2),
            Value::Null,
            Value::from(false),
        );
        eval_between_op(
            Value::from(1),
            Value::from(2),
            Value::Missing,
            Value::from(false),
        );
    }

    #[test]
    fn select_with_cross_join() {
        let mut lg = LogicalPlan::new();

        // Example 9 from spec with projected columns from different tables demonstrates a cross join:
        // SELECT c.id, c.name, o.custId, o.productId FROM customers AS c, orders AS o
        let from_lhs = lg.add_operator(scan("customers", "c"));
        let from_rhs = lg.add_operator(scan("orders", "o"));

        let project = lg.add_operator(Project(logical::Project {
            exprs: HashMap::from([
                ("id".to_string(), path_var("c", "id")),
                ("name".to_string(), path_var("c", "name")),
                ("custId".to_string(), path_var("o", "custId")),
                ("productId".to_string(), path_var("o", "productId")),
            ]),
        }));

        let join = lg.add_operator(BindingsExpr::Join(logical::Join {
            kind: JoinKind::Cross,
            on: None,
        }));

        let sink = lg.add_operator(BindingsExpr::Sink);
        lg.add_flow_with_branch_num(from_lhs, join, 0);
        lg.add_flow_with_branch_num(from_rhs, join, 1);
        lg.add_flow_with_branch_num(join, project, 0);
        lg.add_flow_with_branch_num(project, sink, 0);

        let out = evaluate(lg, join_data());
        println!("{:?}", &out);

        assert_matches!(out, Value::Bag(bag) => {
            let expected = partiql_bag![
                partiql_tuple![("custId", 7), ("name", "Joe"), ("id", 5), ("productId", 101)],
                partiql_tuple![("custId", 7), ("name", "Joe"), ("id", 5), ("productId", 523)],
                partiql_tuple![("custId", 7), ("name", "Mary"), ("id", 7), ("productId", 101)],
                partiql_tuple![("custId", 7), ("name", "Mary"), ("id", 7), ("productId", 523)],
            ];
            assert_eq!(*bag, expected);
        });
    }

    #[test]
    fn select_with_join_and_on() {
        let mut lg = LogicalPlan::new();

        // Similar to ex 9 from spec with projected columns from different tables with an inner JOIN and ON condition
        // SELECT c.id, c.name, o.custId, o.productId FROM customers AS c, orders AS o ON c.id = o.custId
        let from_lhs = lg.add_operator(scan("customers", "c"));
        let from_rhs = lg.add_operator(scan("orders", "o"));

        let project = lg.add_operator(Project(logical::Project {
            exprs: HashMap::from([
                ("id".to_string(), path_var("c", "id")),
                ("name".to_string(), path_var("c", "name")),
                ("custId".to_string(), path_var("o", "custId")),
                ("productId".to_string(), path_var("o", "productId")),
            ]),
        }));

        let join = lg.add_operator(BindingsExpr::Join(logical::Join {
            kind: JoinKind::Inner,
            on: Some(ValueExpr::BinaryExpr(
                BinaryOp::Eq,
                Box::new(path_var("c", "id")),
                Box::new(path_var("o", "custId")),
            )),
        }));

        let sink = lg.add_operator(BindingsExpr::Sink);
        lg.add_flow_with_branch_num(from_lhs, join, 0);
        lg.add_flow_with_branch_num(from_rhs, join, 1);
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
        let sink = lg.add_operator(BindingsExpr::Sink);
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
        let sink = lg.add_operator(BindingsExpr::Sink);
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
        let sink = lg.add_operator(BindingsExpr::Sink);
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
        let sink = lg.add_operator(BindingsExpr::Sink);
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

    fn eval_is_op(not: bool, expr: Value, is_type: Type, expected_first_elem: Value) {
        let mut plan = LogicalPlan::new();
        let scan = plan.add_operator(BindingsExpr::Scan(logical::Scan {
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
                        vec![PathComponent::Key("expr".to_string())],
                    )),
                    is_type,
                }),
            )]),
        }));

        let sink = plan.add_operator(BindingsExpr::Sink);
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
                    vec![PathComponent::Key("a".to_string())],
                ),
            )]),
        }));

        let sink = lg.add_operator(BindingsExpr::Sink);

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

        let sink = lg.add_operator(BindingsExpr::Sink);

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
        // Plan for `select value {'test': a} from data`
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

        let sink = lg.add_operator(BindingsExpr::Sink);

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

        let sink = lg.add_operator(BindingsExpr::Sink);

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
        // Plan for `select value {'test': a} from data`
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

        let sink = lg.add_operator(BindingsExpr::Sink);

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

        let sink = lg.add_operator(BindingsExpr::Sink);

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

        let sink = lg.add_operator(BindingsExpr::Sink);

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
        // Plan for `select value {'test': a} from data`
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

        let sink = lg.add_operator(BindingsExpr::Sink);

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
        // Plan for `select value {'test': a} from data`
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

        let sink = lg.add_operator(BindingsExpr::Sink);

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

        let sink = lg.add_operator(BindingsExpr::Sink);

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

        let sink = lg.add_operator(BindingsExpr::Sink);

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

        let sink = lg.add_operator(BindingsExpr::Sink);

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

        let sink = lg.add_operator(BindingsExpr::Sink);

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

        let filter = logical.add_operator(BindingsExpr::Filter(logical::Filter {
            expr: ValueExpr::BinaryExpr(
                BinaryOp::Gt,
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
                        BinaryOp::Concat,
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
