use partiql_parser::lalr_parse;

#[test]
fn SFW_without_WHERE_expression() {
    let parse_result = lalr_parse("SELECT * FROM foo WHERE");
    assert!(parse_result.is_err());
}
