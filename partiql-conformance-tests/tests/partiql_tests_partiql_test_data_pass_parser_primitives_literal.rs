mod literal_namespace {
    #[test]
    fn int_test() {
        let parse_result = partiql_parser::lalr_parse("5");
        assert!(parse_result.is_ok());
    }

    #[test]
    fn null_test() {
        let parse_result = partiql_parser::lalr_parse("null");
        assert!(parse_result.is_ok());
    }

    #[test]
    fn missing_test() {
        let parse_result = partiql_parser::lalr_parse("missing");
        assert!(parse_result.is_ok());
    }

    #[test]
    fn list_test() {
        let parse_result = partiql_parser::lalr_parse("[a, 5]");
        assert!(parse_result.is_ok());
    }

    #[test]
    fn list_with_binary_operation_test() {
        let parse_result = partiql_parser::lalr_parse("[a, 5, (b + 6)]");
        assert!(parse_result.is_ok());
    }

    #[test]
    fn list_function_test() {
        let parse_result = partiql_parser::lalr_parse("LIST(a, 5)");
        assert!(parse_result.is_ok());
    }

    #[test]
    fn sexp_function_test() {
        let parse_result = partiql_parser::lalr_parse("SEXP(a, 5)");
        assert!(parse_result.is_ok());
    }

    #[test]
    fn sexp_function_with_binary_operation_test() {
        let parse_result = partiql_parser::lalr_parse("SEXP(a, 5, (b + 6))");
        assert!(parse_result.is_ok());
    }

    #[test]
    fn struct_test() {
        let parse_result = partiql_parser::lalr_parse("{'x':a, 'y':5 }");
        assert!(parse_result.is_ok());
    }

    #[test]
    fn struct_with_binary_operation_test() {
        let parse_result = partiql_parser::lalr_parse("{'x':a, 'y':5, 'z':(b + 6)}");
        assert!(parse_result.is_ok());
    }

    #[test]
    fn nested_empty_list_test() {
        let parse_result = partiql_parser::lalr_parse("[[]]");
        assert!(parse_result.is_ok());
    }

    #[test]
    fn nested_empty_bag_test() {
        let parse_result = partiql_parser::lalr_parse("<<<<>>>>");
        assert!(parse_result.is_ok());
    }

    #[test]
    fn nested_empty_struct_test() {
        let parse_result = partiql_parser::lalr_parse("{'a':{}}");
        assert!(parse_result.is_ok());
    }
}