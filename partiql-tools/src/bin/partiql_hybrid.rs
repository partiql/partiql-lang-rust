use partiql_tools::common;

use common::{
    count_rows_from_file, create_catalog, lower, parse, random_catalog, simple_catalog,
    CompiledSourceFactory,
};
use partiql_eval::value::Shape;
use partiql_eval::{CompilationContext, ExecutionCatalog, ExecutionContext, PlanCompiler};
use partiql_value::{Tuple, Value};
use std::time::Instant;

const BATCH_SIZE: usize = 1;
const NUM_BATCHES: usize = 10_000;

fn main() {
    // Parse command line arguments
    let args: Vec<String> = std::env::args().collect();
    if args.len() < 2 {
        eprintln!(
            "Usage: {} <query> --data-source <mem|ion|rand> [--data-path <path>]",
            args[0]
        );
        eprintln!("\nExamples:");
        eprintln!("  {} \"SELECT a, b FROM !input WHERE a % 1000 = 0\" --data-source ion --data-path test_data/data_b1024_n10000.ion", args[0]);
        eprintln!("  {} \"SELECT * FROM !input\" --data-source mem", args[0]);
        eprintln!(
            "  {} \"SELECT * FROM data WHERE a > 0 LIMIT 10\" --data-source rand",
            args[0]
        );
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

    if data_source != "mem" && data_source != "rand" && data_path.is_none() {
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

    let total_rows = if data_source == "mem" || data_source == "rand" {
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

    // Set up compilation context - use two-phase catalog pattern for ALL data sources
    let mut context = CompilationContext::new();

    // Create appropriate catalog based on data source - all use two-phase pattern
    // The execution catalog is wrapped in an enum to handle different concrete types
    enum ExecCatalog {
        Random(common::RandomExecutionCatalog),
        Simple(common::SimpleExecutionCatalog),
    }

    let (comp_catalog, mut exec_catalog_inner) = match data_source.as_str() {
        "rand" => {
            // Random catalog for custom reader demonstration
            let (comp, exec) =
                random_catalog(vec![("data".to_string(), total_rows, column_names.clone())]);
            (comp, ExecCatalog::Random(exec))
        }
        "mem" | "ion" | "ionb" => {
            // Simple catalog for mem/ion data sources - use CompiledSourceFactory
            let factory = match data_source.as_str() {
                "mem" => CompiledSourceFactory::mem(total_rows, column_names.clone()),
                "ion" | "ionb" => CompiledSourceFactory::ion(data_path.clone().unwrap_or_default()),
                _ => unreachable!(),
            };
            let (comp, exec) = simple_catalog(vec![("data".to_string(), factory)]);
            (comp, ExecCatalog::Simple(exec))
        }
        _ => {
            eprintln!("Unsupported data source: {}", data_source);
            std::process::exit(1);
        }
    };

    // Add catalog and CAPTURE the returned catalog_id - this is the ONLY place catalog_id is assigned
    let catalog_id = context.add_catalog("default", comp_catalog);

    let mut compiler = PlanCompiler::new(&context);
    let compiled = match compiler.compile(&logical) {
        Ok(p) => p,
        Err(e) => {
            eprintln!("Compile error: {:?}", e);
            std::process::exit(1);
        }
    };
    let compile_time = compile_start.elapsed();

    // Dump compiled plan for debugging
    println!("Compiled Plan:");
    println!("{}", compiled);
    println!();

    // Phase 4: Execute
    let exec_start = Instant::now();

    // Prepare execution catalog with catalog-specific scans (ScanId-based pattern)
    let catalog_scans = compiled.scans_for_catalog(catalog_id);
    match &mut exec_catalog_inner {
        ExecCatalog::Random(exec) => exec.prepare(&catalog_scans),
        ExecCatalog::Simple(exec) => exec.prepare(&catalog_scans),
    }

    // Create ExecutionContext and ALWAYS populate it with execution catalog
    let mut exec_context = ExecutionContext::new();
    match exec_catalog_inner {
        ExecCatalog::Random(exec) => exec_context.add_catalog(catalog_id, Box::new(exec)),
        ExecCatalog::Simple(exec) => exec_context.add_catalog(catalog_id, Box::new(exec)),
    }

    let mut vm = match partiql_eval::PartiQLVM::new(compiled, &exec_context) {
        Ok(p) => p,
        Err(e) => {
            eprintln!("Execution setup error: {:?}", e);
            std::process::exit(1);
        }
    };

    let shape = vm.shape().clone();
    let mut row_count = 0usize;

    let (prefix, tab, suffix) = match shape {
        Shape::Bag(_) => (Some("<<"), "  ", Some(">>")),
        Shape::List(_) => (Some("["), "  ", Some("]")),
        Shape::Single(_) => (None, "", None),
    };

    println!("Results:");
    println!("{}", "=".repeat(60));

    let mut is_first = true;
    match vm.execute() {
        Ok(partiql_eval::ExecutionResult::Query(iter)) => {
            if let Some(p) = prefix {
                println!("{}", p);
            }
            for row_result in iter {
                match row_result {
                    Ok(row) => {
                        row_count += 1;
                        if is_first {
                            is_first = false;
                        } else {
                            println!(",");
                        }
                        let value = row_to_value(&row, &shape);
                        print!("{}", tab);
                        print!("{:?}", value);
                    }
                    Err(e) => {
                        eprintln!("Execution error: {:?}", e);
                        std::process::exit(1);
                    }
                }
            }
            println!();
            if let Some(s) = suffix {
                println!("{}", s);
            }
        }
        Err(e) => {
            eprintln!("Execution setup error: {:?}", e);
            std::process::exit(1);
        }
    }
    let exec_time = exec_start.elapsed();

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

fn row_to_value(
    row: &partiql_eval::value::RegisterReader<'_>,
    shape: &partiql_eval::value::Shape,
) -> Value {
    use partiql_eval::value::{FieldName, RowShape};

    match shape.row_shape() {
        RowShape::Struct(fields) => {
            // Construct tuple with all fields (including single field case)
            let mut tuple = Tuple::new();
            for field in fields.iter() {
                let name = match &field.name {
                    FieldName::Static(s) => s.clone(),
                    FieldName::Register(reg) => {
                        // Dynamic name - get from register as string
                        row.get_str(*reg).unwrap_or("?").to_string()
                    }
                };
                // Get register index from the field's value shape
                let reg_idx = match &field.value {
                    RowShape::Register(idx, _) => *idx,
                    _ => continue, // nested structs not yet supported here
                };
                let mut view = row.get_value_view(reg_idx).expect("register should exist");
                tuple.insert(&name, value_view_to_value(&mut view));
            }
            Value::Tuple(Box::new(tuple))
        }
        RowShape::Register(idx, _) => {
            // Scalar - return value directly
            let mut view = row.get_value_view(*idx).expect("register should exist");
            value_view_to_value(&mut view)
        }
    }
}

/// Convert a ValueView cursor to a Value
fn value_view_to_value(view: &mut partiql_eval::value::ValueView<'_>) -> Value {
    use partiql_eval::value::ValueType;

    match view.get_type() {
        ValueType::Missing => Value::Missing,
        ValueType::Null => Value::Null,
        ValueType::Bool => Value::Boolean(view.get_bool().unwrap()),
        ValueType::Integer => Value::Integer(view.get_i64().unwrap()),
        ValueType::Decimal => Value::Decimal(Box::new(view.get_decimal().unwrap())),
        ValueType::Float => Value::Real(view.get_f64().unwrap().into()),
        ValueType::String => Value::String(Box::new(view.get_str().unwrap().to_string())),
        ValueType::Bytes => Value::Blob(Box::new(view.get_bytes().unwrap().to_vec())),
        ValueType::Tuple => {
            // Navigate tuple fields with support for nested tuples
            let mut tuple = Tuple::new();
            if view.step_in().is_ok() {
                loop {
                    let field_name = view.get_field_name().unwrap().to_string();
                    let field_value = value_view_to_value(view);
                    tuple.insert(&field_name, field_value);

                    // Move to next field
                    if !view.advance().unwrap_or(false) {
                        break;
                    }
                }
                let _ = view.step_out();
            }
            Value::Tuple(Box::new(tuple))
        }
        ValueType::List => {
            // Navigate list elements
            let mut items = Vec::new();
            if view.step_in().is_ok() {
                loop {
                    let item_value = value_view_to_value(view);
                    items.push(item_value);

                    if !view.advance().unwrap_or(false) {
                        break;
                    }
                }
                let _ = view.step_out();
            }
            Value::List(Box::new(items.into()))
        }
        ValueType::Bag => {
            // Navigate bag elements
            let mut items = Vec::new();
            if view.step_in().is_ok() {
                loop {
                    let item_value = value_view_to_value(view);
                    items.push(item_value);

                    if !view.advance().unwrap_or(false) {
                        break;
                    }
                }
                let _ = view.step_out();
            }
            Value::Bag(Box::new(items.into()))
        }
    }
}
