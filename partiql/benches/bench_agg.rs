use std::ops::Deref;
use std::time::Duration;

use criterion::{black_box, criterion_group, criterion_main, Criterion};
use itertools::Itertools;
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

fn numbers() -> impl Iterator<Item = Value> {
    (0..1000i64).map(Value::from)
}

fn data() -> MapBindings<Value> {
    let mut shards = [
        "df589d89-b866-4eac-865a-9ed29ce37e4d",
        "9b4fd584-77e7-46d0-80c8-52af06fdb56d",
        "cb6b8f09-cc76-426f-b5ad-9786f4b4473d",
    ]
    .into_iter()
    .cycle();
    let numbers: Bag = numbers()
        .map(|n| tuple![("shard", shards.next().unwrap()), ("n", n)])
        .collect();
    let data = tuple![("numbers", numbers)];
    data.into()
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

fn create_query(aggs: &[(&'static str, bool)], group: bool, group_as: bool) -> (String, String) {
    let agg_fns = aggs
        .iter()
        .map(|(name, distinct)| {
            format!(
                "{}({}v.n) as agg_{}",
                name.to_uppercase(),
                if *distinct { "DISTINCT " } else { "" },
                name.to_lowercase()
            )
        })
        .join(", ");
    let group_by_clause = if group {
        "GROUP BY shard, v.n%2 = 0 as is_even"
    } else {
        ""
    };
    let group_as_clause = if group_as { "GROUP AS g" } else { "" };
    let query = format!(
        "SELECT shard, is_even, {agg_fns}, g FROM numbers AS v {group_by_clause} {group_as_clause}"
    );

    let agg_name = aggs
        .iter()
        .map(|(name, distinct)| {
            format!(
                "{}{}",
                name.to_lowercase(),
                if *distinct { "_distinct" } else { "" }
            )
        })
        .join("-");
    let group_by_name = if group { "group_by" } else { "" }.to_string();
    let group_as_name = if group_as { "group_as" } else { "" }.to_string();
    let name = [
        "arith_agg".to_string(),
        agg_name,
        group_by_name,
        group_as_name,
    ]
    .iter()
    .filter(|s| !s.is_empty())
    .join("-");
    (name, query)
}

fn create_tests() -> Vec<(String, String)> {
    let aggs = ["avg", "count", "min", "max", "sum"];
    let distincts = [false, true];

    let all_aggs = aggs.iter().cartesian_product(distincts.iter());
    let groups = [(false, false), (true, false), (true, true)].iter();

    let simple = all_aggs
        .clone()
        .map(|(&a, d)| create_query(vec![(a, *d)].as_slice(), false, false));

    let aggs_all = aggs.into_iter().cartesian_product([false]).collect_vec();
    let aggs_distinct = aggs.into_iter().cartesian_product([true]).collect_vec();

    let full_aggs_all = groups
        .clone()
        .map(|(g_by, g_as)| create_query(&aggs_all, *g_by, *g_as));
    let full_aggs_distinct = groups
        .clone()
        .map(|(g_by, g_as)| create_query(&aggs_distinct, *g_by, *g_as));

    simple
        .chain(full_aggs_all)
        .chain(full_aggs_distinct)
        .collect_vec()
}

fn bench_agg(c: &mut Criterion) {
    let catalog: &PartiqlSharedCatalog = SHARED_CATALOG.deref();
    let bindings = data();

    for (name, query) in create_tests() {
        let compiled = compile(catalog, &parse(query.as_str()).unwrap());
        c.bench_function(name.as_str(), |b| {
            b.iter(|| {
                let plan = plan(catalog, &compiled);
                let bindings = bindings.clone();
                evaluate(black_box(plan), black_box(bindings))
            })
        });
    }
}

criterion_group! {
    name = eval;
    config = Criterion::default().measurement_time(Duration::new(5, 0));
    targets = bench_agg
}

criterion_main!(eval);
