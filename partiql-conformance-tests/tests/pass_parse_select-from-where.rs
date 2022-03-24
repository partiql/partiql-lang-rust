use partiql_parser::lalr_parse;

#[test]
fn SELECT_with_single_FROM() {
    let parse_result = lalr_parse("SELECT a FROM table1");
    assert!(parse_result.is_ok());
}
