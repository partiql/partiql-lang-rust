use crate::common::parse;
use itertools::Itertools;
use partiql_ast::ast::{AstNode, TopLevelQuery};
use partiql_common::pretty::ToPretty;
use partiql_value::{bag, list, tuple, DateTime, Value};
use rust_decimal::prelude::FromPrimitive;
use time::macros::{date, datetime, offset, time};

mod common;

#[track_caller]
#[inline]
fn pretty_print_test(name: &str, statement: &str) {
    let res = common::parse(statement);
    assert!(res.is_ok());
    let res = res.unwrap();

    // First test that the pretty printed version is parseable
    pretty_print_roundtrip_test(&res.ast);

    // Then snapshot the pretty printed version
    pretty_print_output_test(name, statement, &res.ast);
}

#[track_caller]
fn pretty_print_output_test(name: &str, statement: &str, statement_ast: &AstNode<TopLevelQuery>) {
    // TODO https://github.com/partiql/partiql-lang-rust/issues/473
    let doc = [180, 120, 80, 40, 30, 20, 10]
        .into_iter()
        .map(|w| {
            let header = format!("{:-<w$}", "");
            let ast = format!("{}\n", statement_ast.to_pretty_string(w).unwrap());
            format!("{header}\n{ast}")
        })
        .join("\n");

    let w = 200;
    let header = format!("{:=<w$}", "");
    let doc = format!("{header}\n{statement}\n{header}\n\n{doc}");

    insta::assert_snapshot!(name, doc)
}

#[track_caller]
fn pretty_print_roundtrip_test(statement_ast: &AstNode<TopLevelQuery>) {
    let pretty = statement_ast.to_pretty_string(40).unwrap();
    let reparsed = parse(pretty.as_str());
    assert!(reparsed.is_ok());

    let pretty2 = reparsed.unwrap().ast.to_pretty_string(40).unwrap();

    assert_eq!(pretty, pretty2);
}

#[track_caller]
#[inline]
fn pretty_print_value_test(name: &str, value: &Value) {
    pretty_print_value_output_test(name, value);
    pretty_print_value_roundtrip_test(value);
}

#[track_caller]
fn pretty_print_value_output_test(name: &str, value: &Value) {
    let doc = [180, 120, 80, 40, 30, 20, 10]
        .into_iter()
        .map(|w| {
            let header = format!("{:-<w$}", "");
            let ast = format!("{}\n", value.to_pretty_string(w).unwrap());
            format!("{header}\n{ast}")
        })
        .join("\n");

    let w = 200;
    let header = format!("{:=<w$}", "");
    let doc = format!("{header}\n\n{doc}");

    insta::assert_snapshot!(name, doc)
}

#[track_caller]
fn pretty_print_value_roundtrip_test(value: &Value) {
    let pretty = value.to_pretty_string(40).unwrap();

    let reparsed = parse(pretty.as_str());
    assert!(reparsed.is_ok());

    let pretty2 = reparsed.unwrap().ast.to_pretty_string(40).unwrap();

    assert_eq!(pretty, pretty2);
}

#[test]
fn pretty_val() {
    let dec = Value::Decimal(Box::new(
        rust_decimal::Decimal::from_f64(2.998e8).expect("deciaml"),
    ));
    let dt_d = Value::DateTime(Box::new(DateTime::Date(date!(2020 - 01 - 01))));
    let dt_t = Value::DateTime(Box::new(DateTime::Time(time!(1:02:03.004_005_006))));
    let dt_ttz = Value::DateTime(Box::new(DateTime::TimeWithTz(
        time!(1:02:03.004_005_006),
        offset!(UTC),
    )));
    let dt_ts = Value::DateTime(Box::new(DateTime::Timestamp(datetime!(2020-01-01 0:00 ))));
    let dt_tstz = Value::DateTime(Box::new(DateTime::TimestampWithTz(
        datetime!(2020-01-01 0:00 UTC),
    )));
    let blob = Value::Blob(Box::new("abcdef".as_bytes().into()));
    let l_val = list!(
        1,
        2,
        999.876,
        dec,
        dt_d,
        dt_t,
        dt_ttz,
        dt_ts,
        dt_tstz,
        blob,
        Value::Missing
    );
    let short_l_val = list!(1, 2, "skip a few", 99, 100);
    let b_val = bag!(
        tuple!(("n", 1)),
        tuple!(("n", 2)),
        tuple!(("n", 3)),
        tuple!(("n", 4)),
        tuple!(("n", 5)),
        tuple!(("n", 6)),
        tuple!(("n", 7)),
        tuple!(("n", 8)),
        tuple!(("n", 9)),
        tuple!(("n", 10))
    );

    #[allow(clippy::approx_constant)]
    let t_val = tuple!(
        ("foo", true),
        ("-foo", false),
        ("bar", 42),
        ("baz", 3.14),
        ("qux", "string"),
        ("thud", Value::Null),
        ("plugh", l_val),
        ("xyzzy", b_val),
        ("waldo", short_l_val)
    )
    .into();
    pretty_print_value_test("pretty_val", &t_val);
}

#[test]
fn pretty() {
    pretty_print_test(
        "pretty",
        "select foo,bar, baz,thud.*,grunt.a[*].b[2].*, count(1) as n from
            <<
                { 'foo': 'foo', 'x': 9, 'y':5, z:-11 },
                { 'foo': 'bar' },
                { 'foo': 'qux' },
                { 'foo': 'bar' },
                { 'foo': 'baz' },
                { 'foo': 'bar' },
                { 'foo': 'baz' }
            >>  group by foo order by n desc",
    );
}

#[test]
fn pretty2() {
    pretty_print_test(
        "pretty2",
        "select foo,bar, baz,thud,grunt, count(1) as n from
            (SELECT * FROM table1)
            where (bar between 3 and 25 AND baz NOT LIKE 'example%') OR foo.a.b[*] IS MISSING
            group by foo
            order by n desc",
    );
}

#[test]
fn pretty_having_limit_offset() {
    pretty_print_test(
        "having_limit_offset",
        "SELECT a FROM foo GROUP BY a HAVING a > 2 ORDER BY a LIMIT 1 OFFSET 1",
    );
}

#[test]
fn pretty_select_value_unpivot() {
    pretty_print_test(
        "select value unpivot",
        "SELECT VALUE foo FROM (SELECT VALUE v FROM UNPIVOT e AS v) AS foo",
    );
}

#[test]
fn pretty_select_value_tuple_ctor() {
    pretty_print_test(
        "pretty_select_value_tuple_ctor",
        "SELECT VALUE {'a':v.a, 'b':v.b} FROM [{'a':1, 'b':1}, {'a':2, 'b':2}] AS v",
    );
}

#[test]
fn pretty_from_comma() {
    pretty_print_test("pretty_from_comma", "SELECT a, b FROM T1, T2");
}

#[test]
fn pretty_expr_in() {
    pretty_print_test("pretty_expr_in", "(a, b) IN ((1, 2), (3, 4))");
}

#[test]
fn pretty_setop() {
    pretty_print_test(
        "pretty_setop",
        "(SELECT a1 FROM b1 ORDER BY c1 LIMIT d1 OFFSET e1)
                            UNION
                            (SELECT a2 FROM b2 ORDER BY c2 LIMIT d2 OFFSET e2)
                            ORDER BY c3 LIMIT d3 OFFSET e3",
    );
}

#[test]
fn pretty_bagop() {
    pretty_print_test(
        "pretty_bagop",
        "
                (
                    (SELECT a1 FROM b1 ORDER BY c1 LIMIT d1 OFFSET e1)
                    UNION DISTINCT
                    (SELECT a2 FROM b2 ORDER BY c2 LIMIT d2 OFFSET e2)
                )
                OUTER UNION ALL
                (SELECT a3 FROM b3 ORDER BY c3 LIMIT d3 OFFSET e3)
                ORDER BY c4 LIMIT d4 OFFSET e4",
    );
}

#[test]
fn pretty_join() {
    pretty_print_test(
        "pretty_join",
        "
                  SELECT t1.id AS id, t1.val AS val1, t2.val AS val2
                  FROM table1 AS t1 JOIN table1_null_row AS t2 ON t1.id = t2.id",
    );
}

#[test]
fn pretty_kw_fns() {
    pretty_print_test("pretty_kw_fn1", "trim(trailing from 'test')");
    pretty_print_test("pretty_kw_fn2", "POSITION('abc' IN 'abcdefg')");
    pretty_print_test("pretty_kw_fn3", "substring('test', 100, 50)");
    pretty_print_test("pretty_kw_fn4", "substring('test', 100)");
}

#[test]
fn pretty_typed_lits() {
    pretty_print_test(
        "pretty_typed_lits1",
        "TIME WITH TIME ZONE '23:59:59.1234567890+18:00'",
    );
    pretty_print_test("pretty_typed_lits2", "TIME (3) WITH TIME ZONE '12:59:31'");
}

#[test]
fn pretty_case() {
    pretty_print_test("pretty_case_1", "SELECT VALUE CASE WHEN x + 1 < i THEN '< ONE' WHEN x + 1 = f THEN 'TWO' WHEN (x + 1 > d) AND (x + 1 < 100) THEN '>= THREE < 100' ELSE '?' END FROM << -1.0000, i, f, d, 100e0, null, missing >> AS x");
    pretty_print_test("pretty_case_2","SELECT VALUE CASE x + 1 WHEN NULL THEN 'shouldnt be null' WHEN MISSING THEN 'shouldnt be missing' WHEN i THEN 'ONE' WHEN f THEN 'TWO' WHEN d THEN 'THREE' END FROM << i, f, d, null, missing >> AS x");
}

#[test]
fn pretty_pivot() {
    pretty_print_test(
        "pretty_pivot",
        "
                  PIVOT foo.a AT foo.b
                  FROM <<{'a': 1, 'b':'I'}, {'a': 2, 'b':'II'}, {'a': 3, 'b':'III'}>> AS foo
                  ORDER BY a
                  LIMIT 1 OFFSET 1
                ",
    );
}

#[test]
fn pretty_ands_and_ors() {
    pretty_print_test(
        "pretty_ands_and_ors",
        "
                SELECT *
                FROM test_data AS test_data
                WHERE ((((test_data.country_code <> 'Distinctio.') AND ((test_data.* < false) AND (NOT (test_data.description LIKE 'Esse solam.') AND NOT (test_data.transaction_id LIKE 'Esset accusata.')))) OR (test_data.test_address <> 'Potest. Sed.')) AND (test_data.* > -28.146858383543243))
                ",
    );
}

mod graph {
    use once_cell::sync::Lazy;
    use std::collections::HashMap;
    use std::sync::atomic::{AtomicU32, Ordering};
    use std::sync::Mutex;
    static COUNTS: Lazy<Mutex<HashMap<String, AtomicU32>>> =
        Lazy::new(|| HashMap::default().into());

    macro_rules! parse_test {
        ($k:literal, $q:expr) => {{
            let k = $k;
            let mut counts = match COUNTS.lock() {
                Ok(guard) => guard,
                Err(poisoned) => poisoned.into_inner(),
            };
            let count = counts
                .entry(k.to_string())
                .or_insert_with(|| AtomicU32::new(1));
            let next = count.fetch_add(1, Ordering::SeqCst);
            drop(counts);

            super::pretty_print_test(&format!("graph_{k}_{next}"), $q);
        }};
    }
    #[test]
    fn edge() {
        macro_rules! parse {
            ($q:expr) => {{
                parse_test!("edge", $q)
            }};
        }
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
    fn labels() {
        macro_rules! parse {
            ($q:expr) => {{
                parse_test!("labels", $q)
            }};
        }

        parse!(r#"SELECT a,b FROM (g MATCH (a:%) -[e:%]-> (b:%))"#);
        parse!(r#"SELECT a,b FROM (g MATCH (a:!Z) -[e:!D]-> (b:!Y))"#);
        parse!(r#"SELECT a,b FROM (g MATCH (a:A|Z) -[e:E|D]-> (b:B|Y))"#);
        parse!(r#"SELECT a,b FROM (g MATCH (a:A&Z) -[e:E&D]-> (b:B&Y))"#);

        parse!(r#"SELECT a,b FROM (g MATCH (a:(A|Z)&!Y) -[e:E&!D]-> (b:%&!(!B)))"#);
    }

    #[test]
    fn quantifiers() {
        macro_rules! parse {
            ($q:expr) => {{
                parse_test!("quantifiers", $q)
            }};
        }
        parse!(r#"SELECT a,b FROM (g MATCH (a:A)-[:edge]->*(b:B))"#);
        parse!(r#"SELECT a,b FROM (g MATCH (a:A)<-[:edge]-+(b:B))"#);
        parse!(r#"SELECT a,b FROM (g MATCH (a:A)~[:edge]~{5,}(b:B))"#);
        parse!(r#"SELECT a,b FROM (g MATCH (a:A)-[e:edge]-{2,6}(b:B))"#);
        parse!(r#"SELECT a,b FROM (g MATCH (a:A)->*(b:B))"#);
        parse!(r#"SELECT a,b FROM (g MATCH (a:A)<-+(b:B))"#);
        parse!(r#"SELECT a,b FROM (g MATCH (a:A)~{5,}(b:B))"#);
        parse!(r#"SELECT a,b FROM (g MATCH (a:A)-{2,6}(b:B))"#);
        parse!(r#"SELECT a,b FROM (g MATCH (a:A) -? (b:B))"#);
    }
    #[test]
    fn patterns() {
        macro_rules! parse {
            ($q:expr) => {{
                parse_test!("patterns", $q)
            }};
        }
        parse!(
            r#"SELECT the_a.name AS src, the_b.name AS dest FROM (my_graph MATCH (the_a:a) -[the_y:y]-> (the_b:b) WHERE the_y.score > 10)"#
        );
        parse!(r#""SELECT a,b FROM (g MATCH (a)-[:has]->()-[:contains]->(b))""#);
        parse!(r#"SELECT a,b FROM GRAPH_TABLE (g MATCH (a) -[:has]-> (x), (x)-[:contains]->(b))"#);
    }
    #[test]
    fn path_var() {
        macro_rules! parse {
            ($q:expr) => {{
                parse_test!("path_var", $q)
            }};
        }
        parse!(r#"SELECT a,b FROM GRAPH_TABLE (g MATCH p = (a:A) -[e:E]-> (b:B))"#);
    }
    #[test]
    fn parenthesized() {
        macro_rules! parse {
            ($q:expr) => {{
                parse_test!("parenthesized", $q)
            }};
        }
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
        macro_rules! parse {
            ($q:expr) => {{
                parse_test!("filters", $q)
            }};
        }
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
        parse!(
            r#"SELECT u as banCandidate
                   FROM (g MATCH
                            (p:Post Where p.isFlagged = true)
                            <-[e:createdPost where e.isMobile = true]-
                            (u)
                         )"#
        );
    }
    #[test]
    fn path_mode() {
        macro_rules! parse {
            ($q:expr) => {{
                parse_test!("path_mode", $q)
            }};
        }
        parse!(
            r#"SELECT p FROM (g MATCH p = TRAIL (a WHERE a.owner='Dave') -[t:Transfer]-> * (b WHERE b.owner='Aretha'))"#
        );
        parse!(
            r#"SELECT p FROM (g MATCH p = SIMPLE (a WHERE a.owner='Dave') -[t:Transfer]-> * (b WHERE b.owner='Aretha'))"#
        );
        parse!(
            r#"SELECT p FROM (g MATCH p = ACYCLIC (a WHERE a.owner='Dave') -[t:Transfer]-> * (b WHERE b.owner='Aretha'))"#
        );
        parse!(
            r#"SELECT p FROM (g MATCH p = WALK (a WHERE a.owner='Dave') -[t:Transfer]-> * (b WHERE b.owner='Aretha'))"#
        );
    }
    #[test]
    fn search_prefix() {
        macro_rules! parse {
            ($q:expr) => {{
                parse_test!("search_prefix", $q)
            }};
        }
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
        parse!(
            r#"SELECT p FROM (g MATCH p = ALL (a WHERE a.owner='Dave') -[t:Transfer]-> * (b WHERE b.owner='Aretha'))"#
        );
    }
    #[test]
    fn match_and_join() {
        macro_rules! parse {
            ($q:expr) => {{
                parse_test!("match_and_join", $q)
            }};
        }
        parse!(
            r#"SELECT a,b,c, t1.x as x, t2.y as y FROM GRAPH_TABLE (g MATCH (a) -> (b), (a) -> (c)), table1 as t1, table2 as t2"#
        );
        parse!(
            r#"SELECT a,b,c, t1.x as x, t2.y as y FROM table1 as t1, table2 as t2, GRAPH_TABLE (g MATCH (a) -> (b), (a) -> (c))"#
        );
    }
    #[test]
    fn union() {
        macro_rules! parse {
            ($q:expr) => {{
                parse_test!("union", $q)
            }};
        }
        parse!(r#"(MyGraph MATCH (x)) UNION SELECT * FROM tbl1"#);
        parse!(r#"SELECT * FROM tbl1 UNION (MyGraph MATCH (x))"#);
    }
    #[test]
    fn etc() {
        macro_rules! parse {
            ($q:expr) => {{
                parse_test!("etc", $q)
            }};
        }
        parse!("SELECT * FROM (g MATCH ALL SHORTEST ( (x)-[e]->*(y) ))");
        parse!("SELECT * FROM (g MATCH ALL SHORTEST ( TRAIL (x)-[e]->*(y) ))");
        parse!(
            "SELECT * FROM (g MATCH \
                                    REPEATABLE ELEMENTS \
                                    ALL SHORTEST ( (x)-[e]->*(y) ) \
                                    KEEP ANY SIMPLE PATHS \
                                    WHERE x.foo = 'bar'
                               )"
        );
        parse!(
            "SELECT * FROM (g MATCH \
                                    DIFFERENT EDGES \
                                    ALL SHORTEST ( (x)-[e]->*(y) ) \
                                    KEEP ANY SIMPLE PATHS \
                                    WHERE x.foo = 'bar'
                               )"
        );
        parse!("SELECT * FROM (g MATCH ( (x)-[e]->*(y) ) | (z) ~ (q) )");
        parse!("SELECT * FROM (g MATCH ( (x)-[e]->*(y) ) |+| (z) ~ (q) )");
    }
    #[test]
    fn shapes() {
        macro_rules! parse {
            ($q:expr) => {{
                parse_test!("shapes", $q)
            }};
        }
        parse!(
            "SELECT * \
                FROM (g MATCH (x)-[e]->*(y) \
                        ONE ROW PER MATCH \
                      )"
        );
        parse!(
            "SELECT * \
                FROM (g MATCH (x)-[e]->*(y) \
                        ONE ROW PER NODE ( x ) \
                      )"
        );
        parse!(
            "SELECT * \
                FROM (g MATCH p = (x)-[e]->*(y) \
                        ONE ROW PER NODE ( x ) IN ( p ) \
                      )"
        );
        parse!(
            "SELECT * \
                FROM (g MATCH (x)-[e]->*(y) \
                        ONE ROW PER STEP ( x, e, y ) \
                      )"
        );
        parse!(
            "SELECT * \
                FROM (g MATCH p = (x)-[e]->*(y)-[e2]-(z)  \
                        ONE ROW PER STEP ( y, e2, z ) IN ( p ) \
                      )"
        );
        parse!(
            "SELECT * \
                FROM (g MATCH (x)-[e]->*(y) \
                        COLUMNS(x as node1, e, y.*) \
                      )"
        );
        parse!(
            "SELECT * \
                FROM (g MATCH (x)-[e]->*(y) \
                        EXPORT ALL SINGLETONS \
                      )"
        );
        parse!(
            "SELECT * \
                FROM (g MATCH (x)-[e]->*(y) \
                        EXPORT ALL SINGLETONS EXCEPT ( e, y ) \
                      )"
        );
        parse!(
            "SELECT * \
                FROM (g MATCH (x)-[e]->*(y) \
                        EXPORT SINGLETONS ( x, e ) \
                      )"
        );
        parse!(
            "SELECT * \
                FROM (g MATCH (x)-[e]->*(y) \
                        EXPORT NO SINGLETONS \
                      )"
        );
        parse!(
            "SELECT * \
                FROM (g MATCH (x)-[e]->*(y) \
                        ONE ROW PER MATCH \
                        COLUMNS(x as node1, e, y.*) \
                        EXPORT ALL SINGLETONS \
                      )"
        );
        parse!(
            "SELECT * \
                FROM (g MATCH (x)-[e]->*(y) \
                        ONE ROW PER MATCH \
                        COLUMNS(x as node1, e, y.*) \
                        EXPORT ALL SINGLETONS \
                      )"
        );
        parse!(
            "SELECT * \
                FROM (g MATCH (x)-[e]->*(y) \
                        ONE ROW PER MATCH \
                        COLUMNS(x,y) \
                        EXPORT NO SINGLETONS \
                      )"
        );
    }
    #[test]
    fn simplified() {
        macro_rules! parse {
            ($q:expr) => {{
                parse_test!("simplified", $q)
            }};
        }
        parse!("SELECT * FROM (g MATCH <-/ start&begin fin> /- )");
        parse!("SELECT * FROM (g MATCH ~/ !begin -fin /~ )");
        parse!("SELECT * FROM (g MATCH -/ start <fin /-> )");
        parse!("SELECT * FROM (g MATCH <~/ start <~fin |+| begin ~fin> /~ )");
        parse!("SELECT * FROM (g MATCH ~/ start <fin> | begin -fin /~> )");
        parse!("SELECT * FROM (g MATCH <-/ start? intermediate{1,} fin  /-> )");
        parse!("SELECT * FROM (g MATCH <-/ start? intermediate* fin  /-> )");
        parse!("SELECT * FROM (g MATCH <-/ edge+ fin  /-> )");
        parse!("SELECT * FROM (g MATCH -/ start !(intermediate other){2,3} fin  /- )");
    }
}

#[test]
fn pretty_exclude() {
    pretty_print_test(
        "pretty_exclude_1",
        "
                    SELECT * EXCLUDE c.ssn, c.address.street FROM [{
                        'name': 'Alan',
                        'custId': 1,
                        'address': {
                            'city': 'Seattle',
                            'zipcode': 98109,
                            'street': '123 Seaplane Dr.'
                        },
                        'ssn': 123456789
                    }] AS c
                ",
    );
    pretty_print_test(
        "pretty_exclude_2",
        "
                    SELECT * EXCLUDE t.a.b.c[0], t.a.b.c[1].field
                    FROM [{
                        'a': {
                        'b': {
                        'c': [
                        {
                        'field': 0    -- c[0]
                        },
                        {
                        'field': 1    -- c[1]
                        },
                        {
                        'field': 2    -- c[2]
                        }
                        ]
                        }
                        },
                        'foo': 'bar'
                        }] AS t
                ",
    );
    pretty_print_test(
        "pretty_exclude_3",
        "
                SELECT *
                    EXCLUDE
                t.a.b.c[*].field_x
                FROM [{
                    'a': {
                        'b': {
                            'c': [
                            {                    -- c[0]
                                'field_x': 0,
                                'field_y': 0
                            },
                            {                    -- c[1]
                                'field_x': 1,
                                'field_y': 1
                            },
                            {                    -- c[2]
                                'field_x': 2,
                                'field_y': 2
                            }
                            ]
                        }
                    },
                    'foo': 'bar'
                }] AS t
                ",
    );
    pretty_print_test(
        "pretty_exclude_4",
        "
                SELECT *
                    EXCLUDE
                t.a.b.c[*].*
                    FROM [{
                        'a': {
                            'b': {
                                'c': [
                                {                    -- c[0]
                                    'field_x': 0,
                                    'field_y': 0
                                },
                                {                    -- c[1]
                                    'field_x': 1,
                                    'field_y': 1
                                },
                                {                    -- c[2]
                                    'field_x': 2,
                                    'field_y': 2
                                }
                                ]
                            }
                        },
                        'foo': 'bar'
                    }] AS t
                ",
    );
}
