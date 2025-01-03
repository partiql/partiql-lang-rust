use crate::common::{eval_query, TestError};
use assert_matches::assert_matches;
use itertools::Itertools;
use partiql_eval::eval::Evaluated;
use partiql_eval::plan::EvaluationMode;
use partiql_value::{Comparable, Value};

mod common;

#[track_caller]
#[inline]
pub fn eval_modes(statement: &str) -> (Result<Evaluated, TestError>, Result<Evaluated, TestError>) {
    let permissive = eval_query(statement, EvaluationMode::Permissive);
    let strict = eval_query(statement, EvaluationMode::Strict);
    (permissive, strict)
}

#[track_caller]
#[inline]
pub fn eval_fail(statement: &str) {
    let (permissive, strict) = eval_modes(statement);

    assert_matches!(permissive, Ok(_));
    let permissive = permissive.unwrap().result;
    assert_matches!(permissive, Value::Missing);

    assert_matches!(strict, Err(_));
    let err = strict.unwrap_err();
    assert_matches!(err, TestError::Eval(_));
}

#[track_caller]
#[inline]
pub fn eval_success(statement: &str) {
    let (permissive, strict) = eval_modes(statement);
    assert_matches!(permissive, Ok(_));
    assert_matches!(strict, Ok(_));
    assert_eq!(permissive.unwrap().result, strict.unwrap().result);
}

#[test]
fn test1() {
    eval_success("[{'a': 1},{'b': 2}]")
}

#[test]
fn test11() {
    eval_success("[{'a': 1+1},{'b': 2}]")
}

#[track_caller]
#[inline]
pub fn eval_op(op: &str) {
    let vals = op_values();
    let pairs = vals.clone().into_iter().cartesian_product(vals);
    for (l, r) in pairs {
        let statement = format!("{l} {op} {r}");
        if l.is_comparable_to(&r) {
            println!("`{statement}` should compare");
            eval_success(&statement);
        } else {
            println!("`{statement}` should error");
            eval_fail(&statement);
        }
    }
}

fn op_values() -> [Value; 4] {
    [
        Value::Integer(1),
        Value::Real(3.14.into()),
        Value::Boolean(true),
        Value::String("foo".to_string().into()),
        /* TODO currently DateTimes can be printed but not yet parsed
        Value::DateTime(Box::new(DateTime::TimestampWithTz(
            time::OffsetDateTime::now_utc(),
        ))),
        */
    ]
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

#[test]
fn between() {
    let vals = op_values();
    let pairs = vals.clone().into_iter().cartesian_product(vals.clone());
    let trios = pairs
        .into_iter()
        .cartesian_product(vals)
        .map(|((l, m), r)| (l, m, r));
    for (l, m, r) in trios {
        let statement = format!("{l} BETWEEN {m} AND {r}");
        if l.is_comparable_to(&r) && l.is_comparable_to(&m) {
            println!("`{statement}` should compare");
            eval_success(&statement);
        } else {
            println!("`{statement}` should error");
            eval_fail(&statement);
        }
    }
}
