use partiql_tools::common;

use common::{
    count_rows_from_file, create_catalog, lower, parse, random_catalog, simple_catalog,
    CompiledSourceFactory,
};
use partiql_eval::plan::EvaluationMode;
use partiql_eval::value::Shape;
use partiql_eval::{CompilationContext, ExecutionCatalog, ExecutionContext, PlanCompiler};
use partiql_value::{Tuple, Value};
use std::borrow::Cow;
use std::time::Instant;

use clap::{Parser, Subcommand};
use reedline::{Prompt, PromptEditMode, PromptHistorySearch, Reedline, Signal};

const BATCH_SIZE: usize = 1;
const NUM_BATCHES: usize = 10_000;

/// Pqlite: An interactive PartiQL database engine and REPL.
///
/// Note: `~input~` in the query is replaced with `data`.
#[derive(Parser)]
#[command(name = "pqlite")]
struct Cli {
    /// Data source: mem | rand | ion | ionb.
    #[arg(long, default_value = "mem", global = true)]
    data_source: String,

    /// Path to the data file (required for file-based data sources).
    #[arg(long, global = true)]
    data_path: Option<String>,

    #[command(subcommand)]
    command: Option<Commands>,
}

#[derive(Subcommand)]
enum Commands {
    /// Execute a single query immediately
    Exec {
        /// The mandatory PartiQL query string to run
        query: String,
    },
}

fn main() {
    let cli = Cli::parse();

    // File-based data sources require an explicit --data-path.
    if cli.data_source != "mem" && cli.data_source != "rand" && cli.data_path.is_none() {
        eprintln!(
            "Error: --data-path is required for file-based data source '{}'",
            cli.data_source
        );
        std::process::exit(1);
    }

    match &cli.command {
        Some(Commands::Exec { query }) => {
            if let Err(e) = execute_query(query, &cli.data_source, cli.data_path.as_ref()) {
                eprintln!("{}", e);
                std::process::exit(1);
            }
        }
        None => {
            run_repl(&cli.data_source, cli.data_path.as_ref());
        }
    }
}

/// Minimal REPL prompt that renders a fixed `pqlite> ` indicator.
struct PqlitePrompt;

impl Prompt for PqlitePrompt {
    fn render_prompt_left(&self) -> Cow<'_, str> {
        Cow::Borrowed("pqlite")
    }

    fn render_prompt_right(&self) -> Cow<'_, str> {
        Cow::Borrowed("")
    }

    fn render_prompt_indicator(&self, _prompt_mode: PromptEditMode) -> Cow<'_, str> {
        Cow::Borrowed("> ")
    }

    fn render_prompt_multiline_indicator(&self) -> Cow<'_, str> {
        Cow::Borrowed("... ")
    }

    fn render_prompt_history_search_indicator(
        &self,
        _history_search: PromptHistorySearch,
    ) -> Cow<'_, str> {
        Cow::Borrowed("(search) ")
    }
}

/// Run the interactive REPL: read a line, execute it as a query, and loop.
///
/// Errors from `execute_query` are reported but never terminate the session.
fn run_repl(data_source: &str, data_path: Option<&String>) {
    let mut line_editor = Reedline::create();
    let prompt = PqlitePrompt;

    loop {
        match line_editor.read_line(&prompt) {
            Ok(Signal::Success(buffer)) => {
                if buffer.trim().is_empty() {
                    continue;
                }
                if let Err(e) = execute_query(&buffer, data_source, data_path) {
                    eprintln!("{}", e);
                }
            }
            Ok(Signal::CtrlC) => {
                println!("^C");
            }
            Ok(Signal::CtrlD) => {
                println!("Exiting...");
                break;
            }
            Ok(_) => {
                // Other signals (HostCommand / ExternalBreak) are not used here.
            }
            Err(e) => {
                eprintln!("REPL error: {}", e);
                break;
            }
        }
    }
}

fn execute_query(
    query_str: &str,
    data_source: &str,
    data_path: Option<&String>,
) -> Result<(), Box<dyn std::error::Error>> {
    // Replace !input with data (no parentheses for hybrid)
    let query = query_str.replace("~input~", "data");

    println!("Query:       {}", query);
    println!("Data Source: {}", data_source);
    if let Some(path) = data_path {
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
    } else if let Some(path) = data_path {
        let total = count_rows_from_file(data_source, path);
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

    let catalog = create_catalog(data_source.to_string(), data_path.cloned());

    // Default column names for in-memory reader
    let column_names = vec!["a".to_string(), "b".to_string()];

    // Phase 1: Parse
    let parse_start = Instant::now();
    let parsed = parse(&query).map_err(|e| format!("Parse error: {:?}", e))?;
    let parse_time = parse_start.elapsed();

    // === PIPELINE TRACE: AST ===
    println!("\n{}", "=".repeat(60));
    println!("PHASE 1: PARSED AST");
    println!("{}", "=".repeat(60));
    println!("{:#?}", parsed);

    // Phase 2: Lower (AST → Logical Plan)
    let lower_start = Instant::now();
    let logical = lower(&*catalog, &parsed).map_err(|e| format!("Lower error: {:?}", e))?;
    let lower_time = lower_start.elapsed();

    // === PIPELINE TRACE: LOGICAL PLAN ===
    println!("\n{}", "=".repeat(60));
    println!("PHASE 2: LOGICAL PLAN");
    println!("{}", "=".repeat(60));
    println!("{:#?}", logical);

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

    let (comp_catalog, mut exec_catalog_inner) = match data_source {
        "rand" => {
            // Random catalog for custom reader demonstration
            let (comp, exec) =
                random_catalog(vec![("data".to_string(), total_rows, column_names.clone())]);
            (comp, ExecCatalog::Random(exec))
        }
        "mem" | "ion" | "ionb" => {
            // Simple catalog for mem/ion data sources - use CompiledSourceFactory
            let factory = match data_source {
                "mem" => CompiledSourceFactory::mem(total_rows, column_names.clone()),
                "ion" | "ionb" => {
                    CompiledSourceFactory::ion(data_path.cloned().unwrap_or_default())
                }
                _ => unreachable!(),
            };
            let (comp, exec) = simple_catalog(vec![("data".to_string(), factory)]);
            (comp, ExecCatalog::Simple(exec))
        }
        _ => {
            return Err(format!("Unsupported data source: {}", data_source).into());
        }
    };

    // Add catalog and CAPTURE the returned catalog_id - this is the ONLY place catalog_id is assigned
    let catalog_id = context.add_catalog("default", comp_catalog);

    let mut compiler = PlanCompiler::new(&context, EvaluationMode::Permissive);
    let compiled = compiler
        .compile(&logical)
        .map_err(|e| format!("Compile error: {:?}", e))?;
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

    let mut vm = partiql_eval::PartiQLVM::new(compiled, &exec_context)
        .map_err(|e| format!("Execution setup error: {:?}", e))?;

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
                        return Err(format!("Execution error: {:?}", e).into());
                    }
                }
            }
            println!();
            if let Some(s) = suffix {
                println!("{}", s);
            }
        }
        Err(e) => {
            return Err(format!("Execution setup error: {:?}", e).into());
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

    Ok(())
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
