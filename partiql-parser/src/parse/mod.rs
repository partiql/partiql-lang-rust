// Copyright Amazon.com, Inc. or its affiliates.

//! Provides the [`parse_partiql`] function to parse a PartiQL query.

use crate::lexer;
use crate::preprocessor::{built_ins, FnExprSet, PreprocessingPartiqlLexer};
use crate::result::{ParseError, ParserResult, UnexpectedTokenData};
use lalrpop_util as lpop;
use lazy_static::lazy_static;
use partiql_ast::ast;
use partiql_source_map::line_offset_tracker::LineOffsetTracker;
use partiql_source_map::location::{ByteOffset, BytePosition, LineAndColumn, ToLocated};

#[allow(clippy::just_underscores_and_digits)] // LALRPOP generates a lot of names like this
#[allow(clippy::clone_on_copy)]
#[allow(clippy::type_complexity)]
#[allow(clippy::needless_lifetimes)]
#[allow(clippy::too_many_arguments)]
#[allow(clippy::ptr_arg)]
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

lazy_static! {
    static ref BUILT_INS: FnExprSet<'static> = built_ins();
}

/// Parse PartiQL query text.
pub fn parse_partiql(s: &str) -> ParserResult {
    let mut offsets = LineOffsetTracker::default();
    let mut errors: Vec<LalrpopErrorRecovery> = vec![];
    let lexer = PreprocessingPartiqlLexer::new(s, &mut offsets, &*BUILT_INS);

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
                Ok(ast) => ast,
                _ => panic!("{:?}", res),
            }
        }};
    }

    // TODO DATE <date string>
    // TODO TIME <time string>
    // TODO TIMESTAMP <timestamp string>
    mod literals {
        use super::*;

        #[test]
        fn null() {
            parse!("NULL");
        }

        #[test]
        fn missing() {
            parse!("MISSING");
        }

        #[test]
        fn true_() {
            parse!("TRUE");
        }

        #[test]
        fn false_() {
            parse!("FALSE");
        }

        #[test]
        fn string() {
            parse!("'foo'");
            parse!("'embe''ded'");
        }

        #[test]
        fn numeric() {
            parse!("42");
            parse!("7.");
            parse!(".00125");
            parse!("5.5");
            parse!("17e2");
            parse!("1.317e-3");
            parse!("3141.59265e-03");
        }

        #[test]
        fn ion() {
            parse!(r#" `[{'a':1, 'b':1}, {'a':2}, "foo"]` "#);
            parse!(r#" `[{'a':1, 'b':1}, {'a':2}, "foo", 'a`b', "a`b", '''`s''', {{"a`b"}}]` "#);
            parse!(
                r#" `{'a':1, // comment ' "
                      'b':1} ` "#
            );
            parse!(
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

        #[test]
        fn identifier() {
            parse!("id");
            parse!(r#""quoted_id""#);
        }
        #[test]
        fn array() {
            parse!(r#"[]"#);
            parse!(r#"[1, 'moo', "some variable", [], 'a', MISSING]"#);
            // In the interest of compatibility to SQL, PartiQL also allows array constructors to be
            // denoted with parentheses instead of brackets, when there are at least two elements in the array
            parse!(r#"(1, 'moo', "some variable", [], 'a', MISSING)"#);
        }
        #[test]
        fn bag() {
            parse!(r#"<<>>"#);
            parse!(r#"<<1>>"#);
            parse!(r#"<<1,2>>"#);
            parse!(r#"<<1, <<>>, 'boo', some_variable, 'a'>>"#);
        }
        #[test]
        fn tuple() {
            parse!(r#"{}"#);
            parse!(r#"{a_variable: 1, 'cow': 'moo', 'a': NULL}"#);
        }
    }

    mod expr {
        use super::*;

        #[test]
        fn or_simple() {
            parse!(r#"TRUE OR FALSE"#);
        }

        #[test]
        fn or() {
            parse!(r#"t1.super OR test(t2.name, t1.name)"#);
        }

        #[test]
        fn and_simple() {
            parse!(r#"TRUE and FALSE"#);
        }

        #[test]
        fn and() {
            parse!(r#"test(t2.name, t1.name) AND t1.id = t2.id"#);
        }

        #[test]
        fn or_and() {
            parse!(r#"t1.super OR test(t2.name, t1.name) AND t1.id = t2.id"#);
        }

        #[test]
        fn infix() {
            parse!(r#"1 + -2 * +3 % 4^5 / 6 - 7  <= 3.14 AND 'foo' || 'bar' LIKE '%oba%'"#);
        }

        #[test]
        fn expr_in() {
            parse!(r#"a in (1,2,3,4)"#);
            parse!(r#"a in [1,2,3,4]"#);
        }

        #[test]
        fn expr_between() {
            parse!(r#"a between 2 and 3"#);
        }
    }

    mod pathexpr {
        use super::*;

        #[test]
        fn nested() {
            parse!(r#"a.b"#);
            parse!(r#"a.b.c['item']."d"[5].e['s'].f[1+2]"#);
            parse!(r#"a.b.*"#);
            parse!(r#"a.b[*]"#);
            parse!(r#"@a.b[*]"#);
            parse!(r#"@"a".b[*]"#);
            parse!(r#"tables.items[*].product.*.nest"#);
        }

        #[test]
        fn tuple() {
            parse!(r#"{'a':1 , 'data': 2}.a"#);
            parse!(r#"{'a':1 , 'data': 2}.'a'"#);
            parse!(r#"{'A':1 , 'data': 2}."A""#);
            parse!(r#"{'A':1 , 'data': 2}['a']"#);
            parse!(r#"{'attr': 1, 'b':2}[v || w]"#);
            parse!(r#"{'a':1, 'b':2}.*"#);
        }

        #[test]
        fn array() {
            parse!(r#"[1,2,3][0]"#);
            parse!(r#"[1,2,3][1 + 1]"#);
            parse!(r#"[1,2,3][*]"#);
        }

        #[test]
        fn query() {
            parse!(r#"(SELECT a FROM table).a"#);
            parse!(r#"(SELECT a FROM table).'a'"#);
            parse!(r#"(SELECT a FROM table)."a""#);
            parse!(r#"(SELECT a FROM table)['a']"#);
            parse!(r#"(SELECT a FROM table).*"#);
            parse!(r#"(SELECT a FROM table)[*]"#);
        }

        #[test]
        fn function_call() {
            parse!(r#"foo(x, y).a"#);
            parse!(r#"foo(x, y).*"#);
            parse!(r#"foo(x, y)[*]"#);
            parse!(r#"foo(x, y)[5]"#);
            parse!(r#"foo(x, y).a.*"#);
        }

        #[test]
        #[should_panic]
        fn erroneous() {
            parse!(r#"a.b.['item']"#);
            parse!(r#"a.b.{'a': 1, 'b': 2}.a"#);
            parse!(r#"a.b.[1, 2, 3][2]"#);
            parse!(r#"a.b.[*]"#);
        }
    }

    mod sfw {
        use super::*;

        #[test]
        fn selectstar() {
            parse!("SELECT *");
        }

        #[test]
        fn select1() {
            parse!("SELECT g");
        }

        #[test]
        fn select_list() {
            parse!("SELECT g, k as ck, h");
        }

        #[test]
        fn fun_call() {
            parse!(r#"fun_call('bar', 1,2,3,4,5,'foo')"#);
        }

        #[test]
        fn select3() {
            parse!("SELECT g, k, function('2') as fn_result");
        }

        #[test]
        fn group() {
            parse!("SELECT g FROM data GROUP BY a");
        }

        #[test]
        fn group_complex() {
            parse!("SELECT g FROM data GROUP BY a AS x, b + c AS y, foo(d) AS z GROUP AS g");
        }

        #[test]
        fn order_by() {
            parse!(r#"SELECT a FROM tb ORDER BY PRESERVE"#);
            parse!(r#"SELECT a FROM tb ORDER BY rk1"#);
            parse!(r#"SELECT a FROM tb ORDER BY rk1 ASC, rk2 DESC"#);
        }

        #[test]
        fn where_simple() {
            parse!(r#"SELECT a FROM tb WHERE hk = 1"#);
        }

        #[test]
        fn where_boolean() {
            parse!(
                r#"SELECT a FROM tb WHERE t1.super OR test(t2.name, t1.name) AND t1.id = t2.id"#
            );
        }

        #[test]
        fn limit() {
            parse!(r#"SELECT * FROM a LIMIT 10"#);
        }

        #[test]
        fn offset() {
            parse!(r#"SELECT * FROM a OFFSET 10"#);
        }

        #[test]
        fn limit_offset() {
            parse!(r#"SELECT * FROM a LIMIT 10 OFFSET 2"#);
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
            parse!(q);
        }

        #[test]
        fn select_with_case() {
            parse!(r#"SELECT a WHERE CASE WHEN x <> 0 THEN y/x > 1.5 ELSE false END"#);
            parse!(
                r#"SELECT a,
                    CASE WHEN a=1 THEN 'one'
                         WHEN a=2 THEN 'two'
                         ELSE 'other'
                    END
                    FROM test"#
            );

            parse!(
                r#"SELECT VALUE
                    {
                        'locationType': R.LocationType,
                        'Location': (
                            CASE WHEN id IS NOT NULL THEN
                                (SELECT VALUE (CASE WHEN R.LocationType = 'z' THEN n ELSE d END)
                                FROM R.Scope AS scope WHERE scope.name = id)
                            ELSE
                                (SELECT VALUE (CASE WHEN R.LocationType = 'z' THEN n ELSE d END)
                                FROM R.Scope AS scope WHERE scope.name = someZone)
                            END
                        ),
                        'marketType' : MarketInfo.marketType,
                    }
                    FROM UNPIVOT R.returnValueMap.success AS "list" AT symb"#
            );
        }

        #[test]
        fn select_with_cross_join_and_at() {
            parse!(r#"SELECT * FROM a AS a CROSS JOIN c AS c AT q"#);
        }

        #[test]
        fn select_with_at_and_cross_join_and_at() {
            parse!(r#"SELECT * FROM a AS a AT b CROSS JOIN c AS c AT q"#);
        }
    }

    mod set_ops {
        use super::*;

        #[test]
        fn set_ops() {
            parse!(
                r#"(SELECT * FROM a LIMIT 10 OFFSET 2) UNION SELECT * FROM b INTERSECT c EXCEPT SELECT * FROM d"#
            );
        }

        #[test]
        fn union_prec() {
            let l = parse!(r#"a union b union c"#);
            let r = parse!(r#"(a union b) union c"#);
            assert_eq!(l, r);
        }

        #[test]
        fn intersec_prec() {
            let l = parse!(r#"a union b intersect c"#);
            let r = parse!(r#"a union (b intersect c)"#);
            assert_eq!(l, r);
        }

        #[test]
        fn limit() {
            let l = parse!(r#"SELECT a FROM b UNION SELECT x FROM y ORDER BY a LIMIT 10 OFFSET 5"#);
            let r =
                parse!(r#"(SELECT a FROM b UNION SELECT x FROM y) ORDER BY a LIMIT 10 OFFSET 5"#);
            assert_eq!(l, r);
            let r2 =
                parse!(r#"SELECT a FROM b UNION (SELECT x FROM y ORDER BY a LIMIT 10 OFFSET 5)"#);
            assert_ne!(l, r2);
            assert_ne!(r, r2);
        }
    }

    mod case_expr {
        use super::*;

        #[test]
        fn searched_case() {
            parse!(r#"CASE WHEN TRUE THEN 2 END"#);
            parse!(r#"CASE WHEN id IS 1 THEN 2 WHEN titanId IS 2 THEN 3 ELSE 1 END"#);
            parse!(r#"CASE hello WHEN id IS NOT NULL THEN (SELECT * FROM data) ELSE 1 END"#);
        }

        #[test]
        #[should_panic]
        fn searched_case_failure() {
            parse!(r#"CASE hello WHEN id IS NOT NULL THEN SELECT * FROM data ELSE 1 END"#);
        }
    }

    mod nonuniform {
        use super::*;

        #[test]
        fn position() {
            parse!(r#"position('oB' in 'FooBar')"#);
        }

        #[test]
        fn substring() {
            parse!(r#"substring('FooBar' from 2 for 3)"#);
            parse!(r#"substring('FooBar' from 2)"#);
            parse!(r#"substring('FooBar' for 3)"#);
        }

        #[test]
        fn trim() {
            parse!(r#"trim(LEADING 'Foo' from 'FooBar')"#);
            parse!(r#"trim(leading from '   Bar')"#);
            parse!(r#"trim(TrAiLiNg 'Bar' from 'FooBar')"#);
            parse!(r#"trim(TRAILING from 'Bar   ')"#);
            parse!(r#"trim(BOTH 'Foo' from 'FooBarBar')"#);
            parse!(r#"trim(botH from '   Bar   ')"#);
            parse!(r#"trim(from '   Bar   ')"#);
        }

        #[test]
        fn cast() {
            parse!(r#"CAST(9 AS b)"#);
            parse!(r#"CAST(a AS VARCHAR)"#);
            parse!(r#"CAST(a AS VARCHAR(20))"#);
            parse!(r#"CAST( TRUE AS INTEGER)"#);
            parse!(r#"CAST( (4 in (1,2,3,4)) AS INTEGER)"#);
        }

        #[test]
        fn extract() {
            parse!(r#"extract(day from a)"#);
            parse!(r#"extract(hour from a)"#);
            parse!(r#"extract(minute from a)"#);
            parse!(r#"extract(second from a)"#);
        }

        #[test]
        fn agg() {
            parse!(r#"count(a)"#);
            parse!(r#"count(distinct a)"#);
            parse!(r#"count(all a)"#);
            parse!(r#"count(*)"#);
        }

        #[test]
        fn composed() {
            parse!(
                r#"cast(trim(LEADING 'Foo' from substring('BarFooBar' from 4 for 6)) AS VARCHAR(20))"#
            );
        }
    }

    mod errors {
        use super::*;
        use crate::result::{LexicalError, UnexpectedToken, UnexpectedTokenData};
        use partiql_source_map::location::{
            CharOffset, LineAndCharPosition, LineOffset, Located, Location,
        };
        use std::borrow::Cow;

        #[test]
        fn eof() {
            let res = parse_partiql(r#"SELECT"#);
            assert!(res.is_err());
            let errors = res.unwrap_err();
            assert_eq!(1, errors.len());
            assert_eq!(errors[0], ParseError::UnexpectedEndOfInput);
        }

        #[test]
        fn unterminated_ion_unicode() {
            let q = r#"/`܋"#;
            let res = parse_partiql(q);
            assert!(res.is_err());
            let errors = res.unwrap_err();
            assert_eq!(2, errors.len());
            assert_eq!(
                errors[0],
                ParseError::UnexpectedToken(UnexpectedToken {
                    inner: UnexpectedTokenData {
                        token: Cow::from("/")
                    },
                    location: Location {
                        start: LineAndCharPosition {
                            line: LineOffset(0),
                            char: CharOffset(0)
                        }
                        .into(),
                        end: LineAndCharPosition {
                            line: LineOffset(0),
                            char: CharOffset(1)
                        }
                        .into(),
                    },
                })
            );
            assert_eq!(
                errors[1],
                ParseError::LexicalError(Located {
                    inner: LexicalError::UnterminatedIonLiteral,
                    location: Location {
                        start: LineAndCharPosition {
                            line: LineOffset(0),
                            char: CharOffset(1)
                        }
                        .into(),
                        end: LineAndCharPosition {
                            line: LineOffset(0),
                            char: CharOffset(3)
                        }
                        .into(),
                    },
                })
            );
        }
    }
}
