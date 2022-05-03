use criterion::{black_box, criterion_group, criterion_main, Criterion};
use partiql_rewriter::experimental::{like_to_re_pattern, similar_to_re_pattern};
use rand::distributions::Alphanumeric;
use rand::{Rng, SeedableRng};
use regex::Regex;
use std::time::Duration;

const LIKE_SIMPLE: &str = r#"foo_.*?_bar"#;

fn like_8k() -> String {
    let mut rng = rand_chacha::ChaCha20Rng::seed_from_u64(987654321);
    let mut like_8k = String::with_capacity(8002);
    like_8k += "%";
    for ch in rng.sample_iter(&Alphanumeric).take(8000) {
        like_8k.push(ch as char);
    }
    like_8k += "%";
    like_8k
}

fn like(c: &mut Criterion) {
    c.bench_function("like-simple-translate", |b| {
        b.iter(|| like_to_re_pattern(black_box(LIKE_SIMPLE), '\\'))
    });

    let pat = like_to_re_pattern(black_box(LIKE_SIMPLE), '\\');
    let re = Regex::new(&pat).unwrap();
    c.bench_function("like-simple-match", |b| {
        b.iter(|| re.is_match("foos.*?%bar"))
    });

    let like_8k = like_8k();
    c.bench_function("like-8k-translate", |b| {
        b.iter(|| like_to_re_pattern(black_box(&like_8k), '\\'))
    });
    let pat = like_to_re_pattern(black_box(&like_8k), '\\');
    let re = Regex::new(&pat).unwrap();

    c.bench_function("like-8k-match", |b| b.iter(|| re.is_match(&like_8k)));
}

criterion_group! {
    name = like_compile;
    config = Criterion::default();
    targets = like
}

criterion_main!(like_compile);
