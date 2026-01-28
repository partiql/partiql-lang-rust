use criterion::{black_box, criterion_group, criterion_main, BenchmarkId, Criterion};
use partiql_catalog::context::SystemContext;
use partiql_eval::env::basic::MapBindings;
use partiql_eval::eval::BasicContext;
use partiql_eval::plan::EvaluationMode;
use partiql_logical::BindingsOp;
use partiql_logical::LogicalPlan;
use partiql_value::{DateTime, Value};
use std::time::Duration;

use partiql_benchmarks::{compile, create_catalog, lower, parse};

/// Simple benchmark query
const QUERY: &str = "SELECT a, b FROM data WHERE a % 100 = 0";
const DATA_SIZE: usize = 10_000;

/// Compiled plan for Legacy engine
struct LegacyPlan {
    plan: partiql_eval::eval::EvalPlan,
}

impl LegacyPlan {
    fn new() -> Self {
        std::env::set_var("TOTAL_ROWS", DATA_SIZE.to_string());
        let non_vec_query = QUERY.replace("data", "data()");
        let catalog = create_catalog("mem".to_string(), None);
        let parsed = parse(&non_vec_query).expect("Parse failed");
        let logical = lower(&*catalog, &parsed).expect("Lower failed");
        let plan = compile(EvaluationMode::Permissive, &*catalog, logical).expect("Compile failed");
        Self { plan }
    }

    fn execute(&self) -> usize {
        let bindings = MapBindings::default();
        let sys = SystemContext {
            now: DateTime::from_system_now_utc(),
        };
        let ctx = BasicContext::new(bindings, sys);
        match self.plan.execute(&ctx) {
            Ok(evaluated) => match evaluated.result {
                Value::Bag(bag) => bag.len(),
                _ => 1,
            },
            Err(_) => 0,
        }
    }
}

/// Compiled plan for Vectorized engine
struct VectorizedPlan {
    plan: partiql_eval_vectorized::VectorizedPlan,
}

impl VectorizedPlan {
    fn new(batch_size: usize) -> Self {
        let catalog = create_catalog("mem".to_string(), None);
        let parsed = parse(QUERY).expect("Parse failed");
        let logical = lower(&*catalog, &parsed).expect("Lower failed");
        let plan = compile_vectorized(&logical, batch_size);
        Self { plan }
    }

    fn execute(mut self) -> usize {
        let mut total_rows = 0;
        for batch_result in self.plan.execute() {
            match batch_result {
                Ok(batch) => {
                    let batch_row_count = if let Some(selection) = batch.selection() {
                        selection.indices.len()
                    } else {
                        batch.row_count()
                    };
                    total_rows += batch_row_count;
                }
                Err(_) => break,
            }
        }
        total_rows
    }
}

/// Compiled plan for Hybrid engine
struct HybridPlan {
    vm: partiql_eval::engine::PartiQLVM,
}

impl HybridPlan {
    fn new() -> Self {
        let catalog = create_catalog("mem".to_string(), None);
        let parsed = parse(QUERY).expect("Parse failed");
        let logical = lower(&*catalog, &parsed).expect("Lower failed");
        let vm = compile_hybrid(&logical).expect("Hybrid compile failed");
        Self { vm }
    }

    fn execute(self) -> usize {
        let mut stream = self.vm.execute().expect("Hybrid execute failed");
        let mut row_count = 0;
        loop {
            match stream.next_row() {
                Ok(Some(_)) => row_count += 1,
                Ok(None) => break,
                Err(_) => break,
            }
        }
        row_count
    }
}

fn compile_vectorized(
    logical: &LogicalPlan<BindingsOp>,
    batch_size: usize,
) -> partiql_eval_vectorized::VectorizedPlan {
    use partiql_eval_vectorized::reader::InMemoryGeneratedReader;
    let num_batches = (DATA_SIZE + batch_size - 1) / batch_size;
    let reader: Box<dyn partiql_eval_vectorized::BatchReader> =
        Box::new(InMemoryGeneratedReader::with_config(batch_size, num_batches));
    let context = partiql_eval_vectorized::CompilerContext::new()
        .with_data_source("data".to_string(), reader);
    let mut compiler = partiql_eval_vectorized::Compiler::new(context);
    compiler.compile(logical).expect("Vectorized compilation failed")
}

fn compile_hybrid(
    logical: &LogicalPlan<BindingsOp>,
) -> partiql_eval::engine::Result<partiql_eval::engine::PartiQLVM> {
    use partiql_eval::engine::{PlanCompiler, ScanProvider};
use partiql_eval::engine::plan::ReaderFactoryEnum;
    use partiql_logical::Scan;

    struct HybridScanProvider {}
    impl ScanProvider for HybridScanProvider {
        fn reader_factory(
            &self,
            _scan: &Scan,
        ) -> partiql_eval::engine::Result<Box<dyn partiql_eval::engine::RowReaderFactory>> {
            Ok(Box::new(
                partiql_eval::engine::InMemGeneratedReaderFactory::new(DATA_SIZE),
            ))
        }
    }

    let provider = HybridScanProvider {};
    let compiler = PlanCompiler::new(&provider);
    let compiled = compiler.compile(logical)?;
    compiler.instantiate(compiled, None)
}

fn bench_simple_comparison(c: &mut Criterion) {
    let mut group = c.benchmark_group("simple_10k");
    group.sample_size(30);

    group.bench_function("legacy", |b| {
        b.iter_batched(
            || LegacyPlan::new(),
            |plan| black_box(plan.execute()),
            criterion::BatchSize::LargeInput,
        );
    });

    group.bench_function("vectorized_1", |b| {
        b.iter_batched(
            || VectorizedPlan::new(1),
            |plan| black_box(plan.execute()),
            criterion::BatchSize::LargeInput,
        );
    });

    group.bench_function("vectorized_1024", |b| {
        b.iter_batched(
            || VectorizedPlan::new(1024),
            |plan| black_box(plan.execute()),
            criterion::BatchSize::LargeInput,
        );
    });

    group.bench_function("hybrid", |b| {
        b.iter_batched(
            || HybridPlan::new(),
            |plan| black_box(plan.execute()),
            criterion::BatchSize::LargeInput,
        );
    });

    group.finish();
}

criterion_group! {
    name = simple;
    config = Criterion::default()
        .measurement_time(Duration::from_secs(5))
        .warm_up_time(Duration::from_secs(2));
    targets = bench_simple_comparison
}

criterion_main!(simple);
