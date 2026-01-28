use criterion::{black_box, criterion_group, criterion_main, BenchmarkId, Criterion};
use partiql_catalog::context::SystemContext;
use partiql_eval::env::basic::MapBindings;
use partiql_eval::eval::BasicContext;
use partiql_eval::plan::EvaluationMode;
use partiql_logical::BindingsOp;
use partiql_logical::LogicalPlan;
use partiql_value::{DateTime, Value};
use std::collections::HashSet;
use std::time::Duration;

use partiql_benchmarks::{compile, create_catalog, lower, parse};

/// Configuration for benchmark filtering via environment variables
struct BenchConfig {
    sizes: Option<HashSet<usize>>,
    engines: Option<HashSet<String>>,
    queries: Option<HashSet<String>>,
    formats: Option<HashSet<String>>,
}

impl BenchConfig {
    fn from_env() -> Self {
        let sizes = std::env::var("BENCH_SIZES")
            .ok()
            .map(|s| {
                s.split(',')
                    .filter_map(|size| size.trim().parse::<usize>().ok())
                    .collect()
            });

        let engines = std::env::var("BENCH_ENGINES")
            .ok()
            .map(|s| {
                s.split(',')
                    .map(|e| e.trim().to_lowercase())
                    .collect()
            });

        let queries = std::env::var("BENCH_QUERIES")
            .ok()
            .map(|s| {
                s.split(',')
                    .map(|q| q.trim().to_string())
                    .collect()
            });

        let formats = std::env::var("BENCH_FORMATS")
            .ok()
            .map(|s| {
                s.split(',')
                    .map(|f| f.trim().to_lowercase())
                    .collect()
            });

        Self { sizes, engines, queries, formats }
    }

    fn should_run_size(&self, size: usize) -> bool {
        self.sizes.as_ref().map_or(true, |s| s.contains(&size))
    }

    fn should_run_engine(&self, engine: &str) -> bool {
        self.engines.as_ref().map_or(true, |e| e.contains(&engine.to_lowercase()))
    }

    fn should_run_query(&self, query: &str) -> bool {
        self.queries.as_ref().map_or(true, |q| q.contains(query))
    }

    fn should_run_format(&self, format: &str) -> bool {
        self.formats.as_ref().map_or(true, |f| f.contains(&format.to_lowercase()))
    }
}

/// Data source configuration for benchmarks
#[derive(Debug, Clone)]
enum DataSource {
    InMemory { rows: usize },
    IonFile { path: String, rows: usize },
}

impl DataSource {
    fn rows(&self) -> usize {
        match self {
            DataSource::InMemory { rows } => *rows,
            DataSource::IonFile { rows, .. } => *rows,
        }
    }
}

/// Expand ~ in file paths to the user's home directory
fn expand_home_path(path: &str) -> String {
    if path.starts_with("~/") {
        if let Some(home) = std::env::var("HOME").ok() {
            return path.replacen("~", &home, 1);
        }
    }
    path.to_string()
}

/// Ion file data sources - switch to this for file-based benchmarks
const ION_FILE_SOURCES: &[(&str, usize)] = &[
    ("~/Desktop/test_data/data_b1_n100.ion", 100),
    ("~/Desktop/test_data/data_b1_n1000.ion", 1000),
];

/// Convert ION_FILE_SOURCES to DataSource array
fn ion_file_sources() -> Vec<DataSource> {
    ION_FILE_SOURCES
        .iter()
        .map(|(path, rows)| DataSource::IonFile {
            path: path.to_string(),
            rows: *rows,
        })
        .collect()
}

/// Benchmark queries - projections and filters only
const QUERIES: &[(&str, &str)] = &[
    ("proj", "SELECT a, b FROM data"),
    (
        "every_other",
        "SELECT a, b FROM data WHERE a % 2 = 0",
    ),
    (
        "every_other_complex",
        "SELECT a, b FROM data WHERE ((a - a + b - b + a - a + b - b) + a % 2) = 0",
    ),
    // (
    //     "filter_range",
    //     "SELECT a, b FROM data WHERE a > 1000 AND a < 9000",
    // ),
];

/// Compiled plan for Legacy engine
struct LegacyPlan {
    plan: partiql_eval::eval::EvalPlan,
}

impl LegacyPlan {
    fn new(query: &str, source: &DataSource) -> Self {
        let (catalog, query_to_use) = match source {
            DataSource::InMemory { rows } => {
                // Set environment variable for data generation
                std::env::set_var("TOTAL_ROWS", rows.to_string());
                let non_vec_query = query.replace("data", "data()");
                let catalog = create_catalog("mem".to_string(), None);
                (catalog, non_vec_query)
            }
            DataSource::IonFile { path, .. } => {
                let expanded_path = expand_home_path(path);
                let non_vec_query = query.replace("data", "data()");
                let catalog = create_catalog("ion".to_string(), Some(expanded_path));
                (catalog, non_vec_query)
            }
        };

        let parsed = parse(&query_to_use).expect("Parse failed");
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
                Value::Bag(bag) => {
                    let mut row_count = 0;
                    for _ in bag.iter() {
                        row_count += 1;
                    }
                    row_count
                }
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
    fn new(query: &str, source: &DataSource, batch_size: usize) -> Self {
        let plan = match source {
            DataSource::InMemory { rows } => {
                let catalog = create_catalog("mem".to_string(), None);
                let parsed = parse(query).expect("Parse failed");
                let logical = lower(&*catalog, &parsed).expect("Lower failed");
                let num_batches = (*rows + batch_size - 1) / batch_size;
                compile_vectorized(&logical, *rows, num_batches, batch_size)
            }
            DataSource::IonFile { path, .. } => {
                let expanded_path = expand_home_path(path);
                let catalog = create_catalog("ion".to_string(), Some(expanded_path.clone()));
                let parsed = parse(query).expect("Parse failed");
                let logical = lower(&*catalog, &parsed).expect("Lower failed");
                compile_vectorized_ion(&logical, &expanded_path, batch_size)
            }
        };

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
    fn new(query: &str, source: &DataSource) -> Self {
        let vm = match source {
            DataSource::InMemory { rows } => {
                let catalog = create_catalog("mem".to_string(), None);
                let parsed = parse(query).expect("Parse failed");
                let logical = lower(&*catalog, &parsed).expect("Lower failed");
                compile_hybrid(&logical, None, *rows).expect("Hybrid compile failed")
            }
            DataSource::IonFile { path, rows } => {
                let expanded_path = expand_home_path(path);
                let catalog = create_catalog("ion".to_string(), Some(expanded_path.clone()));
                let parsed = parse(query).expect("Parse failed");
                let logical = lower(&*catalog, &parsed).expect("Lower failed");
                compile_hybrid(&logical, Some(expanded_path), *rows).expect("Hybrid compile failed")
            }
        };

        Self { vm }
    }

    fn execute(&mut self) -> usize {
        let mut row_count = 0;

        loop {
            match self.vm.next_row() {
                Ok(Some(_)) => row_count += 1,
                Ok(None) => break,
                Err(_) => break,
            }
        }

        row_count
    }
    
    fn reset(&mut self) {
        self.vm.reset().expect("Reset failed");
    }
}

/// Helper to compile vectorized plan with in-memory data
fn compile_vectorized(
    logical: &LogicalPlan<BindingsOp>,
    data_size: usize,
    num_batches: usize,
    batch_size: usize,
) -> partiql_eval_vectorized::VectorizedPlan {
    use partiql_eval_vectorized::reader::InMemoryGeneratedReader;

    // Adjust num_batches to account for actual data size
    let actual_batches = if batch_size * num_batches > data_size {
        (data_size + batch_size - 1) / batch_size
    } else {
        num_batches
    };

    let reader: Box<dyn partiql_eval_vectorized::BatchReader> =
        Box::new(InMemoryGeneratedReader::with_config(batch_size, actual_batches));

    let context = partiql_eval_vectorized::CompilerContext::new()
        .with_data_source("data".to_string(), reader);

    let mut compiler = partiql_eval_vectorized::Compiler::new(context);
    compiler
        .compile(logical)
        .expect("Vectorized compilation failed")
}

/// Helper to compile vectorized plan with Ion file data
fn compile_vectorized_ion(
    logical: &LogicalPlan<BindingsOp>,
    ion_file_path: &str,
    batch_size: usize,
) -> partiql_eval_vectorized::VectorizedPlan {
    use partiql_eval_vectorized::reader::PIonTextReader;

    let reader: Box<dyn partiql_eval_vectorized::BatchReader> =
        Box::new(PIonTextReader::from_ion_file(ion_file_path, batch_size)
            .expect("Failed to create Ion text reader"));

    let context = partiql_eval_vectorized::CompilerContext::new()
        .with_data_source("data".to_string(), reader);

    let mut compiler = partiql_eval_vectorized::Compiler::new(context);
    compiler
        .compile(logical)
        .expect("Vectorized compilation failed")
}

/// Helper to compile hybrid plan
fn compile_hybrid(
    logical: &LogicalPlan<BindingsOp>,
    ion_file_path: Option<String>,
    total_rows: usize,
) -> partiql_eval::engine::Result<partiql_eval::engine::PartiQLVM> {
    use partiql_eval::engine::{PlanCompiler, ScanProvider, ReaderFactoryEnum};
    use partiql_logical::Scan;

    struct HybridScanProvider {
        ion_file_path: Option<String>,
        num_rows: usize,
    }

    impl ScanProvider for HybridScanProvider {
        fn reader_factory(
            &self,
            _scan: &Scan,
        ) -> partiql_eval::engine::Result<ReaderFactoryEnum> {
            match &self.ion_file_path {
                Some(path) => Ok(ReaderFactoryEnum::Ion(
                    partiql_eval::engine::IonRowReaderFactory::new(path.clone()),
                )),
                None => Ok(ReaderFactoryEnum::InMem(
                    partiql_eval::engine::InMemGeneratedReaderFactory::new(self.num_rows),
                )),
            }
        }
    }

    let provider = HybridScanProvider { 
        ion_file_path,
        num_rows: total_rows 
    };
    let compiler = PlanCompiler::new(&provider);
    let compiled = compiler.compile(logical)?;
    compiler.instantiate(compiled, None)
}

/// Helper function to generate a group name for a data source
fn group_name_for_source(source: &DataSource) -> String {
    match source {
        DataSource::InMemory { rows } => format!("{}_rows_mem", rows),
        DataSource::IonFile { rows, .. } => format!("{}_rows_ion", rows),
    }
}

/// Benchmark all engines for a specific data source
fn bench_data_source(c: &mut Criterion, source: DataSource) {
    let config = BenchConfig::from_env();
    
    // Check if we should run benchmarks for this data size
    if !config.should_run_size(source.rows()) {
        return;
    }
    
    // Check if we should run benchmarks for this data format
    let format = match &source {
        DataSource::InMemory { .. } => "mem",
        DataSource::IonFile { .. } => "ion",
    };
    if !config.should_run_format(format) {
        return;
    }
    
    let group_name = group_name_for_source(&source);
    let mut group = c.benchmark_group(&group_name);
    
    // Adjust sample size for larger datasets
    if source.rows() >= 100_000 {
        group.sample_size(20);
    }

    for &(query_name, query) in QUERIES {
        // Check if we should run this query
        if !config.should_run_query(query_name) {
            continue;
        }
        
        // Legacy
        if config.should_run_engine("legacy") {
            group.bench_with_input(
                BenchmarkId::new(query_name, "legacy"),
                &(query, &source),
                |b, &(query, source)| {
                    b.iter_batched(
                        || LegacyPlan::new(query, source),
                        |plan| black_box(plan.execute()),
                        criterion::BatchSize::LargeInput,
                    );
                },
            );
        }

        // Vectorized-1
        if config.should_run_engine("vectorized_1") {
            group.bench_with_input(
                BenchmarkId::new(query_name, "vectorized_1"),
                &(query, &source),
                |b, &(query, source)| {
                    b.iter_batched(
                        || VectorizedPlan::new(query, source, 1),
                        |plan| black_box(plan.execute()),
                        criterion::BatchSize::LargeInput,
                    );
                },
            );
        }

        // Vectorized-1024
        if config.should_run_engine("vectorized_1024") {
            group.bench_with_input(
                BenchmarkId::new(query_name, "vectorized_1024"),
                &(query, &source),
                |b, &(query, source)| {
                    b.iter_batched(
                        || VectorizedPlan::new(query, source, 1024),
                        |plan| black_box(plan.execute()),
                        criterion::BatchSize::LargeInput,
                    );
                },
            );
        }

        // Hybrid
        if config.should_run_engine("hybrid") {
            group.bench_with_input(
                BenchmarkId::new(query_name, "hybrid"),
                &(query, &source),
                |b, &(query, source)| {
                    b.iter_batched_ref(
                        || HybridPlan::new(query, source),
                        |plan| {
                            let count = black_box(plan.execute());
                            plan.reset();
                            count
                        },
                        criterion::BatchSize::LargeInput,
                    );
                },
            );
        }
    }

    group.finish();
}

/// Data sizes to benchmark
const DATA_SIZES: &[usize] = &[1, 10, 100, 1_000, 10_000, 100_000, 1_000_000];

/// Benchmark all engine/query/size/format configurations
/// Filtering is controlled via environment variables (see BENCH_CONFIG.md)
fn bench_all_configurations(c: &mut Criterion) {
    // Benchmark in-memory sources for all data sizes
    for &size in DATA_SIZES {
        bench_data_source(c, DataSource::InMemory { rows: size });
    }
    
    // Benchmark Ion file sources (if they exist)
    for source in ion_file_sources() {
        bench_data_source(c, source);
    }
}

criterion_group! {
    name = engine_comparison;
    config = Criterion::default()
        .measurement_time(Duration::from_secs(10))
        .warm_up_time(Duration::from_secs(3));
    targets = bench_all_configurations
}

criterion_main!(engine_comparison);
