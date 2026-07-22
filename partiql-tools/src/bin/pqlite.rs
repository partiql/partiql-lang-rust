use partiql_tools::common;

use common::{create_table_fn_catalog, lower_statement, parse, parse_statements};
use partiql_common::catalog::CatalogId;
use partiql_eval::plan::EvaluationMode;
use partiql_eval::value::Shape;
use partiql_eval::{CompilationContext, ExecutionCatalog, ExecutionContext, PlanCompiler};
use partiql_logical::LogicalStatement;
use partiql_tools::catalog::{HeedCompilationCatalog, HeedExecutionCatalog};
use partiql_tools::storage::{HeedDB, StorageError};
use partiql_value::{Tuple, Value};
use std::borrow::Cow;
use std::sync::{Arc, Mutex};
use std::time::Instant;

use clap::{Parser, Subcommand};
use reedline::{
    FileBackedHistory, History, Prompt, PromptEditMode, PromptHistorySearch, Reedline, Signal,
    ValidationResult, Validator,
};

const HISTORY_CAPACITY: usize = 1000;

/// The schema version this binary bootstraps to and requires.
const CURRENT_SCHEMA_VERSION: u32 = 1;

/// Whether `execute_query` prints. Bootstrap runs muted.
#[derive(Clone, Copy)]
enum OutputMode {
    User,
    Silent,
}

/// Crate version joined with the git commit SHA captured in build.rs.
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

    /// Path to the database file. The parent directory must already exist.
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
            let db = match cli.db.as_deref() {
                Some(p) => match open_and_bootstrap(p) {
                    Ok(db) => Some(db),
                    Err(e) => {
                        eprintln!("Error: could not open database at {}: {}", p.display(), e);
                        std::process::exit(1);
                    }
                },
                None => None,
            };
            if let Err(e) = execute_query(query, &debug, db, OutputMode::User) {
                eprintln!("{}", e);
                std::process::exit(1);
            }
        }
        None => {
            // Require explicit --db rather than inventing a default path.
            let db_path = cli.db.clone().unwrap_or_else(|| {
                eprintln!("Error: The `--db <PATH>` option is required to open the database.");
                std::process::exit(1);
            });

            run_repl(&debug, &db_path);
        }
    }
}

/// Trim outer whitespace and strip a single trailing `;` so the REPL can use
/// `;` as a completion signal without it leaking into the PartiQL grammar.
/// TODO: drop this once the parser accepts `;` natively.
fn normalize_query(input: &str) -> &str {
    let trimmed = input.trim();
    trimmed.strip_suffix(';').unwrap_or(trimmed).trim_end()
}

/// Open the DB and bring it to CURRENT_SCHEMA_VERSION. Idempotent + crash-safe.
fn open_and_bootstrap(path: &std::path::Path) -> Result<Arc<HeedDB>, Box<dyn std::error::Error>> {
    let db = Arc::new(HeedDB::open(path)?);
    let version = db.read_schema_version()?;
    if version > CURRENT_SCHEMA_VERSION {
        return Err(format!(
            "database schema version {version} is newer than this binary \
             supports ({CURRENT_SCHEMA_VERSION}); upgrade pqlite"
        )
        .into());
    }
    // A stamped current version must satisfy its postcondition (one extra
    // catalog lookup at startup — a deliberate integrity check).
    if version == CURRENT_SCHEMA_VERSION && !db.tables_has_self_entry()? {
        return Err("database reports schema version 1 but the _tables catalog \
                    is missing or incomplete"
            .into());
    }
    // Migration ladder: each future migration is its own `version < N` block
    // that stamps N, so an older DB runs every step in order. Bumping
    // CURRENT_SCHEMA_VERSION alone is not enough — add a block.
    if version < 1 {
        bootstrap_v1(&db)?;
        let stamped = db.set_schema_version(1)?;
        // Guards a concurrent writer that stamped a newer version between our
        // read above and this stamp; monotonic set returns that higher version.
        if stamped > CURRENT_SCHEMA_VERSION {
            return Err(format!(
                "database schema version {stamped} is newer than this binary \
                 supports ({CURRENT_SCHEMA_VERSION}); upgrade pqlite"
            )
            .into());
        }
    }
    Ok(db)
}

/// Run the v1 bootstrap script. The `;`-separated script is parsed as a unit
/// and each statement executed in order. Idempotent: a crash-recovery DB whose
/// _tables already exists yields a typed StorageError::TableExists, reconciled
/// against the self-entry; any other error propagates.
fn bootstrap_v1(db: &Arc<HeedDB>) -> Result<(), Box<dyn std::error::Error>> {
    let script = include_str!("../bootstrap/v1.pql");
    let debug = DebugFlags::from_args(&[]);
    let parsed = parse_statements(script).map_err(|e| format!("Parse error: {:?}", e))?;
    for stmt in &parsed.statements {
        // parse_time is a bootstrap step, not user-facing; Silent mode prints no
        // timing footer, so a zero placeholder is never observed.
        match execute_statement(
            stmt,
            &debug,
            Some(Arc::clone(db)),
            OutputMode::Silent,
            std::time::Duration::ZERO,
        ) {
            Ok(()) => {}
            Err(e) => {
                let is_self_tables_exists = e
                    .downcast_ref::<StorageError>()
                    .map(|se| matches!(se, StorageError::TableExists(n) if n == "_tables"))
                    .unwrap_or(false);
                // Crash-recovery: _tables already created on a prior run. Skip and
                // continue; any other error is real and aborts bootstrap.
                if is_self_tables_exists && db.tables_has_self_entry()? {
                    continue;
                }
                return Err(e);
            }
        }
    }
    Ok(())
}

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

/// Marks a REPL entry as complete when it ends with `;` or is a `.`-prefixed
/// meta-command; anything else keeps reading on the next line.
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

/// File-backed history at `~/.pqlite_history`; falls back to in-memory on
/// resolution or open failure.
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

    match FileBackedHistory::new(HISTORY_CAPACITY) {
        Ok(history) => Box::new(history),
        Err(_) => Box::new(FileBackedHistory::default()),
    }
}

fn home_dir() -> Option<std::path::PathBuf> {
    // `std::env::home_dir` was un-deprecated in Rust 1.85.
    #[allow(deprecated)]
    std::env::home_dir()
}

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

enum MetaOutcome {
    Handled,
    Quit,
}

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

/// Written to stderr to keep stdout reserved for query output.
fn print_startup_banner(db_path: &std::path::Path) {
    eprintln!("pqlite version {VERSION}");
    eprintln!("database: {}", db_path.display());
    eprintln!("For usage information, enter \".help\".");
}

/// `execute_query` errors are reported but never terminate the session.
fn run_repl(debug: &DebugFlags, db_path: &std::path::Path) {
    // Hard-fail rather than fall back to in-memory: a silently non-persisting
    // db is worse than refusing to start.
    let db = match open_and_bootstrap(db_path) {
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

    print_startup_banner(db_path);

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

                if trimmed.starts_with('.') {
                    match handle_meta_command(trimmed) {
                        MetaOutcome::Handled => continue,
                        MetaOutcome::Quit => break,
                    }
                }

                let query = normalize_query(trimmed);

                // A lone `;` normalizes to empty — skip rather than parse "".
                if query.is_empty() {
                    continue;
                }

                if let Err(e) = execute_query(query, debug, Some(Arc::clone(&db)), OutputMode::User)
                {
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
            Ok(_) => {}
            Err(e) => {
                eprintln!("REPL error: {}", e);
                break;
            }
        }
    }
}

/// SQL-idiomatic rendering: quoted identifiers re-quoted, bare ones bare.
fn format_table_name(name: &partiql_value::BindingsName<'_>) -> String {
    match name {
        partiql_value::BindingsName::CaseSensitive(s) => format!("\"{}\"", s),
        partiql_value::BindingsName::CaseInsensitive(s) => s.as_ref().to_string(),
    }
}

/// Bare identifiers fold to ASCII lowercase; quoted identifiers are verbatim.
/// `HeedDB::create_table` rejects interior NULs that would panic heed.
///
/// ASCII-only — full Unicode-consistent folding (matching the engine's UniCase
/// fold for non-ASCII identifiers) is a later-PR refinement.
fn canonical_table_key(name: &partiql_value::BindingsName<'_>) -> String {
    match name {
        partiql_value::BindingsName::CaseInsensitive(s) => s.to_lowercase(),
        partiql_value::BindingsName::CaseSensitive(s) => s.to_string(),
    }
}

fn print_ddl_executed_footer(elapsed: std::time::Duration) {
    eprintln!("(took {:.1}ms)", elapsed.as_secs_f64() * 1000.0);
}

/// Per-stage timing footer for the write statements (CTAS / INSERT).
fn print_write_footer(
    parse: std::time::Duration,
    lower: std::time::Duration,
    compile: std::time::Duration,
    exec: std::time::Duration,
) {
    let total = parse + lower + compile + exec;
    eprintln!(
        "(took {:.1}ms — parse: {:.1}ms, lower: {:.1}ms, compile: {:.1}ms, exec: {:.1}ms)",
        total.as_secs_f64() * 1000.0,
        parse.as_secs_f64() * 1000.0,
        lower.as_secs_f64() * 1000.0,
        compile.as_secs_f64() * 1000.0,
        exec.as_secs_f64() * 1000.0,
    );
}

/// Bundles `HeedCompilationCatalog` with the existing `TableFnCompilationCatalog`
/// under `PlanCompiler`'s single `"default"` catalog name. Also records the
/// first unresolved bare table name so `build_compiled` can fail fast with
/// "Table not found" instead of letting Permissive mode yield a MISSING row.
struct CombinedCatalog {
    table_fns: common::TableFnCompilationCatalog,
    heed: Option<HeedCompilationCatalog>,
    unresolved_table_name: Arc<Mutex<Option<String>>>,
}

impl partiql_eval::CompilationCatalog for CombinedCatalog {
    fn get_table(
        &self,
        path: &[partiql_value::BindingsName<'_>],
    ) -> Option<partiql_eval::source::DataSourceHandle> {
        if let Some(handle) = self.heed.as_ref().and_then(|h| h.get_table(path)) {
            return Some(handle);
        }
        // Only record bare single-element bindings — schema-qualified paths
        // never match anything here and aren't useful to surface as table names.
        // The compiler may probe a name multiple times; keep the first.
        if path.len() == 1 {
            if let Ok(mut guard) = self.unresolved_table_name.lock() {
                if guard.is_none() {
                    let name = match &path[0] {
                        partiql_value::BindingsName::CaseSensitive(s) => s.to_string(),
                        partiql_value::BindingsName::CaseInsensitive(s) => s.to_string(),
                    };
                    *guard = Some(name);
                }
            }
        }
        None
    }

    fn get_table_function(&self, name: &str) -> Option<partiql_eval::source::TableFunctionHandle> {
        self.table_fns.get_table_function(name)
    }
}

/// CatalogId is reused by the matching ExecutionCatalog.
fn build_compiled(
    logical: &partiql_logical::LogicalPlan<partiql_logical::BindingsOp>,
    debug: &DebugFlags,
    db: Option<Arc<HeedDB>>,
) -> Result<(partiql_eval::CompiledPlan, CatalogId), Box<dyn std::error::Error>> {
    if debug.plan {
        eprintln!("[Plan] {:?}", logical);
    }

    let mut context = CompilationContext::new();

    let column_names = vec!["a".to_string(), "b".to_string()];
    let unresolved_table_name: Arc<Mutex<Option<String>>> = Arc::new(Mutex::new(None));
    let combined = CombinedCatalog {
        table_fns: common::TableFnCompilationCatalog::new(column_names),
        heed: db.map(HeedCompilationCatalog::new),
        unresolved_table_name: Arc::clone(&unresolved_table_name),
    };
    let catalog: Arc<dyn partiql_eval::CompilationCatalog> = Arc::new(combined);
    let catalog_id = context.add_catalog("default", catalog);

    let mut compiler = PlanCompiler::new(&context, EvaluationMode::Permissive);
    let compiled = compiler
        .compile(logical)
        .map_err(|e| format!("Compile error: {:?}", e))?;

    let first_unresolved = unresolved_table_name
        .lock()
        .map(|mut g| g.take())
        .unwrap_or(None);
    if let Some(name) = first_unresolved {
        return Err(format!("Error: Table '{}' not found", name).into());
    }

    if debug.program {
        eprintln!("[Program]\n{}", compiled);
    }

    Ok((compiled, catalog_id))
}

/// The Heed catalog is `prepare()`-d from `compiled` before being boxed so
/// `ScanId → table-name` mappings are populated before the VM calls `create()`.
fn build_exec_context(
    db: Option<Arc<HeedDB>>,
    catalog_id: CatalogId,
    compiled: &partiql_eval::CompiledPlan,
) -> ExecutionContext {
    let mut exec_context = ExecutionContext::new();
    exec_context.register_table_function("rand", Arc::new(common::RandTableFunction));
    exec_context.register_table_function("mem", Arc::new(common::MemTableFunction));
    exec_context.register_table_function("scan_ion", Arc::new(common::ScanIonTableFunction));

    if let Some(db) = db {
        let mut heed_exec = HeedExecutionCatalog::new(db);
        let scans = compiled.scans_for_catalog(catalog_id);
        heed_exec.prepare(&scans);
        exec_context.add_catalog(catalog_id, Box::new(heed_exec));
    }

    exec_context
}

/// Buffered source rows plus the timing a write-path caller needs to finish its
/// own exec timer: `(encoded rows, compile time, exec start instant)`.
type DrainedSource = (Vec<Vec<u8>>, std::time::Duration, Instant);

/// Compile the source plan, run it, and buffer every result row into owned
/// bytes. The source is fully drained here — before any caller opens a write
/// txn — because a streaming source holds a read txn open across iteration, and
/// LMDB rejects opening a table handle in a write txn while one is live
/// (MDB_BAD_DBI). Returns the buffered rows, the compile time, and the exec
/// start instant so the caller can finish timing after its own write + commit.
fn drain_source_rows(
    query: &partiql_logical::LogicalPlan<partiql_logical::BindingsOp>,
    debug: &DebugFlags,
    db: &Arc<HeedDB>,
) -> Result<DrainedSource, Box<dyn std::error::Error>> {
    let compile_start = Instant::now();
    let (compiled, catalog_id) = build_compiled(query, debug, Some(Arc::clone(db)))?;
    let compile_time = compile_start.elapsed();

    let exec_start = Instant::now();
    let exec_context = build_exec_context(Some(Arc::clone(db)), catalog_id, &compiled);
    let mut vm = partiql_eval::PartiQLVM::new(compiled, &exec_context)
        .map_err(|e| format!("Execution setup error: {:?}", e))?;

    // Snapshot RowShape before vm.execute() borrows the VM mutably.
    let row_shape = vm.shape().row_shape().clone();

    let mut rows: Vec<Vec<u8>> = Vec::new();
    // Reused warm across rows (serialize_row clears on entry); each retained row
    // is a tight clone (cap == len), so no per-row 4 KiB is held.
    let mut scratch: Vec<u8> = Vec::with_capacity(4096);
    match vm.execute() {
        Ok(partiql_eval::ExecutionResult::Query(iter)) => {
            // SAFETY: QueryIterator::next lifetime-extends its RegisterReader;
            // aliasing across next() is UB. Consume `row` before the next pull.
            for r in iter {
                let row = r.map_err(|e| {
                    format!("Error: {}", StorageError::Execution(format!("{:?}", e)))
                })?;
                partiql_tools::row_codec::serialize_row(&row, &row_shape, &mut scratch)
                    .map_err(|e| format!("Error: {}", StorageError::Codec(format!("{e}"))))?;
                rows.push(scratch.clone());
            }
        }
        Err(e) => return Err(format!("Execution setup error: {:?}", e).into()),
    }
    Ok((rows, compile_time, exec_start))
}

fn execute_query(
    query_str: &str,
    debug: &DebugFlags,
    db: Option<Arc<HeedDB>>,
    mode: OutputMode,
) -> Result<(), Box<dyn std::error::Error>> {
    let parse_start = Instant::now();
    let parsed = parse(query_str).map_err(|e| format!("Parse error: {:?}", e))?;
    let parse_time = parse_start.elapsed();

    if debug.ast {
        eprintln!("[AST] {:?}", parsed);
    }

    // `exec` and the REPL run one statement at a time; a `;`-separated script is
    // parsed with `parse_statements` and each statement run via `execute_statement`
    // in a loop (see `bootstrap_v1`).
    let stmt = match parsed.statements.as_slice() {
        [stmt] => stmt,
        // Wording matches `LogicalPlanner::lower` for parity across error sites.
        _ => return Err("Lower error: multi-statement input".into()),
    };
    execute_statement(stmt, debug, db, mode, parse_time)
}

/// Execute a single already-parsed statement. Split from [`execute_query`] so a
/// parsed script can dispatch each statement in a loop without re-parsing.
fn execute_statement(
    stmt: &partiql_ast::ast::AstNode<partiql_ast::ast::Statement>,
    debug: &DebugFlags,
    db: Option<Arc<HeedDB>>,
    mode: OutputMode,
    parse_time: std::time::Duration,
) -> Result<(), Box<dyn std::error::Error>> {
    let catalog = create_table_fn_catalog();

    let lower_start = Instant::now();
    let statement =
        lower_statement(&*catalog, stmt).map_err(|e| format!("Lower error: {:?}", e))?;
    let lower_time = lower_start.elapsed();

    let logical = match statement {
        LogicalStatement::Query(plan) => plan,
        LogicalStatement::CreateTableAs { table_name, query } => {
            let key = canonical_table_key(&table_name);

            // Resolve --db before compile + VM setup so a missing flag fails
            // cleanly before any expensive work.
            let db = db
                .as_ref()
                .ok_or("Error: The `--db <PATH>` option is required for CREATE TABLE AS.")?;

            // Encode the catalog value before draining so a deterministic
            // encode error fails fast, ahead of any query execution.
            let mut tables_value = Vec::new();
            partiql_tools::row_codec::serialize_name_row(&[&key], &mut tables_value)
                .map_err(|e| format!("Error: {}", e))?;

            let (rows, compile_time, exec_start) = drain_source_rows(&query, debug, db)?;

            let n = {
                let mut writer = db
                    .create_table(&key, &tables_value)
                    .map_err(|e| format!("Error: {}", e))?;
                for encoded in &rows {
                    writer
                        .push_row(encoded)
                        .map_err(|e| format!("Error: {}", e))?;
                }
                writer.commit().map_err(|e| format!("Error: {}", e))?
            };
            let exec_time = exec_start.elapsed();

            if matches!(mode, OutputMode::User) {
                eprintln!(
                    "Created table {} ({} rows)",
                    format_table_name(&table_name),
                    n
                );
                print_write_footer(parse_time, lower_time, compile_time, exec_time);
            }
            return Ok(());
        }
        LogicalStatement::InsertInto { table_name, query } => {
            let key = canonical_table_key(&table_name);

            if partiql_tools::storage::is_system_table(&key) {
                return Err("Error: cannot INSERT into system table '_tables'".into());
            }

            // Resolve --db before compile + VM setup so a missing flag fails
            // cleanly before any expensive work.
            let db = db
                .as_ref()
                .ok_or("Error: The `--db <PATH>` option is required for INSERT.")?;

            let (rows, compile_time, exec_start) = drain_source_rows(&query, debug, db)?;

            // Row count is tallied here, not from commit(): open_table_for_append
            // seeds row_id at the high-water mark, so commit() returns the absolute
            // counter (existing + inserted), not this statement's insert count.
            let inserted = rows.len() as u64;
            {
                let mut writer = db
                    .open_table_for_append(&key)
                    .map_err(|e| format!("Error: {}", e))?;
                for encoded in &rows {
                    writer
                        .push_row(encoded)
                        .map_err(|e| format!("Error: {}", e))?;
                }
                writer.commit().map_err(|e| format!("Error: {}", e))?;
            }
            let exec_time = exec_start.elapsed();

            if matches!(mode, OutputMode::User) {
                eprintln!(
                    "Inserted {} rows into {}",
                    inserted,
                    format_table_name(&table_name)
                );
                print_write_footer(parse_time, lower_time, compile_time, exec_time);
            }
            return Ok(());
        }
        LogicalStatement::CreateTable { table_name } => {
            let key = canonical_table_key(&table_name);
            let db = db
                .as_ref()
                .ok_or("Error: The `--db <PATH>` option is required for CREATE TABLE.")?;

            // Bare `?` (not `.map_err(format!)`) so the typed StorageError /
            // SerializeError boxes intact — the startup bootstrap downcasts to
            // StorageError::TableExists to reconcile a crash-recovery re-run.
            let mut tables_value = Vec::new();
            partiql_tools::row_codec::serialize_name_row(&[&key], &mut tables_value)?;
            let exec_start = Instant::now();
            db.create_table(&key, &tables_value)?.commit()?;
            let exec_time = exec_start.elapsed();

            if matches!(mode, OutputMode::User) {
                eprintln!("Created table {}", format_table_name(&table_name));
                print_ddl_executed_footer(parse_time + lower_time + exec_time);
            }
            return Ok(());
        }
    };

    let compile_start = Instant::now();
    let (compiled, catalog_id) = build_compiled(&logical, debug, db.clone())?;
    let compile_time = compile_start.elapsed();

    let exec_start = Instant::now();
    let exec_context = build_exec_context(db, catalog_id, &compiled);
    let mut vm = partiql_eval::PartiQLVM::new(compiled, &exec_context)
        .map_err(|e| format!("Execution setup error: {:?}", e))?;

    let shape = vm.shape().clone();
    let mut row_count = 0usize;

    let (prefix, tab, suffix) = match shape {
        Shape::Bag(_) => (Some("<<"), "  ", Some(">>")),
        Shape::List(_) => (Some("["), "  ", Some("]")),
        Shape::Single(_) => (None, "", None),
    };

    let user_output = matches!(mode, OutputMode::User);
    let mut is_first = true;
    match vm.execute() {
        Ok(partiql_eval::ExecutionResult::Query(iter)) => {
            if user_output {
                if let Some(p) = prefix {
                    println!("{}", p);
                }
            }
            for row_result in iter {
                match row_result {
                    Ok(row) => {
                        row_count += 1;
                        if user_output {
                            if is_first {
                                is_first = false;
                            } else {
                                println!(",");
                            }
                            let value = row_to_value(&row, &shape);
                            print!("{}", tab);
                            print!("{:?}", value);
                        }
                    }
                    Err(e) => {
                        return Err(format!("Execution error: {:?}", e).into());
                    }
                }
            }
            if user_output {
                println!();
                if let Some(s) = suffix {
                    println!("{}", s);
                }
            }
        }
        Err(e) => {
            return Err(format!("Execution setup error: {:?}", e).into());
        }
    }
    let exec_time = exec_start.elapsed();

    if user_output {
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
    }

    Ok(())
}

fn row_to_value(
    row: &partiql_eval::value::RegisterReader<'_>,
    shape: &partiql_eval::value::Shape,
) -> Value {
    use partiql_eval::value::{FieldName, RowShape};

    match shape.row_shape() {
        RowShape::Struct(fields) => {
            let mut tuple = Tuple::new();
            for field in fields.iter() {
                let name: &str = match &field.name {
                    FieldName::Static(s) => s,
                    FieldName::Register(reg) => row.get_str(*reg).unwrap_or("?"),
                };
                let reg_idx = match &field.value {
                    RowShape::Register(idx, _) => *idx,
                    _ => continue, // nested structs not yet supported here
                };
                let mut view = row.get_value_view(reg_idx).expect("register should exist");
                tuple.insert(name, value_view_to_value(&mut view));
            }
            Value::Tuple(Box::new(tuple))
        }
        RowShape::Register(idx, _) => {
            let mut view = row.get_value_view(*idx).expect("register should exist");
            value_view_to_value(&mut view)
        }
    }
}

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
            let mut tuple = Tuple::new();
            if view.step_in().is_ok() {
                loop {
                    let field_name = view.get_field_name().unwrap().to_string();
                    let field_value = value_view_to_value(view);
                    tuple.insert(&field_name, field_value);
                    if !view.advance().unwrap_or(false) {
                        break;
                    }
                }
                let _ = view.step_out();
            }
            Value::Tuple(Box::new(tuple))
        }
        ValueType::List => {
            let mut items = Vec::new();
            if view.step_in().is_ok() {
                loop {
                    items.push(value_view_to_value(view));
                    if !view.advance().unwrap_or(false) {
                        break;
                    }
                }
                let _ = view.step_out();
            }
            Value::List(Box::new(items.into()))
        }
        ValueType::Bag => {
            let mut items = Vec::new();
            if view.step_in().is_ok() {
                loop {
                    items.push(value_view_to_value(view));
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
