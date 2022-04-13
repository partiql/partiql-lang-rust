// Copyright Amazon.com, Inc. or its affiliates.

//! Provides the [`parse_partiql`] function to parse a PartiQL query.

use crate::lexer;
use crate::lexer::PartiqlLexer;
use crate::result::{ParseError, ParserResult, UnexpectedTokenData};
use lalrpop_util as lpop;
use partiql_ast::experimental::ast;
use partiql_source_map::line_offset_tracker::LineOffsetTracker;
use partiql_source_map::location::{ByteOffset, BytePosition, LineAndColumn, ToLocated};

#[allow(clippy::just_underscores_and_digits)] // LALRPOP generates a lot of names like this
#[allow(clippy::clone_on_copy)]
#[allow(clippy::type_complexity)]
#[allow(clippy::needless_lifetimes)]
#[allow(clippy::too_many_arguments)]
#[allow(clippy::vec_box)]
#[allow(unused_variables)]
#[allow(dead_code)]
mod grammar {
    include!(concat!(env!("OUT_DIR"), "/partiql.rs"));
}

type LalrpopError<'input> =
    lpop::ParseError<ByteOffset, lexer::Token<'input>, ParseError<'input, BytePosition>>;
type LalrpopResult<'input> = Result<Box<ast::Expr>, LalrpopError<'input>>;
type LalrpopErrorRecovery<'input> =
    lpop::ErrorRecovery<ByteOffset, lexer::Token<'input>, ParseError<'input, BytePosition>>;

/// Parse a text PartiQL query.
pub fn parse_partiql(s: &str) -> ParserResult {
    let mut offsets = LineOffsetTracker::default();
    let mut errors: Vec<LalrpopErrorRecovery> = vec![];
    let lexer = PartiqlLexer::new(s, &mut offsets);

    let parsed: LalrpopResult = grammar::QueryParser::new().parse(s, &mut errors, lexer);

    process_errors(s, &offsets, parsed, errors)
}

fn process_errors<'input, T>(
    s: &'input str,
    offsets: &LineOffsetTracker,
    result: Result<T, LalrpopError<'input>>,
    errors: Vec<LalrpopErrorRecovery<'input>>,
) -> Result<T, Vec<ParseError<'input, LineAndColumn>>> {
    fn map_error<'input>(
        s: &'input str,
        offsets: &LineOffsetTracker,
        e: LalrpopError<'input>,
    ) -> ParseError<'input, LineAndColumn> {
        ParseError::from(e).map_loc(|byte_loc| offsets.at(s, byte_loc).unwrap().into())
    }

    let mut parser_errors: Vec<_> = errors
        .into_iter()
        // TODO do something with error_recovery.dropped_tokens?
        .map(|e| map_error(s, offsets, e.error))
        .collect();

    match (result, parser_errors.is_empty()) {
        (Ok(ast), true) => Ok(ast),
        (Ok(_), false) => Err(parser_errors),
        (Err(e), true) => Err(vec![map_error(s, offsets, e)]),
        (Err(e), false) => {
            parser_errors.push(map_error(s, offsets, e));
            Err(parser_errors)
        }
    }
}

impl<'input> From<LalrpopErrorRecovery<'input>> for ParseError<'input, BytePosition> {
    fn from(error_recovery: LalrpopErrorRecovery<'input>) -> Self {
        // TODO do something with error_recovery.dropped_tokens?
        error_recovery.error.into()
    }
}
impl<'input> From<LalrpopError<'input>> for ParseError<'input, BytePosition> {
    #[inline]
    fn from(error: LalrpopError<'input>) -> Self {
        match error {
            // TODO do something with UnrecognizedToken.expected
            lalrpop_util::ParseError::UnrecognizedToken {
                token: (start, token, end),
                expected: _,
            } => ParseError::UnexpectedToken(
                UnexpectedTokenData {
                    token: token.to_string().into(),
                }
                .to_located(start.into()..end.into()),
            ),

            lalrpop_util::ParseError::InvalidToken { location } => {
                ParseError::UnknownParseError(location.into())
            }

            // TODO do something with UnrecognizedEOF.expected
            lalrpop_util::ParseError::UnrecognizedEOF { expected: _, .. } => {
                ParseError::UnexpectedEndOfInput
            }

            lalrpop_util::ParseError::ExtraToken {
                token: (start, token, end),
            } => ParseError::UnexpectedToken(
                UnexpectedTokenData {
                    token: token.to_string().into(),
                }
                .to_located(start.into()..end.into()),
            ),

            lalrpop_util::ParseError::User { error } => error,
        }
    }
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
                let mut errors = vec![];
                let lexer = lexer::PartiqlLexer::new($q, &mut offsets);
                let res = grammar::LiteralParser::new().parse($q, &mut errors, lexer);
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
                let mut errors = vec![];
                let lexer = lexer::PartiqlLexer::new($q, &mut offsets);
                let res = grammar::ExprTermParser::new().parse($q, &mut errors, lexer);
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
    }

    mod errors {
        use super::*;
        use crate::result::{UnexpectedToken, UnexpectedTokenData};
        use partiql_source_map::location::{CharOffset, LineAndCharPosition, LineOffset, Location};
        use std::borrow::Cow;

        #[test]
        fn improper_at() {
            let res = parse_partiql(r#"SELECT * FROM a AS a CROSS JOIN c AS c AT q"#);
            assert!(res.is_err());
            let errors = res.unwrap_err();
            assert_eq!(1, errors.len());
            assert_eq!(
                "Unexpected token `AT` at `(1:40..1:42)`",
                errors[0].to_string()
            );
        }

        #[test]
        fn improper_at_multi() {
            let res = parse_partiql(r#"SELECT * FROM a AS a AT b CROSS JOIN c AS c AT q"#);
            assert!(res.is_err());
            let errors = res.unwrap_err();
            assert_eq!(2, errors.len());
            assert_eq!(
                "Unexpected token `AT` at `(1:22..1:24)`",
                errors[0].to_string()
            );
            assert_eq!(
                "Unexpected token `AT` at `(1:45..1:47)`",
                errors[1].to_string()
            );
            assert_eq!(
                errors[0],
                ParseError::UnexpectedToken(UnexpectedToken {
                    inner: UnexpectedTokenData {
                        token: Cow::from("AT")
                    },
                    location: Location {
                        start: LineAndCharPosition {
                            line: LineOffset(0),
                            char: CharOffset(21)
                        }
                        .into(),
                        end: LineAndCharPosition {
                            line: LineOffset(0),
                            char: CharOffset(23)
                        }
                        .into(),
                    },
                })
            );
            assert_eq!(
                errors[1],
                ParseError::UnexpectedToken(UnexpectedToken {
                    inner: UnexpectedTokenData {
                        token: Cow::from("AT")
                    },
                    location: Location {
                        start: LineAndCharPosition {
                            line: LineOffset(0),
                            char: CharOffset(44)
                        }
                        .into(),
                        end: LineAndCharPosition {
                            line: LineOffset(0),
                            char: CharOffset(46)
                        }
                        .into(),
                    },
                })
            );
        }

        #[test]
        fn eof() {
            let res = parse_partiql(r#"SELECT"#);
            assert!(res.is_err());
            let errors = res.unwrap_err();
            assert_eq!(1, errors.len());
            assert_eq!(errors[0], ParseError::UnexpectedEndOfInput);
        }
    }
}
