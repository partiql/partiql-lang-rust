use partiql_perf::common;

use common::{create_catalog, parse, lower, compile, count_rows_from_file};
use partiql_eval::engine::{
    IonRowReaderFactory, PlanCompiler, RowReaderFactory, ScanProvider,
};
use partiql_eval::engine::ReaderFactoryEnum;
use partiql_eval::env::basic::MapBindings;
use partiql_eval::eval::BasicContext;
use partiql_eval::plan::EvaluationMode;
use partiql_logical::Scan;
use partiql_value::{tuple, DateTime, Value};
use std::{io::{self, Write}, time::{Duration, Instant}};

/// Represents a benchmark query
#[derive(Clone)]
struct BenchmarkQuery {
    name: String,
    query: String,
}

/// Data format types - each variant holds the data it needs
#[derive(Clone, PartialEq, Eq)]
enum DataFormat {
    InMemory { num_batches: usize },
    Ion { path: String },
    IonBinary { path: String },
    Arrow { path: String },
    Parquet { path: String },
}

impl DataFormat {
    fn name(&self) -> &str {
        match self {
            DataFormat::InMemory { .. } => "In-Memory",
            DataFormat::Ion { .. } => "Ion",
            DataFormat::IonBinary { .. } => "Ion Binary",
            DataFormat::Arrow { .. } => "Arrow",
            DataFormat::Parquet { .. } => "Parquet",
        }
    }
    
    fn format_str(&self) -> &str {
        match self {
            DataFormat::InMemory { .. } => "mem",
            DataFormat::Ion { .. } => "ion",
            DataFormat::IonBinary { .. } => "ion",
            DataFormat::Arrow { .. } => "arrow",
            DataFormat::Parquet { .. } => "parquet",
        }
    }
    
    fn row_count(&self) -> usize {
        match self {
            DataFormat::InMemory { num_batches } => {
                // Batch size 1 for legacy compatibility in row counting
                1 * num_batches
            }
            DataFormat::Ion { path } => count_rows_from_file("ion", path),
            DataFormat::IonBinary { path } => count_rows_from_file("ionb", path),
            DataFormat::Arrow { path } => count_rows_from_file("arrow", path),
            DataFormat::Parquet { path } => count_rows_from_file("parquet", path),
        }
    }
}

/// Results from a single benchmark run
#[derive(Clone)]
struct BenchmarkResult {
    query_name: String,
    engine: String,
    data_format: String,
    data_source_rows: usize,  // Input data source size
    output_rows: usize,        // Actual output rows from query
    min_ms: f64,
    max_ms: f64,
    avg_ms: f64,
    iterations: usize,
    error: Option<String>,
}

impl BenchmarkResult {
    /// Get formatted data format without row count
    fn formatted_data_format(&self) -> String {
        self.data_format.clone()
    }
}

/// Process explicit file paths into DataFormat instances
fn process_file_paths(paths: Vec<String>) -> Result<Vec<DataFormat>, String> {
    use std::path::Path;
    
    let mut formats = Vec::new();
    
    for path_str in paths {
        let path = Path::new(&path_str);
        
        if !path.exists() {
            return Err(format!("File not found: {}", path_str));
        }
        
        if !path.is_file() {
            return Err(format!("Not a file: {}", path_str));
        }
        
        if let Some(ext) = path.extension().and_then(|e| e.to_str()) {
            match ext {
                "ion" => formats.push(DataFormat::Ion { path: path_str }),
                "10n" => formats.push(DataFormat::IonBinary { path: path_str }),
                "arrow" => formats.push(DataFormat::Arrow { path: path_str }),
                "parquet" => formats.push(DataFormat::Parquet { path: path_str }),
                _ => {
                    return Err(format!(
                        "Unsupported file extension '{}' for file: {}",
                        ext, path_str
                    ));
                }
            }
        } else {
            return Err(format!("File has no extension: {}", path_str));
        }
    }
    
    if formats.is_empty() {
        return Err("No valid data files provided".to_string());
    }
    
    Ok(formats)
}

fn main() {
    // Parse command line arguments
    let args: Vec<String> = std::env::args().collect();
    let mut iterations = 5;
    let mut warmup_iterations = 1;
    let mut include_mem: Option<usize> = None;
    let mut file_paths: Vec<String> = Vec::new();

    // Simple argument parsing
    let mut i = 1;
    while i < args.len() {
        match args[i].as_str() {
            "--iterations" => {
                if i + 1 < args.len() {
                    iterations = args[i + 1].parse().unwrap_or(5);
                    i += 2;
                } else {
                    eprintln!("Error: --iterations requires a value");
                    std::process::exit(1);
                }
            }
            "--warmup" => {
                if i + 1 < args.len() {
                    warmup_iterations = args[i + 1].parse().unwrap_or(1);
                    i += 2;
                } else {
                    eprintln!("Error: --warmup requires a value");
                    std::process::exit(1);
                }
            }
            "--include-mem" => {
                if i + 1 < args.len() {
                    include_mem = Some(args[i + 1].parse().unwrap_or(10_000));
                    i += 2;
                } else {
                    eprintln!("Error: --include-mem requires number of batches");
                    std::process::exit(1);
                }
            }
            "--help" | "-h" => {
                print_usage(&args[0]);
                std::process::exit(0);
            }
            arg if arg.starts_with("--") => {
                eprintln!("Unknown argument: {}", arg);
                print_usage(&args[0]);
                std::process::exit(1);
            }
            _ => {
                // Positional argument - treat as file path
                file_paths.push(args[i].clone());
                i += 1;
            }
        }
    }

    // Define batch size for in-memory data generation
    let batch_size = 1;

    // Build list of data formats
    let mut data_formats = Vec::new();
    
    // Add in-memory if requested
    if let Some(num_batches) = include_mem {
        data_formats.push(DataFormat::InMemory { num_batches });
    }
    
    // Process file paths if provided
    if !file_paths.is_empty() {
        match process_file_paths(file_paths) {
            Ok(mut formats) => {
                data_formats.append(&mut formats);
            }
            Err(e) => {
                eprintln!("Error: {}", e);
                std::process::exit(1);
            }
        }
    }
    
    // For each unique data size from file formats, add corresponding in-memory format
    let file_based_formats: Vec<_> = data_formats.iter()
        .filter(|f| !matches!(f, DataFormat::InMemory { .. }))
        .collect();
    
    if !file_based_formats.is_empty() {
        use std::collections::HashSet;
        let mut unique_batch_counts = HashSet::new();
        
        // Collect unique batch counts from file formats
        for format in file_based_formats {
            let row_count = format.row_count();
            let num_batches = row_count / batch_size;
            unique_batch_counts.insert(num_batches);
        }
        
        // Add in-memory formats for each unique size
        for num_batches in unique_batch_counts {
            // Only add if we don't already have an in-memory format with this size
            let already_exists = data_formats.iter().any(|f| {
                matches!(f, DataFormat::InMemory { num_batches: n } if *n == num_batches)
            });
            
            if !already_exists {
                data_formats.push(DataFormat::InMemory { num_batches });
            }
        }
    }
    
    // Validate we have at least one data format
    if data_formats.is_empty() {
        eprintln!("Error: No data sources specified. Provide file paths or use --include-mem");
        eprintln!();
        print_usage(&args[0]);
        std::process::exit(1);
    }

    println!("╔═══════════════════════════════════════════════════════════════════════════╗");
    println!("║                        PARTIQL BENCHMARK SUITE                            ║");
    println!("╚═══════════════════════════════════════════════════════════════════════════╝");
    println!();
    println!("Configuration:");
    println!("  Iterations per benchmark: {}", iterations);
    println!("  Warmup iterations:        {}", warmup_iterations);
    println!("  Data sources:             {}", data_formats.len());
    for format in &data_formats {
        println!("    - {} ({} rows)", format.name(), format_number(format.row_count()));
    }
    println!();

    // Define benchmark queries
    let queries = vec![
        // BenchmarkQuery {
        //     name: "Simple Projection".to_string(),
        //     query: "SELECT a FROM ~input~".to_string(),
        // },
        BenchmarkQuery {
            name: "Simple Projection with filter".to_string(),
            query: "SELECT a, b FROM ~input~ WHERE ((a - a + b - b + a - a + b - b) + a % 100000) = 0".to_string(),
        },
    ];

    let mut all_results = Vec::new();

    // Run benchmarks for each query
    for query in &queries {
        println!("═══════════════════════════════════════════════════════════════════════════");
        println!("Query: {}", query.query);
        println!("═══════════════════════════════════════════════════════════════════════════");
        println!();

        // Legacy engine: all formats
        println!("Running Legacy Engine benchmarks...");
        for format in &data_formats {
            print!("  {} with {} ({} rows)... ", query.name, format.name(), format_number(format.row_count()));
            io::stdout().flush().unwrap();
            let result = run_legacy_benchmark(
                query,
                format,
                iterations,
                warmup_iterations,
            );
            if result.error.is_none() {
                println!("✓ (avg: {:.2}ms)", result.avg_ms);
            } else {
                println!("✗ ({})", result.error.as_ref().unwrap());
            }
            all_results.push(result);
        }

        println!();
        println!("Running Vectorized-1 Engine benchmarks...");
        // Vectorized engine with batch size 1: all formats
        for format in &data_formats {
            print!("  {} with {} ({} rows)... ", query.name, format.name(), format_number(format.row_count()));
            io::stdout().flush().unwrap();
            let result = run_vectorized_benchmark(
                query,
                format,
                1, // batch_size
                iterations,
                warmup_iterations,
            );
            if result.error.is_none() {
                println!("✓ (avg: {:.2}ms)", result.avg_ms);
            } else {
                println!("✗ ({})", result.error.as_ref().unwrap());
            }
            all_results.push(result);
        }

        println!();
        println!("Running Vectorized-1024 Engine benchmarks...");
        // Vectorized engine with batch size 1024: all formats
        for format in &data_formats {
            print!("  {} with {} ({} rows)... ", query.name, format.name(), format_number(format.row_count()));
            io::stdout().flush().unwrap();
            let result = run_vectorized_benchmark(
                query,
                format,
                1024, // batch_size
                iterations,
                warmup_iterations,
            );
            if result.error.is_none() {
                println!("✓ (avg: {:.2}ms)", result.avg_ms);
            } else {
                println!("✗ ({})", result.error.as_ref().unwrap());
            }
            all_results.push(result);
        }

        println!();
        println!("Running Hybrid Engine benchmarks...");
        for format in &data_formats {
            print!("  {} with {} ({} rows)... ", query.name, format.name(), format_number(format.row_count()));
            io::stdout().flush().unwrap();
            let result = run_hybrid_benchmark(
                query,
                format,
                iterations,
                warmup_iterations,
            );
            if result.error.is_none() {
                println!("✓ (avg: {:.2}ms)", result.avg_ms);
            } else {
                println!("✗ ({})", result.error.as_ref().unwrap());
            }
            all_results.push(result);
        }

        println!();
    }

    // Print results table
    print_results_table(&all_results);
}

fn run_legacy_benchmark(
    query: &BenchmarkQuery,
    format: &DataFormat,
    iterations: usize,
    warmup_iterations: usize,
) -> BenchmarkResult {
    let non_vec_query = query.query.replace("~input~", "data()");
    
    // Batch size for legacy engine (always 1)
    let batch_size = 1;
    
    // Calculate total rows and get path
    let (total_rows, path) = match format {
        DataFormat::InMemory { num_batches } => {
            (batch_size * num_batches, None)
        }
        DataFormat::Ion { path } => {
            (count_rows_from_file("ion", path), Some(path.clone()))
        }
        DataFormat::IonBinary { path } => {
            (count_rows_from_file("ionb", path), Some(path.clone()))
        }
        DataFormat::Arrow { path } => {
            (count_rows_from_file("arrow", path), Some(path.clone()))
        }
        DataFormat::Parquet { path } => {
            (count_rows_from_file("parquet", path), Some(path.clone()))
        }
    };

    // Set environment variable for legacy evaluator
    std::env::set_var("TOTAL_ROWS", total_rows.to_string());

    // Create catalog
    let catalog = create_catalog(format.format_str().to_string(), path);

    // Parse and compile (once, outside the benchmark loop)
    let parsed = match parse(&non_vec_query) {
        Ok(p) => p,
        Err(e) => {
            return BenchmarkResult {
                query_name: query.name.clone(),
                engine: "Legacy".to_string(),
                data_format: format.name().to_string(),
                data_source_rows: total_rows,
                output_rows: 0,
                min_ms: 0.0,
                max_ms: 0.0,
                avg_ms: 0.0,
                iterations: 0,
                error: Some(format!("Parse error: {:?}", e)),
            };
        }
    };

    let logical = match lower(&*catalog, &parsed) {
        Ok(l) => l,
        Err(e) => {
            return BenchmarkResult {
                query_name: query.name.clone(),
                engine: "Legacy".to_string(),
                data_format: format.name().to_string(),
                data_source_rows: total_rows,
                output_rows: 0,
                min_ms: 0.0,
                max_ms: 0.0,
                avg_ms: 0.0,
                iterations: 0,
                error: Some(format!("Lower error: {:?}", e)),
            };
        }
    };

    let plan = match compile(EvaluationMode::Permissive, &*catalog, logical) {
        Ok(p) => p,
        Err(e) => {
            return BenchmarkResult {
                query_name: query.name.clone(),
                engine: "Legacy".to_string(),
                data_format: format.name().to_string(),
                data_source_rows: total_rows,
                output_rows: 0,
                min_ms: 0.0,
                max_ms: 0.0,
                avg_ms: 0.0,
                iterations: 0,
                error: Some(format!("Compile error: {:?}", e)),
            };
        }
    };

    // Warmup iterations
    for _ in 0..warmup_iterations {
        let bindings = MapBindings::default();
        let sys = partiql_catalog::context::SystemContext {
            now: DateTime::from_system_now_utc(),
        };
        let ctx = BasicContext::new(bindings, sys);
        
        if let Err(_) = plan.execute(&ctx) {
            // Ignore warmup errors
        }
    }

    // Benchmark iterations
    let mut timings = Vec::new();
    let mut row_count = 0;

    for _ in 0..iterations {
        let bindings = MapBindings::default();
        let sys = partiql_catalog::context::SystemContext {
            now: DateTime::from_system_now_utc(),
        };
        let ctx = BasicContext::new(bindings, sys);

        let start = Instant::now();
        match plan.execute(&ctx) {
            Ok(evaluated) => {
                match evaluated.result {
                    Value::Bag(bag) => {
                        row_count = bag.len();
                    }
                    _ => {
                        row_count = 1;
                    }
                }
            }
            Err(e) => {
                return BenchmarkResult {
                    query_name: query.name.clone(),
                    engine: "Legacy".to_string(),
                    data_format: format.name().to_string(),
                    data_source_rows: total_rows,
                    output_rows: 0,
                    min_ms: 0.0,
                    max_ms: 0.0,
                    avg_ms: 0.0,
                    iterations: 0,
                    error: Some(format!("Execution error: {:?}", e)),
                };
            }
        }
        let elapsed = start.elapsed();
        timings.push(elapsed);
    }

    let (min_ms, max_ms, avg_ms) = calculate_stats(&timings);

    BenchmarkResult {
        query_name: query.name.clone(),
        engine: "Legacy".to_string(),
        data_format: format.name().to_string(),
        data_source_rows: total_rows,
        output_rows: row_count,
        min_ms,
        max_ms,
        avg_ms,
        iterations,
        error: None,
    }
}

fn run_vectorized_benchmark(
    query: &BenchmarkQuery,
    format: &DataFormat,
    batch_size: usize,
    iterations: usize,
    warmup_iterations: usize,
) -> BenchmarkResult {
    let vec_query = query.query.replace("~input~", "data");
    
    // Get the data source size
    let data_source_rows = format.row_count();

    // Engine name includes batch size
    let engine_name = format!("Vectorized-{}", batch_size);

    // Create catalog (needed for parsing/lowering)
    let catalog = create_catalog("mem".to_string(), None);

    // Parse and lower (once, outside the benchmark loop)
    let parsed = match parse(&vec_query) {
        Ok(p) => p,
        Err(e) => {
            return BenchmarkResult {
                query_name: query.name.clone(),
                engine: engine_name,
                data_format: format.name().to_string(),
                data_source_rows,
                output_rows: 0,
                min_ms: 0.0,
                max_ms: 0.0,
                avg_ms: 0.0,
                iterations: 0,
                error: Some(format!("Parse error: {:?}", e)),
            };
        }
    };

    let logical = match lower(&*catalog, &parsed) {
        Ok(l) => l,
        Err(e) => {
            return BenchmarkResult {
                query_name: query.name.clone(),
                engine: engine_name,
                data_format: format.name().to_string(),
                data_source_rows,
                output_rows: 0,
                min_ms: 0.0,
                max_ms: 0.0,
                avg_ms: 0.0,
                iterations: 0,
                error: Some(format!("Lower error: {:?}", e)),
            };
        }
    };

    // Warmup iterations
    for _ in 0..warmup_iterations {
        let mut plan = compile_vectorized(&logical, format, batch_size);
        for batch_result in plan.execute() {
            if let Err(_) = batch_result {
                // Ignore warmup errors
                break;
            }
        }
    }

    // Benchmark iterations
    let mut timings = Vec::new();
    let mut total_rows = 0;

    for _ in 0..iterations {
        let mut plan = compile_vectorized(&logical, format, batch_size);

        let start = Instant::now();
        let mut row_count = 0;

        for batch_result in plan.execute() {
            match batch_result {
                Ok(batch) => {
                    let batch_row_count = if let Some(selection) = batch.selection() {
                        selection.indices.len()
                    } else {
                        batch.row_count()
                    };
                    row_count += batch_row_count;
                }
                Err(e) => {
                    return BenchmarkResult {
                        query_name: query.name.clone(),
                        engine: engine_name.clone(),
                        data_format: format.name().to_string(),
                        data_source_rows,
                        output_rows: 0,
                        min_ms: 0.0,
                        max_ms: 0.0,
                        avg_ms: 0.0,
                        iterations: 0,
                        error: Some(format!("Execution error: {:?}", e)),
                    };
                }
            }
        }

        let elapsed = start.elapsed();
        timings.push(elapsed);
        total_rows = row_count;
    }

    let (min_ms, max_ms, avg_ms) = calculate_stats(&timings);

    BenchmarkResult {
        query_name: query.name.clone(),
        engine: engine_name,
        data_format: format.name().to_string(),
        data_source_rows,
        output_rows: total_rows,
        min_ms,
        max_ms,
        avg_ms,
        iterations,
        error: None,
    }
}

fn run_hybrid_benchmark(
    query: &BenchmarkQuery,
    format: &DataFormat,
    iterations: usize,
    warmup_iterations: usize,
) -> BenchmarkResult {
    let hybrid_query = query.query.replace("~input~", "data");
    let data_source_rows = format.row_count();
    let catalog = create_catalog("mem".to_string(), None);

    let parsed = match parse(&hybrid_query) {
        Ok(p) => p,
        Err(e) => {
            return BenchmarkResult {
                query_name: query.name.clone(),
                engine: "Hybrid".to_string(),
                data_format: format.name().to_string(),
                data_source_rows,
                output_rows: 0,
                min_ms: 0.0,
                max_ms: 0.0,
                avg_ms: 0.0,
                iterations: 0,
                error: Some(format!("Parse error: {:?}", e)),
            };
        }
    };

    let logical = match lower(&*catalog, &parsed) {
        Ok(l) => l,
        Err(e) => {
            return BenchmarkResult {
                query_name: query.name.clone(),
                engine: "Hybrid".to_string(),
                data_format: format.name().to_string(),
                data_source_rows,
                output_rows: 0,
                min_ms: 0.0,
                max_ms: 0.0,
                avg_ms: 0.0,
                iterations: 0,
                error: Some(format!("Lower error: {:?}", e)),
            };
        }
    };

    // Compile once outside the timing loop
    let mut vm = match compile_hybrid(&logical, format, data_source_rows) {
        Ok(v) => v,
        Err(e) => {
            return BenchmarkResult {
                query_name: query.name.clone(),
                engine: "Hybrid".to_string(),
                data_format: format.name().to_string(),
                data_source_rows,
                output_rows: 0,
                min_ms: 0.0,
                max_ms: 0.0,
                avg_ms: 0.0,
                iterations: 0,
                error: Some(format!("Compile error: {:?}", e)),
            };
        }
    };

    // Warmup iterations
    for _ in 0..warmup_iterations {
        while let Ok(Some(_row)) = vm.next_row() {}
        vm.reset().ok();
    }

    let mut timings = Vec::new();
    let mut total_rows = 0;

    for _ in 0..iterations {
        let start = Instant::now();
        let mut row_count = 0;
        loop {
            match vm.next_row() {
                Ok(Some(_row)) => row_count += 1,
                Ok(None) => break,
                Err(e) => {
                    return BenchmarkResult {
                        query_name: query.name.clone(),
                        engine: "Hybrid".to_string(),
                        data_format: format.name().to_string(),
                        data_source_rows,
                        output_rows: 0,
                        min_ms: 0.0,
                        max_ms: 0.0,
                        avg_ms: 0.0,
                        iterations: 0,
                        error: Some(format!("Execution error: {:?}", e)),
                    };
                }
            }
        }

        let elapsed = start.elapsed();
        timings.push(elapsed);
        total_rows = row_count;
        
        // Reset for next iteration
        if let Err(e) = vm.reset() {
            return BenchmarkResult {
                query_name: query.name.clone(),
                engine: "Hybrid".to_string(),
                data_format: format.name().to_string(),
                data_source_rows,
                output_rows: 0,
                min_ms: 0.0,
                max_ms: 0.0,
                avg_ms: 0.0,
                iterations: 0,
                error: Some(format!("Reset error: {:?}", e)),
            };
        }
    }

    let (min_ms, max_ms, avg_ms) = calculate_stats(&timings);

    BenchmarkResult {
        query_name: query.name.clone(),
        engine: "Hybrid".to_string(),
        data_format: format.name().to_string(),
        data_source_rows,
        output_rows: total_rows,
        min_ms,
        max_ms,
        avg_ms,
        iterations,
        error: None,
    }
}

fn compile_vectorized(
    logical: &partiql_logical::LogicalPlan<partiql_logical::BindingsOp>,
    format: &DataFormat,
    batch_size: usize,
) -> partiql_eval_vectorized::VectorizedPlan {
    use partiql_eval_vectorized::reader::{ArrowReader, InMemoryGeneratedReader, PIonReader, PIonTextReader, ParquetReader};

    let reader: Box<dyn partiql_eval_vectorized::BatchReader> = match format {
        DataFormat::InMemory { num_batches } => {
            Box::new(InMemoryGeneratedReader::with_config(batch_size, *num_batches))
        }
        DataFormat::Arrow { path } => {
            Box::new(ArrowReader::from_file(path, batch_size).expect("Failed to create ArrowReader"))
        }
        DataFormat::Parquet { path } => {
            Box::new(ParquetReader::from_file(path, batch_size).expect("Failed to create ParquetReader"))
        }
        DataFormat::Ion { path } => {
            // Text Ion - uses string-based field names
            Box::new(PIonTextReader::from_ion_file(path, batch_size).expect("Failed to create IonTextReader"))
        }
        DataFormat::IonBinary { path } => {
            // Binary Ion - uses symbol IDs for optimal performance
            Box::new(PIonReader::from_ion_file(path, batch_size).expect("Failed to create IonBinaryReader"))
        }
    };

    let context = partiql_eval_vectorized::CompilerContext::new()
        .with_data_source("data".to_string(), reader);

    let mut compiler = partiql_eval_vectorized::Compiler::new(context);
    compiler
        .compile(logical)
        .expect("Vectorized compilation failed")
}

fn compile_hybrid(
    logical: &partiql_logical::LogicalPlan<partiql_logical::BindingsOp>,
    format: &DataFormat,
    total_rows: usize,
) -> partiql_eval::engine::Result<partiql_eval::engine::PartiQLVM> {
    let provider = HybridScanProvider::new(format, total_rows);
    let compiler = PlanCompiler::new(&provider);
    let compiled = compiler.compile(logical)?;
    compiler.instantiate(compiled, None)
}

struct HybridScanProvider {
    data_source: String,
    data_path: Option<String>,
    num_rows: Option<usize>,
}

impl HybridScanProvider {
    fn new(format: &DataFormat, total_rows: usize) -> Self {
        match format {
            DataFormat::InMemory { .. } => HybridScanProvider {
                data_source: "mem".to_string(),
                data_path: None,
                num_rows: Some(total_rows),
            },
            DataFormat::Ion { path } => HybridScanProvider {
                data_source: "ion".to_string(),
                data_path: Some(path.clone()),
                num_rows: None,
            },
            DataFormat::IonBinary { path } => HybridScanProvider {
                data_source: "ionb".to_string(),
                data_path: Some(path.clone()),
                num_rows: None,
            },
            _ => HybridScanProvider {
                data_source: "unsupported".to_string(),
                data_path: None,
                num_rows: None,
            },
        }
    }
}

impl ScanProvider for HybridScanProvider {
    fn reader_factory(&self, _scan: &Scan) -> partiql_eval::engine::Result<ReaderFactoryEnum> {
        match self.data_source.as_str() {
            "mem" => {
                let num_rows = self.num_rows.ok_or_else(|| {
                    partiql_eval::engine::EngineError::ReaderError(
                        "num_rows required for mem source".to_string(),
                    )
                })?;
                Ok(ReaderFactoryEnum::InMem(partiql_eval::engine::InMemGeneratedReaderFactory::new(num_rows)))
            }
            "ion" | "ionb" => {
                let path = self.data_path.clone().ok_or_else(|| {
                    partiql_eval::engine::EngineError::ReaderError(
                        "ion path required".to_string(),
                    )
                })?;
                Ok(ReaderFactoryEnum::Ion(partiql_eval::engine::IonRowReaderFactory::new(path)))
            }
            _ => Err(partiql_eval::engine::EngineError::ReaderError(
                "Hybrid supports mem/ion/ionb only".to_string(),
            )),
        }
    }
}

fn calculate_stats(timings: &[Duration]) -> (f64, f64, f64) {
    let min_ms = timings.iter().min().unwrap().as_secs_f64() * 1000.0;
    let max_ms = timings.iter().max().unwrap().as_secs_f64() * 1000.0;
    let sum_ms: f64 = timings.iter().map(|d| d.as_secs_f64() * 1000.0).sum();
    let avg_ms = sum_ms / timings.len() as f64;
    (min_ms, max_ms, avg_ms)
}

fn print_results_table(results: &[BenchmarkResult]) {
    // Filter out results with errors
    let successful_results: Vec<_> = results.iter().filter(|r| r.error.is_none()).collect();

    if successful_results.is_empty() {
        println!("No successful benchmark results to display.");
        return;
    }

    println!("═══════════════════════════════════════════════════════════════════════════");
    println!("                            BENCHMARK RESULTS                              ");
    println!("═══════════════════════════════════════════════════════════════════════════");
    println!();

    // Group by query name
    let mut current_query = "";
    for result in &successful_results {
        if result.query_name != current_query {
            if !current_query.is_empty() {
                println!();
            }
            println!("Query: {}", result.query_name);
            println!();
            println!("┌──────────────┬─────────────┬─────────────┬─────────────┬──────────────┬──────────────┬──────────────┬────────────┐");
            println!("│ Engine       │ Data Format │ Source Rows │ Output Rows │ Min(ms)      │ Max(ms)      │ Avg(ms)      │ Iterations │");
            println!("├──────────────┼─────────────┼─────────────┼─────────────┼──────────────┼──────────────┼──────────────┼────────────┤");
            current_query = &result.query_name;
        }

        println!(
            "│ {:12} │ {:11} │ {:>11} │ {:>11} │ {:>12.2} │ {:>12.2} │ {:>12.2} │ {:>10} │",
            result.engine,
            result.formatted_data_format(),
            format_number(result.data_source_rows),
            format_number(result.output_rows),
            result.min_ms,
            result.max_ms,
            result.avg_ms,
            result.iterations
        );
    }
    println!("└──────────────┴─────────────┴─────────────┴─────────────┴──────────────┴──────────────┴──────────────┴────────────┘");
    println!();

    // Calculate and display speedups
    print_speedup_analysis(&successful_results);

    // Display errors if any
    let errors: Vec<_> = results.iter().filter(|r| r.error.is_some()).collect();
    if !errors.is_empty() {
        println!();
        println!("═══════════════════════════════════════════════════════════════════════════");
        println!("                              ERRORS                                       ");
        println!("═══════════════════════════════════════════════════════════════════════════");
        for error_result in errors {
            println!(
                "- {} / {} / {}: {}",
                error_result.query_name,
                error_result.engine,
                error_result.data_format,
                error_result.error.as_ref().unwrap()
            );
        }
    }
}

fn print_speedup_analysis(results: &[&BenchmarkResult]) {
    println!();
    println!("═══════════════════════════════════════════════════════════════════════════");
    println!("                          SPEEDUP ANALYSIS                                 ");
    println!("═══════════════════════════════════════════════════════════════════════════");
    println!();

    // Section 1: Cross-Engine Comparisons (Same Storage Format)
    print_cross_engine_analysis(results);
    
    // Section 2: Cross-Format Comparisons (Same Engine)
    print_cross_format_analysis(results);
    
    // Section 3: Best-Case Scenarios
    print_best_case_analysis(results);
    
    // Section 4: Data Size Scaling (if applicable)
    if has_multiple_data_sizes(results) {
        print_scaling_analysis(results);
    }
    
    // Section 5: Speedup Matrix
    print_speedup_matrix(results);
    
    // Section 6: Format Comparison Table
    print_format_comparison_table(results);
}

/// Cross-Engine Comparisons: Same storage format, different engines
fn print_cross_engine_analysis(results: &[&BenchmarkResult]) {
    println!("1. Cross-Engine Comparisons (Same Storage Format)");
    println!("   Shows vectorization benefit for each storage format");
    println!();

    // Group by data size
    let mut sizes: Vec<usize> = results.iter().map(|r| r.data_source_rows).collect();
    sizes.sort();
    sizes.dedup();

    let multiple_sizes = sizes.len() > 1;

    for size in sizes {
        if multiple_sizes {
            println!("   Data Size: {} rows", format_number(size));
            println!();
        }

        let size_results: Vec<_> = results.iter().filter(|r| r.data_source_rows == size).copied().collect();

        // Group by data format
        let mut formats: Vec<&str> = size_results.iter()
            .map(|r| r.data_format.as_str())
            .collect();
        formats.sort();
        formats.dedup();

        let engine_pairs = [
            ("Legacy", "Vectorized-1"),
            ("Legacy", "Vectorized-1024"),
            ("Legacy", "Hybrid"),
            ("Hybrid", "Vectorized-1"),
            ("Hybrid", "Vectorized-1024"),
            ("Vectorized-1", "Vectorized-1024"),
        ];

        let mut found_any = false;
        for format in formats {
            for (left, right) in engine_pairs {
                let left_res = size_results
                    .iter()
                    .find(|r| r.engine == left && r.data_format == format);
                let right_res = size_results
                    .iter()
                    .find(|r| r.engine == right && r.data_format == format);

                if let (Some(l), Some(r)) = (left_res, right_res) {
                    if l.avg_ms > 0.0 && r.avg_ms > 0.0 {
                        let speedup = l.avg_ms / r.avg_ms;
                        let indent = if multiple_sizes { "     " } else { "   " };
                        println!(
                            "{}{} {} → {} {}: {:.2}x",
                            indent, left, format, right, format, speedup
                        );
                        found_any = true;
                    }
                }
            }
        }

        if !found_any {
            let indent = if multiple_sizes { "     " } else { "   " };
            println!("{}(No comparable engine pairs found)", indent);
        }
        
        if multiple_sizes {
            println!();
        }
    }
    
    if !multiple_sizes {
        println!();
    }
}

/// Cross-Format Comparisons: Same engine, different storage formats
fn print_cross_format_analysis(results: &[&BenchmarkResult]) {
    println!("2. Storage Format Impact (Same Engine)");
    println!("   Shows file format performance relative to slowest format");
    println!();

    // Group by data size
    let mut sizes: Vec<usize> = results.iter().map(|r| r.data_source_rows).collect();
    sizes.sort();
    sizes.dedup();

    let multiple_sizes = sizes.len() > 1;

    for size in sizes {
        if multiple_sizes {
            println!("   Data Size: {} rows", format_number(size));
            println!();
        }

        let size_results: Vec<_> = results.iter().filter(|r| r.data_source_rows == size).copied().collect();

        for engine in &["Legacy", "Vectorized-1", "Vectorized-1024", "Hybrid"] {
            // Filter to only file-based formats (exclude in-memory)
            let file_results: Vec<_> = size_results.iter()
                .filter(|r| r.engine == *engine && r.data_format != "In-Memory")
                .collect();
            
            if file_results.is_empty() {
                continue;
            }

            // Use slowest file-based format as baseline
            let baseline = file_results.iter()
                .max_by(|a, b| a.avg_ms.partial_cmp(&b.avg_ms).unwrap_or(std::cmp::Ordering::Equal))
                .copied();
            
            if let Some(baseline) = baseline {
                let mut found_any = false;
                let baseline_name = baseline.data_format.as_str();
                let indent = if multiple_sizes { "     " } else { "   " };
                println!("{}{} Engine (baseline: {}):", indent, engine, baseline_name);
                
                let other_formats: Vec<_> = file_results.iter()
                    .filter(|r| r.data_format != baseline_name)
                    .collect();

                for result in other_formats {
                    if baseline.avg_ms > 0.0 && result.avg_ms > 0.0 {
                        let speedup = baseline.avg_ms / result.avg_ms;
                        let overhead_pct = ((result.avg_ms - baseline.avg_ms) / baseline.avg_ms * 100.0).abs();
                        let detail_indent = if multiple_sizes { "       " } else { "     " };
                        
                        if speedup >= 1.0 {
                            println!("{}{} → {}: {:.2}x faster ({:.0}% overhead reduction)", 
                                     detail_indent, baseline_name, result.data_format, speedup, overhead_pct);
                        } else {
                            println!("{}{} → {}: {:.2}x slower ({:.0}% overhead increase)", 
                                     detail_indent, baseline_name, result.data_format, 1.0 / speedup, overhead_pct);
                        }
                        found_any = true;
                    }
                }
                
                if !found_any {
                    let detail_indent = if multiple_sizes { "       " } else { "     " };
                    println!("{}(Only {} available)", detail_indent, baseline_name);
                }
            }
        }
        
        if multiple_sizes {
            println!();
        }
    }
    
    if !multiple_sizes {
        println!();
    }
}

/// Best-Case Scenario Analysis
fn print_best_case_analysis(results: &[&BenchmarkResult]) {
    println!("3. Best-Case Scenarios");
    println!("   Optimal file format configurations grouped by data size");
    println!();

    // Group by data size (excluding in-memory)
    let mut size_groups: std::collections::HashMap<usize, Vec<&BenchmarkResult>> = 
        std::collections::HashMap::new();

    for result in results.iter() {
        if result.data_format != "In-Memory" {
            size_groups.entry(result.data_source_rows).or_insert_with(Vec::new).push(*result);
        }
    }

    if size_groups.is_empty() {
        println!("   (No file-based results available)");
        println!();
        return;
    }

    // Sort sizes
    let mut sizes: Vec<usize> = size_groups.keys().copied().collect();
    sizes.sort();

    // Analyze each data size separately
    for size in sizes {
        let size_results = &size_groups[&size];
        
        println!("   Data Size: {} rows", format_number(size));

        let engines = ["Legacy", "Vectorized-1", "Vectorized-1024", "Hybrid"];
        let mut best_by_engine: std::collections::HashMap<&str, BenchmarkResult> =
            std::collections::HashMap::new();

        for engine in engines {
            let engine_results: Vec<_> = size_results
                .iter()
                .filter(|r| r.engine == engine)
                .copied()
                .collect();
            if engine_results.is_empty() {
                continue;
            }
            let best = engine_results.iter().min_by(|a, b| {
                a.avg_ms.partial_cmp(&b.avg_ms).unwrap_or(std::cmp::Ordering::Equal)
            });
            let worst = engine_results.iter().max_by(|a, b| {
                a.avg_ms.partial_cmp(&b.avg_ms).unwrap_or(std::cmp::Ordering::Equal)
            });

            if let (Some(best), Some(worst)) = (best, worst) {
                best_by_engine.insert(engine, (*best).clone());
                if engine_results.len() > 1 {
                    let ratio = worst.avg_ms / best.avg_ms;
                    println!(
                        "     {} format range: {} (best) to {} (worst) = {:.2}x difference",
                        engine, best.data_format, worst.data_format, ratio
                    );
                }
            }
        }

        if let (Some(best_leg), Some(best_vec1)) =
            (best_by_engine.get("Legacy"), best_by_engine.get("Vectorized-1"))
        {
            let speedup = best_leg.avg_ms / best_vec1.avg_ms;
            println!(
                "     Best Legacy ({}) → Best Vectorized-1 ({}): {:.2}x",
                best_leg.data_format, best_vec1.data_format, speedup
            );
        }

        if let (Some(best_leg), Some(best_vec1024)) =
            (best_by_engine.get("Legacy"), best_by_engine.get("Vectorized-1024"))
        {
            let speedup = best_leg.avg_ms / best_vec1024.avg_ms;
            println!(
                "     Best Legacy ({}) → Best Vectorized-1024 ({}): {:.2}x",
                best_leg.data_format, best_vec1024.data_format, speedup
            );
        }

        if let (Some(best_leg), Some(best_hybrid)) =
            (best_by_engine.get("Legacy"), best_by_engine.get("Hybrid"))
        {
            let speedup = best_leg.avg_ms / best_hybrid.avg_ms;
            println!(
                "     Best Legacy ({}) → Best Hybrid ({}): {:.2}x",
                best_leg.data_format, best_hybrid.data_format, speedup
            );
        }

        if let (Some(best_hybrid), Some(best_vec1)) =
            (best_by_engine.get("Hybrid"), best_by_engine.get("Vectorized-1"))
        {
            let speedup = best_hybrid.avg_ms / best_vec1.avg_ms;
            println!(
                "     Best Hybrid ({}) → Best Vectorized-1 ({}): {:.2}x",
                best_hybrid.data_format, best_vec1.data_format, speedup
            );
        }

        if let (Some(best_hybrid), Some(best_vec1024)) =
            (best_by_engine.get("Hybrid"), best_by_engine.get("Vectorized-1024"))
        {
            let speedup = best_hybrid.avg_ms / best_vec1024.avg_ms;
            println!(
                "     Best Hybrid ({}) → Best Vectorized-1024 ({}): {:.2}x",
                best_hybrid.data_format, best_vec1024.data_format, speedup
            );
        }

        if let (Some(best_vec1), Some(best_vec1024)) =
            (best_by_engine.get("Vectorized-1"), best_by_engine.get("Vectorized-1024"))
        {
            let speedup = best_vec1.avg_ms / best_vec1024.avg_ms;
            println!(
                "     Best Vectorized-1 ({}) → Best Vectorized-1024 ({}): {:.2}x",
                best_vec1.data_format, best_vec1024.data_format, speedup
            );
        }

        println!();
    }
}

/// Data Size Scaling Analysis
fn print_scaling_analysis(results: &[&BenchmarkResult]) {
    println!("4. Data Size Scaling Analysis");
    println!("   How speedups change with data size");
    println!();

    // Group results by (engine, format) and then by data size
    let mut size_groups: std::collections::HashMap<(String, String), Vec<&BenchmarkResult>> = 
        std::collections::HashMap::new();

    for result in results {
        let key = (result.engine.clone(), result.data_format.clone());
        size_groups.entry(key).or_insert_with(Vec::new).push(*result);
    }

    // For each configuration that appears at multiple sizes
    for ((engine, format), mut group) in size_groups.iter_mut() {
        if group.len() > 1 {
            // Sort by row count
            group.sort_by_key(|r| r.data_source_rows);
            
            println!("   {} {}:", engine, format);
            for (i, result) in group.iter().enumerate() {
                let throughput = (result.data_source_rows as f64 / result.avg_ms) * 1000.0; // rows/sec
                println!("     {}M rows: {:.2}ms ({:.0} rows/sec)",
                         result.data_source_rows / 1_000_000, result.avg_ms, throughput);
                
                if i > 0 {
                    let prev = group[i - 1];
                    let size_ratio = result.data_source_rows as f64 / prev.data_source_rows as f64;
                    let time_ratio = result.avg_ms / prev.avg_ms;
                    let scaling_efficiency = size_ratio / time_ratio;
                    println!("       Scaling efficiency: {:.2}x", scaling_efficiency);
                }
            }
        }
    }

    println!();
}

/// Check if results contain multiple data sizes
fn has_multiple_data_sizes(results: &[&BenchmarkResult]) -> bool {
    let mut sizes: Vec<usize> = results.iter().map(|r| r.data_source_rows).collect();
    sizes.sort();
    sizes.dedup();
    sizes.len() > 1
}

/// Format comparison table showing Legacy vs other engine performance
fn print_format_comparison_table(results: &[&BenchmarkResult]) {
    println!();
    println!("6. Format Performance Comparison");
    println!("   Shows Legacy vs other engines by format");
    println!();

    // Group by data size
    let mut sizes: Vec<usize> = results.iter().map(|r| r.data_source_rows).collect();
    sizes.sort();
    sizes.dedup();

    let multiple_sizes = sizes.len() > 1;

    for size in sizes {
        if multiple_sizes {
            println!("   Data Size: {} rows", format_number(size));
            println!();
        }

        let size_results: Vec<_> = results.iter().filter(|r| r.data_source_rows == size).copied().collect();

        // Collect all formats that appear at this size
        let mut formats: Vec<&str> = size_results.iter()
            .map(|r| r.data_format.as_str())
            .collect();
        formats.sort();
        formats.dedup();

        if formats.is_empty() {
            let indent = if multiple_sizes { "     " } else { "   " };
            println!("{}(No results available)", indent);
            if multiple_sizes {
                println!();
            }
            continue;
        }

        let indent = if multiple_sizes { "     " } else { "   " };

        // Print table header
        println!("{}┌─────────────┬──────────────────┬──────────────────┬──────────────────┬──────────────────┬──────────────────┬──────────────────┬──────────────────┐", indent);
        println!("{}│ Format      │ Legacy Avg (ms)  │ Vec-1 Avg (ms)   │ Vec-1024 Avg(ms) │ Hybrid Avg (ms)  │ Vec-1 Speedup    │ Vec-1024 Speedup │ Hybrid Speedup   │", indent);
        println!("{}├─────────────┼──────────────────┼──────────────────┼──────────────────┼──────────────────┼──────────────────┼──────────────────┼──────────────────┤", indent);

        // Print table rows
        for format in formats {
            let legacy = size_results.iter().find(|r| r.engine == "Legacy" && r.data_format == format);
            let vec1 = size_results.iter().find(|r| r.engine == "Vectorized-1" && r.data_format == format);
            let vec1024 = size_results.iter().find(|r| r.engine == "Vectorized-1024" && r.data_format == format);
            let hybrid = size_results.iter().find(|r| r.engine == "Hybrid" && r.data_format == format);

            let legacy_str = if let Some(leg) = legacy {
                format!("{:.2}", leg.avg_ms)
            } else {
                "N/A".to_string()
            };

            let vec1_str = if let Some(v) = vec1 {
                format!("{:.2}", v.avg_ms)
            } else {
                "N/A".to_string()
            };

            let vec1024_str = if let Some(v) = vec1024 {
                format!("{:.2}", v.avg_ms)
            } else {
                "N/A".to_string()
            };

            let hybrid_str = if let Some(hyb) = hybrid {
                format!("{:.2}", hyb.avg_ms)
            } else {
                "N/A".to_string()
            };

            let vec1_speedup_str = if let (Some(leg), Some(v)) = (legacy, vec1) {
                if leg.avg_ms > 0.0 && v.avg_ms > 0.0 {
                    let speedup = leg.avg_ms / v.avg_ms;
                    format_speedup(speedup)
                } else {
                    "N/A".to_string()
                }
            } else {
                "N/A".to_string()
            };

            let vec1024_speedup_str = if let (Some(leg), Some(v)) = (legacy, vec1024) {
                if leg.avg_ms > 0.0 && v.avg_ms > 0.0 {
                    let speedup = leg.avg_ms / v.avg_ms;
                    format_speedup(speedup)
                } else {
                    "N/A".to_string()
                }
            } else {
                "N/A".to_string()
            };

            let hybrid_speedup_str = if let (Some(leg), Some(hyb)) = (legacy, hybrid) {
                if leg.avg_ms > 0.0 && hyb.avg_ms > 0.0 {
                    let speedup = leg.avg_ms / hyb.avg_ms;
                    format_speedup(speedup)
                } else {
                    "N/A".to_string()
                }
            } else {
                "N/A".to_string()
            };

            println!("{}│ {:11} │ {:>16} │ {:>16} │ {:>16} │ {:>16} │ {:>16} │ {:>16} │ {:>16} │",
                     indent,
                     format,
                     legacy_str,
                     vec1_str,
                     vec1024_str,
                     hybrid_str,
                     vec1_speedup_str,
                     vec1024_speedup_str,
                     hybrid_speedup_str);
        }

        println!("{}└─────────────┴──────────────────┴──────────────────┴──────────────────┴──────────────────┴──────────────────┴──────────────────┴──────────────────┘", indent);

        if multiple_sizes {
            println!();
        }
    }

    if !multiple_sizes {
        println!();
    }
}

/// Generate comprehensive speedup matrix
fn print_speedup_matrix(results: &[&BenchmarkResult]) {
    println!("5. Comprehensive Speedup Matrix");
    println!("   Legacy (rows) → Other Engines (columns) speedup comparisons");
    println!();

    // Group by data size
    let mut sizes: Vec<usize> = results.iter().map(|r| r.data_source_rows).collect();
    sizes.sort();
    sizes.dedup();

    let multiple_sizes = sizes.len() > 1;

    for size in sizes {
        if multiple_sizes {
            println!("   Data Size: {} rows", format_number(size));
            println!();
        }

        let size_results: Vec<_> = results.iter().filter(|r| r.data_source_rows == size).copied().collect();

        // Separate Legacy configurations and other engine configurations for this size
        let mut legacy_configs: Vec<&str> = size_results.iter()
            .filter(|r| r.engine == "Legacy")
            .map(|r| r.data_format.as_str())
            .collect();
        legacy_configs.sort();
        legacy_configs.dedup();

        let mut other_configs: Vec<(String, String)> = size_results
            .iter()
            .filter(|r| r.engine != "Legacy")
            .map(|r| (r.engine.clone(), r.data_format.clone()))
            .collect();
        other_configs.sort();
        other_configs.dedup();

        if legacy_configs.is_empty() || other_configs.is_empty() {
            let indent = if multiple_sizes { "     " } else { "   " };
            println!("{}(Need both Legacy and other engine results for matrix)", indent);
            if multiple_sizes {
                println!();
            }
            continue;
        }

        // Create labels
        let create_label = |engine: &str, fmt: &str| -> String {
            let fmt_label = match fmt {
                "In-Memory" => "Mem",
                "Ion" => "Ion",
                "Ion Binary" => "IonB",
                "Arrow" => "Arr",
                "Parquet" => "Pqt",
                other => other,
            };
            let eng_label = match engine {
                "Vectorized" => "Vec",
                "Hybrid" => "Hy",
                other => other,
            };
            format!("{}-{}", eng_label, fmt_label)
        };

        let row_labels: Vec<String> = legacy_configs.iter()
            .map(|fmt| create_label("Legacy", fmt))
            .collect();
        let col_labels: Vec<String> = other_configs.iter()
            .map(|(engine, fmt)| create_label(engine, fmt))
            .collect();

        let max_label_len = row_labels.iter().map(|l| l.len()).max().unwrap_or(10);
        let col_width = 8;
        let indent = if multiple_sizes { "     " } else { "   " };

        // Print header
        print!("{}{:width$}", indent, "", width = max_label_len);
        for label in &col_labels {
            print!("│{:^width$}", truncate_label(label, col_width), width = col_width);
        }
        println!("│");

        // Print separator
        print!("{}{}", indent, "─".repeat(max_label_len));
        for _ in &col_labels {
            print!("┼{}", "─".repeat(col_width));
        }
        println!("┤");

        // Print matrix rows
        for (i, legacy_fmt) in legacy_configs.iter().enumerate() {
            print!("{}{:width$}", indent, row_labels[i], width = max_label_len);
            
            let legacy_result = size_results.iter().find(|r| 
                r.engine == "Legacy" && 
                r.data_format == *legacy_fmt
            );

            for (engine, fmt) in &other_configs {
                let other_result = size_results.iter().find(|r| 
                    r.engine == *engine && 
                    r.data_format == *fmt
                );

                let speedup_str = if let (Some(legacy), Some(other)) = (legacy_result, other_result) {
                    if legacy.avg_ms > 0.0 && other.avg_ms > 0.0 {
                        let speedup = legacy.avg_ms / other.avg_ms;
                        format_speedup(speedup)
                    } else {
                        "N/A".to_string()
                    }
                } else {
                    "N/A".to_string()
                };

                print!("│{:>width$}", speedup_str, width = col_width);
            }
            println!("│");
        }

        if multiple_sizes {
            println!();
        }
    }

    // Print legend
    if !multiple_sizes {
        println!();
    }
    println!("   Legend: >1.0x = Legacy faster, <1.0x = Other engine faster");
    println!("   Rows = Legacy configurations, Columns = Other engine configurations");
    println!("   Abbreviations: Vec=Vectorized, Hy=Hybrid, Mem=In-Memory, IonB=Ion Binary, Arr=Arrow, Pqt=Parquet");
}

/// Format speedup value with appropriate precision
fn format_speedup(speedup: f64) -> String {
    if speedup >= 10.0 {
        format!("{:.1}x", speedup)
    } else if speedup >= 1.0 {
        format!("{:.2}x", speedup)
    } else {
        format!("{:.2}x", speedup)
    }
}

/// Truncate label to fit column width
fn truncate_label(label: &str, max_len: usize) -> String {
    if label.len() <= max_len {
        label.to_string()
    } else {
        format!("{}…", &label[..max_len - 1])
    }
}

fn format_number(n: usize) -> String {
    if n < 1_000 {
        format!("{}", n)
    } else if n < 1_000_000 {
        format!("{}K", n / 1_000)
    } else {
        format!("{}M", n / 1_000_000)
    }
}

fn print_usage(program: &str) {
    println!("Usage: {} [OPTIONS] [FILES...]", program);
    println!();
    println!("Options:");
    println!("  --include-mem <N>       Include in-memory benchmarks with N batches");
    println!("                          (in-memory formats are also auto-added for file sizes)");
    println!("  --iterations <N>        Number of iterations per benchmark (default: 5)");
    println!("  --warmup <N>            Number of warmup iterations (default: 1)");
    println!("  --help, -h              Show this help message");
    println!();
    println!("Arguments:");
    println!("  FILES                   Data files to benchmark (shell glob expansion supported)");
    println!("                          Supported formats: .ion, .10n, .arrow, .parquet");
    println!();
    println!("At least one data file or --include-mem must be specified.");
    println!();
    println!("Examples:");
    println!("  # Benchmark only in-memory with 10K batches");
    println!("  {} --include-mem 10000", program);
    println!();
    println!("  # Benchmark specific files (shell expands the glob)");
    println!("  {} test_data/data_b1024_n10000.*", program);
    println!();
    println!("  # Benchmark multiple files with brace expansion");
    println!("  {} test_data/data_b1024_n{{1000,10000}}.*", program);
    println!();
    println!("  # Benchmark all Ion files");
    println!("  {} test_data/*.ion", program);
    println!();
    println!("  # Combine in-memory + specific files with custom iterations");
    println!("  {} --include-mem 5000 --iterations 10 test_data/*.parquet", program);
    println!();
    println!("  # Mix different file types");
    println!("  {} test_data/data_b1024_n1000.ion test_data/data_b1024_n10000.parquet", program);
}
