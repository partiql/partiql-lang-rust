use std::collections::HashMap;
use std::time::Duration;

use criterion::{black_box, criterion_group, criterion_main, Criterion};

use partiql_eval::env::basic::MapBindings;
use partiql_eval::eval::{
    BasicContext, EvalPath, EvalPathComponent, EvalPlan, EvalScan, EvalVarRef, Evaluable, Evaluator,
};
use partiql_eval::plan;
use partiql_logical as logical;
use partiql_logical::BindingsExpr::Project;
use partiql_logical::{BinaryOp, BindingsExpr, JoinKind, LogicalPlan, PathComponent, ValueExpr};
use partiql_value::{
    partiql_bag, partiql_list, partiql_tuple, Bag, BindingsName, List, Tuple, Value,
};

fn data() -> MapBindings<Value> {
    let hr = partiql_tuple![(
        "employeesNestScalars",
        partiql_bag![
            partiql_tuple![
                ("id", 3),
                ("name", "Bob Smith"),
                ("title", Value::Null),
                (
                    "projects",
                    partiql_list![
                        "AWS Redshift Spectrum querying",
                        "AWS Redshift security",
                        "AWS Aurora security",
                    ]
                ),
            ],
            partiql_tuple![
                ("id", 4),
                ("name", "Susan Smith"),
                ("title", "Dev Mgr"),
                ("projects", partiql_list![]),
            ],
            partiql_tuple![
                ("id", 6),
                ("name", "Jane Smith"),
                ("title", "Software Eng 2"),
                ("projects", partiql_list!["AWS Redshift security"]),
            ],
        ]
    )];

    let mut p0: MapBindings<Value> = MapBindings::default();
    p0.insert("hr", hr.into());
    p0
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

fn logical_plan() -> LogicalPlan<BindingsExpr> {
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

    lg
}

fn eval_plan(logical: &LogicalPlan<BindingsExpr>) -> EvalPlan {
    let planner = plan::EvaluatorPlanner;

    planner.compile(logical)
}

fn evaluate(plan: EvalPlan, bindings: MapBindings<Value>) -> Value {
    let mut evaluator = Evaluator::new(bindings);

    if let Ok(out) = evaluator.execute(plan) {
        out.result
    } else {
        Value::Missing
    }
}

fn eval_bench(c: &mut Criterion) {
    let join_data = join_data();
    let logical_plan = logical_plan();

    c.bench_function("join", |b| {
        b.iter(|| {
            let eval_plan = eval_plan(black_box(&logical_plan));
            let bindings = join_data.clone();
            evaluate(black_box(eval_plan), bindings)
        })
    });

    fn eval(eval: bool) {
        // eval plan for SELECT * FROM hr.employeesNestScalars
        let mut from = EvalScan::new(
            Box::new(EvalPath {
                expr: Box::new(EvalVarRef {
                    name: BindingsName::CaseInsensitive("hr".to_string()),
                }),
                components: vec![EvalPathComponent::Key("employeesNestScalars".to_string())],
            }),
            "x",
        );

        let ctx = BasicContext::new(data());
        if eval {
            from.evaluate(&ctx);
        }
    }

    let _dummy = "dummy";
    c.bench_function("simple", |b| b.iter(|| eval(black_box(true))));
    c.bench_function("simple-no", |b| b.iter(|| eval(black_box(false))));
    c.bench_function("numbers", |b| {
        b.iter(|| {
            black_box(Value::Integer(0));
            black_box(Value::Integer(7));
            black_box(Value::Integer(29));
            black_box(Value::Integer(119));
            black_box(Value::Integer(1209));
            black_box(Value::Integer(12209));
            black_box(Value::Integer(122039));
            black_box(Value::Integer(1220339));
            black_box(Value::Integer(12203392));
            black_box(Value::Integer(122033942));
        })
    });
}

criterion_group! {
    name = eval;
    config = Criterion::default().measurement_time(Duration::new(5, 0));
    targets = eval_bench
}

criterion_main!(eval);
