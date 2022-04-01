use criterion::{black_box, criterion_group, criterion_main, Criterion};
use partiql_parser::lalr_parse;
use partiql_parser::logos_lex;
use std::time::Duration;

const Q_STAR: &str = "SELECT *";

const Q_ION: &str = "SELECT a FROM `{'a':1,  'b':1}`";

const Q_GROUP: &str = "SELECT g FROM data GROUP BY a AS x, b + c AS y, foo(d) AS z GROUP AS g";

const Q_COMPLEX: &str = r#"
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

fn logos_benchmark(c: &mut Criterion) {
    let parse = logos_lex;
    c.bench_function("logos-simple", |b| b.iter(|| parse(black_box(Q_STAR))));
    c.bench_function("logos-ion", |b| b.iter(|| parse(black_box(Q_ION))));
    c.bench_function("logos-group", |b| b.iter(|| parse(black_box(Q_GROUP))));
    c.bench_function("logos-complex", |b| b.iter(|| parse(black_box(Q_COMPLEX))));
}

fn lalr_benchmark(c: &mut Criterion) {
    let parse = lalr_parse;
    c.bench_function("lalr-simple", |b| b.iter(|| parse(black_box(Q_STAR))));
    c.bench_function("lalr-ion", |b| b.iter(|| parse(black_box(Q_ION))));
    c.bench_function("lalr-group", |b| b.iter(|| parse(black_box(Q_GROUP))));
    c.bench_function("lalr-complex", |b| b.iter(|| parse(black_box(Q_COMPLEX))));
}

criterion_group! {
    name = parse;
    config = Criterion::default().measurement_time(Duration::new(10, 0));
    targets = logos_benchmark, lalr_benchmark
}

criterion_main!(parse);
