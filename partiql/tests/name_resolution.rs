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
/// key reads the FROM binding's attribute, not the key: in SELECT, HAVING and ORDER BY.
#[test]
fn aggregate_argument_sees_names_from_before_group_by() {
    // Per group `c`: SUM(c) is 5 for c = 1, 3 for c = 3 and 4 for c = 4, so ordering by
    // SUM(c) differs from ordering by `c`.
    let data = "<< {'c': 1}, {'c': 1}, {'c': 1}, {'c': 1}, {'c': 1}, {'c': 3}, {'c': 4} >>";
    assert_eq!(
        eval(&format!(
            "SELECT c, SUM(c) AS s FROM {data} AS e GROUP BY c"
        )),
        Value::from(partiql_value::bag![
            tuple![("c", 1), ("s", 5)],
            tuple![("c", 3), ("s", 3)],
            tuple![("c", 4), ("s", 4)]
        ])
    );
    // A NULL SUM would drop every group.
    assert_eq!(
        eval(&format!(
            "SELECT VALUE c FROM {data} AS e GROUP BY c HAVING SUM(c) > 3"
        )),
        Value::from(partiql_value::bag![1, 4])
    );
    assert_eq!(
        eval(&format!(
            "SELECT VALUE c FROM {data} AS e GROUP BY c ORDER BY SUM(c)"
        )),
        Value::from(list![3, 4, 1])
    );
}
