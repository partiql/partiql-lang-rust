use partiql_ast_passes::error::AstTransformationError;
use partiql_catalog::call_defs::{CallDef, CallSpec};
use partiql_catalog::catalog::{MutableCatalog, PartiqlCatalog, SharedCatalog};
use partiql_catalog::context::SessionContext;
use partiql_catalog::table_fn::{
    BaseTableExpr, BaseTableExprResult, BaseTableFunctionInfo, TableFunction,
};
use partiql_eval::error::PlanErr;
use partiql_eval::eval::EvalPlan;
use partiql_eval::plan::{EvaluationMode, EvaluatorPlanner};
use partiql_logical::LogicalPlan;
use partiql_logical_planner::LogicalPlanner;
use partiql_parser::{Parsed, Parser, ParserError};
use partiql_value::{Tuple, Value};
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
        let iter = (0..10_240_000).map(|i| {
            let tuple = Tuple::from([("a", Value::Integer(i)), ("b", Value::Integer(i + 100))]);
            Ok(Value::Tuple(Box::new(tuple)))
        });

        Ok(Box::new(iter))
    }
}

fn main() {
    // Parse command line arguments
    let args: Vec<String> = std::env::args().collect();
    if args.len() < 2 {
        eprintln!("Usage: {} <non-vectorized-query> [vectorized-query]", args[0]);
        eprintln!("Examples:");
        eprintln!("  {} \"SELECT a FROM data()\"", args[0]);
        eprintln!("  {} \"SELECT * FROM data()\" \"SELECT a FROM data\"", args[0]);
        eprintln!("\nIf only one query is provided, it will be used for both evaluators.");
        std::process::exit(1);
    }

    let non_vec_query = &args[1];
    let vec_query = if args.len() >= 3 {
        &args[2]
    } else {
        non_vec_query
    };
    
    println!("Non-Vectorized Query: {}", non_vec_query);
    println!("Vectorized Query:     {}\n", vec_query);

    // Create catalog and add the data table function
    let mut catalog = PartiqlCatalog::default();
    let data_fn = TableFunction::new(Box::new(DataTableFunction));
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
    let non_vec_parsed = match parse(non_vec_query) {
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
    let vec_parsed = match parse(vec_query) {
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
    let mut vec_plan = compile_vectorized(&vec_logical);
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

fn compile_vectorized(
    logical: &LogicalPlan<partiql_logical::BindingsOp>,
) -> partiql_eval_vectorized::VectorizedPlan {
    // TODO: This is complier's work - convert LogicalPlan to PhysicalPlan
    // They should analyze the logical plan and create appropriate ProjectionSpec
    // For now, using mock data source for compatibility

    use partiql_eval_vectorized::reader::InMemoryGeneratedReader;

    let reader: Box<dyn partiql_eval_vectorized::BatchReader> =
        Box::new(InMemoryGeneratedReader::new());

    let context = partiql_eval_vectorized::CompilerContext::new()
        .with_data_source("data".to_string(), reader);

    let mut compiler = partiql_eval_vectorized::Compiler::new(context);
    compiler
        .compile(logical)
        .expect("Vectorized compilation failed")
}

/// Print a batch with SelectionVector support
/// Handles Int64 and Boolean types
fn print_batch(batch: &partiql_eval_vectorized::VectorizedBatch) {
    let schema = batch.schema();
    let row_count = batch.row_count();

    // DIAGNOSTIC INFO
    // println!("[DEBUG] Batch info:");
    // println!("  - Row count: {}", row_count);
    // println!("  - Field count: {}", schema.field_count());
    // print!("  - Fields: ");
    // for field in schema.fields() {
        // print!("{} ({:?}), ", field.name, field.type_info);
    // }
    // println!();

    // If row_count is 0, nothing to print
    if row_count == 0 {
        // println!("[DEBUG] Skipping batch with 0 rows");
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
        println!(
            "({} rows selected out of {} total)",
            rows_to_print.len(),
            row_count
        );
    } else {
        println!("({} rows)", rows_to_print.len());
    }
    println!();
}

/// Format a single value from a physical vector
fn format_value(
    physical: &partiql_eval_vectorized::PhysicalVectorEnum,
    idx: usize,
    logical_type: partiql_eval_vectorized::LogicalType,
) -> String {
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
