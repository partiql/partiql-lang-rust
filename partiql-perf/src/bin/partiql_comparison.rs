use partiql_perf::common;

use common::{parse, lower, compile, count_rows_from_file, create_catalog};
use partiql_eval::env::basic::MapBindings;
use partiql_eval::eval::BasicContext;
use partiql_eval::plan::EvaluationMode;
use partiql_value::{DateTime, Value};
use std::time::Instant;

const BATCH_SIZE: usize = 1;
const NUM_BATCHES: usize = 10_000;

fn main() {
    // Parse command line arguments
    let args: Vec<String> = std::env::args().collect();
    if args.len() < 2 {
        eprintln!("Usage: {} <query> --data-source-old <mem|ion> [--data-path-old <path>] --data-source-new <mem|arrow|parquet|ion> [--data-path-new <path>]", args[0]);
        eprintln!("\nExamples:");
        eprintln!("  {} \"SELECT a, b FROM !input WHERE a % 1000 = 0\" --data-source-old ion --data-path-old test_data/data_b1024_n10000.ion --data-source-new ion --data-path-new test_data/data_b1024_n10000.ion", args[0]);
        eprintln!("\nNote: !input will be replaced with 'data()' for legacy and 'data' for vectorized");
        std::process::exit(1);
    }

    let mut query_arg = None;
    let mut data_source_old = "mem".to_string();
    let mut data_path_old = None;
    let mut data_source_new = "mem".to_string();
    let mut data_path_new = None;

    // Parse arguments
    let mut i = 1;
    while i < args.len() {
        match args[i].as_str() {
            "--data-source-old" => {
                if i + 1 < args.len() {
                    data_source_old = args[i + 1].clone();
                    i += 2;
                } else {
                    eprintln!("Error: --data-source-old requires a value");
                    std::process::exit(1);
                }
            }
            "--data-path-old" => {
                if i + 1 < args.len() {
                    data_path_old = Some(args[i + 1].clone());
                    i += 2;
                } else {
                    eprintln!("Error: --data-path-old requires a value");
                    std::process::exit(1);
                }
            }
            "--data-source-new" => {
                if i + 1 < args.len() {
                    data_source_new = args[i + 1].clone();
                    i += 2;
                } else {
                    eprintln!("Error: --data-source-new requires a value");
                    std::process::exit(1);
                }
            }
            "--data-path-new" => {
                if i + 1 < args.len() {
                    data_path_new = Some(args[i + 1].clone());
                    i += 2;
                } else {
                    eprintln!("Error: --data-path-new requires a value");
                    std::process::exit(1);
                }
            }
            arg if !arg.starts_with("--") => {
                if query_arg.is_none() {
                    query_arg = Some(arg.to_string());
                } else {
                    eprintln!("Error: Only one query is allowed");
                    std::process::exit(1);
                }
                i += 1;
            }
            _ => {
                eprintln!("Unknown argument: {}", args[i]);
                std::process::exit(1);
            }
        }
    }

    let query_arg = query_arg.unwrap_or_else(|| {
        eprintln!("Error: Query is required");
        std::process::exit(1);
    });

    // Validate that file-based sources have a path
    if data_source_old != "mem" && data_path_old.is_none() {
        eprintln!("Error: --data-path-old is required for file-based data source '{}'", data_source_old);
        std::process::exit(1);
    }
    if data_source_new != "mem" && data_path_new.is_none() {
        eprintln!("Error: --data-path-new is required for file-based data source '{}'", data_source_new);
        std::process::exit(1);
    }

    // Replace !input with data() for legacy and data for vectorized
    let non_vec_query = query_arg.replace("~input~", "data()");
    let vec_query = query_arg.replace("~input~", "data");
    
    println!("Query (Legacy):       {}", non_vec_query);
    println!("Query (Vectorized):   {}", vec_query);
    println!("Data Source (Old):    {}", data_source_old);
    if let Some(ref path) = data_path_old {
        println!("Data Path (Old):      {}", path);
    }
    println!("Data Source (New):    {}", data_source_new);
    if let Some(ref path) = data_path_new {
        println!("Data Path (New):      {}", path);
    }
    
    // Calculate total rows and set environment variable for non-vectorized evaluator
    let total_rows = if data_source_old == "mem" {
        let total = BATCH_SIZE * NUM_BATCHES;
        println!("Reader Config (Old):  batch_size={}, num_batches={}, total_rows={}", 
                 BATCH_SIZE, NUM_BATCHES, common::format_with_commas(total));
        total
    } else if let Some(ref path) = data_path_old {
        let total = count_rows_from_file(&data_source_old, path);
        println!("Reader Config (Old):  total_rows={} (from file)", common::format_with_commas(total));
        total
    } else {
        0
    };
    
    // Calculate total rows for vectorized evaluator
    if data_source_new == "mem" {
        let total = BATCH_SIZE * NUM_BATCHES;
        println!("Reader Config (New):  batch_size={}, num_batches={}, total_rows={}", 
                 BATCH_SIZE, NUM_BATCHES, common::format_with_commas(total));
    } else if let Some(ref path) = data_path_new {
        let total = count_rows_from_file(&data_source_new, path);
        println!("Reader Config (New):  total_rows={} (from file)", common::format_with_commas(total));
    }
    
    // Set environment variable for non-vectorized evaluator
    std::env::set_var("TOTAL_ROWS", total_rows.to_string());
    
    println!();

    // Create catalog
    let catalog = create_catalog(data_source_old.clone(), data_path_old.clone());

    // Phase 1: Parse Non-Vectorized Query
    let non_vec_parse_start = Instant::now();
    let non_vec_parsed = match parse(&non_vec_query) {
        Ok(p) => p,
        Err(e) => {
            eprintln!("Non-vectorized parse error: {:?}", e);
            std::process::exit(1);
        }
    };
    let non_vec_parse_time = non_vec_parse_start.elapsed();

    // Phase 2: Lower Non-Vectorized (AST → Logical Plan)
    let non_vec_lower_start = Instant::now();
    let non_vec_logical = match lower(&*catalog, &non_vec_parsed) {
        Ok(l) => l,
        Err(e) => {
            eprintln!("Non-vectorized lower error: {:?}", e);
            std::process::exit(1);
        }
    };
    let non_vec_lower_time = non_vec_lower_start.elapsed();

    // Phase 3: Compile Non-Vectorized (Logical → Physical/Executable Plan)
    let non_vec_compile_start = Instant::now();
    let plan = match compile(EvaluationMode::Permissive, &*catalog, non_vec_logical) {
        Ok(p) => p,
        Err(e) => {
            eprintln!("Non-vectorized compile error: {:?}", e);
            std::process::exit(1);
        }
    };
    let non_vec_compile_time = non_vec_compile_start.elapsed();

    // Phase 4: Parse Vectorized Query
    let vec_parse_start = Instant::now();
    let vec_parsed = match parse(&vec_query) {
        Ok(p) => p,
        Err(e) => {
            eprintln!("Vectorized parse error: {:?}", e);
            std::process::exit(1);
        }
    };
    let vec_parse_time = vec_parse_start.elapsed();

    // Phase 5: Lower Vectorized (AST → Logical Plan)
    let vec_lower_start = Instant::now();
    let vec_logical = match lower(&*catalog, &vec_parsed) {
        Ok(l) => l,
        Err(e) => {
            eprintln!("Vectorized lower error: {:?}", e);
            std::process::exit(1);
        }
    };
    let vec_lower_time = vec_lower_start.elapsed();

    // Phase 6: Vectorized Compilation (Logical → Physical Vectorized Operators)
    let vec_compile_start = Instant::now();
    let mut vec_plan = compile_vectorized(&vec_logical, &data_source_new, data_path_new.as_deref());
    let vec_compile_time = vec_compile_start.elapsed();

    println!("{}", "=".repeat(60));
    println!("EXECUTION COMPARISON: Non-Vectorized vs Vectorized");
    println!("{}\n", "=".repeat(60));

    // Execute Non-Vectorized Version
    println!("=== NON-VECTORIZED EXECUTION ===");
    let non_vec_exec_start = Instant::now();
    let mut non_vec_row_count = 0;

    // Create evaluation context
    let bindings = MapBindings::default();
    let sys = partiql_catalog::context::SystemContext {
        now: DateTime::from_system_now_utc(),
    };
    let ctx = BasicContext::new(bindings, sys);

    // Execute and iterate through results
    match plan.execute(&ctx) {
        Ok(evaluated) => {
            match evaluated.result {
                Value::Bag(bag) => {
                    non_vec_row_count = bag.len();
                    println!("Execution completed successfully");
                }
                _ => {
                    non_vec_row_count = 1;
                    println!("Execution completed (non-bag result)");
                }
            }
        }
        Err(e) => {
            eprintln!("Non-vectorized execution error: {:?}", e);
        }
    }
    let non_vec_exec_time = non_vec_exec_start.elapsed();

    println!(
        "Execution time: {:.3}ms",
        non_vec_exec_time.as_secs_f64() * 1000.0
    );
    println!("Rows processed: {}", non_vec_row_count);

    // Execute Vectorized Version
    println!("\n=== VECTORIZED EXECUTION ===");
    let vec_exec_start = Instant::now();
    let mut batch_count = 0;
    let mut vec_row_count = 0;

    for batch_result in vec_plan.execute() {
        match batch_result {
            Ok(batch) => {
                batch_count += 1;

                // Calculate row count based on selection vector if present
                let batch_row_count = if let Some(selection) = batch.selection() {
                    selection.indices.len()
                } else {
                    batch.row_count()
                };

                vec_row_count += batch_row_count;
            }
            Err(e) => {
                eprintln!("Vectorized execution error: {:?}", e);
                std::process::exit(1);
            }
        }
    }
    let vec_exec_time = vec_exec_start.elapsed();

    println!(
        "Execution time: {:.3}ms",
        vec_exec_time.as_secs_f64() * 1000.0
    );
    println!("Batches processed: {}", batch_count);
    println!("Rows processed: {}", vec_row_count);

    // Summary Comparison
    println!("\n{}", "=".repeat(60));
    println!("TIMING SUMMARY");
    println!("{}", "=".repeat(60));
    
    println!("\nNon-Vectorized Planning Phase:");
    println!("  Parse:                {:.3}ms", non_vec_parse_time.as_secs_f64() * 1000.0);
    println!("  Lower:                {:.3}ms", non_vec_lower_time.as_secs_f64() * 1000.0);
    println!("  Compile:              {:.3}ms", non_vec_compile_time.as_secs_f64() * 1000.0);
    
    println!("\nVectorized Planning Phase:");
    println!("  Parse:                {:.3}ms", vec_parse_time.as_secs_f64() * 1000.0);
    println!("  Lower:                {:.3}ms", vec_lower_time.as_secs_f64() * 1000.0);
    println!("  Compile:              {:.3}μs", vec_compile_time.as_secs_f64() * 1_000_000.0);
    
    println!("\nExecution Phase:");
    println!(
        "  Non-Vectorized:       {:.3}ms",
        non_vec_exec_time.as_secs_f64() * 1000.0
    );
    println!(
        "  Vectorized:           {:.3}ms",
        vec_exec_time.as_secs_f64() * 1000.0
    );

    if non_vec_exec_time.as_secs_f64() > 0.0 && vec_exec_time.as_secs_f64() > 0.0 {
        let speedup = non_vec_exec_time.as_secs_f64() / vec_exec_time.as_secs_f64();
        println!("  Speedup:              {:.2}x", speedup);
    }
}

fn compile_vectorized(
    logical: &partiql_logical::LogicalPlan<partiql_logical::BindingsOp>,
    data_source: &str,
    data_path: Option<&str>,
) -> partiql_eval_vectorized::VectorizedPlan {
    use partiql_eval_vectorized::reader::{ArrowReader, InMemoryGeneratedReader, PIonReader, PIonTextReader, ParquetReader};

    let reader: Box<dyn partiql_eval_vectorized::BatchReader> = match data_source {
        "mem" => {
            let batch_size = BATCH_SIZE;
            let num_batches = NUM_BATCHES;
            Box::new(InMemoryGeneratedReader::with_config(batch_size, num_batches))
        }
        "arrow" => {
            let path = data_path.expect("--data-path-new required for arrow data source");
            Box::new(ArrowReader::from_file(path, BATCH_SIZE).expect("Failed to create ArrowReader"))
        }
        "parquet" => {
            let path = data_path.expect("--data-path-new required for parquet data source");
            let batch_size = BATCH_SIZE;
            Box::new(ParquetReader::from_file(path, batch_size).expect("Failed to create ParquetReader"))
        }
        "ion" => {
            // Text Ion - uses string-based field names
            let path = data_path.expect("--data-path-new required for ion data source");
            let batch_size = BATCH_SIZE;
            Box::new(PIonTextReader::from_ion_file(path, batch_size).expect("Failed to create IonTextReader"))
        }
        "ionb" => {
            // Binary Ion - uses symbol IDs for optimal performance
            let path = data_path.expect("--data-path-new required for ionb data source");
            let batch_size = BATCH_SIZE;
            Box::new(PIonReader::from_ion_file(path, batch_size).expect("Failed to create IonBinaryReader"))
        }
        _ => {
            panic!("Unknown data source: {}", data_source);
        }
    };

    let context = partiql_eval_vectorized::CompilerContext::new()
        .with_data_source("data".to_string(), reader);

    let mut compiler = partiql_eval_vectorized::Compiler::new(context);
    compiler
        .compile(logical)
        .expect("Vectorized compilation failed")
}
