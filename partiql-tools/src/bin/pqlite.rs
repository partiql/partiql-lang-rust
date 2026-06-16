use partiql_tools::common;

use common::{create_table_fn_catalog, lower_statement, parse};
use partiql_eval::plan::EvaluationMode;
use partiql_eval::value::Shape;
use partiql_eval::{CompilationContext, ExecutionContext, PlanCompiler};
use partiql_logical::LogicalStatement;
use partiql_tools::storage::{default_db_path, HeedDB};
use partiql_value::{Tuple, Value};
use std::borrow::Cow;
use std::time::Instant;

use clap::{Parser, Subcommand};
use reedline::{
    FileBackedHistory, History, Prompt, PromptEditMode, PromptHistorySearch, Reedline, Signal,
    ValidationResult, Validator,
};

/// Maximum number of lines retained in the REPL history file.
const HISTORY_CAPACITY: usize = 1000;

/// Version string shared by the `--version` flag and the REPL startup banner:
/// the crate version joined with the git commit SHA captured in build.rs.
const VERSION: &str = concat!(env!("CARGO_PKG_VERSION"), "@", env!("PQLITE_GIT_SHA"));

/// Pqlite: An interactive PartiQL database engine and REPL.
///
/// Use table functions in queries to access data:
///   SELECT t.a FROM mem(100, 2) t;
///   SELECT t.name FROM scan_ion('data.ion') t;
#[derive(Parser)]
#[command(name = "pqlite", version = VERSION)]
struct Cli {
    /// Print debug info for pipeline stages. Accepts: ast, plan, program, or * for all.
    #[arg(long, global = true, value_delimiter = ',')]
    debug: Vec<String>,

    /// Path to the database file. Defaults to ~/.pqlite/default.pal.
    #[arg(long, global = true)]
    db: Option<std::path::PathBuf>,

    #[command(subcommand)]
    command: Option<Commands>,
}

struct DebugFlags {
    ast: bool,
    plan: bool,
    program: bool,
}

impl DebugFlags {
    fn from_args(args: &[String]) -> Self {
        let all = args.iter().any(|s| s == "*");
        DebugFlags {
            ast: all || args.iter().any(|s| s == "ast"),
            plan: all || args.iter().any(|s| s == "plan"),
            program: all || args.iter().any(|s| s == "program"),
        }
    }
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
    let debug = DebugFlags::from_args(&cli.debug);

    match &cli.command {
        Some(Commands::Exec { query }) => {
            let query = normalize_query(query);
            if query.is_empty() {
                eprintln!("Error: empty query");
                std::process::exit(1);
            }
            if let Err(e) = execute_query(query, &debug) {
                eprintln!("{}", e);
                std::process::exit(1);
            }
        }
        None => {
            // Open the database lazily — only the interactive REPL needs it.
            // A one-shot `exec` query is read-only today, so it must not
            // manifest a database file on disk as a side effect (e.g. silently
            // creating ~/.pqlite/default.pal). When the write path lands, exec
            // will open the db explicitly where it needs to.

            // Resolve the database path: explicit --db wins, else the default.
            let db_path = cli.db.clone().or_else(default_db_path).unwrap_or_else(|| {
                eprintln!(
                    "Error: could not resolve a database path (home directory not found); pass --db <PATH>"
                );
                std::process::exit(1);
            });

            // Open (or create) the database up front. A database that silently
            // stops persisting is worse than one that refuses to start, so this
            // hard-fails (unlike the REPL history file, which falls back to
            // in-memory by design).
            let db = match HeedDB::open(&db_path) {
                Ok(db) => db,
                Err(e) => {
                    eprintln!(
                        "Error: could not open database at {}: {}",
                        db_path.display(),
                        e
                    );
                    std::process::exit(1);
                }
            };

            run_repl(&debug, &db);
        }
    }
}

/// Normalize a query string before handing it to the engine: trim outer
/// whitespace and strip a single trailing `;` statement terminator, while
/// preserving any internal newlines. The `;` is accepted as a convenience
/// terminator (and is what the REPL validator uses to detect a complete
/// entry) but is not part of the PartiQL grammar, so it must be removed.
///
/// TODO: support the trailing `;` statement terminator natively in the
/// PartiQL grammar/parser so semicolons are a first-class part of the
/// language, and remove this stripping workaround once that lands.
fn normalize_query(input: &str) -> &str {
    let trimmed = input.trim();
    trimmed.strip_suffix(';').unwrap_or(trimmed).trim_end()
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

/// Validator that decides when a multi-line REPL entry is complete.
///
/// An input is considered complete when, after trimming trailing whitespace,
/// it either:
///   * ends with a semicolon (`;`) — a finished PartiQL statement, or
///   * is a meta-command starting with a dot (`.`).
///
/// Anything else is treated as `Incomplete`, so pressing Enter inserts a
/// newline and lets the user keep typing on the following line.
struct PqliteValidator;

impl Validator for PqliteValidator {
    fn validate(&self, line: &str) -> ValidationResult {
        let trimmed = line.trim_end();
        if trimmed.trim_start().starts_with('.') || trimmed.ends_with(';') {
            ValidationResult::Complete
        } else {
            ValidationResult::Incomplete
        }
    }
}

/// Build a persistent, file-backed history at `~/.pqlite_history`.
///
/// Falls back to in-memory history (never crashing) if the home directory
/// cannot be resolved or the history file cannot be opened.
fn build_history() -> Box<dyn History> {
    let history_path = home_dir().map(|mut p| {
        p.push(".pqlite_history");
        p
    });

    if let Some(path) = history_path {
        match FileBackedHistory::with_file(HISTORY_CAPACITY, path.clone()) {
            Ok(history) => return Box::new(history),
            Err(e) => {
                eprintln!(
                    "Warning: could not open history file {}: {} (using in-memory history)",
                    path.display(),
                    e
                );
            }
        }
    } else {
        eprintln!("Warning: could not resolve home directory (using in-memory history)");
    }

    // Fallback: in-memory history. If even this fails, run without history.
    match FileBackedHistory::new(HISTORY_CAPACITY) {
        Ok(history) => Box::new(history),
        Err(_) => Box::new(FileBackedHistory::default()),
    }
}

/// Resolve the user's home directory across platforms.
fn home_dir() -> Option<std::path::PathBuf> {
    // `std::env::home_dir` was un-deprecated in Rust 1.85 and is correct on
    // all supported platforms, so no extra crate is needed here.
    #[allow(deprecated)]
    std::env::home_dir()
}

/// Print the REPL help menu listing meta-commands and query usage.
fn print_help() {
    println!("PartiQL REPL — available commands:");
    println!("  .help     Show this help message");
    println!("  .quit     Exit the REPL (alias: .exit)");
    println!("  .exit     Exit the REPL (alias: .quit)");
    println!();
    println!("To run a query, type any PartiQL statement and press Enter.");
    println!("Available table functions:");
    println!("  mem(rows, cols)       — sequential integer data");
    println!("  rand(rows, cols)      — random integer data");
    println!("  scan_ion(path)        — read Ion file");
    println!();
    println!("Example: SELECT t.a, t.b FROM mem(100, 2) t LIMIT 5;");
}

/// Outcome of handling a REPL meta-command (a line starting with `.`).
enum MetaOutcome {
    /// The command was handled; continue the loop.
    Handled,
    /// The user requested to quit; break the loop.
    Quit,
}

/// Handle a dot-prefixed meta-command. `input` must be the trimmed buffer.
fn handle_meta_command(input: &str) -> MetaOutcome {
    match input {
        ".quit" | ".exit" => MetaOutcome::Quit,
        ".help" => {
            print_help();
            MetaOutcome::Handled
        }
        _ => {
            println!("Unrecognized command. Type .help for a list of commands.");
            MetaOutcome::Handled
        }
    }
}

/// Print the REPL startup banner: the crate version, the git commit it was
/// built from, the active database path, and a pointer to the help command.
///
/// Written to stderr, not stdout: the banner is startup chrome, and stdout is
/// reserved for the query result stream so it can be piped/redirected cleanly.
fn print_startup_banner(db_path: &std::path::Path) {
    // Reuse the same VERSION string that backs the `--version` flag so the two
    // can never drift (crate version + git SHA captured in build.rs).
    eprintln!("pqlite version {VERSION}");
    eprintln!("database: {}", db_path.display());
    eprintln!("For usage information, enter \".help\".");
}

/// Run the interactive REPL: read a line, execute it as a query, and loop.
///
/// Errors from `execute_query` are reported but never terminate the session.
fn run_repl(debug: &DebugFlags, db: &HeedDB) {
    print_startup_banner(db.path());

    let mut line_editor = Reedline::create()
        .with_history(build_history())
        .with_validator(Box::new(PqliteValidator));
    let prompt = PqlitePrompt;

    loop {
        match line_editor.read_line(&prompt) {
            Ok(Signal::Success(buffer)) => {
                let trimmed = buffer.trim();
                if trimmed.is_empty() {
                    continue;
                }

                // Meta-commands start with a dot and are handled locally.
                if trimmed.starts_with('.') {
                    match handle_meta_command(trimmed) {
                        MetaOutcome::Handled => continue,
                        MetaOutcome::Quit => break,
                    }
                }

                // Strip the trailing `;` terminator (preserving internal
                // newlines) before handing the full multi-line text to the
                // engine — shared with the `exec` subcommand for consistency.
                let query = normalize_query(trimmed);

                // A lone `;` (e.g. on its own line) leaves nothing to run once
                // the terminator is stripped — skip it rather than handing an
                // empty string to the parser.
                if query.is_empty() {
                    continue;
                }

                if let Err(e) = execute_query(query, debug) {
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

/// Render a `BindingsName` the SQL-idiomatic way, so the printed form round-trips
/// the case-sensitivity the user typed (instead of the `CaseInsensitive("t")` Debug
/// wrapper, which both leaks internals and reads identically for quoted vs bare):
///   - case-sensitive (originally quoted) -> re-quoted, e.g. `"My_Table"`
///   - case-insensitive (originally bare) -> bare, e.g. `my_table`
fn format_table_name(name: &partiql_value::BindingsName<'_>) -> String {
    match name {
        partiql_value::BindingsName::CaseSensitive(s) => format!("\"{}\"", s),
        partiql_value::BindingsName::CaseInsensitive(s) => s.as_ref().to_string(),
    }
}

/// Shared stderr footer for a planning-only DDL statement.
fn print_ddl_planned_footer(elapsed: std::time::Duration) {
    eprintln!(
        "(planned in {:.1}ms — execution not yet implemented)",
        elapsed.as_secs_f64() * 1000.0
    );
}

fn execute_query(query_str: &str, debug: &DebugFlags) -> Result<(), Box<dyn std::error::Error>> {
    let query = query_str.to_string();

    let catalog = create_table_fn_catalog();

    // Phase 1: Parse
    let parse_start = Instant::now();
    let parsed = parse(&query).map_err(|e| format!("Parse error: {:?}", e))?;
    let parse_time = parse_start.elapsed();

    if debug.ast {
        eprintln!("[AST] {:?}", parsed);
    }

    // Phase 2: Lower (AST → Logical Statement)
    // pqlite runs exactly one statement per submission; reject anything else here,
    // since lowering operates on a single statement.
    let stmt = match parsed.statements.as_slice() {
        [stmt] => stmt,
        // Match the planner's wording for this condition so the failure reads the
        // same whether it surfaces here or via `LogicalPlanner::lower`.
        _ => return Err("Lower error: multi-statement input".into()),
    };
    let lower_start = Instant::now();
    let statement =
        lower_statement(&*catalog, stmt).map_err(|e| format!("Lower error: {:?}", e))?;
    let lower_time = lower_start.elapsed();

    // DDL statements are planning-only for now: print the lowered plan and stop.
    // (Compile/execute and storage are a later slice; the consumer would
    // orchestrate writes around the inner query's execution.)
    let logical = match statement {
        LogicalStatement::Query(plan) => plan,
        LogicalStatement::CreateTableAs { table_name, query } => {
            // `{}` (Display) on the plan, not `{:?}`: LogicalPlan has a readable
            // Display impl; Debug would dump the raw nodes/edges struct.
            println!(
                "Planned CREATE TABLE {} AS:",
                format_table_name(&table_name)
            );
            println!("{}", query);
            print_ddl_planned_footer(parse_time + lower_time);
            return Ok(());
        }
        LogicalStatement::CreateTable { table_name } => {
            println!(
                "Planned CREATE TABLE {} (no source query)",
                format_table_name(&table_name)
            );
            print_ddl_planned_footer(parse_time + lower_time);
            return Ok(());
        }
    };

    if debug.plan {
        eprintln!("[Plan] {:?}", logical);
    }

    // Phase 3: Compile (Logical → CompiledPlan)
    let compile_start = Instant::now();

    // Set up compilation context with table function support
    let mut context = CompilationContext::new();

    // Use TableFnCompilationCatalog which supports both table lookups and table functions
    let column_names = vec!["a".to_string(), "b".to_string()];
    let comp_catalog: std::sync::Arc<dyn partiql_eval::CompilationCatalog> =
        std::sync::Arc::new(common::TableFnCompilationCatalog::new(column_names));
    let _catalog_id = context.add_catalog("default", comp_catalog);

    let mut compiler = PlanCompiler::new(&context, EvaluationMode::Permissive);
    let compiled = compiler
        .compile(&logical)
        .map_err(|e| format!("Compile error: {:?}", e))?;
    let compile_time = compile_start.elapsed();

    if debug.program {
        eprintln!("[Program]\n{}", compiled);
    }

    // Phase 4: Execute
    let exec_start = Instant::now();

    // Create ExecutionContext with table functions registered
    let mut exec_context = ExecutionContext::new();
    exec_context.register_table_function("rand", std::sync::Arc::new(common::RandTableFunction));
    exec_context.register_table_function("mem", std::sync::Arc::new(common::MemTableFunction));
    exec_context.register_table_function(
        "scan_ion",
        std::sync::Arc::new(common::ScanIonTableFunction),
    );

    let mut vm = partiql_eval::PartiQLVM::new(compiled, &exec_context)
        .map_err(|e| format!("Execution setup error: {:?}", e))?;

    let shape = vm.shape().clone();
    let mut row_count = 0usize;

    let (prefix, tab, suffix) = match shape {
        Shape::Bag(_) => (Some("<<"), "  ", Some(">>")),
        Shape::List(_) => (Some("["), "  ", Some("]")),
        Shape::Single(_) => (None, "", None),
    };

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

    let total_time = parse_time + lower_time + compile_time + exec_time;
    eprintln!(
        "({} rows in {:.1}ms — parse: {:.1}ms, lower: {:.1}ms, compile: {:.1}ms, exec: {:.1}ms)",
        row_count,
        total_time.as_secs_f64() * 1000.0,
        parse_time.as_secs_f64() * 1000.0,
        lower_time.as_secs_f64() * 1000.0,
        compile_time.as_secs_f64() * 1000.0,
        exec_time.as_secs_f64() * 1000.0,
    );

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
