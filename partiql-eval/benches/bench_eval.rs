use std::time::Duration;

use criterion::{black_box, criterion_group, criterion_main, Criterion};

use partiql_eval::env::basic::MapBindings;
use partiql_eval::eval::{
    BasicContext, EvalPath, EvalPathComponent, EvalScan, EvalVarRef, Evaluable,
};
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

fn eval_bench(c: &mut Criterion) {
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
