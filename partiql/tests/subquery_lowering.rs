use crate::common::{eval_query, TestError};
use assert_matches::assert_matches;
use partiql_eval::eval::Evaluated;
use partiql_eval::plan::EvaluationMode;
use partiql_value::{bag, tuple, Value};

mod common;

#[track_caller]
#[inline]
fn eval(statement: &str) -> Result<Evaluated, TestError<'_>> {
    eval_query(statement, EvaluationMode::Permissive)
}

/// A subquery in a projection-list item lowers and evaluates as a `SubQueryExpr` value.
#[test]
fn subquery_in_projection_list() {
    let res = eval("SELECT (SELECT VALUE 2 FROM [0] AS z) AS s FROM [0] AS t");
    assert_matches!(res, Ok(_));
    assert_eq!(
        res.unwrap().result,
        Value::from(bag![tuple![("s", bag![2i64])]])
    );
}

/// A subquery as a struct value in `SELECT VALUE { ... }` lowers and evaluates.
#[test]
fn subquery_as_struct_value() {
    let res = eval("SELECT VALUE {'s': (SELECT VALUE 2 FROM [0] AS z)} FROM [0] AS t");
    assert_matches!(res, Ok(_));
    assert_eq!(
        res.unwrap().result,
        Value::from(bag![tuple![("s", bag![2i64])]])
    );
}

/// A subquery nested inside a scalar operator (here, a function-call argument) lowers and
/// evaluates as an ordinary operand.
#[test]
fn subquery_in_call_arg() {
    let res = eval("SELECT VALUE CARDINALITY((SELECT VALUE 1 FROM [0, 0] AS z)) FROM [0] AS t");
    assert_matches!(res, Ok(_));
    assert_eq!(res.unwrap().result, Value::from(bag![2i64]));
}

/// A subquery nested inside a `CASE` branch lowers and evaluates.
#[test]
fn subquery_in_case_branch() {
    let res = eval(
        "SELECT VALUE CASE WHEN true THEN (SELECT VALUE 1 FROM [0] AS z) ELSE 0 END FROM [0] AS t",
    );
    assert_matches!(res, Ok(_));
    assert_eq!(res.unwrap().result, Value::from(bag![bag![1i64]]));
}

/// A scalar-position subquery returns the subquery's collection as-is; it is not coerced to a
/// scalar. An empty subquery therefore yields an empty bag (matching the `SubQueryExpr`
/// evaluator semantics; see the `subquery_in_project` evaluator test).
#[test]
fn subquery_empty_result() {
    let res = eval("SELECT (SELECT VALUE 2 FROM [] AS z) AS s FROM [0] AS t");
    assert_matches!(res, Ok(_));
    assert_eq!(
        res.unwrap().result,
        Value::from(bag![tuple![("s", bag![])]])
    );
}

/// A set-operation (`UNION`/`EXCEPT`/`INTERSECT`) subquery in scalar position is not handled by
/// this change — only scalar-position `SELECT` subqueries are lowered. It still fails lowering,
/// as it did before; documented here to bound the change.
#[test]
fn setop_subquery_in_scalar_position_unsupported() {
    let res = eval(
        "SELECT (SELECT VALUE 1 FROM [0] AS z UNION SELECT VALUE 2 FROM [0] AS z) AS s FROM [0] AS t",
    );
    assert_matches!(res, Err(TestError::Lower(_)));
}
