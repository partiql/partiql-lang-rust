// Copyright Amazon.com, Inc. or its affiliates.

//! Provides the [`parse_partiql`] function to parse a `PartiQL` query.

mod parse_util;
mod parser_state;

use crate::error::{ParseError, UnexpectedTokenData};
use crate::lexer;
use crate::lexer::CommentSkippingLexer;
use crate::parse::parser_state::ParserState;
use crate::preprocessor::{PreprocessingPartiqlLexer, BUILT_INS};
use lalrpop_util as lpop;
use partiql_ast::ast;
use partiql_common::node::NodeIdGenerator;
use partiql_common::syntax::line_offset_tracker::LineOffsetTracker;
use partiql_common::syntax::location::{ByteOffset, BytePosition, ToLocated};
use partiql_common::syntax::metadata::LocationMap;

#[allow(clippy::just_underscores_and_digits)] // LALRPOP generates a lot of names like this
#[allow(clippy::all)]
#[allow(clippy::pedantic)]
#[allow(unused_variables)]
#[allow(dead_code)]
#[allow(unused_extern_crates)]
#[allow(explicit_outlives_requirements)]
mod grammar {
    include!(concat!(env!("OUT_DIR"), "/partiql.rs"));
}

type LalrpopError<'input> =
    lpop::ParseError<ByteOffset, lexer::Token<'input>, ParseError<'input, BytePosition>>;
type LalrpopResult<'input> = Result<ast::AstNode<ast::TopLevelQuery>, LalrpopError<'input>>;
type LalrpopErrorRecovery<'input> =
    lpop::ErrorRecovery<ByteOffset, lexer::Token<'input>, ParseError<'input, BytePosition>>;

#[derive(Debug, Clone)]
pub(crate) struct AstData {
    pub ast: ast::AstNode<ast::TopLevelQuery>,
    pub locations: LocationMap,
    pub offsets: LineOffsetTracker,
}

#[derive(Debug, Clone)]
pub(crate) struct ErrorData<'input> {
    pub errors: Vec<ParseError<'input, BytePosition>>,
    pub offsets: LineOffsetTracker,
}

pub(crate) type AstResult<'input> = Result<AstData, ErrorData<'input>>;

/// Parse `PartiQL` query text into an AST.
pub(crate) fn parse_partiql(s: &str) -> AstResult<'_> {
    parse_partiql_with_state(s, ParserState::default())
}

fn parse_partiql_with_state<'input, Id: NodeIdGenerator>(
    s: &'input str,
    mut state: ParserState<'input, Id>,
) -> AstResult<'input> {
    let mut offsets = LineOffsetTracker::default();
    let lexer = PreprocessingPartiqlLexer::new(s, &mut offsets, &BUILT_INS);
    let lexer = CommentSkippingLexer::new(lexer);

    let result: LalrpopResult<'_> = grammar::TopLevelQueryParser::new().parse(s, &mut state, lexer);

    let ParserState {
        locations, errors, ..
    } = state;

    let mut errors: Vec<_> = errors
        .into_iter()
        // TODO do something with error_recovery.dropped_tokens?
        .map(|e| ParseError::from(e.error))
        .collect();

    match (result, errors.is_empty()) {
        (Ok(_), false) => Err(ErrorData { errors, offsets }),
        (Err(e), true) => {
            let errors = vec![ParseError::from(e)];
            Err(ErrorData { errors, offsets })
        }
        (Err(e), false) => {
            errors.push(ParseError::from(e));
            Err(ErrorData { errors, offsets })
        }
        (Ok(ast), true) => Ok(AstData {
            ast,
            locations,
            offsets,
        }),
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
                ParseError::Unknown(location.into())
            }

            // TODO do something with UnrecognizedEof.expected
            lalrpop_util::ParseError::UnrecognizedEof {
                location,
                expected: _,
            } => ParseError::UnexpectedEndOfInput(location.into()),

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
    fn parse_partiql(s: &str) -> AstResult<'_> {
        super::parse_partiql(s)
    }

    macro_rules! parse {
        ($q:expr) => {{
            let res = parse_partiql($q);
            println!("{:#?}", res);
            match res {
                Ok(data) => data.ast,
                _ => panic!("{:?}", res),
            }
        }};
    }

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
        fn time() {
            parse!("time '22:12'");
            parse!("time(10) '22:12'");
            parse!("time WITH TIME ZONE '22:12'");
            parse!("time WITHOUT TIME ZONE '22:12'");
            parse!("time(10) WITH TIME ZONE '22:12'");
            parse!("time(10) WITHOUT TIME ZONE '22:12'");
            parse!("time (10) WITH TIME ZONE '22:12'");
            parse!("time (10) WITHOUT TIME ZONE '22:12'");
        }

        #[test]
        fn ion() {
            parse!(r#" `[{'a':1, 'b':1}, {'a':2}, "foo"]` "#);
            parse!(
                r#" ```[{'a':1, 'b':1}, {'a':2}, "foo", 'a`b', "a`b", '''`s''', {{"a`b"}}]``` "#
            );
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
            parse!(r"[]");
            parse!(r#"[1, 'moo', "some variable", [], 'a', MISSING]"#);
            // In the interest of compatibility to SQL, PartiQL also allows array constructors to be
            // denoted with parentheses instead of brackets, when there are at least two elements in the array
            parse!(r#"(1, 'moo', "some variable", [], 'a', MISSING)"#);
        }
        #[test]
        fn bag() {
            parse!(r"<<>>");
            parse!(r"<<1>>");
            parse!(r"<<1,2>>");
            parse!(r"<<1, <<>>, 'boo', some_variable, 'a'>>");
        }
        #[test]
        fn tuple() {
            parse!(r"{}");
            parse!(r"{a_variable: 1, 'cow': 'moo', 'a': NULL}");
        }
    }

    mod expr {
        use super::*;

        #[test]
        fn or_simple() {
            parse!(r"TRUE OR FALSE");
        }

        #[test]
        fn or() {
            parse!(r"t1.super OR test(t2.name, t1.name)");
        }

        #[test]
        fn and_simple() {
            parse!(r"TRUE and FALSE");
        }

        #[test]
        fn and() {
            parse!(r"test(t2.name, t1.name) AND t1.id = t2.id");
        }

        #[test]
        fn or_and() {
            parse!(r"t1.super OR test(t2.name, t1.name) AND t1.id = t2.id");
        }

        #[test]
        fn infix() {
            parse!(r"1 + -2 * +3 % 4^5 / 6 - 7  <= 3.14 AND 'foo' || 'bar' LIKE '%oba%'");
        }

        #[test]
        fn expr_in() {
            parse!(r"a in (1,2,3,4)");
            parse!(r"a in [1,2,3,4]");
        }

        #[test]
        fn expr_between() {
            parse!(r"a between 2 and 3");
        }
    }

    mod pathexpr {
        use super::*;

        #[test]
        fn nested() {
            parse!(r"a.b");
            parse!(r#"a.b.c['item']."d"[5].e['s'].f[1+2]"#);
            parse!(r"a.b.*");
            parse!(r"a.b[*]");
            parse!(r"@a.b[*]");
            parse!(r#"@"a".b[*]"#);
            parse!(r"tables.items[*].product.*.nest");
            parse!(r#"a.b.c['item']."d"[5].e['s'].f[1+2]"#);
        }

        #[test]
        fn tuple() {
            parse!(r"{'a':1 , 'data': 2}.a");
            parse!(r"{'a':1 , 'data': 2}.'a'");
            parse!(r#"{'A':1 , 'data': 2}."A""#);
            parse!(r"{'A':1 , 'data': 2}['a']");
            parse!(r"{'attr': 1, 'b':2}[v || w]");
            parse!(r"{'a':1, 'b':2}.*");
        }

        #[test]
        fn array() {
            parse!(r"[1,2,3][0]");
            parse!(r"[1,2,3][1 + 1]");
            parse!(r"[1,2,3][*]");
        }

        #[test]
        fn query() {
            parse!(r"(SELECT a FROM t).a");
            parse!(r"(SELECT a FROM t).'a'");
            parse!(r#"(SELECT a FROM t)."a""#);
            parse!(r"(SELECT a FROM t)['a']");
            parse!(r"(SELECT a FROM t).*");
            parse!(r"(SELECT a FROM t)[*]");
        }

        #[test]
        fn function_call() {
            parse!(r"foo(x, y).a");
            parse!(r"foo(x, y).*");
            parse!(r"foo(x, y)[*]");
            parse!(r"foo(x, y)[5]");
            parse!(r"foo(x, y).a.*");
            parse!(r"foo(x, y)[*].*.b[5]");
        }

        #[test]
        fn test_pathexpr_struct() {
            let res = parse!(r#"a.b.c['item']."d"[5].e['s'].f[1+2]"#);

            if let ast::AstNode {
                node:
                    ast::TopLevelQuery {
                        query:
                            ast::AstNode {
                                node:
                                    ast::Query {
                                        set:
                                            ast::AstNode {
                                                node: ast::QuerySet::Expr(ref e),
                                                ..
                                            },
                                        ..
                                    },
                                ..
                            },
                        ..
                    },
                ..
            } = res
            {
                if let ast::Expr::Path(p) = &**e {
                    assert_eq!(9, p.node.steps.len());
                } else {
                    panic!("PathExpr test failed!");
                }
            } else {
                panic!("PathExpr test failed!");
            }
        }

        #[test]
        #[should_panic]
        fn erroneous() {
            parse!(r"a.b.['item']");
            parse!(r"a.b.{'a': 1, 'b': 2}.a");
            parse!(r"a.b.[1, 2, 3][2]");
            parse!(r"a.b.[*]");
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
            parse!(r"fun_call('bar', 1,2,3,4,5,'foo')");
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
            parse!(r"SELECT a FROM tb ORDER BY PRESERVE");
            parse!(r"SELECT a FROM tb ORDER BY rk1");
            parse!(r"SELECT a FROM tb ORDER BY rk1 ASC, rk2 DESC");
        }

        #[test]
        fn where_simple() {
            parse!(r"SELECT a FROM tb WHERE hk = 1");
        }

        #[test]
        fn where_boolean() {
            parse!(r"SELECT a FROM tb WHERE t1.super OR test(t2.name, t1.name) AND t1.id = t2.id");
        }

        #[test]
        fn limit() {
            parse!(r"SELECT * FROM a LIMIT 10");
        }

        #[test]
        fn offset() {
            parse!(r"SELECT * FROM a OFFSET 10");
        }

        #[test]
        fn limit_offset() {
            parse!(r"SELECT * FROM a LIMIT 10 OFFSET 2");
        }

        #[test]
        fn complex() {
            let q = r"
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
            ";
            parse!(q);
        }

        #[test]
        fn select_with_case() {
            parse!(r"SELECT a WHERE CASE WHEN x <> 0 THEN y/x > 1.5 ELSE false END");
            parse!(
                r"SELECT a,
                    CASE WHEN a=1 THEN 'one'
                         WHEN a=2 THEN 'two'
                         ELSE 'other'
                    END
                    FROM test"
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
            parse!(r"SELECT * FROM a AS a CROSS JOIN c AS c AT q");
        }

        #[test]
        fn select_with_at_and_cross_join_and_at() {
            parse!(r"SELECT * FROM a AS a AT b CROSS JOIN c AS c AT q");
        }

        #[test]
        fn multiline_with_comments() {
            parse!(
                r"SELECT * FROM hr.employees               -- T1
                                  UNION
                                  SELECT title FROM engineering.employees  -- T2"
            );
        }
    }

    mod set_ops {
        use super::*;
        use partiql_common::node::NullIdGenerator;

        impl<'input> ParserState<'input, NullIdGenerator> {
            pub(crate) fn new_null_id() -> ParserState<'input, NullIdGenerator> {
                ParserState::with_id_gen(NullIdGenerator::default())
            }
        }

        fn parse_partiql_null_id(s: &str) -> AstResult<'_> {
            super::parse_partiql_with_state(s, ParserState::new_null_id())
        }

        // parse partiql query with all AST nodes having an id of `0` for ease of comparison regardless
        //   of parse order
        macro_rules! parse_null_id {
            ($q:expr) => {{
                let res = parse_partiql_null_id($q);
                println!("{:#?}", res);
                match res {
                    Ok(data) => data.ast,
                    _ => panic!("{:?}", res),
                }
            }};
        }

        #[test]
        fn set_ops() {
            parse!(
                r"(SELECT * FROM a LIMIT 10 OFFSET 2) UNION SELECT * FROM b INTERSECT c EXCEPT SELECT * FROM d"
            );
        }

        #[test]
        fn union_prec() {
            let l = parse_null_id!(r"a union b union c");
            let r = parse_null_id!(r"(a union b) union c");
            assert_eq!(l, r);
        }

        // TODO: will need to fix set op precedence after parse (parser was changed to support `OUTER UNION/INTERSECT/EXCEPT`
        #[test]
        #[ignore]
        fn intersec_prec() {
            let l = parse_null_id!(r"a union b intersect c");
            let r = parse_null_id!(r"a union (b intersect c)");
            assert_eq!(l, r);
        }

        #[test]
        fn limit() {
            let l = parse_null_id!(
                r"SELECT a FROM b UNION SELECT x FROM y ORDER BY a LIMIT 10 OFFSET 5"
            );
            let r = parse_null_id!(
                r"(SELECT a FROM b UNION SELECT x FROM y) ORDER BY a LIMIT 10 OFFSET 5"
            );
            assert_eq!(l, r);
            let r2 = parse_null_id!(
                r"SELECT a FROM b UNION (SELECT x FROM y ORDER BY a LIMIT 10 OFFSET 5)"
            );
            assert_ne!(l, r2);
            assert_ne!(r, r2);
        }

        #[test]
        fn complex_set() {
            parse_null_id!(
                r"(SELECT a1 FROM b1 ORDER BY c1 LIMIT d1 OFFSET e1)
                   OUTER UNION ALL
                   (SELECT a2 FROM b2 ORDER BY c2 LIMIT d2 OFFSET e2)
                   ORDER BY c3 LIMIT d3 OFFSET e3"
            );
            parse_null_id!(
                r"(SELECT a1 FROM b1 ORDER BY c1 LIMIT d1 OFFSET e1)
                   OUTER INTERSECT ALL
                   (SELECT a2 FROM b2 ORDER BY c2 LIMIT d2 OFFSET e2)
                   ORDER BY c3 LIMIT d3 OFFSET e3"
            );
            parse_null_id!(
                r"(SELECT a1 FROM b1 ORDER BY c1 LIMIT d1 OFFSET e1)
                   OUTER EXCEPT ALL
                   (SELECT a2 FROM b2 ORDER BY c2 LIMIT d2 OFFSET e2)
                   ORDER BY c3 LIMIT d3 OFFSET e3"
            );
            parse_null_id!(
                r"(
                       (SELECT a1 FROM b1 ORDER BY c1 LIMIT d1 OFFSET e1)
                       UNION DISTINCT
                       (SELECT a2 FROM b2 ORDER BY c2 LIMIT d2 OFFSET e2)
                   )
                   OUTER UNION ALL
                   (SELECT a3 FROM b3 ORDER BY c3 LIMIT d3 OFFSET e3)
                   ORDER BY c4 LIMIT d4 OFFSET e4"
            );
        }
    }

    mod case_expr {
        use super::*;

        #[test]
        fn searched_case() {
            parse!(r"CASE WHEN TRUE THEN 2 END");
            parse!(r"CASE WHEN id IS 1 THEN 2 WHEN titanId IS 2 THEN 3 ELSE 1 END");
            parse!(r"CASE hello WHEN id IS NOT NULL THEN (SELECT * FROM data) ELSE 1 END");
        }

        #[test]
        #[should_panic]
        fn searched_case_failure() {
            parse!(r"CASE hello WHEN id IS NOT NULL THEN SELECT * FROM data ELSE 1 END");
        }
    }

    mod nonuniform {
        use super::*;

        #[test]
        fn position() {
            parse!(r"position('oB' in 'FooBar')");
        }

        #[test]
        fn substring() {
            parse!(r"substring('FooBar' from 2 for 3)");
            parse!(r"substring('FooBar' from 2)");
            parse!(r"substring('FooBar' for 3)");
        }

        #[test]
        fn trim() {
            parse!(r"trim(LEADING 'Foo' from 'FooBar')");
            parse!(r"trim(leading from '   Bar')");
            parse!(r"trim(TrAiLiNg 'Bar' from 'FooBar')");
            parse!(r"trim(TRAILING from 'Bar   ')");
            parse!(r"trim(BOTH 'Foo' from 'FooBarBar')");
            parse!(r"trim(botH from '   Bar   ')");
            parse!(r"trim(from '   Bar   ')");
        }

        #[test]
        fn cast() {
            parse!(r"CAST(9 AS b)");
            parse!(r"CAST(a AS VARCHAR)");
            parse!(r"CAST(a AS VARCHAR(20))");
            parse!(r"CAST(a AS TIME)");
            parse!(r"CAST(a AS TIME(20))");
            parse!(r"CAST( TRUE AS INTEGER)");
            parse!(r"CAST( (4 in (1,2,3,4)) AS INTEGER)");
            parse!(r"CAST(a AS TIME WITH TIME ZONE)");
            parse!(r"CAST(a AS TIME WITH TIME ZONE)");
            parse!(r"CAST(a AS TIME(20) WITH TIME ZONE)");
        }

        #[test]
        fn extract() {
            parse!(r"extract(day from a)");
            parse!(r"extract(hour from a)");
            parse!(r"extract(minute from a)");
            parse!(r"extract(second from a)");
        }

        #[test]
        fn agg() {
            parse!(r"count(a)");
            parse!(r"count(distinct a)");
            parse!(r"count(all a)");
            parse!(r"count(*)");
        }

        #[test]
        fn composed() {
            parse!(
                r"cast(trim(LEADING 'Foo' from substring('BarFooBar' from 4 for 6)) AS VARCHAR(20))"
            );
        }
    }

    /// In the future, the following identifiers may be converted into reserved keywords. In that case,
    /// the following tests will need to be modified.
    mod non_reserved_keywords {
        use super::*;

        #[test]
        fn projection_list_trim_spec() {
            parse!(r"SELECT leading FROM t");
            parse!(r"SELECT leading, a FROM t");
            parse!(r"SELECT leading + trailing, b FROM t");
            parse!(r"SELECT both + leading + trailing, a, b, c FROM t");
        }

        #[test]
        fn from_source_trim_spec() {
            parse!(r"SELECT leading, trailing, both FROM leading, trailing, both");
        }

        #[test]
        fn complex_trim() {
            parse!(
                r"SELECT leading + trim(leading leading FROM '  hello world'), both FROM leading, trailing, both"
            );
        }

        #[test]
        fn user_public_domain() {
            parse!(r"SELECT user, puBlIC, DOMAIN FROM USER, pUbLIc, domain");
            parse!(r"USER");
            parse!(r"pUbLIC");
            parse!(r"domain");
        }
    }

    mod graph {
        use super::*;

        #[test]
        fn no_labels() {
            parse!(r#"SELECT 1 FROM GRAPH_TABLE (my_graph MATCH ())"#);

            parse!(r#"SELECT 1 FROM GRAPH_TABLE (my_graph MATCH ()) WHERE contains_value('1')"#);

            parse!(
                r#"SELECT x.info AS info FROM GRAPH_TABLE (my_graph MATCH (x)) WHERE x.name LIKE 'foo'"#
            );
            parse!(r#"SELECT 1 FROM GRAPH_TABLE (g MATCH -[]->) "#);
        }

        #[test]
        fn lone_match_expr() {
            parse!(r#"(MyGraph MATCH (x))"#);
            parse!(r#"(MyGraph MATCH (x), (y) )"#);
            parse!(r#"(MyGraph MATCH (x), -[u]-> )"#);
        }

        #[test]
        fn labelled_nodes() {
            parse!(r#"SELECT x AS target FROM (my_graph MATCH (x:Label) WHERE x.has_data = true)"#);
        }

        #[test]
        fn edges() {
            parse!(r#"SELECT a,b FROM (g MATCH (a:A) -[e:E]-> (b:B))"#);
            parse!(r#"SELECT a,b FROM (g MATCH (a:A) -> (b:B))"#);
            parse!(r#"SELECT a,b FROM (g MATCH (a:A) ~[e:E]~ (b:B))"#);
            parse!(r#"SELECT a,b FROM (g MATCH (a:A) ~ (b:B))"#);
            parse!(r#"SELECT a,b FROM (g MATCH (a:A) <-[e:E]- (b:B))"#);
            parse!(r#"SELECT a,b FROM (g MATCH (a:A) <- (b:B))"#);
            parse!(r#"SELECT a,b FROM (g MATCH (a:A) ~[e:E]~> (b:B))"#);
            parse!(r#"SELECT a,b FROM (g MATCH (a:A) ~> (b:B))"#);
            parse!(r#"SELECT a,b FROM (g MATCH (a:A) <~[e:E]~ (b:B))"#);
            parse!(r#"SELECT a,b FROM (g MATCH (a:A) <~ (b:B))"#);
            parse!(r#"SELECT a,b FROM (g MATCH (a:A) <-[e:E]-> (b:B))"#);
            parse!(r#"SELECT a,b FROM (g MATCH (a:A) <-> (b:B))"#);
            parse!(r#"SELECT a,b FROM (g MATCH (a:A) -[e:E]- (b:B))"#);
            parse!(r#"SELECT a,b FROM (g MATCH (a:A) - (b:B))"#);
        }

        #[test]
        fn quantifiers() {
            parse!(r#"SELECT a,b FROM (g MATCH (a:A)-[:edge]->*(b:B))"#);
            parse!(r#"SELECT a,b FROM (g MATCH (a:A)<-[:edge]-+(b:B))"#);
            parse!(r#"SELECT a,b FROM (g MATCH (a:A)~[:edge]~{5,}(b:B))"#);
            parse!(r#"SELECT a,b FROM (g MATCH (a:A)-[e:edge]-{2,6}(b:B))"#);
            parse!(r#"SELECT a,b FROM (g MATCH (a:A)->*(b:B))"#);
            parse!(r#"SELECT a,b FROM (g MATCH (a:A)<-+(b:B))"#);
            parse!(r#"SELECT a,b FROM (g MATCH (a:A)~{5,}(b:B))"#);
            parse!(r#"SELECT a,b FROM (g MATCH (a:A)-{2,6}(b:B))"#);
        }

        #[test]
        fn patterns() {
            parse!(
                r#"SELECT the_a.name AS src, the_b.name AS dest FROM (my_graph MATCH (the_a:a) -[the_y:y]-> (the_b:b) WHERE the_y.score > 10)"#
            );
            parse!(r#""SELECT a,b FROM (g MATCH (a)-[:has]->()-[:contains]->(b))""#);
            parse!(
                r#"SELECT a,b FROM GRAPH_TABLE (g MATCH (a) -[:has]-> (x), (x)-[:contains]->(b))"#
            );
        }

        #[test]
        fn path_var() {
            parse!(r#"SELECT a,b FROM GRAPH_TABLE (g MATCH p = (a:A) -[e:E]-> (b:B))"#);
        }

        #[test]
        fn paranthesized() {
            parse!(
                r#"SELECT a,b FROM GRAPH_TABLE (g MATCH ((a:A)-[e:Edge]->(b:A) WHERE a.owner=b.owner){2,5})"#
            );
            parse!(
                r#"SELECT a,b FROM GRAPH_TABLE (g MATCH pathVar = (a:A)(()-[e:Edge]->()){1,3}(b:B))"#
            );
            parse!(r#"SELECT a,b FROM GRAPH_TABLE (g MATCH pathVar = (a:A)(-[e:Edge]->)*(b:B))"#);
        }

        #[test]
        fn filters() {
            parse!(
                r#"SELECT u as banCandidate
                   FROM (g MATCH
                            (p:Post Where p.isFlagged = true)
                            <-[:createdPost]-
                            (u:User WHERE u.isBanned = false AND u.karma < 20)
                            -[:createdComment]->
                            (c:Comment WHERE c.isFlagged = true)
                          WHERE p.title LIKE '%considered harmful%')"#
            );
            parse!(
                r#"SELECT u as banCandidate
                   FROM (g MATCH
                            (p:Post Where p.isFlagged = true)
                            <-[:createdPost]-
                            (u:User WHERE u.isBanned = false AND u.karma < 20)
                            -[:createdComment]->
                            (c:Comment WHERE c.isFlagged = true)
                         )
                   WHERE p.title LIKE '%considered harmful%'"#
            );
        }

        #[test]
        fn path_mode() {
            parse!(
                r#"SELECT p FROM (g MATCH p = TRAIL (a WHERE a.owner='Dave') -[t:Transfer]-> * (b WHERE b.owner='Aretha'))"#
            );
            parse!(
                r#"SELECT p FROM (g MATCH p = SIMPLE (a WHERE a.owner='Dave') -[t:Transfer]-> * (b WHERE b.owner='Aretha'))"#
            );
            parse!(
                r#"SELECT p FROM (g MATCH p = ACYCLIC (a WHERE a.owner='Dave') -[t:Transfer]-> * (b WHERE b.owner='Aretha'))"#
            );
        }

        #[test]
        fn search_prefix() {
            parse!(
                r#"SELECT p FROM (g MATCH p = ANY SHORTEST (a WHERE a.owner='Dave') -[t:Transfer]-> * (b WHERE b.owner='Aretha'))"#
            );
            parse!(
                r#"SELECT p FROM (g MATCH p = ALL SHORTEST (a WHERE a.owner='Dave') -[t:Transfer]-> * (b WHERE b.owner='Aretha'))"#
            );
            parse!(
                r#"SELECT p FROM (g MATCH p = ANY (a WHERE a.owner='Dave') -[t:Transfer]-> * (b WHERE b.owner='Aretha'))"#
            );
            parse!(
                r#"SELECT p FROM (g MATCH p = ANY 5 (a WHERE a.owner='Dave') -[t:Transfer]-> * (b WHERE b.owner='Aretha'))"#
            );
            parse!(
                r#"SELECT p FROM (g MATCH p = SHORTEST 5 (a WHERE a.owner='Dave') -[t:Transfer]-> * (b WHERE b.owner='Aretha'))"#
            );
            parse!(
                r#"SELECT p FROM (g MATCH p = SHORTEST 5 GROUP (a WHERE a.owner='Dave') -[t:Transfer]-> * (b WHERE b.owner='Aretha'))"#
            );
        }

        #[test]
        fn match_and_join() {
            parse!(
                r#"SELECT a,b,c, t1.x as x, t2.y as y FROM GRAPH_TABLE (g MATCH (a) -> (b), (a) -> (c)), table1 as t1, table2 as t2"#
            );
            parse!(
                r#"SELECT a,b,c, t1.x as x, t2.y as y FROM table1 as t1, table2 as t2, GRAPH_TABLE (g MATCH (a) -> (b), (a) -> (c))"#
            );
        }

        #[test]
        fn union() {
            parse!(r#"(MyGraph MATCH (x)) UNION SELECT * FROM tbl1"#);
            parse!(r#"SELECT * FROM tbl1 UNION (MyGraph MATCH (x))"#);
        }

        #[test]
        fn etc() {
            parse!("SELECT * FROM (g MATCH ALL SHORTEST ( (x)-[e]->*(y) ))");
            parse!("SELECT * FROM (g MATCH ALL SHORTEST ( TRAIL (x)-[e]->*(y) ))");
        }
    }

    mod errors {
        use super::*;
        use crate::error::{LexError, UnexpectedToken, UnexpectedTokenData};
        use partiql_common::syntax::location::{Located, Location};
        use std::borrow::Cow;

        #[test]
        fn eof() {
            let res = parse_partiql(r"SELECT");
            assert!(res.is_err());
            let err_data = res.unwrap_err();
            assert_eq!(1, err_data.errors.len());
            assert_eq!(
                err_data.errors[0],
                ParseError::UnexpectedEndOfInput(BytePosition::from(6))
            );
        }

        #[test]
        fn unterminated_ion_unicode() {
            let q = r"/`܋";
            let res = parse_partiql(q);
            assert!(res.is_err());
            let err_data = res.unwrap_err();
            assert_eq!(2, err_data.errors.len());
            assert_eq!(
                err_data.errors[0],
                ParseError::UnexpectedToken(UnexpectedToken {
                    inner: UnexpectedTokenData {
                        token: Cow::from("/")
                    },
                    location: Location {
                        start: BytePosition::from(0),
                        end: BytePosition::from(1),
                    },
                })
            );
            assert_eq!(
                err_data.errors[1],
                ParseError::LexicalError(Located {
                    inner: LexError::UnterminatedDocLiteral,
                    location: Location {
                        start: BytePosition::from(1),
                        end: BytePosition::from(4),
                    },
                })
            );
        }
    }
}
