#[test]
fn select_with_single_from_test() {
    let parse_result = partiql_parser::lalr_parse("SELECT a FROM table1");
    assert!(parse_result.is_ok());
}