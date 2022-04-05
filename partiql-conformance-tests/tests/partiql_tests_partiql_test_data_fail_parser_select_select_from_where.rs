#[test]
fn sfw_without_where_expression_test() {
    let parse_result = partiql_parser::lalr_parse("SELECT * FROM foo WHERE");
    assert!(parse_result.is_err());
}