use crate::common::eval_query;
use partiql_eval::plan::EvaluationMode;
use partiql_value::{list, tuple, Value};

mod common;

#[track_caller]
fn eval(statement: &str) -> Value {
    eval_query(statement, EvaluationMode::Permissive)
        .expect("evaluates")
        .result
}

/// ORDER BY on a set operation sorts by columns of the rows it produces.
#[test]
fn set_op_order_by_sorts_by_result_columns() {
    let union =
        "(SELECT a FROM << {'a': 2}, {'a': 3} >> t OUTER UNION ALL SELECT a FROM << {'a': 1} >> u)";
    assert_eq!(
        eval(&format!("{union} ORDER BY a")),
        Value::from(list![tuple![("a", 1)], tuple![("a", 2)], tuple![("a", 3)]])
    );
    assert_eq!(
        eval(&format!("{union} ORDER BY a DESC LIMIT 2")),
        Value::from(list![tuple![("a", 3)], tuple![("a", 2)]])
    );
}

/// A quoted name matches exactly: `"E"` is not the binding `e`.
#[test]
fn quoted_name_is_case_sensitive() {
    assert_eq!(
        eval("SELECT VALUE \"E\" FROM << {'a': 1} >> AS e"),
        eval("SELECT VALUE MISSING FROM << {'a': 1} >> AS e")
    );
    assert_eq!(
        eval("SELECT VALUE \"e\".a FROM << {'a': 1} >> AS e"),
        Value::from(partiql_value::bag![1])
    );
}
