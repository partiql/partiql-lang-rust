use std::ops::Deref;
use std::time::Duration;

use criterion::{black_box, criterion_group, criterion_main, Criterion};
use partiql_catalog::catalog::{Catalog, PartiqlCatalog, PartiqlSharedCatalog, SharedCatalog};
use partiql_catalog::context::SystemContext;
use partiql_eval::env::basic::MapBindings;
use partiql_eval::eval::{BasicContext, EvalPlan};
use partiql_eval::plan::{EvaluationMode, EvaluatorPlanner};
use partiql_logical::{BindingsOp, LogicalPlan};
use partiql_logical_planner::LogicalPlanner;

use partiql_parser::{Parser, ParserResult};
use partiql_value::{tuple, Bag, DateTime, Value};

use once_cell::sync::Lazy;
pub(crate) static SHARED_CATALOG: Lazy<PartiqlSharedCatalog> = Lazy::new(init_shared_catalog);

fn init_shared_catalog() -> PartiqlSharedCatalog {
    PartiqlCatalog::default().to_shared_catalog()
}

fn tables() -> MapBindings<Value> {
    let mut txt = ["foo", "bar", "baz", "qux", "etc"].into_iter().cycle();
    let mut names = ["bob", "fido", "foo"].into_iter().cycle();
    let nums = 0..500i64;

    let table1 = nums
        .clone()
        .map(|idx| tuple![("id", idx), ("txt", txt.next().unwrap())])
        .collect::<Bag>();

    let table2 = nums
        .clone()
        .map(|idx| tuple![("id", idx * 2), ("name", names.next().unwrap())])
        .collect::<Bag>();

    tuple![("table1", table1), ("table2", table2)].into()
}

#[inline]
fn parse(text: &str) -> ParserResult {
    Parser::default().parse(text)
}
#[inline]
fn compile(
    catalog: &dyn SharedCatalog,
    parsed: &partiql_parser::Parsed,
) -> LogicalPlan<BindingsOp> {
    let planner = LogicalPlanner::new(catalog);
    planner.lower(parsed).expect("Expect no lower error")
}
#[inline]
fn plan(catalog: &dyn SharedCatalog, logical: &LogicalPlan<BindingsOp>) -> EvalPlan {
    EvaluatorPlanner::new(EvaluationMode::Permissive, catalog)
        .compile(logical)
        .expect("Expect no plan error")
}
#[inline]
pub(crate) fn evaluate(eval: EvalPlan, bindings: MapBindings<Value>) -> Value {
    let sys = SystemContext {
        now: DateTime::from_system_now_utc(),
    };
    let ctx = BasicContext::new(bindings, sys);
    if let Ok(out) = eval.execute(&ctx) {
        out.result
    } else {
        Value::Missing
    }
}

fn bench_join(c: &mut Criterion) {
    let catalog: &PartiqlSharedCatalog = SHARED_CATALOG.deref();
    let bindings = tables();

    let query = "SELECT * FROM table1, table2";
    let compiled = compile(catalog, &parse(query).unwrap());
    c.bench_function("cartesian join", |b| {
        b.iter(|| {
            let plan = plan(catalog, &compiled);
            let bindings = bindings.clone();
            evaluate(black_box(plan), black_box(bindings))
        })
    });

    let query = "SELECT * FROM table1 INNER JOIN table2 ON table1.id = table2.id";
    let compiled = compile(catalog, &parse(query).unwrap());
    c.bench_function("inner equi join", |b| {
        b.iter(|| {
            let plan = plan(catalog, &compiled);
            let bindings = bindings.clone();
            evaluate(black_box(plan), black_box(bindings))
        })
    });
}

criterion_group! {
    name = eval;
    config = Criterion::default().measurement_time(Duration::new(5, 0));
    targets = bench_join
}

criterion_main!(eval);
