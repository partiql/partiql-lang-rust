use partiql_ast_passes::error::AstTransformationError;
use partiql_catalog::call_defs::{CallDef, CallSpec};
use partiql_catalog::catalog::{MutableCatalog, PartiqlCatalog, SharedCatalog};
use partiql_catalog::context::SessionContext;
use partiql_catalog::table_fn::{BaseTableExpr, BaseTableExprResult, BaseTableFunctionInfo, TableFunction};
use partiql_eval::error::PlanErr;
use partiql_eval::eval::EvalPlan;
use partiql_eval::plan::{EvaluationMode, EvaluatorPlanner};
use partiql_eval_vectorized::{Compiler, CompilerContext};
use partiql_logical::LogicalPlan;
use partiql_logical_planner::LogicalPlanner;
use partiql_parser::{Parsed, Parser, ParserError};
use partiql_value::{Value, Tuple};
use std::borrow::Cow;
use std::time::Instant;

// Table function that generates 1,024,000 rows with fields 'a' and 'b'
#[derive(Debug)]
struct DataTableFunction;

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
        Box::new(DataTableExpr)
    }
}

#[derive(Debug)]
struct DataTableExpr;

impl BaseTableExpr for DataTableExpr {
    fn evaluate<'c>(
        &self,
        _args: &[Cow<'_, Value>],
        _ctx: &'c dyn SessionContext,
    ) -> BaseTableExprResult<'c> {
        // Create an iterator that generates 1,024,000 tuples
        let iter = (0..10_024_000).map(|i| {
            let tuple = Tuple::from([
                ("a", Value::Integer(i)),
                ("b", Value::Integer(i + 100)),
            ]);
            Ok(Value::Tuple(Box::new(tuple)))
        });

        Ok(Box::new(iter))
    }
}

fn main() {
    // Parse command line arguments
    let args: Vec<String> = std::env::args().collect();
    if args.len() < 2 {
        eprintln!("Usage: {} <query>", args[0]);
        eprintln!("Example: {} \"SELECT a, b FROM data()\"", args[0]);
        std::process::exit(1);
    }

    let query = &args[1];
    println!("Query: {}\n", query);

    // Create catalog and add the data table function
    let mut catalog = PartiqlCatalog::default();
    let data_fn = TableFunction::new(Box::new(DataTableFunction));
    catalog.add_table_function(data_fn).expect("Failed to add table function");
    let catalog = catalog.to_shared_catalog();

    // Phase 1: Parse
    let parse_start = Instant::now();
    let parsed = match parse(query) {
        Ok(p) => p,
        Err(e) => {
            eprintln!("Parse error: {:?}", e);
            std::process::exit(1);
        }
    };
    let parse_time = parse_start.elapsed();

    // Phase 2: Lower (AST → Logical Plan)
    let lower_start = Instant::now();
    let logical = match lower(&catalog, &parsed) {
        Ok(l) => l,
        Err(e) => {
            eprintln!("Lower error: {:?}", e);
            std::process::exit(1);
        }
    };
    let lower_time = lower_start.elapsed();

    // Phase 3: Compile (Logical → Physical/Executable Plan) - Non-Vectorized
    let compile_start = Instant::now();
    let plan = match compile(EvaluationMode::Permissive, &catalog, logical.clone()) {
        Ok(p) => p,
        Err(e) => {
            eprintln!("Non-vectorized compile error: {:?}", e);
            std::process::exit(1);
        }
    };
    let compile_time = compile_start.elapsed();

    // Phase 4: Vectorized Compilation (convert to vectorized operators)
    let vec_compile_start = Instant::now();
    let mut vec_plan = compile_vectorized(&logical);
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
    use partiql_value::{DateTime, Bag};
    let bindings = MapBindings::default();
    let sys = partiql_catalog::context::SystemContext { now: DateTime::from_system_now_utc() };
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
    
    println!("Execution time: {:.3}ms", non_vec_exec_time.as_secs_f64() * 1000.0);
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
                    // Count selected rows (length of indices vector)
                    selection.indices.len()
                } else {
                    // No selection vector, use total row count
                    batch.row_count()
                };
                
                vec_row_count += batch_row_count;
                // Optionally print batch details
                // print_batch(&batch);
            }
            Err(e) => {
                eprintln!("Vectorized execution error: {:?}", e);
                std::process::exit(1);
            }
        }
    }
    let vec_exec_time = vec_exec_start.elapsed();
    
    println!("Execution time: {:.3}ms", vec_exec_time.as_secs_f64() * 1000.0);
    println!("Batches processed: {}", batch_count);
    println!("Rows processed: {}", vec_row_count);

    // Summary Comparison
    println!("\n{}", "=".repeat(60));
    println!("TIMING SUMMARY");
    println!("{}", "=".repeat(60));
    
    println!("\nPlanning Phase:");
    println!("  Parse:                {:.3}ms", parse_time.as_secs_f64() * 1000.0);
    println!("  Lower:                {:.3}ms", lower_time.as_secs_f64() * 1000.0);
    println!("  Non-Vec Compile:      {:.3}ms", compile_time.as_secs_f64() * 1000.0);
    println!("  Vectorized Compile:   {:.3}μs", vec_compile_time.as_secs_f64() * 1_000_000.0);
    
    println!("\nExecution Phase:");
    println!("  Non-Vectorized:       {:.3}ms", non_vec_exec_time.as_secs_f64() * 1000.0);
    println!("  Vectorized:           {:.3}ms", vec_exec_time.as_secs_f64() * 1000.0);
    
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

fn compile_vectorized(
    logical: &LogicalPlan<partiql_logical::BindingsOp>,
) -> partiql_eval_vectorized::VectorizedPlan {
    use partiql_eval_vectorized::{BatchReader, Field, SourceTypeDef, Tuple, TupleIteratorReader, LogicalType};
    
    // Create a dummy schema for the "data" table
    let schema = SourceTypeDef::new(vec![
        Field {
            name: "a".to_string(),
            type_info: LogicalType::Int64,
        },
        Field {
            name: "b".to_string(),
            type_info: LogicalType::Int64,
        },
    ]);
    
    // Create a dummy reader (would be replaced with actual data source)
    let tuples: Vec<Tuple> = vec![];
    let reader: Box<dyn BatchReader> = Box::new(TupleIteratorReader::new(
        Box::new(tuples.into_iter()),
        schema,
        1024,
    ));
    
    // Create compiler context with data source
    let context = CompilerContext::new()
        .with_data_source("data".to_string(), reader);
    
    // Create compiler and compile the logical plan
    let mut compiler = Compiler::new(context);
    compiler.compile(logical).expect("Vectorized compilation failed")
}

/// Print a batch with SelectionVector support
/// Handles Int64 and Boolean types
fn print_batch(batch: &partiql_eval_vectorized::VectorizedBatch) {
    let schema = batch.schema();
    let row_count = batch.row_count();
    
    // DIAGNOSTIC INFO
    println!("[DEBUG] Batch info:");
    println!("  - Row count: {}", row_count);
    println!("  - Field count: {}", schema.field_count());
    print!("  - Fields: ");
    for field in schema.fields() {
        print!("{} ({:?}), ", field.name, field.type_info);
    }
    println!();
    
    // If row_count is 0, nothing to print
    if row_count == 0 {
        println!("[DEBUG] Skipping batch with 0 rows");
        return;
    }
    
    // Get selection vector if present
    let selection = batch.selection();
    
    // Determine which rows to print
    let rows_to_print: Vec<usize> = if let Some(sel) = selection {
        // Use selection indices
        sel.indices.clone()
    } else {
        // Print all rows
        (0..row_count).collect()
    };
    
    // Print header
    print!("|");
    for field in schema.fields() {
        print!(" {:>10} |", field.name);
    }
    println!();
    
    // Print separator
    print!("|");
    for _ in 0..schema.field_count() {
        print!("------------|");
    }
    println!();
    
    // Print rows
    for &row_idx in &rows_to_print {
        print!("|");
        for col_idx in 0..schema.field_count() {
            let column = batch.column(col_idx).expect("Column should exist");
            let value_str = format_value(&column.physical, row_idx, column.logical_type());
            print!(" {:>10} |", value_str);
        }
        println!();
    }
    
    // Print summary
    if let Some(_sel) = selection {
        println!("({} rows selected out of {} total)", rows_to_print.len(), row_count);
    } else {
        println!("({} rows)", rows_to_print.len());
    }
    println!();
}

/// Format a single value from a physical vector
fn format_value(physical: &partiql_eval_vectorized::PhysicalVectorEnum, idx: usize, logical_type: partiql_eval_vectorized::LogicalType) -> String {
    use partiql_eval_vectorized::PhysicalVectorEnum;
    match physical {
        PhysicalVectorEnum::Int64(vec) => {
            let slice = vec.as_slice();
            if idx < slice.len() {
                slice[idx].to_string()
            } else {
                "NULL".to_string()
            }
        }
        PhysicalVectorEnum::Boolean(vec) => {
            let slice = vec.as_slice();
            if idx < slice.len() {
                if slice[idx] { "true" } else { "false" }.to_string()
            } else {
                "NULL".to_string()
            }
        }
        PhysicalVectorEnum::Float64(_) => {
            format!("<Float64:{}:unsupported>", logical_type as u8)
        }
        PhysicalVectorEnum::String(_) => {
            format!("<String:{}:unsupported>", logical_type as u8)
        }
    }
}
