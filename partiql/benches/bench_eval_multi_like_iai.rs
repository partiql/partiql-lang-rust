use criterion::black_box;

use once_cell::sync::Lazy;

use partiql_eval::env::basic::MapBindings;
use partiql_eval::eval::EvalPlan;
use partiql_eval::plan::EvaluatorPlanner;
use partiql_logical::{BindingsOp, LogicalPlan};

use crate::multi_like_data::{employee_data, QUERY_1, QUERY_15, QUERY_30};
use partiql_parser::{Parsed, Parser, ParserResult};
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

/// benchmark parsing of query that filters 1 `LIKE` expressions over 10201 rows of tuples containing an id and a string
fn bench_parse_1() -> ParserResult<'static> {
    parse(black_box(QUERY_1))
}
/// benchmark parsing of query that filters 15 `OR`ed `LIKE` expressions over 10201 rows of tuples containing an id and a string
fn bench_parse_15() -> ParserResult<'static> {
    parse(black_box(QUERY_15))
}
/// benchmark parsing of query that filters 30 `OR`ed `LIKE` expressions over 10201 rows of tuples containing an id and a string
fn bench_parse_30() -> ParserResult<'static> {
    parse(black_box(QUERY_30))
}

//pub(crate) static BUILT_INS: Lazy<FnExprSet<'static>> = Lazy::new(built_ins);

static PARSED_1: Lazy<Parsed<'static>> = Lazy::new(|| parse(QUERY_1).unwrap());
static PARSED_15: Lazy<Parsed<'static>> = Lazy::new(|| parse(QUERY_15).unwrap());
static PARSED_30: Lazy<Parsed<'static>> = Lazy::new(|| parse(QUERY_30).unwrap());

/// benchmark compiling of query that filters 1 `LIKE` expressions over 10201 rows of tuples containing an id and a string
fn bench_compile_1() -> LogicalPlan<BindingsOp> {
    compile(black_box(&PARSED_1))
}
/// benchmark compiling of query that filters 15 `OR`ed `LIKE` expressions over 10201 rows of tuples containing an id and a string
fn bench_compile_15() -> LogicalPlan<BindingsOp> {
    compile(black_box(&PARSED_15))
}
/// benchmark compiling of query that filters 30 `OR`ed `LIKE` expressions over 10201 rows of tuples containing an id and a string
fn bench_compile_30() -> LogicalPlan<BindingsOp> {
    compile(black_box(&PARSED_30))
}

static COMPILED_1: Lazy<LogicalPlan<BindingsOp>> = Lazy::new(|| compile(&PARSED_1));
static COMPILED_15: Lazy<LogicalPlan<BindingsOp>> = Lazy::new(|| compile(&PARSED_15));
static COMPILED_30: Lazy<LogicalPlan<BindingsOp>> = Lazy::new(|| compile(&PARSED_30));

/// benchmark planning of query that filters 1 `LIKE` expressions over 10201 rows of tuples containing an id and a string
fn bench_plan_1() -> EvalPlan {
    plan(black_box(&COMPILED_1))
}
/// benchmark planning of query that filters 15 `OR`ed `LIKE` expressions over 10201 rows of tuples containing an id and a string
fn bench_plan_15() -> EvalPlan {
    plan(black_box(&COMPILED_15))
}
/// benchmark planning of query that filters 30 `OR`ed `LIKE` expressions over 10201 rows of tuples containing an id and a string
fn bench_plan_30() -> EvalPlan {
    plan(black_box(&COMPILED_30))
}
/// benchmark evaluating of query that filters 1 `LIKE` expressions over 10201 rows of tuples containing an id and a string
fn bench_eval_1() -> Value {
    let bindings = employee_data();
    let evaluator = plan(black_box(&COMPILED_1));
    evaluate(evaluator, bindings)
}
/// benchmark evaluating of query that filters 15 `OR`ed `LIKE` expressions over 10201 rows of tuples containing an id and a string
fn bench_eval_15() -> Value {
    let bindings = employee_data();
    let evaluator = plan(black_box(&COMPILED_15));
    evaluate(evaluator, bindings)
}
/// benchmark evaluating of query that filters 30 `OR`ed `LIKE` expressions over 10201 rows of tuples containing an id and a string
fn bench_eval_30() -> Value {
    let bindings = employee_data();
    let evaluator = plan(black_box(&COMPILED_30));
    evaluate(evaluator, bindings)
}

iai::main!(
    bench_parse_1,
    bench_parse_15,
    bench_parse_30,
    bench_compile_1,
    bench_compile_15,
    bench_compile_30,
    bench_plan_1,
    bench_plan_15,
    bench_plan_30,
    bench_eval_1,
    bench_eval_15,
    bench_eval_30,
);
