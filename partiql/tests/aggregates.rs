use partiql_eval::plan::EvaluationMode;

mod common;

#[track_caller]
fn assert_eval(query: &str, expected: &str) {
    let res = common::eval_query(query, EvaluationMode::Permissive).expect("eval");
    assert_eq!(format!("{:?}", res.result), expected, "{query}");
}

/// Without GROUP BY the input is one group, even when empty.
#[test]
fn aggregate_without_group_by() {
    assert_eval(
        "SELECT COUNT(*) AS c, SUM(t.a) AS s FROM <<>> t",
        "<<{ 'c': 0, 's': NULL }>>",
    );
    assert_eval(
        "SELECT COUNT(*) AS c, SUM(t.a) AS s FROM <<{'a': 1}, {'a': 2}>> t",
        "<<{ 'c': 2, 's': 3 }>>",
    );
}

/// With GROUP BY, empty input has no groups.
#[test]
fn group_by_over_empty_input_has_no_groups() {
    assert_eval(
        "SELECT t.k AS k, COUNT(*) AS c FROM <<>> t GROUP BY t.k",
        "<<>>",
    );
}
