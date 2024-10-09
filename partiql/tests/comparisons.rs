use crate::common::{eval_query, TestError};
use assert_matches::assert_matches;
use partiql_eval::plan::EvaluationMode;
use partiql_value::Value;

mod common;

#[track_caller]
#[inline]
pub fn eval(statement: &str) {
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
pub fn eval_op(op: &str) {
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
