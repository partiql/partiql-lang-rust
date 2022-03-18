// Copyright Amazon.com, Inc. or its affiliates.

//! Provides a parser for the [PartiQL][partiql] query language.
//!
//! # Usage
//!
//! ```
//! use partiql_parser::prelude::*;
//! use partiql_parser::peg_parse;
//!
//!     peg_parse("SELECT g FROM data GROUP BY a").expect("successful parse");
//! ```
//!
//! [partiql]: https://partiql.org
//!
use pest::iterators::Pairs;
use pest::Parser;

use crate::result::ParserResult;

mod ast_builder;
mod grammar;

pub(crate) use crate::peg::ast_builder::{build_literal, build_query};
pub(crate) use crate::peg::grammar::{PartiQLParser, Rule};
use partiql_ast::experimental::ast;

/// Parser for PartiQL queries.
///
/// Returns `Ok([Pairs<Rule>])` in the case that the input is valid PartiQL.  
/// Returns `Err([ParserError])` in the case that the input is not valid PartiQL.
pub fn parse_partiql(input: &str) -> ParserResult<Pairs<Rule>> {
    Ok(PartiQLParser::parse(Rule::query_full, input)?)
}

pub fn parse_partiql_to_ast(input: &str) -> ParserResult<Box<ast::Expr>> {
    build_query(parse_partiql(input)?)
}

#[cfg(test)]
mod tests {
    use super::*;

    macro_rules! parse {
        ($q:expr) => {{
            let res = parse_partiql($q);
            println!("{:#?}", res);
            match res {
                Ok(_) => (),
                _ => assert!(false, "{:?}", res),
            }
        }};
    }
    macro_rules! parse_to_ast {
        ($q:expr) => {{
            let res = build_query(parse_partiql($q).expect("parse"));
            println!("{:#?}", res);
            match res {
                Ok(_) => (),
                _ => assert!(false, "{:?}", res),
            }
        }};
    }

    mod literals {
        use super::*;

        macro_rules! literal {
            ($q:expr) => {{
                let res = PartiQLParser::parse(Rule::literal, $q);
                println!("{:#?}", res);
                match res {
                    Ok(_) => (),
                    _ => assert!(false, "{:?}", res),
                }
            }};
        }
        macro_rules! lit_and_parse {
            ($q:expr) => {{
                literal!($q);
                parse!($q);
                parse_to_ast!($q);
            }};
        }

        #[test]
        fn null() {
            lit_and_parse!("NULL")
        }
        #[test]
        fn missing() {
            lit_and_parse!("MISSING")
        }
        #[test]
        fn true_() {
            lit_and_parse!("TRUE")
        }
        #[test]
        fn false_() {
            lit_and_parse!("FALSE")
        }
        #[test]
        fn string() {
            lit_and_parse!("'foo'");
            lit_and_parse!("'embe''ded'");
        }
        #[test]
        fn numeric() {
            lit_and_parse!("42");
            lit_and_parse!("7.");
            lit_and_parse!(".00125");
            lit_and_parse!("5.5");
            lit_and_parse!("17e2");
            lit_and_parse!("1.317e-3");
            lit_and_parse!("3141.59265e-03");
        }
        #[test]
        fn array() {
            lit_and_parse!(r#"[]"#);
            lit_and_parse!(r#"[1, 'moo', [], 'a', MISSING]"#);
        }
        #[test]
        fn bag() {
            lit_and_parse!(r#"<<>>"#);
            lit_and_parse!(r#"<<1>>"#);
            lit_and_parse!(r#"<<1,2>>"#);
            lit_and_parse!(r#"<<1, <<>>, 'boo', 'a'>>"#);
        }
        #[test]
        fn tuple() {
            lit_and_parse!(r#"{}"#);
            lit_and_parse!(r#"{'str': 1, 'cow': 'moo', 'a': NULL}"#);
        }
    }

    mod non_literal_values {
        use super::*;

        macro_rules! value {
            ($q:expr) => {{
                let res = PartiQLParser::parse(Rule::expr_term, $q);
                println!("{:#?}", res);
                match res {
                    Ok(_) => (),
                    _ => assert!(false, "{:?}", res),
                }
            }};
        }
        macro_rules! value_and_parse {
            ($q:expr) => {{
                value!($q);
                parse!($q);
                parse_to_ast!($q);
            }};
        }
        #[test]
        fn identifier() {
            value_and_parse!("id");
            value_and_parse!(r#""quoted_id""#);
        }
        #[test]
        fn array() {
            value_and_parse!(r#"[]"#);
            value_and_parse!(r#"[1, 'moo', "some variable", [], 'a', MISSING]"#);
        }
        #[test]
        fn bag() {
            value_and_parse!(r#"<<>>"#);
            value_and_parse!(r#"<<1>>"#);
            value_and_parse!(r#"<<1,2>>"#);
            value_and_parse!(r#"<<1, <<>>, 'boo', some_variable, 'a'>>"#);
        }
        #[test]
        fn tuple() {
            value_and_parse!(r#"{}"#);
            value_and_parse!(r#"{a_variable: 1, 'cow': 'moo', 'a': NULL}"#);
        }
    }

    mod expr {
        use super::*;

        macro_rules! parse_and_ast {
            ($q:expr) => {{
                parse!($q);
                parse_to_ast!($q);
            }};
        }

        #[test]
        fn or_simple() {
            parse_and_ast!(r#"TRUE OR FALSE"#)
        }

        #[test]
        fn or() {
            parse_and_ast!(r#"t1.super OR test(t2.name, t1.name)"#)
        }

        #[test]
        fn and_simple() {
            parse_and_ast!(r#"TRUE and FALSE"#)
        }

        #[test]
        fn and() {
            parse_and_ast!(r#"test(t2.name, t1.name) AND t1.id = t2.id"#)
        }

        #[test]
        fn or_and() {
            parse_and_ast!(r#"t1.super OR test(t2.name, t1.name) AND t1.id = t2.id"#)
        }
    }

    mod sfw {
        use super::*;

        macro_rules! parse_and_ast {
            ($q:expr) => {{
                parse!($q);
                parse_to_ast!($q);
            }};
        }

        #[test]
        fn select1() {
            parse_and_ast!("SELECT g")
        }

        #[test]
        fn select_list() {
            parse_and_ast!("SELECT g, k as ck, h")
        }

        #[test]
        fn fun_call() {
            parse_and_ast!(r#"fun_call('bar', 1,2,3,4,5,'foo')"#)
        }

        #[test]
        fn select3() {
            parse_and_ast!("SELECT g, k, function('2') as fn_result")
        }

        #[test]
        fn group() {
            parse_and_ast!("SELECTg FROM data GROUP BY a")
        }

        #[test]
        fn group_complex() {
            parse_and_ast!("SELECT g FROM data GROUP BY a AS x, b + c AS y, foo(d) AS z GROUP AS g")
        }

        #[test]
        fn order_by() {
            parse_and_ast!(r#"SELECT a FROM tb ORDER BY PRESERVE"#);
            parse_and_ast!(r#"SELECT a FROM tb ORDER BY rk1"#);
            parse_and_ast!(r#"SELECT a FROM tb ORDER BY rk1 ASC, rk2 DESC"#);
        }

        #[test]
        fn where_simple() {
            parse_and_ast!(r#"SELECT a FROM tb WHERE hk = 1"#)
        }

        #[test]
        fn where_boolean() {
            parse_and_ast!(
                r#"SELECT a FROM tb WHERE t1.super OR test(t2.name, t1.name) AND t1.id = t2.id"#
            )
        }

        #[test]
        fn limit() {
            parse_and_ast!(r#"SELECT * FROM a LIMIT 10"#)
        }

        #[test]
        fn offset() {
            parse_and_ast!(r#"SELECT * FROM a OFFSET 10"#)
        }

        #[test]
        fn limit_offset() {
            parse_and_ast!(r#"SELECT * FROM a LIMIT 10 OFFSET 2"#)
        }

        #[test]
        fn complex() {
            let q = r#"
            SELECT (
                SELECT numRec, data
                FROM delta_full_transactions.deltas delta,
                (
                    SELECT u.id, review, rindex
                    FROM delta.data as u CROSS JOIN UNPIVOT u.reviews as review AT rindex
                ) as data,
                delta.numRec as numRec
            )
            AS deltas FROM SOURCE_VIEW_DELTA_FULL_TRANSACTIONS delta_full_transactions
            "#;
            parse_and_ast!(q)
        }
    }
}
