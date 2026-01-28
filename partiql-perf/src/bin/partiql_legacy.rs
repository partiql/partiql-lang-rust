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
        eprintln!("Usage: {} <query> --data-source <mem|ion> [--data-path <path>]", args[0]);
        eprintln!("\nExamples:");
        eprintln!("  {} \"SELECT a, b FROM !input WHERE a % 1000 = 0\" --data-source ion --data-path test_data/data_b1024_n10000.ion", args[0]);
        eprintln!("  {} \"SELECT * FROM !input\" --data-source mem", args[0]);
        eprintln!("\nNote: !input will be replaced with 'data()' in the query");
        std::process::exit(1);
    }

    let mut query_arg = None;
    let mut data_source = "mem".to_string();
    let mut data_path = None;

    // Parse arguments
    let mut i = 1;
    while i < args.len() {
        match args[i].as_str() {
            "--data-source" => {
                if i + 1 < args.len() {
                    data_source = args[i + 1].clone();
                    i += 2;
                } else {
                    eprintln!("Error: --data-source requires a value");
                    std::process::exit(1);
                }
            }
            "--data-path" => {
                if i + 1 < args.len() {
                    data_path = Some(args[i + 1].clone());
                    i += 2;
                } else {
                    eprintln!("Error: --data-path requires a value");
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
    if data_source != "mem" && data_path.is_none() {
        eprintln!("Error: --data-path is required for file-based data source '{}'", data_source);
        std::process::exit(1);
    }

    // Replace !input with data() (function call for legacy)
    let query = query_arg.replace("~input~", "data()");

    println!("Query:       {}", query);
    println!("Data Source: {}", data_source);
    if let Some(ref path) = data_path {
        println!("Data Path:   {}", path);
    }

    // Calculate and display total rows
    let total_rows = if data_source == "mem" {
        let batch_size = BATCH_SIZE;
        let num_batches = NUM_BATCHES;
        let total = batch_size * num_batches;
        println!("Reader Config: batch_size={}, num_batches={}, total_rows={}", 
                 batch_size, num_batches, common::format_with_commas(total));
        total
    } else if let Some(ref path) = data_path {
        let total = count_rows_from_file(&data_source, path);
        println!("Reader Config: total_rows={} (from file)", common::format_with_commas(total));
        total
    } else {
        0
    };
    
    // Set environment variable for the table function
    std::env::set_var("TOTAL_ROWS", total_rows.to_string());
    
    println!();

    // Create catalog
    let catalog = create_catalog(data_source.clone(), data_path.clone());

    // Phase 1: Parse
    let parse_start = Instant::now();
    let parsed = match parse(&query) {
        Ok(p) => p,
        Err(e) => {
            eprintln!("Parse error: {:?}", e);
            std::process::exit(1);
        }
    };
    let parse_time = parse_start.elapsed();

    // Phase 2: Lower (AST → Logical Plan)
    let lower_start = Instant::now();
    let logical = match lower(&*catalog, &parsed) {
        Ok(l) => l,
        Err(e) => {
            eprintln!("Lower error: {:?}", e);
            std::process::exit(1);
        }
    };
    let lower_time = lower_start.elapsed();

    // Phase 3: Compile (Logical → Physical/Executable Plan)
    let compile_start = Instant::now();
    let plan = match compile(EvaluationMode::Permissive, &*catalog, logical) {
        Ok(p) => p,
        Err(e) => {
            eprintln!("Compile error: {:?}", e);
            std::process::exit(1);
        }
    };
    let compile_time = compile_start.elapsed();

    // Phase 4: Execute
    let exec_start = Instant::now();
    
    // Create evaluation context
    let bindings = MapBindings::default();
    let sys = partiql_catalog::context::SystemContext {
        now: DateTime::from_system_now_utc(),
    };
    let ctx = BasicContext::new(bindings, sys);

    let row_count = match plan.execute(&ctx) {
        Ok(evaluated) => {
            match evaluated.result {
                Value::Bag(ref bag) => {
                    let count = bag.len();
                    
                    // Print results
                    println!("Results:");
                    println!("{}", "=".repeat(60));
                    for value in bag.iter() {
                        println!("{:?}", value);
                    }
                    
                    count
                }
                other => {
                    // For non-bag results, print the single value
                    println!("Results:");
                    println!("{}", "=".repeat(60));
                    println!("{:?}", other);
                    1
                }
            }
        }
        Err(e) => {
            eprintln!("Execution error: {:?}", e);
            std::process::exit(1);
        }
    };
    
    let exec_time = exec_start.elapsed();

    // Print timing summary
    println!("\n{}", "=".repeat(60));
    println!("TIMING SUMMARY");
    println!("{}", "=".repeat(60));
    println!("Parse time:       {:.3}ms", parse_time.as_secs_f64() * 1000.0);
    println!("Lower time:       {:.3}ms", lower_time.as_secs_f64() * 1000.0);
    println!("Compile time:     {:.3}ms", compile_time.as_secs_f64() * 1000.0);
    println!("Execution time:   {:.3}ms", exec_time.as_secs_f64() * 1000.0);
    println!("Rows returned:     {}", row_count);
}
