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
use partiql_value::{DateTime, Value};
use std::time::Instant;

/// Engine type to profile
#[derive(Clone, Copy, PartialEq, Eq)]
enum Engine {
    Legacy,
    Vectorized1,
    Vectorized1024,
    Hybrid,
}

impl Engine {
    fn name(&self) -> &str {
        match self {
            Engine::Legacy => "legacy",
            Engine::Vectorized1 => "vectorized-1",
            Engine::Vectorized1024 => "vectorized-1024",
            Engine::Hybrid => "hybrid",
        }
    }
}

/// Data format for profiling
#[derive(Clone)]
enum DataFormat {
    InMemory { num_rows: usize },
    Ion { path: String },
    IonBinary { path: String },
}

impl DataFormat {
    fn name(&self) -> &str {
        match self {
            DataFormat::InMemory { .. } => "mem",
            DataFormat::Ion { .. } => "ion",
            DataFormat::IonBinary { .. } => "ionb",
        }
    }

    fn row_count(&self) -> usize {
        match self {
            DataFormat::InMemory { num_rows } => *num_rows,
            DataFormat::Ion { path } => count_rows_from_file("ion", path),
            DataFormat::IonBinary { path } => count_rows_from_file("ionb", path),
        }
    }
}

fn main() {
    // Parse command line arguments
    let args: Vec<String> = std::env::args().collect();
    
    let mut engine = Engine::Legacy;
    let mut data_format: Option<DataFormat> = None;
    let mut query = "SELECT a, b FROM ~input~ WHERE a % 100000 = 0".to_string();
    let mut iterations = 100; // More iterations for better profiling

    // Parse arguments
    let mut i = 1;
    while i < args.len() {
        match args[i].as_str() {
            "--engine" => {
                if i + 1 < args.len() {
                    engine = match args[i + 1].as_str() {
                        "legacy" => Engine::Legacy,
                        "vectorized-1" => Engine::Vectorized1,
                        "vectorized-1024" => Engine::Vectorized1024,
                        "hybrid" => Engine::Hybrid,
                        other => {
                            eprintln!("Unknown engine: {}", other);
                            print_usage(&args[0]);
                            std::process::exit(1);
                        }
                    };
                    i += 2;
                } else {
                    eprintln!("Error: --engine requires a value");
                    std::process::exit(1);
                }
            }
            "--data" => {
                if i + 1 < args.len() {
                    let data_arg = &args[i + 1];
                    if data_arg.starts_with("mem:") {
                        let num_rows: usize = data_arg[4..].parse().unwrap_or(1_000_000);
                        data_format = Some(DataFormat::InMemory { num_rows });
                    } else if data_arg.ends_with(".ion") {
                        data_format = Some(DataFormat::Ion { path: data_arg.clone() });
                    } else if data_arg.ends_with(".10n") {
                        data_format = Some(DataFormat::IonBinary { path: data_arg.clone() });
                    } else {
                        eprintln!("Error: Unsupported data format. Use mem:<rows>, *.ion, or *.10n");
                        std::process::exit(1);
                    }
                    i += 2;
                } else {
                    eprintln!("Error: --data requires a value");
                    std::process::exit(1);
                }
            }
            "--query" => {
                if i + 1 < args.len() {
                    query = args[i + 1].clone();
                    i += 2;
                } else {
                    eprintln!("Error: --query requires a value");
                    std::process::exit(1);
                }
            }
            "--iterations" => {
                if i + 1 < args.len() {
                    iterations = args[i + 1].parse().unwrap_or(100);
                    i += 2;
                } else {
                    eprintln!("Error: --iterations requires a value");
                    std::process::exit(1);
                }
            }
            "--help" | "-h" => {
                print_usage(&args[0]);
                std::process::exit(0);
            }
            _ => {
                eprintln!("Unknown argument: {}", args[i]);
                print_usage(&args[0]);
                std::process::exit(1);
            }
        }
    }

    // Default to in-memory with 1M rows if not specified
    let data_format = data_format.unwrap_or(DataFormat::InMemory { num_rows: 1_000_000 });

    println!("╔═══════════════════════════════════════════════════════════════════════════╗");
    println!("║                     PARTIQL PROFILING WORKLOAD                            ║");
    println!("╚═══════════════════════════════════════════════════════════════════════════╝");
    println!();
    println!("Configuration:");
    println!("  Engine:     {}", engine.name());
    println!("  Data:       {} ({} rows)", data_format.name(), format_number(data_format.row_count()));
    println!("  Query:      {}", query);
    println!("  Iterations: {}", iterations);
    println!();
    println!("Starting profiling workload...");
    println!();

    // Run the profiling workload
    let start = Instant::now();
    let total_rows = run_profile(&engine, &data_format, &query, iterations);
    let elapsed = start.elapsed();

    println!();
    println!("Profiling complete!");
    println!("  Total time:   {:.2}s", elapsed.as_secs_f64());
    println!("  Iterations:   {}", iterations);
    println!("  Rows/iter:    {}", format_number(total_rows));
    println!("  Rows/sec:     {}", format_number((total_rows as f64 * iterations as f64 / elapsed.as_secs_f64()) as usize));
}

fn run_profile(engine: &Engine, format: &DataFormat, query: &str, iterations: usize) -> usize {
    match engine {
        Engine::Legacy => run_legacy_profile(format, query, iterations),
        Engine::Vectorized1 => run_vectorized_profile(format, query, 1, iterations),
        Engine::Vectorized1024 => run_vectorized_profile(format, query, 1024, iterations),
        Engine::Hybrid => run_hybrid_profile(format, query, iterations),
    }
}

fn run_legacy_profile(format: &DataFormat, query: &str, iterations: usize) -> usize {
    let non_vec_query = query.replace("~input~", "data()");
    let total_rows = format.row_count();

    // Set environment variable for legacy evaluator
    std::env::set_var("TOTAL_ROWS", total_rows.to_string());

    // Create catalog
    let path = match format {
        DataFormat::InMemory { .. } => None,
        DataFormat::Ion { path } => Some(path.clone()),
        DataFormat::IonBinary { path } => Some(path.clone()),
    };
    let catalog = create_catalog(format.name().to_string(), path);

    // Parse and compile once
    let parsed = parse(&non_vec_query).expect("Parse failed");
    let logical = lower(&*catalog, &parsed).expect("Lower failed");
    let plan = compile(EvaluationMode::Permissive, &*catalog, logical).expect("Compile failed");

    // Run iterations
    let mut row_count = 0;
    for _ in 0..iterations {
        let bindings = MapBindings::default();
        let sys = partiql_catalog::context::SystemContext {
            now: DateTime::from_system_now_utc(),
        };
        let ctx = BasicContext::new(bindings, sys);

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
                eprintln!("Execution error: {:?}", e);
                std::process::exit(1);
            }
        }
    }

    row_count
}

fn run_vectorized_profile(format: &DataFormat, query: &str, batch_size: usize, iterations: usize) -> usize {
    let vec_query = query.replace("~input~", "data");
    let catalog = create_catalog("mem".to_string(), None);

    let parsed = parse(&vec_query).expect("Parse failed");
    let logical = lower(&*catalog, &parsed).expect("Lower failed");

    let mut total_rows = 0;
    for _ in 0..iterations {
        let mut plan = compile_vectorized(&logical, format, batch_size);
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
                    eprintln!("Execution error: {:?}", e);
                    std::process::exit(1);
                }
            }
        }
        total_rows = row_count;
    }

    total_rows
}

fn run_hybrid_profile(format: &DataFormat, query: &str, iterations: usize) -> usize {
    let hybrid_query = query.replace("~input~", "data");
    let total_rows = format.row_count();
    let catalog = create_catalog("mem".to_string(), None);

    let parsed = parse(&hybrid_query).expect("Parse failed");
    let logical = lower(&*catalog, &parsed).expect("Lower failed");

    let mut output_rows = 0;
    for _ in 0..iterations {
        let mut stream = compile_hybrid(&logical, format, total_rows)
            .and_then(|plan| plan.execute())
            .expect("Compile failed");

        let mut row_count = 0;
        loop {
            match stream.next_row() {
                Ok(Some(_row)) => row_count += 1,
                Ok(None) => break,
                Err(e) => {
                    eprintln!("Execution error: {:?}", e);
                    std::process::exit(1);
                }
            }
        }
        output_rows = row_count;
    }

    output_rows
}

fn compile_vectorized(
    logical: &partiql_logical::LogicalPlan<partiql_logical::BindingsOp>,
    format: &DataFormat,
    batch_size: usize,
) -> partiql_eval_vectorized::VectorizedPlan {
    use partiql_eval_vectorized::reader::{InMemoryGeneratedReader, PIonReader, PIonTextReader};

    let reader: Box<dyn partiql_eval_vectorized::BatchReader> = match format {
        DataFormat::InMemory { num_rows } => {
            Box::new(InMemoryGeneratedReader::with_config(batch_size, *num_rows / batch_size))
        }
        DataFormat::Ion { path } => {
            Box::new(PIonTextReader::from_ion_file(path, batch_size).expect("Failed to create IonTextReader"))
        }
        DataFormat::IonBinary { path } => {
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
    println!("PartiQL Profiling Workload Runner");
    println!();
    println!("Usage: {} [OPTIONS]", program);
    println!();
    println!("Options:");
    println!("  --engine <ENGINE>       Engine to profile: legacy, vectorized-1, vectorized-1024, hybrid");
    println!("                          (default: legacy)");
    println!("  --data <DATA>           Data source:");
    println!("                            mem:<rows>  - In-memory data with specified row count");
    println!("                            <file>.ion  - Ion text file");
    println!("                            <file>.10n  - Ion binary file");
    println!("                          (default: mem:1000000)");
    println!("  --query <QUERY>         Query to run (use ~input~ placeholder)");
    println!("                          (default: 'SELECT a, b FROM ~input~ WHERE a % 100000 = 0')");
    println!("  --iterations <N>        Number of iterations (default: 100)");
    println!("  --help, -h              Show this help message");
    println!();
    println!("Examples:");
    println!("  # Profile legacy engine with 1M in-memory rows");
    println!("  {} --engine legacy --data mem:1000000", program);
    println!();
    println!("  # Profile vectorized-1024 engine with Ion file");
    println!("  {} --engine vectorized-1024 --data test_data/data.ion", program);
    println!();
    println!("  # Profile hybrid engine with custom query");
    println!("  {} --engine hybrid --query 'SELECT a FROM ~input~ WHERE a > 500000'", program);
    println!();
    println!("Usage with cargo-flamegraph:");
    println!("  cargo flamegraph --bin partiql-profile -- --engine legacy --data mem:1000000");
    println!("  cargo flamegraph --bin partiql-profile -- --engine vectorized-1024");
    println!("  cargo flamegraph --bin partiql-profile -- --engine hybrid");
}
