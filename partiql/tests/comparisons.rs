use crate::common::{
    compile, eval_query, eval_query_with_catalog, evaluate, lower, parse, TestError,
};
use assert_matches::assert_matches;
use partiql_catalog::catalog::{Catalog, PartiqlCatalog};
use partiql_catalog::extension::Extension;
use partiql_eval::eval::Evaluated;
use partiql_eval::plan::EvaluationMode;
use partiql_extension_value_functions::PartiqlValueFnExtension;
use partiql_value::Value;
use std::os::macos::raw::stat;

mod common;

#[track_caller]
#[inline]
pub fn eval<'a>(statement: &'a str) {
    dbg!(&statement);
    let res = eval_query(statement, EvaluationMode::Permissive);
    assert_matches!(res, Ok(_));
    let res = res.unwrap().result;
    assert_matches!(res, Value::Missing);

    let res = eval_query(statement, EvaluationMode::Strict);
    assert_matches!(res, Err(_));
    let err = res.unwrap_err();
    assert_matches!(err, TestError::Eval(_));
}

#[track_caller]
#[inline]
pub fn eval_op<'a>(op: &'a str) {
    eval(&format!("1 {op} 'foo'"))
}

#[test]
fn lt() {
    eval_op("<")
}

#[test]
fn gt() {
    eval_op(">")
}

#[test]
fn lte() {
    eval_op("<=")
}

#[test]
fn gte() {
    eval_op(">=")
}
