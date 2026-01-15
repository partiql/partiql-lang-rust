mod common;

use common::{parse, lower, count_rows_from_file, create_catalog};
use partiql_eval_vectorized::batch::VectorizedBatch;
use partiql_eval_vectorized::batch::PhysicalVectorEnum;
use std::time::Instant;

const BATCH_SIZE: usize = 1;
const NUM_BATCHES: usize = 10_000;

fn main() {
    // Parse command line arguments
    let args: Vec<String> = std::env::args().collect();
    if args.len() < 2 {
        eprintln!("Usage: {} <query> --data-source <mem|arrow|parquet|ion> [--data-path <path>]", args[0]);
        eprintln!("\nExamples:");
        eprintln!("  {} \"SELECT a, b FROM !input WHERE a % 1000 = 0\" --data-source ion --data-path test_data/data_b1024_n10000.ion", args[0]);
        eprintln!("  {} \"SELECT * FROM !input\" --data-source mem", args[0]);
        eprintln!("\nNote: !input will be replaced with 'data' in the query");
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

    // Replace !input with data (no parentheses for vectorized)
    let query = query_arg.replace("~input~", "data");

    println!("Query:       {}", query);
    println!("Data Source: {}", data_source);
    if let Some(ref path) = data_path {
        println!("Data Path:   {}", path);
    }

    // Calculate and display total rows
    if data_source == "mem" {
        let batch_size = BATCH_SIZE;
        let num_batches = NUM_BATCHES;
        let total = batch_size * num_batches;
        println!("Reader Config: batch_size={}, num_batches={}, total_rows={}", 
                 batch_size, num_batches, common::format_with_commas(total));
    } else if let Some(ref path) = data_path {
        let total = count_rows_from_file(&data_source, path);
        println!("Reader Config: total_rows={} (from file)", common::format_with_commas(total));
    }
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

    // Phase 3: Compile (Logical → Vectorized Plan)
    let compile_start = Instant::now();
    let mut vec_plan = compile_vectorized(&logical, &data_source, data_path.as_deref());
    let compile_time = compile_start.elapsed();

    // Phase 4: Execute
    let exec_start = Instant::now();
    let mut batch_count = 0;
    let mut row_count = 0;

    // Print output schema once before processing batches
    print_schema(vec_plan.output_schema());

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

                row_count += batch_row_count;

                // Print batch rows only (without header)
                print_batch_rows(&batch);
            }
            Err(e) => {
                eprintln!("Execution error: {:?}", e);
                std::process::exit(1);
            }
        }
    }
    let exec_time = exec_start.elapsed();

    // Print timing summary
    println!("\n{}", "=".repeat(60));
    println!("TIMING SUMMARY");
    println!("{}", "=".repeat(60));
    println!("Parse time:       {:.3}ms", parse_time.as_secs_f64() * 1000.0);
    println!("Lower time:       {:.3}ms", lower_time.as_secs_f64() * 1000.0);
    println!("Compile time:     {:.3}μs", compile_time.as_secs_f64() * 1_000_000.0);
    println!("Execution time:   {:.3}ms", exec_time.as_secs_f64() * 1000.0);
    println!("Batches processed: {}", batch_count);
    println!("Rows returned:     {}", row_count);
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
            let path = data_path.expect("--data-path required for arrow data source");
            Box::new(ArrowReader::from_file(path).expect("Failed to create ArrowReader"))
        }
        "parquet" => {
            let path = data_path.expect("--data-path required for parquet data source");
            let batch_size = BATCH_SIZE;
            Box::new(ParquetReader::from_file(path, batch_size).expect("Failed to create ParquetReader"))
        }
        "ion" => {
            // Text Ion - uses string-based field names
            let path = data_path.expect("--data-path required for ion data source");
            let batch_size = BATCH_SIZE;
            Box::new(PIonTextReader::from_ion_file(path, batch_size).expect("Failed to create IonTextReader"))
        }
        "ionb" => {
            // Binary Ion - uses symbol IDs for optimal performance
            let path = data_path.expect("--data-path required for ionb data source");
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

fn print_schema(schema: &partiql_eval_vectorized::batch::SourceTypeDef) {
    let fields = schema.fields();
    
    // Print header
    print!("| ");
    for (i, field) in fields.iter().enumerate() {
        if i > 0 {
            print!(" | ");
        }
        print!("{}", field.name);
    }
    println!(" |");

    // Print separator
    print!("|");
    for _ in 0..fields.len() {
        print!("----------|");
    }
    println!();
}

fn print_batch_rows(batch: &VectorizedBatch) {
    // Get the row indices to print based on selection vector
    let row_indices: Vec<usize> = if let Some(selection) = batch.selection() {
        // Only print selected rows
        selection.indices.clone()
    } else {
        // Print all rows
        (0..batch.row_count()).collect()
    };

    if row_indices.is_empty() {
        return;
    }

    // Print rows
    for &row_idx in &row_indices {
        print!("| ");
        for col_idx in 0..batch.source_column_count() {
            if col_idx > 0 {
                print!(" | ");
            }
            
            if let Ok(column) = batch.column(col_idx) {
                match &column.physical {
                    PhysicalVectorEnum::Int64(vec) => {
                        let slice = vec.as_slice();
                        if row_idx < slice.len() {
                            print!("{:8}", slice[row_idx]);
                        } else {
                            print!("{:8}", "NULL");
                        }
                    }
                    PhysicalVectorEnum::Float64(vec) => {
                        let slice = vec.as_slice();
                        if row_idx < slice.len() {
                            print!("{:8.2}", slice[row_idx]);
                        } else {
                            print!("{:8}", "NULL");
                        }
                    }
                    PhysicalVectorEnum::Boolean(vec) => {
                        let slice = vec.as_slice();
                        if row_idx < slice.len() {
                            print!("{:8}", slice[row_idx]);
                        } else {
                            print!("{:8}", "NULL");
                        }
                    }
                    PhysicalVectorEnum::String(vec) => {
                        let slice = vec.as_slice();
                        if row_idx < slice.len() {
                            print!("{:8}", slice[row_idx]);
                        } else {
                            print!("{:8}", "NULL");
                        }
                    }
                }
            }
        }
        println!(" |");
    }
}
