use criterion::{black_box, criterion_group, criterion_main, Criterion};
use partiql_parser::lalr_parse;
use partiql_parser::logos_lex;
use partiql_parser::peg_parse;
use partiql_parser::result::ParserResult;
use pest::iterators::Pairs;

const Q_STAR: &'static str = "SELECT *";

const Q_GROUP: &'static str =
    "SELECT g FROM data GROUP BY a AS x, b + c AS y, foo(d) AS z GROUP AS g";

const Q_COMPLEX: &'static str = r#"
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

fn pest_benchmark(c: &mut Criterion) {
    let parse = peg_parse;
    c.bench_function("peg-simple", |b| b.iter(|| parse(black_box(Q_STAR))));
    c.bench_function("peg-group", |b| b.iter(|| parse(black_box(Q_GROUP))));
    c.bench_function("peg-complex", |b| b.iter(|| parse(black_box(Q_COMPLEX))));
}

fn logos_benchmark(c: &mut Criterion) {
    let parse = |s| logos_lex(s).count(); // Just use `.count` to consume the lexer iterator
    c.bench_function("logos-simple", |b| b.iter(|| parse(black_box(Q_STAR))));
    c.bench_function("logos-group", |b| b.iter(|| parse(black_box(Q_GROUP))));
    c.bench_function("logos-complex", |b| b.iter(|| parse(black_box(Q_COMPLEX))));
}

fn lalr_benchmark(c: &mut Criterion) {
    let parse = lalr_parse;
    c.bench_function("lalr-simple", |b| b.iter(|| parse(black_box(Q_STAR))));
    c.bench_function("lalr-group", |b| b.iter(|| parse(black_box(Q_GROUP))));
    c.bench_function("lalr-complex", |b| b.iter(|| parse(black_box(Q_COMPLEX))));
}

criterion_group!(parse, pest_benchmark, logos_benchmark, lalr_benchmark);
criterion_main!(parse);
