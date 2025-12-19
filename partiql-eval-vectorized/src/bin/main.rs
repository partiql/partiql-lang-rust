use partiql_ast_passes::error::AstTransformationError;
use partiql_catalog::catalog::{PartiqlCatalog, SharedCatalog};
use partiql_eval::error::PlanErr;
use partiql_eval::eval::EvalPlan;
use partiql_eval::plan::{EvaluationMode, EvaluatorPlanner};
use partiql_eval_vectorized::{Compiler, CompilerContext};
use partiql_logical::LogicalPlan;
use partiql_logical_planner::LogicalPlanner;
use partiql_parser::{Parsed, Parser, ParserError};
use std::time::Instant;

fn main() {
    // Parse command line arguments
    let args: Vec<String> = std::env::args().collect();
    if args.len() < 2 {
        eprintln!("Usage: {} <query>", args[0]);
        eprintln!("Example: {} \"SELECT a, b FROM data WHERE a > 10\"", args[0]);
        std::process::exit(1);
    }

    let query = &args[1];
    println!("Query: {}\n", query);

    // Create catalog
    let catalog = PartiqlCatalog::default().to_shared_catalog();

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

    // Phase 3: Compile (Logical → Physical/Executable Plan)
    let compile_start = Instant::now();
    let _plan = match compile(EvaluationMode::Permissive, &catalog, logical.clone()) {
        Ok(p) => p,
        Err(e) => {
            eprintln!("Compile error: {:?}", e);
            std::process::exit(1);
        }
    };
    let compile_time = compile_start.elapsed();

    // Phase 4: Vectorized Compilation (convert to vectorized operators)
    let vec_compile_start = Instant::now();
    let mut _vec_plan = compile_vectorized(&logical);
    let vec_compile_time = vec_compile_start.elapsed();

    // Phase 5: Execute the vectorized plan
    let exec_start = Instant::now();
    let mut batch_count = 0;
    let mut row_count = 0;
    
    for batch_result in _vec_plan.execute() {
        match batch_result {
            Ok(batch) => {
                batch_count += 1;
                row_count += batch.row_count();
            }
            Err(e) => {
                eprintln!("Execution error: {:?}", e);
                std::process::exit(1);
            }
        }
    }
    let exec_time = exec_start.elapsed();

    // Calculate total planning time
    let total_planning_time = parse_time + lower_time + compile_time + vec_compile_time;

    // Print results
    println!("=== Timing Results ===");
    println!("Planning time: {:.3}ms", total_planning_time.as_secs_f64() * 1000.0);
    println!("  - Parse:           {:.3}ms", parse_time.as_secs_f64() * 1000.0);
    println!("  - Lower:           {:.3}ms", lower_time.as_secs_f64() * 1000.0);
    println!("  - Compile:         {:.3}ms", compile_time.as_secs_f64() * 1000.0);
    println!("  - Vec Compile:     {:.3}μs", vec_compile_time.as_secs_f64() * 1_000_000.0);
    
    println!("\n=== Execution Results ===");
    println!("Execution time: {:.3}ms", exec_time.as_secs_f64() * 1000.0);
    println!("Batches processed: {}", batch_count);
    println!("Rows processed: {}", row_count);
    println!("\nNote: Using mock generated data (1000 batches × 1024 rows). Replace generate_mock_batch() with real data source.");
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
    use partiql_eval_vectorized::{BatchReader, Field, SourceTypeDef, Tuple, TupleIteratorReader, TypeInfo};
    
    // Create a dummy schema for the "data" table
    let schema = SourceTypeDef::new(vec![
        Field {
            name: "a".to_string(),
            type_info: TypeInfo::Int64,
        },
        Field {
            name: "b".to_string(),
            type_info: TypeInfo::Int64,
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
