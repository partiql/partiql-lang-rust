use std::time::Duration;

use criterion::{black_box, criterion_group, criterion_main, Criterion};

use partiql_eval::env::basic::MapBindings;
use partiql_eval::eval::EvalPlan;
use partiql_eval::plan::EvaluatorPlanner;
use partiql_logical::{BindingsOp, LogicalPlan};

use crate::multi_like_data::{employee_data, QUERY_1, QUERY_15, QUERY_30};
use partiql_parser::{Parser, ParserResult};
use partiql_value::Value;

// Benchmarks:
//  - parsing,
//  - compiling
//  - planning
//  - evaluation
//
// of queries that  filter against 1, 15, or 30 `OR`ed `LIKE` expressions
// over 10201 rows of tuples containing an id and a string

mod multi_like_data;

#[inline]
fn parse(text: &str) -> ParserResult {
    Parser::default().parse(text)
}
#[inline]
fn compile(parsed: &partiql_parser::Parsed) -> LogicalPlan<BindingsOp> {
    partiql_logical_planner::lower(parsed)
}
#[inline]
fn plan(logical: &LogicalPlan<BindingsOp>) -> EvalPlan {
    EvaluatorPlanner::default().compile(logical)
}
#[inline]
pub(crate) fn evaluate(mut eval: EvalPlan, bindings: MapBindings<Value>) -> Value {
    if let Ok(out) = eval.execute_mut(bindings) {
        out.result
    } else {
        Value::Missing
    }
}

/// benchmark parsing of queries that
/// filter against 1, 15, or 30 `OR`ed `LIKE` expressions
/// over 10201 rows of tuples containing an id and a string
fn bench_parse(c: &mut Criterion) {
    let parsed_1 = parse(QUERY_1);
    assert!(parsed_1.is_ok());
    let parsed_15 = parse(QUERY_15);
    assert!(parsed_15.is_ok());
    let parsed_30 = parse(QUERY_30);
    assert!(parsed_30.is_ok());

    c.bench_function("parse-1", |b| b.iter(|| parse(black_box(QUERY_1))));
    c.bench_function("parse-15", |b| b.iter(|| parse(black_box(QUERY_15))));
    c.bench_function("parse-30", |b| b.iter(|| parse(black_box(QUERY_30))));
}

/// benchmark compiling of queries that
/// filter against 1, 15, or 30 `OR`ed `LIKE` expressions
/// over 10201 rows of tuples containing an id and a string
fn bench_compile(c: &mut Criterion) {
    let parsed_1 = parse(QUERY_1).unwrap();
    let parsed_15 = parse(QUERY_15).unwrap();
    let parsed_30 = parse(QUERY_30).unwrap();

    let compiled_1 = compile(&parsed_1);
    assert_eq!(compiled_1.operator_count(), 4);
    let compiled_15 = compile(&parsed_15);
    assert_eq!(compiled_15.operator_count(), 4);
    let compiled_30 = compile(&parsed_30);
    assert_eq!(compiled_30.operator_count(), 4);

    c.bench_function("compile-1", |b| b.iter(|| compile(black_box(&parsed_1))));
    c.bench_function("compile-15", |b| b.iter(|| compile(black_box(&parsed_15))));
    c.bench_function("compile-30", |b| b.iter(|| compile(black_box(&parsed_30))));
}

/// benchmark planning of queries that
/// filter against 1, 15, or 30 `OR`ed `LIKE` expressions
/// over 10201 rows of tuples containing an id and a string
fn bench_plan(c: &mut Criterion) {
    let compiled_1 = compile(&parse(QUERY_1).unwrap());
    let compiled_15 = compile(&parse(QUERY_15).unwrap());
    let compiled_30 = compile(&parse(QUERY_30).unwrap());

    let _planned_1 = plan(&compiled_1);
    let _planned_15 = plan(&compiled_15);
    let _planned_30 = plan(&compiled_30);

    c.bench_function("plan-1", |b| b.iter(|| plan(black_box(&compiled_1))));
    c.bench_function("plan-15", |b| b.iter(|| plan(black_box(&compiled_15))));
    c.bench_function("plan-30", |b| b.iter(|| plan(black_box(&compiled_30))));
}

/// benchmark evaluation of queries that
/// filter against 1, 15, or 30 `OR`ed `LIKE` expressions
/// over 10201 rows of tuples containing an id and a string
fn bench_eval(c: &mut Criterion) {
    let compiled_1 = compile(&parse(QUERY_1).unwrap());
    let compiled_15 = compile(&parse(QUERY_15).unwrap());
    let compiled_30 = compile(&parse(QUERY_30).unwrap());

    let bindings = employee_data();

    c.bench_function("eval-1", |b| {
        b.iter(|| {
            let plan = plan(&compiled_1);
            let bindings = bindings.clone();
            evaluate(black_box(plan), black_box(bindings))
        })
    });
    c.bench_function("eval-15", |b| {
        b.iter(|| {
            let plan = plan(&compiled_15);
            let bindings = bindings.clone();
            evaluate(black_box(plan), black_box(bindings))
        })
    });
    c.bench_function("eval-30", |b| {
        b.iter(|| {
            let plan = plan(&compiled_30);
            let bindings = bindings.clone();
            evaluate(black_box(plan), black_box(bindings))
        })
    });
}

criterion_group! {
    name = eval;
    config = Criterion::default().measurement_time(Duration::new(5, 0));
    targets = bench_parse, bench_compile, bench_plan, bench_eval
}

criterion_main!(eval);
