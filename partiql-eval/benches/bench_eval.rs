use std::collections::HashMap;
use std::time::Duration;

use criterion::{black_box, criterion_group, criterion_main, Criterion};
use partiql_catalog::PartiqlCatalog;

use partiql_eval::env::basic::MapBindings;
use partiql_eval::eval::EvalPlan;
use partiql_eval::plan;
use partiql_eval::plan::EvaluationMode;
use partiql_logical as logical;
use partiql_logical::BindingsOp::{Project, ProjectAll};
use partiql_logical::{BinaryOp, BindingsOp, JoinKind, LogicalPlan, PathComponent, ValueExpr};
use partiql_value::{bag, list, tuple, BindingsName, Value};

fn data() -> MapBindings<Value> {
    let hr = tuple![(
        "employeesNestScalars",
        bag![
            tuple![
                ("id", 3),
                ("name", "Bob Smith"),
                ("title", Value::Null),
                (
                    "projects",
                    list![
                        "AWS Redshift Spectrum querying",
                        "AWS Redshift security",
                        "AWS Aurora security",
                    ]
                ),
            ],
            tuple![
                ("id", 4),
                ("name", "Susan Smith"),
                ("title", "Dev Mgr"),
                ("projects", list![]),
            ],
            tuple![
                ("id", 6),
                ("name", "Jane Smith"),
                ("title", "Software Eng 2"),
                ("projects", list!["AWS Redshift security"]),
            ],
        ]
    )];

    let mut p0: MapBindings<Value> = MapBindings::default();
    p0.insert("hr", hr.into());
    p0
}

fn join_data() -> MapBindings<Value> {
    let customers = list![
        tuple![("id", 5), ("name", "Joe")],
        tuple![("id", 7), ("name", "Mary")],
    ];

    let orders = list![
        tuple![("custId", 7), ("productId", 101)],
        tuple![("custId", 7), ("productId", 523)],
    ];

    let mut bindings = MapBindings::default();
    bindings.insert("customers", customers.into());
    bindings.insert("orders", orders.into());
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

fn logical_plan() -> LogicalPlan<BindingsOp> {
    let mut lg = LogicalPlan::new();

    // Similar to ex 9 from spec with projected columns from different tables with an inner JOIN and ON condition
    // SELECT c.id, c.name, o.custId, o.productId FROM customers AS c, orders AS o ON c.id = o.custId
    let from_lhs = scan("customers", "c");
    let from_rhs = scan("orders", "o");

    let project = lg.add_operator(Project(logical::Project {
        exprs: HashMap::from([
            ("id".to_string(), path_var("c", "id")),
            ("name".to_string(), path_var("c", "name")),
            ("custId".to_string(), path_var("o", "custId")),
            ("productId".to_string(), path_var("o", "productId")),
        ]),
    }));

    let join = lg.add_operator(BindingsOp::Join(logical::Join {
        kind: JoinKind::Inner,
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

    lg
}

fn eval_plan(logical: &LogicalPlan<BindingsOp>) -> EvalPlan {
    let catalog = PartiqlCatalog::default();
    let mut planner = plan::EvaluatorPlanner::new(EvaluationMode::Permissive, &catalog);
    planner.compile(logical).expect("Expect no plan error")
}

fn evaluate(mut plan: EvalPlan, bindings: MapBindings<Value>) -> Value {
    if let Ok(out) = plan.execute_mut(bindings) {
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
        // logical plan for SELECT * FROM hr.employeesNestScalars
        let mut logical_plan = LogicalPlan::new();

        let from = logical_plan.add_operator(BindingsOp::Scan(logical::Scan {
            expr: ValueExpr::Path(
                Box::new(ValueExpr::VarRef(BindingsName::CaseInsensitive(
                    "hr".to_string(),
                ))),
                vec![PathComponent::Key(BindingsName::CaseInsensitive(
                    "employeesNestScalars".to_string(),
                ))],
            ),
            as_key: "x".to_string(),
            at_key: None,
        }));
        let project_all = logical_plan.add_operator(ProjectAll);
        let sink = logical_plan.add_operator(BindingsOp::Sink);

        logical_plan.add_flow(from, project_all);
        logical_plan.add_flow(project_all, sink);

        let eval_plan = eval_plan(black_box(&logical_plan));
        if eval {
            evaluate(eval_plan, data());
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
