use ion_rs::ReaderBuilder;
use partiql_ast_passes::error::AstTransformationError;
use partiql_catalog::call_defs::{CallDef, CallSpec};
use partiql_catalog::catalog::{MutableCatalog, PartiqlCatalog, SharedCatalog};
use partiql_catalog::context::SessionContext;
use partiql_catalog::extension::ExtensionResultError;
use partiql_catalog::table_fn::{
    BaseTableExpr, BaseTableExprResult, BaseTableFunctionInfo, TableFunction,
};
use partiql_eval::error::PlanErr;
use partiql_eval::eval::EvalPlan;
use partiql_eval::plan::{EvaluationMode, EvaluatorPlanner};
use partiql_extension_ion::decode::{IonDecoderBuilder, IonDecoderConfig, IonValueIter};
use partiql_extension_ion::Encoding;
use partiql_logical::LogicalPlan;
use partiql_logical_planner::LogicalPlanner;
use partiql_parser::{Parsed, Parser, ParserError};
use partiql_value::{Tuple, Value};
use std::borrow::Cow;
use std::fs::File;
use std::io::BufReader;
use std::time::Instant;

// Iterator that owns the file and decoder for lazy Ion reading
struct IonFileIterator {
    decoder: IonValueIter<'static>,
}

impl Iterator for IonFileIterator {
    type Item = Result<Value, Value>;

    fn next(&mut self) -> Option<Self::Item> {
        self.decoder.next().map(|result| {
            result.map_err(|e| Value::from(format!("Ion decode error: {:?}", e)))
        })
    }
}

// Table function that generates or reads data with fields 'a' and 'b'
#[derive(Debug)]
struct DataTableFunction {
    data_source: String,
    data_path: Option<String>,
}

impl DataTableFunction {
    fn new(data_source: String, data_path: Option<String>) -> Self {
        Self {
            data_source,
            data_path,
        }
    }
}

impl BaseTableFunctionInfo for DataTableFunction {
    fn call_def(&self) -> &CallDef {
        // Define the function signature (no arguments)
        static CALL_DEF: std::sync::OnceLock<CallDef> = std::sync::OnceLock::new();
        CALL_DEF.get_or_init(|| CallDef {
            names: vec!["data"],
            overloads: vec![CallSpec {
                input: vec![],
                output: Box::new(|args| {
                    partiql_logical::ValueExpr::Call(partiql_logical::CallExpr {
                        name: partiql_logical::CallName::ByName("data".to_string()),
                        arguments: args,
                    })
                }),
            }],
        })
    }

    fn plan_eval(&self) -> Box<dyn BaseTableExpr> {
        Box::new(DataTableExpr {
            data_source: self.data_source.clone(),
            data_path: self.data_path.clone(),
        })
    }
}

#[derive(Debug)]
struct DataTableExpr {
    data_source: String,
    data_path: Option<String>,
}

impl BaseTableExpr for DataTableExpr {
    fn evaluate<'c>(
        &self,
        _args: &[Cow<'_, Value>],
        _ctx: &'c dyn SessionContext,
    ) -> BaseTableExprResult<'c> {
        match self.data_source.as_str() {
            "mem" => {
                // Generate in-memory data
                let total_rows = if let Ok(rows_str) = std::env::var("TOTAL_ROWS") {
                    rows_str.parse().unwrap_or_else(|_| {
                        let batch_size = std::env::var("BATCH_SIZE")
                            .ok()
                            .and_then(|s| s.parse().ok())
                            .unwrap_or(1024);
                        let num_batches = std::env::var("NUM_BATCHES")
                            .ok()
                            .and_then(|s| s.parse().ok())
                            .unwrap_or(10_000);
                        batch_size * num_batches
                    })
                } else {
                    let batch_size = std::env::var("BATCH_SIZE")
                        .ok()
                        .and_then(|s| s.parse().ok())
                        .unwrap_or(1024);
                    let num_batches = std::env::var("NUM_BATCHES")
                        .ok()
                        .and_then(|s| s.parse().ok())
                        .unwrap_or(10_000);
                    batch_size * num_batches
                };
                
                let iter = (0..total_rows as i64).map(|i| {
                    let tuple = Tuple::from([("a", Value::Integer(i)), ("b", Value::Integer(i + 100))]);
                    Ok(Value::Tuple(Box::new(tuple)))
                });

                Ok(Box::new(iter))
            }
            "ion" => {
                // Read from Ion file - streaming approach
                let file_path = self.data_path.as_ref()
                    .expect("Ion data source requires --data-path-old");

                // Open the file
                let file = File::open(file_path)
                    .expect(&format!("Failed to open Ion file: {}", file_path));
                
                let buf_reader = BufReader::new(file);

                // Create Ion reader from the file
                let reader = ReaderBuilder::new().build(buf_reader)
                    .expect("Failed to create Ion reader");

                // Create Ion decoder - this will stream values lazily
                let decoder = IonDecoderBuilder::new(IonDecoderConfig::default().with_mode(Encoding::Ion))
                    .build(reader)
                    .expect("Failed to create Ion decoder");

                // Return the decoder directly as an iterator (lazy evaluation)
                Ok(Box::new(decoder.map(|result| {
                    result.map_err(|e| ExtensionResultError::ReadError(Box::new(std::io::Error::new(
                        std::io::ErrorKind::InvalidData,
                        format!("Ion decode error: {:?}", e)
                    ))))
                })))
            }
            _ => {
                // Unsupported data source
                panic!("Unsupported data source: {}. Only 'mem' and 'ion' are supported for non-vectorized evaluator.", self.data_source)
            }
        }
    }
}

fn main() {
    // Parse command line arguments
    let args: Vec<String> = std::env::args().collect();
    if args.len() < 2 {
        eprintln!("Usage: {} <non-vectorized-query> [vectorized-query] [--data-source-old <mem|ion>] [--data-path-old <path>] [--data-source-new <mem|arrow|parquet|ion>] [--data-path-new <path>]", args[0]);
        eprintln!("Examples:");
        eprintln!("  {} \"SELECT a FROM data()\"", args[0]);
        eprintln!("  {} \"SELECT * FROM data()\" \"SELECT a FROM data\"", args[0]);
        eprintln!("  {} \"SELECT a FROM data()\" \"SELECT a FROM data\" --data-source-old ion --data-path-old ./data/data1.ion --data-source-new ion --data-path-new ./data/data2.ion", args[0]);
        eprintln!("\nIf only one query is provided, it will be used for both evaluators.");
        eprintln!("\nData source options:");
        eprintln!("  --data-source-old mem      Use in-memory generated reader for non-vectorized (default)");
        eprintln!("  --data-source-old ion      Read from Ion text file for non-vectorized");
        eprintln!("  --data-source-new mem      Use in-memory generated reader for vectorized (default)");
        eprintln!("  --data-source-new arrow    Read from Arrow IPC file for vectorized");
        eprintln!("  --data-source-new parquet  Read from Parquet file for vectorized");
        eprintln!("  --data-source-new ion      Read from Ion text file for vectorized");
        eprintln!("  --data-path-old <path>     Path to data file (required for file-based sources)");
        eprintln!("  --data-path-new <path>     Path to data file (required for file-based sources)");
        std::process::exit(1);
    }

    let mut non_vec_query = None;
    let mut vec_query = None;
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
                if non_vec_query.is_none() {
                    non_vec_query = Some(arg.to_string());
                } else if vec_query.is_none() {
                    vec_query = Some(arg.to_string());
                } else {
                    eprintln!("Error: Too many positional arguments");
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

    let non_vec_query = non_vec_query.ok_or_else(|| {
        eprintln!("Error: At least one query is required");
        std::process::exit(1);
    }).unwrap();
    let vec_query = vec_query.unwrap_or_else(|| non_vec_query.clone());
    
    // Validate that file-based sources have a path
    if data_source_old != "mem" && data_path_old.is_none() {
        eprintln!("Error: --data-path-old is required for file-based data source '{}'", data_source_old);
        std::process::exit(1);
    }
    if data_source_new != "mem" && data_path_new.is_none() {
        eprintln!("Error: --data-path-new is required for file-based data source '{}'", data_source_new);
        std::process::exit(1);
    }
    
    println!("Non-Vectorized Query: {}", non_vec_query);
    println!("Vectorized Query:     {}", vec_query);
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
        let batch_size = std::env::var("BATCH_SIZE")
            .ok()
            .and_then(|s| s.parse().ok())
            .unwrap_or(1024);
        let num_batches = std::env::var("NUM_BATCHES")
            .ok()
            .and_then(|s| s.parse().ok())
            .unwrap_or(10_000);
        let total = batch_size * num_batches;
        println!("Reader Config (Old):  batch_size={}, num_batches={}, total_rows={}", 
                 batch_size, num_batches, total);
        total
    } else if let Some(ref path) = data_path_old {
        // Count total rows from file
        let total = count_rows_from_file(&data_source_old, path);
        println!("Reader Config (Old):  total_rows={} (from file)", total);
        total
    } else {
        0
    };
    
    // Calculate total rows for vectorized evaluator
    if data_source_new == "mem" {
        let batch_size = std::env::var("BATCH_SIZE")
            .ok()
            .and_then(|s| s.parse().ok())
            .unwrap_or(1024);
        let num_batches = std::env::var("NUM_BATCHES")
            .ok()
            .and_then(|s| s.parse().ok())
            .unwrap_or(10_000);
        let total = batch_size * num_batches;
        println!("Reader Config (New):  batch_size={}, num_batches={}, total_rows={}", 
                 batch_size, num_batches, total);
    } else if let Some(ref path) = data_path_new {
        let total = count_rows_from_file(&data_source_new, path);
        println!("Reader Config (New):  total_rows={} (from file)", total);
    }
    
    // Set environment variable for non-vectorized evaluator
    std::env::set_var("TOTAL_ROWS", total_rows.to_string());
    
    println!();

    // Create catalog and add the data table function
    let mut catalog = PartiqlCatalog::default();
    let data_fn = TableFunction::new(Box::new(DataTableFunction::new(
        data_source_old.clone(),
        data_path_old.clone(),
    )));
    catalog.add_table_function(data_fn).expect("Failed to add table function");
    
    // Add type entry for "data" table so it can be referenced without parentheses
    use partiql_catalog::catalog::TypeEnvEntry;
    use partiql_types::{PartiqlShapeBuilder, StructField, Static, StructType, struct_fields, StructConstraint};
    use indexmap::IndexSet;
    
    let mut bld = PartiqlShapeBuilder::default();
    let fields = IndexSet::from([
        StructField::new("a", bld.new_static(Static::Int)),
        StructField::new("b", bld.new_static(Static::Int)),
    ]);
    let data_type = bld.new_struct(StructType::new(IndexSet::from([
        StructConstraint::Fields(fields),
        StructConstraint::Open(false),
    ])));
    
    let data_type_entry = TypeEnvEntry::new("data", &[], data_type);
    catalog.add_type_entry(data_type_entry).expect("Failed to add type entry");
    
    let catalog = catalog.to_shared_catalog();

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
    let non_vec_logical = match lower(&catalog, &non_vec_parsed) {
        Ok(l) => l,
        Err(e) => {
            eprintln!("Non-vectorized lower error: {:?}", e);
            std::process::exit(1);
        }
    };
    let non_vec_lower_time = non_vec_lower_start.elapsed();

    // Phase 3: Compile Non-Vectorized (Logical → Physical/Executable Plan)
    let non_vec_compile_start = Instant::now();
    let plan = match compile(EvaluationMode::Permissive, &catalog, non_vec_logical) {
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
    let vec_logical = match lower(&catalog, &vec_parsed) {
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

    println!("\n{}", "=".repeat(60));
    println!("EXECUTION COMPARISON: Non-Vectorized vs Vectorized");
    println!("{}\n", "=".repeat(60));

    // Execute Non-Vectorized Version
    println!("=== NON-VECTORIZED EXECUTION ===");
    let non_vec_exec_start = Instant::now();
    let mut non_vec_row_count = 0;

    // Create evaluation context
    use partiql_eval::env::basic::MapBindings;
    use partiql_eval::eval::BasicContext;
    use partiql_value::{Bag, DateTime};
    let bindings = MapBindings::default();
    let sys = partiql_catalog::context::SystemContext {
        now: DateTime::from_system_now_utc(),
    };
    let ctx = BasicContext::new(bindings, sys);

    // Execute and iterate through results
    match plan.execute(&ctx) {
        Ok(evaluated) => {
            // The result is an Evaluated type containing a Value
            match evaluated.result {
                Value::Bag(bag) => {
                    non_vec_row_count = bag.len();
                    println!("Execution completed successfully");
                }
                _ => {
                    // For non-bag results, count as 1
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
                    // Partial selection: count selected rows (length of indices vector)
                    selection.indices.len()
                } else {
                    // No selection vector: all rows in batch passed (or no filter)
                    // Use the batch's row_count
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

    println!("\nNote: Using mock data. Replace data sources with real implementations for accurate benchmarks.");
}

fn parse(statement: &str) -> Result<Parsed<'_>, ParserError<'_>> {
    Parser::default().parse(statement)
}

fn lower(
    catalog: &dyn SharedCatalog,
    parsed: &Parsed<'_>,
) -> Result<LogicalPlan<partiql_logical::BindingsOp>, AstTransformationError> {
    let planner = LogicalPlanner::new(catalog);
    planner.lower(parsed)
}

fn compile(
    mode: EvaluationMode,
    catalog: &dyn SharedCatalog,
    logical: LogicalPlan<partiql_logical::BindingsOp>,
) -> Result<EvalPlan, PlanErr> {
    let mut planner = EvaluatorPlanner::new(mode, catalog);
    planner.compile(&logical)
}

/// Count total rows from a file-based data source
fn count_rows_from_file(data_source: &str, file_path: &str) -> usize {
    match data_source {
        "arrow" => {
            use arrow_ipc::reader::FileReader;
            use std::fs::File;
            if let Ok(file) = File::open(file_path) {
                if let Ok(reader) = FileReader::try_new(file, None) {
                    let mut total = 0;
                    for batch_result in reader {
                        if let Ok(batch) = batch_result {
                            total += batch.num_rows();
                        }
                    }
                    return total;
                }
            }
            0
        }
        "parquet" => {
            use parquet::arrow::arrow_reader::ParquetRecordBatchReaderBuilder;
            use std::fs::File;
            if let Ok(file) = File::open(file_path) {
                if let Ok(builder) = ParquetRecordBatchReaderBuilder::try_new(file) {
                    if let Ok(reader) = builder.build() {
                        let mut total = 0;
                        for batch_result in reader {
                            if let Ok(batch) = batch_result {
                                total += batch.num_rows();
                            }
                        }
                        return total;
                    }
                }
            }
            0
        }
        "ion" => {
            // For Ion, try to parse the filename pattern first (e.g., data_b4096_n244.ion)
            // If that fails, parse the Ion file to count rows
            if let Some(filename) = std::path::Path::new(file_path).file_name() {
                if let Some(name_str) = filename.to_str() {
                    // Try to extract batch_size and num_batches from filename
                    // Pattern: data_b<batch_size>_n<num_batches>.ion
                    if let Some(b_pos) = name_str.find("_b") {
                        if let Some(n_pos) = name_str.find("_n") {
                            if let Some(ext_pos) = name_str.rfind('.') {
                                if let Ok(batch_size) = name_str[b_pos + 2..n_pos].parse::<usize>() {
                                    if let Ok(num_batches) = name_str[n_pos + 2..ext_pos].parse::<usize>() {
                                        return batch_size * num_batches;
                                    }
                                }
                            }
                        }
                    }
                }
            }
            // Fallback: parse Ion file to count rows (more expensive)
            if let Ok(contents) = std::fs::read_to_string(file_path) {
                if let Ok(reader) = ReaderBuilder::new().build(contents) {
                    if let Ok(mut decoder) = IonDecoderBuilder::new(IonDecoderConfig::default().with_mode(Encoding::Ion))
                        .build(reader) {
                        let mut count = 0;
                        while let Some(result) = decoder.next() {
                            if result.is_ok() {
                                count += 1;
                            }
                        }
                        return count;
                    }
                }
            }
            0
        }
        _ => 0,
    }
}

fn compile_vectorized(
    logical: &LogicalPlan<partiql_logical::BindingsOp>,
    data_source: &str,
    data_path: Option<&str>,
) -> partiql_eval_vectorized::VectorizedPlan {
    use partiql_eval_vectorized::reader::{ArrowReader, InMemoryGeneratedReader, PIonReader, ParquetReader};

    let reader: Box<dyn partiql_eval_vectorized::BatchReader> = match data_source {
        "mem" => {
            // Configure reader size via environment variables
            let batch_size = std::env::var("BATCH_SIZE")
                .ok()
                .and_then(|s| s.parse().ok())
                .unwrap_or(1024);
            let num_batches = std::env::var("NUM_BATCHES")
                .ok()
                .and_then(|s| s.parse().ok())
                .unwrap_or(10_000);
            Box::new(InMemoryGeneratedReader::with_config(batch_size, num_batches))
        }
        "arrow" => {
            let path = data_path.expect("--data-path-new required for arrow data source");
            Box::new(ArrowReader::from_file(path).expect("Failed to create ArrowReader"))
        }
        "parquet" => {
            let path = data_path.expect("--data-path-new required for parquet data source");
            let batch_size = std::env::var("BATCH_SIZE")
                .ok()
                .and_then(|s| s.parse().ok())
                .unwrap_or(1024);
            Box::new(ParquetReader::from_file(path, batch_size).expect("Failed to create ParquetReader"))
        }
        "ion" => {
            let path = data_path.expect("--data-path-new required for ion data source");
            let batch_size = std::env::var("BATCH_SIZE")
                .ok()
                .and_then(|s| s.parse().ok())
                .unwrap_or(1024);
            Box::new(PIonReader::from_ion_file(path, batch_size).expect("Failed to create IonReader"))
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
