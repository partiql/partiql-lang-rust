use partiql_catalog::catalog::{Catalog, PartiqlCatalog};
use partiql_catalog::context::SystemContext;
use partiql_catalog::extension::Extension;
use partiql_eval::env::basic::MapBindings;
use partiql_eval::error::EvaluationError;
use partiql_eval::eval::BasicContext;
use partiql_eval::plan::EvaluationMode;
use partiql_extension_csv::CsvExtension;
use partiql_parser::{Parsed, ParserResult};
use partiql_value::{bag, tuple, DateTime, Value};
use std::path::PathBuf;

#[track_caller]
#[inline]
pub(crate) fn parse(statement: &str) -> ParserResult<'_> {
    partiql_parser::Parser::default().parse(statement)
}

#[track_caller]
#[inline]
pub(crate) fn lower(
    catalog: &dyn Catalog,
    parsed: &Parsed<'_>,
) -> partiql_logical::LogicalPlan<partiql_logical::BindingsOp> {
    let planner = partiql_logical_planner::LogicalPlanner::new(catalog);
    planner.lower(parsed).expect("lower")
}

#[track_caller]
#[inline]
pub(crate) fn evaluate(
    catalog: &dyn Catalog,
    logical: partiql_logical::LogicalPlan<partiql_logical::BindingsOp>,
    bindings: MapBindings<Value>,
) -> (Value, Vec<EvaluationError>) {
    let mut planner =
        partiql_eval::plan::EvaluatorPlanner::new(EvaluationMode::Permissive, catalog);

    let mut plan = planner.compile(&logical).expect("Expect no plan error");

    let sys = SystemContext {
        now: DateTime::from_system_now_utc(),
    };
    let ctx = BasicContext::new(bindings, sys);
    let value = if let Ok(out) = plan.execute_mut(&ctx) {
        out.result
    } else {
        Value::Missing
    };
    let errs = ctx.errors.take();
    (value, errs)
}

#[inline]
#[allow(dead_code)]
pub(crate) fn evaluate_with_csv_scan(
    statement: &str,
    env: &Option<Value>,
) -> (Value, Vec<EvaluationError>) {
    let mut catalog = PartiqlCatalog::default();
    let ext = CsvExtension {};
    ext.load(&mut catalog)
        .expect("ion extension load to succeed");

    let parsed = parse(statement);
    let lowered = lower(&catalog, &parsed.expect("parse"));
    let bindings = env
        .as_ref()
        .map(std::convert::Into::into)
        .unwrap_or_default();
    evaluate(&catalog, lowered, bindings)
}

#[inline]
#[allow(dead_code)]
pub(crate) fn pass_eval(statement: &str, env: &Option<Value>, expected: &Value) {
    let (out, errs) = evaluate_with_csv_scan(statement, env);

    assert!(out.is_bag());
    assert!(errs.is_empty());
    assert_eq!(&out, expected);
}

fn csv_scan_range_over(file: &str, query_item: &str) {
    let mut path = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
    path.push("resources/test");
    path.push(file);
    let path = path.as_path().display();

    let query = format!("SELECT {query_item} from scan_csv('{path}') as csv_data");
    let (result, errs) = evaluate_with_csv_scan(&query, &None);

    insta::assert_snapshot!(file, result);
    insta::assert_debug_snapshot!(format!("{file}.errors"), errs);
}

#[test]
fn people_csv() {
    csv_scan_range_over("people.csv", "*");
}

#[test]
fn pets_csv() {
    csv_scan_range_over("pets.csv", "*");
}

#[test]
fn join() {
    let mut people = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
    people.push("resources/test/people.csv");
    let people = people.as_path().display();

    let mut pets = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
    pets.push("resources/test/pets.csv");
    let pets = pets.as_path().display();

    let query = format!(
        "SELECT people.Color, pets.Food \
         FROM scan_csv('{people}') as people \
            INNER JOIN scan_csv('{pets}') as pets \
                    ON people.Pet=pets.Pet"
    );
    let (result, errs) = evaluate_with_csv_scan(&query, &None);

    insta::assert_snapshot!(result);
}
