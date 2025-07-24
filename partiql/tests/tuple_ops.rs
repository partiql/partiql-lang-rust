use crate::common::{eval_query_with_catalog, TestError};
use assert_matches::assert_matches;
use partiql_catalog::catalog::PartiqlCatalog;
use partiql_catalog::extension::Extension;
use partiql_eval::eval::Evaluated;
use partiql_eval::plan::EvaluationMode;
use partiql_extension_value_functions::PartiqlValueFnExtension;
use partiql_value::Value;

mod common;

#[track_caller]
#[inline]
pub fn eval(statement: &str, mode: EvaluationMode) -> Result<Evaluated, TestError<'_>> {
    let mut catalog = PartiqlCatalog::default();
    let ext = PartiqlValueFnExtension::default();
    ext.load(&mut catalog)?;
    let catalog = catalog.to_shared_catalog();

    eval_query_with_catalog(statement, &catalog, mode)
}

#[test]
fn tupleunion() {
    let query = "tupleunion({ 'bob': 1, 'sally': 'error' }, { 'sally': 1 }, { 'sally': 2 }, { 'sally': 3 }, { 'sally': 4 })";

    let res = eval(query, EvaluationMode::Permissive);
    assert_matches!(res, Ok(_));

    let res = res.unwrap().result;
    assert_matches!(res, Value::Tuple(_));
    let tuple = res.as_tuple_ref();
    assert_eq!(tuple.len(), 6);

    insta::assert_debug_snapshot!(tuple);
}

#[test]
fn tupleconcat() {
    let query = "tupleconcat({ 'bob': 1, 'sally': 'error' }, { 'sally': 1 }, { 'sally': 2 }, { 'sally': 3 }, { 'sally': 4 })";

    let res = eval(query, EvaluationMode::Permissive);
    assert_matches!(res, Ok(_));

    let res = res.unwrap().result;
    assert_matches!(res, Value::Tuple(_));
    let tuple = res.as_tuple_ref();
    assert_eq!(tuple.len(), 2);

    insta::assert_debug_snapshot!(tuple);
}
