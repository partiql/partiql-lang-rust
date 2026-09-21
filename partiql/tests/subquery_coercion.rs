use crate::common::eval_query;
use assert_matches::assert_matches;
use partiql_eval::plan::EvaluationMode;
use partiql_value::{bag, tuple, Value};

mod common;

#[track_caller]
fn eval(q: &str) -> Value {
    eval_query(q, EvaluationMode::Permissive)
        .expect("eval")
        .result
}

/// Coercion of a SELECT subquery into a scalar (PartiQL spec §9.1): an SQL-style `SELECT`
/// (projection list / `*`) appearing in a scalar-expecting position coerces to the single
/// contained value. A singleton result with one attribute yields that attribute's value.
#[test]
fn sql_select_subquery_coerces_to_scalar() {
    assert_matches!(
        eval("1 + (SELECT v.n FROM [{'n': 41}] AS v)"),
        Value::Integer(42)
    );
}

/// Non-singleton results coerce to `MISSING` (permissive; never fails).
#[test]
fn empty_and_multi_row_coerce_to_missing() {
    // empty
    assert_matches!(
        eval("1 + (SELECT v.n FROM [{'n': 41}] AS v WHERE v.n > 100)"),
        Value::Missing
    );
    // more than one row
    assert_matches!(
        eval("1 + (SELECT v.n FROM [{'n': 1}, {'n': 2}] AS v)"),
        Value::Missing
    );
}

/// `SELECT VALUE` explicitly constructs a collection and is NOT coerced, even in a
/// scalar-adjacent position — it remains a bag (here counted by CARDINALITY).
#[test]
fn select_value_is_not_coerced() {
    assert_matches!(
        eval("CARDINALITY(SELECT VALUE v.n FROM [{'n': 1}, {'n': 2}] AS v)"),
        Value::Integer(2)
    );
}

/// `coll_to_scalar` is an internal definitional builtin (injected by the coercion above),
/// not a user-callable function — calling it by name must not resolve.
#[test]
fn coll_to_scalar_is_not_user_callable() {
    let r = eval_query("coll_to_scalar(<<{'n': 1}>>)", EvaluationMode::Permissive);
    assert_matches!(r, Err(_));
}

// The coercion is context-sensitive (matching the Kotlin `SubqueryCoercionVisitorTransform`):
// a scalar-position SQL `SELECT` coerces to a single value ONLY where the enclosing context
// expects one. The tests below pair the non-coercing contexts (a value in a struct/collection
// constructor, a `CASE` branch, a non-`EXISTS` call argument), where the subquery must remain
// its collection, with the coercing ones (a projection-list item, a comparison operand).

/// A subquery used as a struct attribute value is not a single-value context; it stays a bag.
#[test]
fn struct_value_subquery_is_not_coerced() {
    assert_eq!(
        eval("{'a': (SELECT v.n FROM [{'n': 7}] AS v)}"),
        Value::from(tuple![("a", bag![tuple![("n", 7i64)]])])
    );
}

/// A subquery in a `CASE` branch is not a single-value context; it stays a bag.
#[test]
fn case_branch_subquery_is_not_coerced() {
    assert_eq!(
        eval("CASE WHEN true THEN (SELECT v.n FROM [{'n': 7}] AS v) ELSE 0 END"),
        Value::from(bag![tuple![("n", 7i64)]])
    );
}

/// A subquery as a (non-`EXISTS`) function-call argument is not a single-value context;
/// `CARDINALITY` sees the one-row collection rather than a coerced scalar.
#[test]
fn call_arg_subquery_is_not_coerced() {
    assert_matches!(
        eval("CARDINALITY((SELECT v.n FROM [{'n': 7}] AS v))"),
        Value::Integer(1)
    );
}

/// A subquery in a projection-list item IS a single-value context; it coerces to the scalar.
#[test]
fn projection_item_subquery_coerces_to_scalar() {
    assert_eq!(
        eval("SELECT (SELECT v.n FROM [{'n': 7}] AS v) AS s FROM [0] AS t"),
        Value::from(bag![tuple![("s", 7i64)]])
    );
}

/// A subquery as a comparison operand (here in `WHERE`) IS a single-value context; it coerces.
#[test]
fn comparison_operand_subquery_coerces_to_scalar() {
    assert_eq!(
        eval("SELECT VALUE t.x FROM [{'x': 7}] AS t WHERE t.x = (SELECT v.n FROM [{'n': 7}] AS v)"),
        Value::from(bag![7i64])
    );
}

/// A subquery that is the ROOT of a path is not itself a single-value context: the enclosing
/// coercing context (here the `+` operand) must not reach through the path and coerce the root
/// (Kotlin's `SubqueryCoercionVisitorTransform` returns `Path` unchanged). The path navigates the
/// subquery's collection result, so it evaluates the same whether or not it sits in such a context.
#[test]
fn path_root_subquery_not_coerced_by_enclosing_context() {
    let top_level = eval("(SELECT v.a FROM <<{'a': {'b': 7}}>> AS v).b");
    let in_coercing_context = eval("(SELECT v.a FROM <<{'a': {'b': 7}}>> AS v).b + 0");
    assert_eq!(top_level, in_coercing_context);
}
