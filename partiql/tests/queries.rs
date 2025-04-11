use crate::common::{eval_query_with_catalog, TestError};
use assert_matches::assert_matches;
use partiql_catalog::catalog::PartiqlCatalog;
use partiql_catalog::extension::Extension;
use partiql_eval::eval::Evaluated;
use partiql_eval::plan::EvaluationMode;
use partiql_extension_value_functions::PartiqlValueFnExtension;

mod common;

#[track_caller]
#[inline]
pub fn eval(statement: &str, mode: EvaluationMode) -> Result<Evaluated, TestError<'_>> {
    let mut catalog = PartiqlCatalog::default();
    let ext = PartiqlValueFnExtension::default();
    ext.load(&mut catalog)?;

    eval_query_with_catalog(statement, &catalog, mode)
}

#[test]
fn select_star_unpivot() {
    let query = "SELECT * FROM UNPIVOT {'amzn': 840.05, 'tdc': 31.06} AS price AT sym";

    let res = eval(query, EvaluationMode::Permissive);
    assert_matches!(res, Ok(_));
    let res = res.unwrap().result;
    insta::assert_debug_snapshot!(res);
}
