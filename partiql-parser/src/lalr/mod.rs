// Copyright Amazon.com, Inc. or its affiliates.

//! Provides a parser for the [PartiQL][partiql] query language.
//!
//! # Usage
//!
//! ```
//! use partiql_parser::{LalrParseResult, lalr_parse};
//!
//!     lalr_parse("SELECT g FROM data GROUP BY a").expect("successful parse");
//! ```
//!
//! [partiql]: https://partiql.org
use crate::lalr::lexer::{LexicalToken, PartiqlLexer};
use lalrpop_util::ParseError;
use partiql_ast::experimental::ast;

#[allow(clippy::just_underscores_and_digits)] // LALRPOP generates a lot of names like this
#[allow(clippy::clone_on_copy)]
#[allow(clippy::type_complexity)]
#[allow(clippy::too_many_arguments)]
#[allow(clippy::vec_box)]
#[allow(unused_variables)]
#[allow(dead_code)]
mod grammar {
    include!(concat!(env!("OUT_DIR"), "/partiql.rs"));
}

mod lexer;
mod util;

pub use lexer::LexicalError;
pub use lexer::LineOffsetTracker;

pub type ParseResult =
    Result<Box<ast::Expr>, ParseError<usize, lexer::Token, (usize, lexer::LexicalError, usize)>>;

/// Parse a text PartiQL query.
pub fn parse_partiql(s: &str) -> ParseResult {
    let mut offsets = LineOffsetTracker::default();
    let lexer = PartiqlLexer::new(s, &mut offsets);
    grammar::QueryParser::new().parse(lexer)
}

/// Lex a text PartiQL query.
// TODO make private
#[deprecated(note = "prototypical lexer implementation")]
pub fn lex_partiql(s: &str) -> Vec<LexicalToken> {
    let mut counter = LineOffsetTracker::default();
    PartiqlLexer::new(s, &mut counter).collect()
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

    mod literals {
        use super::*;

        macro_rules! literal {
            ($q:expr) => {{
                let mut offsets = LineOffsetTracker::default();
                let lexer = lexer::PartiqlLexer::new($q, &mut offsets);
                let res = grammar::LiteralParser::new().parse(lexer);
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
        fn ion() {
            lit_and_parse!(r#" `[{'a':1, 'b':1}, {'a':2}, "foo"]` "#);
            lit_and_parse!(
                r#" `[{'a':1, 'b':1}, {'a':2}, "foo", 'a`b', "a`b", '''`s''', {{"a`b"}}]` "#
            );
            lit_and_parse!(
                r#" `{'a':1, // comment ' "
                      'b':1} ` "#
            );
            lit_and_parse!(
                r#" `{'a' // comment ' "
                       :1, /* 
                               comment 
                              */
                      'b':1} ` "#
            );
        }
    }

    mod non_literal_values {
        use super::*;

        macro_rules! value {
            ($q:expr) => {{
                let mut offsets = LineOffsetTracker::default();
                let lexer = lexer::PartiqlLexer::new($q, &mut offsets);
                let res = grammar::ExprTermParser::new().parse(lexer);
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

        #[test]
        fn or_simple() {
            parse!(r#"TRUE OR FALSE"#)
        }

        #[test]
        fn or() {
            parse!(r#"t1.super OR test(t2.name, t1.name)"#)
        }

        #[test]
        fn and_simple() {
            parse!(r#"TRUE and FALSE"#)
        }

        #[test]
        fn and() {
            parse!(r#"test(t2.name, t1.name) AND t1.id = t2.id"#)
        }

        #[test]
        fn or_and() {
            parse!(r#"t1.super OR test(t2.name, t1.name) AND t1.id = t2.id"#)
        }
    }

    mod sfw {
        use super::*;
        use crate::lalr::lexer::Token;

        #[test]
        fn selectstar() {
            parse!("SELECT *")
        }

        #[test]
        fn select1() {
            parse!("SELECT g")
        }

        #[test]
        fn select_list() {
            parse!("SELECT g, k as ck, h")
        }

        #[test]
        fn fun_call() {
            parse!(r#"fun_call('bar', 1,2,3,4,5,'foo')"#)
        }

        #[test]
        fn select3() {
            parse!("SELECT g, k, function('2') as fn_result")
        }

        #[test]
        fn group() {
            parse!("SELECT g FROM data GROUP BY a")
        }

        #[test]
        fn group_complex() {
            parse!("SELECT g FROM data GROUP BY a AS x, b + c AS y, foo(d) AS z GROUP AS g")
        }

        #[test]
        fn order_by() {
            parse!(r#"SELECT a FROM tb ORDER BY PRESERVE"#);
            parse!(r#"SELECT a FROM tb ORDER BY rk1"#);
            parse!(r#"SELECT a FROM tb ORDER BY rk1 ASC, rk2 DESC"#);
        }

        #[test]
        fn where_simple() {
            parse!(r#"SELECT a FROM tb WHERE hk = 1"#)
        }

        #[test]
        fn where_boolean() {
            parse!(r#"SELECT a FROM tb WHERE t1.super OR test(t2.name, t1.name) AND t1.id = t2.id"#)
        }

        #[test]
        fn limit() {
            parse!(r#"SELECT * FROM a LIMIT 10"#)
        }

        #[test]
        fn offset() {
            parse!(r#"SELECT * FROM a OFFSET 10"#)
        }

        #[test]
        fn limit_offset() {
            parse!(r#"SELECT * FROM a LIMIT 10 OFFSET 2"#)
        }

        #[test]
        fn complex() {
            let q = r#"
            SELECT (
                SELECT numRec, data
                FROM delta_full_transactions.deltas delta0,
                (
                    SELECT u.id, review, rindex
                    FROM delta1.data as u CROSS JOIN UNPIVOT u.reviews as review AT rindex
                ) as data,
                delta2.numRec as numRec
            )
            AS deltas FROM SOURCE_VIEW_DELTA_FULL_TRANSACTIONS delta_full_transactions
            "#;
            parse!(q)
        }

        #[test]
        fn improper_at() {
            let res = parse_partiql(r#"SELECT * FROM a AS a AT b"#);
            assert!(res.is_err());
            let error = res.unwrap_err();
            assert!(matches!(
                error,
                lalrpop_util::ParseError::UnrecognizedToken {
                    token: (21, Token::At, 23),
                    ..
                }
            ));
        }
    }
}
