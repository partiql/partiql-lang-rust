use partiql_parser::lalr_parse;

#[test]
fn literal_int() {
    let parse_result = lalr_parse("5");
    assert!(parse_result.is_ok());
}

#[test]
fn literal_null() {
    let parse_result = lalr_parse("null");
    assert!(parse_result.is_ok());
}

#[test]
fn literal_missing() {
    let parse_result = lalr_parse("missing");
    assert!(parse_result.is_ok());
}

#[test]
fn literal_list() {
    let parse_result = lalr_parse("[a, 5]");
    assert!(parse_result.is_ok());
}

#[test]
fn literal_list_with_binary_operation() {
    let parse_result = lalr_parse("[a, 5, (b + 6)]");
    assert!(parse_result.is_ok());
}

#[test]
fn literal_list_function() {
    let parse_result = lalr_parse("LIST(a, 5)");
    assert!(parse_result.is_ok());
}

#[test]
fn literal_sexp_function() {
    let parse_result = lalr_parse("SEXP(a, 5)");
    assert!(parse_result.is_ok());
}

#[test]
fn literal_sexp_function_with_binary_operation() {
    let parse_result = lalr_parse("SEXP(a, 5, (b + 6))");
    assert!(parse_result.is_ok());
}

#[test]
fn literal_struct() {
    let parse_result = lalr_parse("{'x':a, 'y':5 }");
    assert!(parse_result.is_ok());
}

#[test]
fn literal_struct_with_binary_operation() {
    let parse_result = lalr_parse("{'x':a, 'y':5, 'z':(b + 6)}");
    assert!(parse_result.is_ok());
}

#[test]
fn literal_nested_empty_list() {
    let parse_result = lalr_parse("[[]]");
    assert!(parse_result.is_ok());
}

#[test]
fn literal_nested_empty_bag() {
    let parse_result = lalr_parse("<<<<>>>>");
    assert!(parse_result.is_ok());
}

#[test]
fn literal_nested_empty_struct() {
    let parse_result = lalr_parse("{'a':{}}");
    assert!(parse_result.is_ok());
}
