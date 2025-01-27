use partiql_catalog::catalog::{Catalog, PartiqlCatalog};
use partiql_catalog::context::SystemContext;
use partiql_catalog::extension::Extension;
use partiql_eval::env::basic::MapBindings;
use partiql_eval::error::EvaluationError;
use partiql_eval::eval::BasicContext;
use partiql_eval::plan::EvaluationMode;
use partiql_extension_ion_functions::IonExtension;
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
pub(crate) fn evaluate_with_ion_scan(
    statement: &str,
    env: &Option<Value>,
) -> (Value, Vec<EvaluationError>) {
    let mut catalog = PartiqlCatalog::default();
    let ext = IonExtension {};
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
    let (out, errs) = evaluate_with_ion_scan(statement, env);

    assert!(out.is_bag());
    assert!(errs.is_empty());
    assert_eq!(&out, expected);
}

fn ion_read_select_distinct(file: &str) {
    let value = bag![
        tuple![("Program", "p1"), ("Operation", "get")],
        tuple![("Program", "p1"), ("Operation", "put")],
        tuple![("Program", "p2"), ("Operation", "get")],
        tuple![("Program", "p2"), ("Operation", "put")],
        tuple![("Program", "p3"), ("Operation", "update")],
    ]
    .into();
    let mut path = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
    path.push("resources/test");
    path.push(file);
    let path = path.as_path().display();

    let query = format!("SELECT DISTINCT Program, Operation from read_ion('{path}') as fel");
    pass_eval(&query, &None, &value);
}

#[macro_export]
macro_rules! ion {
    ($x:expr) => {
        partiql_extension_ion::boxed_ion::BoxedIonType {}
            .value_from_str($x)
            .expect("boxed ion construct")
            .into_value()
    };
}

fn ion_scan_select_distinct(file: &str) {
    let value = bag![
        tuple![("Program", ion!("\"p1\"")), ("Operation", ion!("\"get\""))],
        tuple![("Program", ion!("\"p1\"")), ("Operation", ion!("\"put\""))],
        tuple![("Program", ion!("\"p2\"")), ("Operation", ion!("\"get\""))],
        tuple![("Program", ion!("\"p2\"")), ("Operation", ion!("\"put\""))],
        tuple![
            ("Program", ion!("\"p3\"")),
            ("Operation", ion!("\"update\""))
        ],
    ]
    .into();
    let mut path = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
    path.push("resources/test");
    path.push(file);
    let path = path.as_path().display();

    let query = format!("SELECT DISTINCT Program, Operation from scan_ion('{path}') as fel");
    pass_eval(&query, &None, &value);
}

fn ion_scan_range_over(file: &str, query_item: &str) {
    let mut path = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
    path.push("resources/test");
    path.push(file);
    let path = path.as_path().display();

    let query = format!("SELECT {query_item} from scan_ion('{path}') as ion_data");
    let (result, errs) = evaluate_with_ion_scan(&query, &None);

    insta::assert_snapshot!(file, result);
    insta::assert_debug_snapshot!(format!("{file}.errors"), errs);
}

#[test]
fn custom_ion_read_text() {
    ion_read_select_distinct("test.ion");
}

#[test]
fn custom_ion_read_binary() {
    ion_read_select_distinct("test.10n");
}

#[test]
fn custom_ion_read_zstd() {
    ion_read_select_distinct("test.10n.zst");
}

#[test]
fn custom_ion_scan_text() {
    ion_scan_select_distinct("test.ion");
}

#[test]
fn custom_ion_scan_binary() {
    ion_scan_select_distinct("test.10n");
}

#[test]
fn custom_ion_scan_zstd() {
    ion_scan_select_distinct("test.10n.zst");
}

#[test]
fn custom_ion_passthrough() {
    ion_scan_range_over("ion_passthrough_test.ion", "ion_data");
}

#[test]
fn custom_ion_passthrough_structs_text() {
    ion_scan_range_over("ion_passthrough_structs_test.ion", "ion_data.data");
}

#[test]
fn custom_ion_passthrough_bad_text() {
    ion_scan_range_over("ion_passthrough_test.bad.ion", "ion_data.data");
}
