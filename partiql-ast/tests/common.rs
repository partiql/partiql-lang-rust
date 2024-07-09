use partiql_ast::pretty::ToPretty;
use partiql_parser::ParserResult;

pub fn setup() {
    // setup test code goes here
}

#[track_caller]
#[inline]
fn parse(statement: &str) -> ParserResult<'_> {
    partiql_parser::Parser::default().parse(statement)
}

#[track_caller]
#[inline]
fn pretty_print_test(statement: &str) {
    let res = parse(statement);
    assert!(res.is_ok());
    let res = res.unwrap();
// TODO https://github.com/partiql/partiql-lang-rust/issues/473
    for w in [180, 120, 80, 40, 30, 20, 10] {
        println!("{:-<w$}", "");
        println!("{}\n", res.ast.to_pretty_string(w).unwrap());
    }
}
#[test]
fn pretty() {
    pretty_print_test(
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
        "select foo,bar, baz,thud,grunt, count(1) as n from
            (SELECT * FROM table1)
            where (bar between 3 and 25 AND baz NOT LIKE 'example%') OR foo.a.b[*] IS MISSING
            group by foo
            order by n desc",
    );
}

#[test]
fn pretty_having_limit_offset() {
    pretty_print_test("SELECT a FROM foo GROUP BY a HAVING a > 2 ORDER BY a LIMIT 1 OFFSET 1");
}

#[test]
fn pretty_select_value_unpivot() {
    pretty_print_test("SELECT VALUE foo FROM (SELECT VALUE v FROM UNPIVOT e AS v) AS foo");
}

#[test]
fn pretty_select_value_tuple_ctor() {
    pretty_print_test("SELECT VALUE {'a':v.a, 'b':v.b} FROM [{'a':1, 'b':1}, {'a':2, 'b':2}] AS v");
}

#[test]
fn pretty_from_comma() {
    pretty_print_test("SELECT a, b FROM T1, T2");
}

#[test]
fn pretty_expr_in() {
    pretty_print_test("(a, b) IN ((1, 2), (3, 4))");
}

#[test]
fn pretty_setop() {
    pretty_print_test(
        "(SELECT a1 FROM b1 ORDER BY c1 LIMIT d1 OFFSET e1)
                            UNION
                            (SELECT a2 FROM b2 ORDER BY c2 LIMIT d2 OFFSET e2)
                            ORDER BY c3 LIMIT d3 OFFSET e3",
    );
}

#[test]
fn pretty_bagop() {
    pretty_print_test(
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
        "
                  SELECT t1.id AS id, t1.val AS val1, t2.val AS val2
                  FROM table1 AS t1 JOIN table1_null_row AS t2 ON t1.id = t2.id",
    );
}

#[test]
fn pretty_kw_fns() {
    pretty_print_test("trim(trailing from 'test')");
    pretty_print_test("POSITION('abc' IN 'abcdefg')");
    pretty_print_test("substring('test', 100, 50)");
    pretty_print_test("substring('test', 100)");
}

#[test]
fn pretty_typed_lits() {
    pretty_print_test("TIME WITH TIME ZONE '23:59:59.1234567890+18:00'");
    pretty_print_test("TIME (3) WITH TIME ZONE '12:59:31'");
}

#[test]
fn pretty_case() {
    pretty_print_test("SELECT VALUE CASE WHEN x + 1 < i THEN '< ONE' WHEN x + 1 = f THEN 'TWO' WHEN (x + 1 > d) AND (x + 1 < 100) THEN '>= THREE < 100' ELSE '?' END FROM << -1.0000, i, f, d, 100e0, null, missing >> AS x");
    pretty_print_test("SELECT VALUE CASE x + 1 WHEN NULL THEN 'shouldnt be null' WHEN MISSING THEN 'shouldnt be missing' WHEN i THEN 'ONE' WHEN f THEN 'TWO' WHEN d THEN 'THREE' END FROM << i, f, d, null, missing >> AS x");
}

#[test]
fn pretty_pivot() {
    pretty_print_test(
        "
                  PIVOT foo.a AT foo.b
                  FROM <<{'a': 1, 'b':'I'}, {'a': 2, 'b':'II'}, {'a': 3, 'b':'III'}>> AS foo
                  ORDER BY a
                  LIMIT 1 OFFSET 1
                ",
    );
}
