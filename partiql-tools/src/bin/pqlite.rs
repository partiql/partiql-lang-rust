use partiql_tools::common;

use common::{create_table_fn_catalog, lower_statement, parse};
use partiql_eval::plan::EvaluationMode;
use partiql_eval::value::Shape;
use partiql_eval::{CompilationContext, ExecutionContext, PlanCompiler};
use partiql_logical::LogicalStatement;
use partiql_tools::storage::{HeedDB, StorageError};
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

    /// Path to the database file (required for the interactive REPL). The
    /// parent directory must already exist; it is not created for you.
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

/// How `execute_query` obtains a database when a statement needs one.
///
/// Only `CREATE TABLE AS` needs storage. The REPL opens one handle for the
/// whole session and passes it as `Open`; `exec` passes `Lazy` so the database
/// is opened only if a write statement actually requires it — a plain query via
/// `exec` never opens or creates a file.
#[derive(Clone, Copy)]
enum DbSource<'a> {
    /// An already-open handle (the REPL session database).
    Open(&'a HeedDB),
    /// A `--db` path to open on demand (the `exec` subcommand); `None` if no
    /// `--db` was given.
    Lazy(Option<&'a std::path::Path>),
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
            if let Err(e) = execute_query(query, &debug, DbSource::Lazy(cli.db.as_deref())) {
                eprintln!("{}", e);
                std::process::exit(1);
            }
        }
        None => {
            // The REPL needs a database. We require an explicit --db rather
            // than inventing a default path: like standard UNIX tools, we don't
            // create files or directories on the user's behalf.
            let db_path = cli.db.clone().unwrap_or_else(|| {
                eprintln!("Error: The `--db <PATH>` option is required to open the database.");
                std::process::exit(1);
            });

            // Open the database up front. Hard-fail rather than fall back (a
            // silently-non-persisting db is worse than one that refuses to
            // start). The parent directory must already exist.
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

                let query = normalize_query(trimmed);

                // A lone `;` strips to empty — skip rather than parse "".
                if query.is_empty() {
                    continue;
                }

                if let Err(e) = execute_query(query, debug, DbSource::Open(db)) {
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

/// Derive the canonical catalog key for a table name. Bare (case-insensitive)
/// identifiers fold to lowercase so `foo`, `FOO`, and `Foo` collide as one
/// table; quoted (case-sensitive) identifiers are kept verbatim. This is
/// ASCII-equivalent to the engine's `UniCase` fold; full Unicode-consistent
/// folding is a later-PR refinement.
///
/// NOTE: `name` may contain arbitrary bytes from a quoted identifier. An
/// interior NUL byte would otherwise panic heed's internal
/// `CString::new(name).unwrap()`; `HeedDB::create_table` guards against that
/// up front and returns `StorageError::InvalidName` instead.
fn canonical_table_key(name: &partiql_value::BindingsName<'_>) -> String {
    match name {
        partiql_value::BindingsName::CaseInsensitive(s) => s.to_lowercase(),
        partiql_value::BindingsName::CaseSensitive(s) => s.to_string(),
    }
}

/// Shared stderr footer for a planning-only DDL statement.
fn print_ddl_planned_footer(elapsed: std::time::Duration) {
    eprintln!(
        "(planned in {:.1}ms — execution not yet implemented)",
        elapsed.as_secs_f64() * 1000.0
    );
}

/// Compile a lowered query plan into an executable program, applying the
/// `plan`/`program` debug dumps. Shared by the `Query` and `CreateTableAs`
/// paths so their compilation setup cannot drift.
///
/// Note: when a debug flag is set, the `[Plan]`/`[Program]` dumps are written
/// inside this call, so the caller's reported `compile:` time includes their
/// cost. That only affects the diagnostic timing line under `--debug`; normal
/// output is unaffected.
fn build_compiled(
    logical: &partiql_logical::LogicalPlan<partiql_logical::BindingsOp>,
    debug: &DebugFlags,
) -> Result<partiql_eval::CompiledPlan, Box<dyn std::error::Error>> {
    if debug.plan {
        eprintln!("[Plan] {:?}", logical);
    }

    let mut context = CompilationContext::new();
    let column_names = vec!["a".to_string(), "b".to_string()];
    let comp_catalog: std::sync::Arc<dyn partiql_eval::CompilationCatalog> =
        std::sync::Arc::new(common::TableFnCompilationCatalog::new(column_names));
    let _catalog_id = context.add_catalog("default", comp_catalog);

    let mut compiler = PlanCompiler::new(&context, EvaluationMode::Permissive);
    let compiled = compiler
        .compile(logical)
        .map_err(|e| format!("Compile error: {:?}", e))?;

    if debug.program {
        eprintln!("[Program]\n{}", compiled);
    }

    Ok(compiled)
}

/// Build the execution context with pqlite's table functions registered.
/// Shared by the `Query` and `CreateTableAs` paths.
fn build_exec_context() -> ExecutionContext {
    let mut exec_context = ExecutionContext::new();
    exec_context.register_table_function("rand", std::sync::Arc::new(common::RandTableFunction));
    exec_context.register_table_function("mem", std::sync::Arc::new(common::MemTableFunction));
    exec_context.register_table_function(
        "scan_ion",
        std::sync::Arc::new(common::ScanIonTableFunction),
    );
    exec_context
}

fn execute_query(
    query_str: &str,
    debug: &DebugFlags,
    db_source: DbSource,
) -> Result<(), Box<dyn std::error::Error>> {
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

    let logical = match statement {
        LogicalStatement::Query(plan) => plan,
        LogicalStatement::CreateTableAs { table_name, query } => {
            let key = canonical_table_key(&table_name);

            // Resolve --db before compile + VM setup so a missing --db on a
            // CTAS path is a clean CLI error before any expensive work.
            let db_path: Option<&std::path::Path> = match db_source {
                DbSource::Open(_) => None,
                DbSource::Lazy(path) => Some(
                    path.ok_or("Error: The `--db <PATH>` option is required for CREATE TABLE AS.")?,
                ),
            };

            let compile_start = Instant::now();
            let compiled = build_compiled(&query, debug)?;
            let compile_time = compile_start.elapsed();

            let exec_start = Instant::now();
            let exec_context = build_exec_context();
            let mut vm = partiql_eval::PartiQLVM::new(compiled, &exec_context)
                .map_err(|e| format!("Execution setup error: {:?}", e))?;

            // Capture RowShape before vm.execute() borrows the VM mutably.
            let row_shape = vm.shape().row_shape().clone();

            // 4 KiB scratch: matches LMDB's typical page size; reused per row.
            let mut scratch_buf: Vec<u8> = Vec::with_capacity(4096);

            // Open the env up front. Mid-stream rejection rolls back the wtxn
            // (no catalog entry, no row bytes), but the env file itself may
            // remain on disk.
            let lazily_opened;
            let db: &HeedDB = match db_source {
                DbSource::Open(db) => db,
                DbSource::Lazy(_) => {
                    let path = db_path.expect("Lazy implies a path");
                    lazily_opened = HeedDB::open(path).map_err(|e| {
                        format!(
                            "Error: could not open database at {}: {}",
                            path.display(),
                            e
                        )
                    })?;
                    &lazily_opened
                }
            };

            let n = match vm.execute() {
                Ok(partiql_eval::ExecutionResult::Query(iter)) => {
                    let mut writer = db.create_table(&key).map_err(|e| format!("Error: {}", e))?;
                    // SAFETY: QueryIterator::next uses unsafe lifetime extension
                    // on its RegisterReader. Aliasing across iter.next() is UB.
                    // Consume `row` synchronously, drop it, then push the bytes.
                    for r in iter {
                        let row = r.map_err(|e| {
                            format!("Error: {}", StorageError::Execution(format!("{:?}", e)))
                        })?;
                        partiql_tools::row_codec::serialize_row(&row, &row_shape, &mut scratch_buf)
                            .map_err(|e| {
                                format!("Error: {}", StorageError::Codec(format!("{e}")))
                            })?;
                        writer
                            .push_row(&scratch_buf)
                            .map_err(|e| format!("Error: {}", e))?;
                    }
                    writer.commit().map_err(|e| format!("Error: {}", e))?
                }
                Err(e) => return Err(format!("Execution setup error: {:?}", e).into()),
            };
            let exec_time = exec_start.elapsed();

            // stdout stays empty for a write; confirmation + timing go to stderr.
            eprintln!(
                "Created table {} ({} rows)",
                format_table_name(&table_name),
                n
            );
            let total_time = parse_time + lower_time + compile_time + exec_time;
            eprintln!(
                "(took {:.1}ms — parse: {:.1}ms, lower: {:.1}ms, compile: {:.1}ms, exec: {:.1}ms)",
                total_time.as_secs_f64() * 1000.0,
                parse_time.as_secs_f64() * 1000.0,
                lower_time.as_secs_f64() * 1000.0,
                compile_time.as_secs_f64() * 1000.0,
                exec_time.as_secs_f64() * 1000.0,
            );
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

    // Phase 3: Compile (Logical → CompiledPlan)
    let compile_start = Instant::now();
    let compiled = build_compiled(&logical, debug)?;
    let compile_time = compile_start.elapsed();

    // Phase 4: Execute
    let exec_start = Instant::now();
    let exec_context = build_exec_context();
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

#[cfg(test)]
mod tests {
    use super::*;
    use partiql_value::BindingsName;
    use std::borrow::Cow;

    #[test]
    fn canonical_key_folds_case_insensitive_to_lowercase() {
        let n = BindingsName::CaseInsensitive(Cow::Borrowed("FOO"));
        assert_eq!(canonical_table_key(&n), "foo");
    }

    #[test]
    fn canonical_key_preserves_case_sensitive_verbatim() {
        let n = BindingsName::CaseSensitive(Cow::Borrowed("Foo"));
        assert_eq!(canonical_table_key(&n), "Foo");
    }

    #[test]
    fn canonical_key_collides_bare_names_regardless_of_case() {
        let a = BindingsName::CaseInsensitive(Cow::Borrowed("Foo"));
        let b = BindingsName::CaseInsensitive(Cow::Borrowed("FOO"));
        assert_eq!(canonical_table_key(&a), canonical_table_key(&b));
    }
}
