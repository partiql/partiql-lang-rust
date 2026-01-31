use partiql_tools::common;

use common::{count_rows_from_file, create_catalog, lower, parse, SimpleDataCatalog};
use partiql_eval::source::DataSourceHandle;
use partiql_eval::{CatalogRegistry, PlanCompiler, ScanProvider};
use partiql_logical::Scan;
use partiql_value::{Tuple, Value};
use std::sync::Arc;
use std::time::Instant;

const BATCH_SIZE: usize = 1;
const NUM_BATCHES: usize = 10_000;

fn main() {
    // Parse command line arguments
    let args: Vec<String> = std::env::args().collect();
    if args.len() < 2 {
        eprintln!(
            "Usage: {} <query> --data-source <mem|ion> [--data-path <path>]",
            args[0]
        );
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

    if data_source != "mem" && data_path.is_none() {
        eprintln!(
            "Error: --data-path is required for file-based data source '{}'",
            data_source
        );
        std::process::exit(1);
    }

    // Replace !input with data (no parentheses for hybrid)
    let query = query_arg.replace("~input~", "data");

    println!("Query:       {}", query);
    println!("Data Source: {}", data_source);
    if let Some(ref path) = data_path {
        println!("Data Path:   {}", path);
    }

    let total_rows = if data_source == "mem" {
        let batch_size = BATCH_SIZE;
        let num_batches = NUM_BATCHES;
        let total = batch_size * num_batches;
        println!(
            "Reader Config: batch_size={}, num_batches={}, total_rows={}",
            batch_size,
            num_batches,
            common::format_with_commas(total)
        );
        total
    } else if let Some(ref path) = data_path {
        let total = count_rows_from_file(&data_source, path);
        println!(
            "Reader Config: total_rows={} (from file)",
            common::format_with_commas(total)
        );
        total
    } else {
        0
    };

    std::env::set_var("TOTAL_ROWS", total_rows.to_string());

    println!();

    let catalog = create_catalog(data_source.clone(), data_path.clone());

    // Default column names for in-memory reader
    let column_names = vec!["a".to_string(), "b".to_string()];

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

    // Phase 3: Compile (Logical → CompiledPlan)
    let compile_start = Instant::now();
    let provider = HybridScanProvider::new(data_source.clone(), data_path.clone(), total_rows);

    // Set up catalog registry with a simple data catalog using builder pattern
    let mut registry = CatalogRegistry::new();
    let data_source_handle = match data_source.as_str() {
        "mem" => DataSourceHandle::mem(total_rows, column_names.clone()),
        "ion" | "ionb" => DataSourceHandle::ion(data_path.clone().unwrap_or_default()),
        _ => DataSourceHandle::mem(total_rows, column_names.clone()),
    };
    let data_catalog =
        Arc::new(SimpleDataCatalog::new(catalog.name()).with_table("data", data_source_handle));
    registry.register_catalog(data_catalog);

    let compiler = PlanCompiler::with_catalogs(&provider, Some(&registry));
    let compiled = match compiler.compile(&logical) {
        Ok(p) => p,
        Err(e) => {
            eprintln!("Compile error: {:?}", e);
            std::process::exit(1);
        }
    };
    let compile_time = compile_start.elapsed();

    // Phase 4: Execute
    let exec_start = Instant::now();
    let mut vm = match partiql_eval::PartiQLVM::new(compiled) {
        Ok(p) => p,
        Err(e) => {
            eprintln!("Execution setup error: {:?}", e);
            std::process::exit(1);
        }
    };

    let schema = vm.schema();
    let mut row_count = 0usize;
    let mut results: Vec<Value> = Vec::new();

    match vm.execute() {
        Ok(partiql_eval::ExecutionResult::Query(iter)) => {
            for row_result in iter {
                match row_result {
                    Ok(row) => {
                        row_count += 1;
                        results.push(row_to_value(&row, &schema));
                    }
                    Err(e) => {
                        eprintln!("Execution error: {:?}", e);
                        std::process::exit(1);
                    }
                }
            }
        }
        Err(e) => {
            eprintln!("Execution setup error: {:?}", e);
            std::process::exit(1);
        }
    }
    let exec_time = exec_start.elapsed();

    println!("Results:");
    println!("{}", "=".repeat(60));
    for value in &results {
        println!("{:?}", value);
    }

    println!("\n{}", "=".repeat(60));
    println!("TIMING SUMMARY");
    println!("{}", "=".repeat(60));
    println!(
        "Parse time:       {:.3}ms",
        parse_time.as_secs_f64() * 1000.0
    );
    println!(
        "Lower time:       {:.3}ms",
        lower_time.as_secs_f64() * 1000.0
    );
    println!(
        "Compile time:     {:.3}ms",
        compile_time.as_secs_f64() * 1000.0
    );
    println!(
        "Execution time:   {:.3}ms",
        exec_time.as_secs_f64() * 1000.0
    );
    println!("Rows returned:     {}", row_count);
}

struct HybridScanProvider {
    data_source: String,
    data_path: Option<String>,
    total_rows: usize,
}

impl HybridScanProvider {
    fn new(data_source: String, data_path: Option<String>, total_rows: usize) -> Self {
        HybridScanProvider {
            data_source,
            data_path,
            total_rows,
        }
    }
}

impl ScanProvider for HybridScanProvider {
    fn data_source(&self, _scan: &Scan) -> partiql_eval::Result<DataSourceHandle> {
        match self.data_source.as_str() {
            "mem" => Ok(DataSourceHandle::mem(
                self.total_rows,
                vec!["a".to_string(), "b".to_string()],
            )),
            "ion" | "ionb" => {
                let path = self.data_path.clone().ok_or_else(|| {
                    partiql_eval::EngineError::ReaderError("ion path required".to_string())
                })?;
                Ok(DataSourceHandle::ion(path))
            }
            other => Err(partiql_eval::EngineError::ReaderError(format!(
                "unsupported data source: {other}"
            ))),
        }
    }
}

fn row_to_value(
    row: &partiql_eval::value::RegisterReader<'_>,
    schema: &partiql_eval::Schema,
) -> Value {
    if schema.columns.len() == 1 {
        row.get_value(0).into()
    } else {
        let mut tuple = Tuple::new();
        for (idx, col) in schema.columns.iter().enumerate() {
            tuple.insert(&col.name, row.get_value(idx).into());
        }
        Value::Tuple(Box::new(tuple))
    }
}
