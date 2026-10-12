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

/// An aggregate's argument is evaluated on each input row, so a name that is also a GROUP BY
/// key reads the FROM binding's attribute, not the key.
#[test]
fn aggregate_argument_sees_names_from_before_group_by() {
    let data = "<< {'c': 1, 'v': 10}, {'c': 1, 'v': 20}, {'c': 2, 'v': 5} >>";
    assert_eq!(
        eval(&format!(
            "SELECT c, SUM(c) AS s FROM {data} AS e GROUP BY c"
        )),
        Value::from(partiql_value::bag![
            tuple![("c", 1), ("s", 2)],
            tuple![("c", 2), ("s", 2)]
        ])
    );
    assert_eq!(
        eval(&format!(
            "SELECT c, SUM(v) AS s FROM {data} AS e GROUP BY c HAVING SUM(c) > 1 ORDER BY SUM(v)"
        )),
        Value::from(list![
            tuple![("c", 2), ("s", 5)],
            tuple![("c", 1), ("s", 30)]
        ])
    );
}
