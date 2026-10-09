use std::borrow::Cow;
use std::io::Write;

use partiql_tools::session::{
    flush_debug, normalize_query, parse_script, render_outcome_ion, render_outcome_text,
    render_query_ion, render_query_text, render_statement_timing, render_total_timing, DebugFlags,
    OutputFormat, PqliteSession, RunOutcome, Script, StatementTiming,
};
use partiql_tools::table_fns::TableFnRegistry;

use clap::builder::{PossibleValue, PossibleValuesParser};
use clap::{CommandFactory, Parser, Subcommand, ValueHint};
use reedline::{
    FileBackedHistory, History, Prompt, PromptEditMode, PromptHistorySearch, Reedline, Signal,
    ValidationResult, Validator,
};

const HISTORY_CAPACITY: usize = 1000;

/// Crate version joined with the git commit SHA captured in build.rs.
const VERSION: &str = concat!(env!("CARGO_PKG_VERSION"), "@", env!("PQLITE_GIT_SHA"));

/// Pqlite: An interactive PartiQL database engine and REPL.
///
/// Use table functions in queries to access data:
///   SELECT t.a FROM mem(100, 2) t;
///   SELECT t.name FROM scan_ion('data.ion') t;
///
/// Shell completions: `pqlite completions <bash|zsh|fish|powershell|elvish>`.
/// See PQLITE.md for install instructions.
#[derive(Parser)]
#[command(name = "pqlite", version = VERSION)]
struct Cli {
    /// Print debug info for pipeline stages. Accepts: ast, plan, program, or all (alias '*').
    #[arg(long, global = true, value_delimiter = ',', value_parser = debug_values())]
    debug: Vec<String>,

    /// UNSTABLE: load table functions from a plugin shared library. Repeatable.
    #[cfg(pqlite_unstable_plugins)]
    #[arg(long = "load", global = true, value_name = "PATH", value_hint = ValueHint::FilePath)]
    load: Vec<std::path::PathBuf>,

    /// UNSTABLE: `key=value` option passed to every loaded plugin. Repeatable.
    #[cfg(pqlite_unstable_plugins)]
    #[arg(long = "plugin-opt", global = true, value_name = "KEY=VALUE", value_parser = parse_kv)]
    plugin_opts: Vec<(String, String)>,

    #[command(subcommand)]
    command: CliCommand,
}

/// Values accepted by `--debug`. `*` is a hidden alias of `all`: completion
/// generators emit possible values unquoted, so a visible `*` would be
/// glob-expanded into the current directory's file names.
fn debug_values() -> PossibleValuesParser {
    PossibleValuesParser::new([
        PossibleValue::new("ast"),
        PossibleValue::new("plan"),
        PossibleValue::new("program"),
        PossibleValue::new("all").alias("*"),
    ])
}

#[derive(Subcommand)]
enum CliCommand {
    /// Open a database file and start the interactive REPL.
    Open {
        /// Path to the database file. The parent directory must already exist.
        #[arg(value_hint = ValueHint::FilePath)]
        db: std::path::PathBuf,
    },
    /// Execute one or more `;`-separated statements immediately, optionally
    /// against a database file. Statements run in order; the first failure
    /// stops the run and exits non-zero.
    Exec {
        /// The PartiQL statement(s) to run, e.g. "SELECT 1; SELECT 2".
        #[arg(value_hint = ValueHint::Other)]
        query: String,
        /// Path to the database file. Omit for db-free queries.
        #[arg(long, value_hint = ValueHint::FilePath)]
        db: Option<std::path::PathBuf>,
        /// Output format: `text` (default), `ion`, or `none` (run to
        /// completion, print nothing to stdout).
        #[arg(long, default_value_t = OutputFormat::Text, value_enum)]
        format: OutputFormat,
    },
    /// Print a shell completion script to stdout. See PQLITE.md for install steps.
    Completions {
        /// The shell to generate completions for.
        shell: clap_complete::Shell,
    },
}

#[cfg(pqlite_unstable_plugins)]
fn parse_kv(s: &str) -> Result<(String, String), String> {
    s.split_once('=')
        .map(|(k, v)| (k.to_string(), v.to_string()))
        .ok_or_else(|| format!("expected KEY=VALUE, got '{s}'"))
}

/// Built-in table functions plus any `--load`ed plugins. Exits on a load error.
fn table_fns(cli: &Cli) -> TableFnRegistry {
    #[allow(unused_mut)]
    let mut fns = TableFnRegistry::builtin();
    #[cfg(pqlite_unstable_plugins)]
    if !cli.load.is_empty() {
        eprintln!("warning: plugin support is unstable and in development; the ABI may change without notice");
    }
    #[cfg(pqlite_unstable_plugins)]
    for path in &cli.load {
        if let Err(e) = partiql_tools::plugin::load_plugin(path, &cli.plugin_opts, &mut fns) {
            eprintln!("Error: {e}");
            std::process::exit(1);
        }
    }
    #[cfg(not(pqlite_unstable_plugins))]
    let _ = cli;
    fns
}

fn main() {
    let cli = Cli::parse();
    let debug = DebugFlags::from_args(&cli.debug);
    // Before loading plugins: completions never need to run plugin code.
    if let CliCommand::Completions { shell } = cli.command {
        let mut command = Cli::command();
        let bin_name = command.get_name().to_string();
        clap_complete::generate(shell, &mut command, bin_name, &mut std::io::stdout());
        return;
    }
    let fns = table_fns(&cli);

    match cli.command {
        CliCommand::Exec { query, db, format } => {
            // Reject empty or unparseable input before opening the database so
            // a `--db <path>` invocation that cannot run does not create the
            // file on disk.
            if normalize_query(&query).is_empty() {
                eprintln!("Error: empty query");
                std::process::exit(1);
            }
            let script = match parse_script(&query, &debug) {
                Ok(s) if s.statements().is_empty() => {
                    eprintln!("Error: empty query");
                    std::process::exit(1);
                }
                Ok(s) => s,
                Err(e) => {
                    eprintln!("{}", e);
                    std::process::exit(1);
                }
            };
            let session = match db.as_deref() {
                Some(p) => match PqliteSession::open(p, debug) {
                    Ok(s) => s.with_table_fns(fns),
                    Err(e) => {
                        eprintln!("Error: could not open database at {}: {}", p.display(), e);
                        std::process::exit(1);
                    }
                },
                None => PqliteSession::open_without_db(debug).with_table_fns(fns),
            };
            let mut stdout = std::io::stdout();
            let mut stderr = std::io::stderr();
            if let Err(e) = run_script(&session, &script, format, &mut stdout, &mut stderr) {
                eprintln!("{}", e);
                std::process::exit(1);
            }
        }
        CliCommand::Open { db } => run_repl(debug, fns, &db),
        CliCommand::Completions { .. } => unreachable!("handled above"),
    }
}

/// Run a parsed script's statements in order. Stops at the first failure and
/// returns its error labeled with the 1-based statement number; output of the
/// statements before it has already been written.
///
/// Outside Ion mode, timing goes to stderr: with several statements, one
/// `Statement N:` line after each, then a `Total Timing:` line (which adds the
/// script's parse time); with one statement, just the `Total Timing:` line.
fn run_script(
    session: &PqliteSession,
    script: &Script<'_>,
    format: OutputFormat,
    stdout: &mut dyn Write,
    stderr: &mut dyn Write,
) -> Result<(), String> {
    let io_err = |e: std::io::Error| e.to_string();
    flush_debug(&script.debug, stderr).map_err(io_err)?;
    // Ion mode is silent on stderr for successful statements; the caller
    // pipes stdout to an Ion parser and expects no interleaved lines.
    let show_timing = !matches!(format, OutputFormat::Ion);
    let statements = script.statements();
    let mut total = StatementTiming {
        parse: script.parse_time,
        ..Default::default()
    };
    let mut total_rows: Option<u64> = None;
    for (i, stmt) in statements.iter().enumerate() {
        let n = i + 1;
        let (rows, timing) = dispatch(session, stmt, format, stdout, stderr)
            .map_err(|e| format!("Statement {n}: {e}"))?;
        total += timing;
        if let Some(r) = rows {
            *total_rows.get_or_insert(0) += r;
        }
        if show_timing && statements.len() > 1 {
            render_statement_timing(n, rows, &timing, stderr).map_err(io_err)?;
        }
    }
    if show_timing {
        render_total_timing(total_rows, &total, stderr).map_err(io_err)?;
    }
    Ok(())
}

/// Run one statement and render its result to `stdout`/`stderr`, returning
/// its row count (rows returned or written; `None` for CREATE TABLE) and
/// timing. Debug capture flushes to stderr BEFORE any query rows land on
/// stdout so the ordering matches every `--debug` test expectation.
fn dispatch(
    session: &PqliteSession,
    stmt: &partiql_ast::ast::AstNode<partiql_ast::ast::Statement>,
    format: OutputFormat,
    stdout: &mut dyn Write,
    stderr: &mut dyn Write,
) -> Result<(Option<u64>, StatementTiming), Box<dyn std::error::Error>> {
    let (result, on_error_capture) = session.run_statement(stmt);
    let outcome = match result {
        Ok(o) => o,
        Err(e) => {
            flush_debug(&on_error_capture, stderr)?;
            return Err(e);
        }
    };

    match outcome {
        RunOutcome::Query(mut handle) => {
            flush_debug(&handle.take_debug(), stderr)?;
            let ((), footer) = handle.drain(|rows, shape| match format {
                OutputFormat::Text => render_query_text(rows, shape, stdout),
                OutputFormat::Ion => render_query_ion(rows, shape, stdout),
                // `drain` consumes any rows the renderer leaves, so the query
                // still runs to completion and the timing is real.
                OutputFormat::None => Ok(()),
            })?;
            Ok((Some(footer.row_count), footer.timing))
        }
        RunOutcome::Statement(outcome) => {
            flush_debug(outcome.debug(), stderr)?;
            match format {
                OutputFormat::Ion => render_outcome_ion(&outcome, stdout)?,
                OutputFormat::Text | OutputFormat::None => render_outcome_text(&outcome, stderr)?,
            }
            Ok((outcome.rows(), *outcome.timing()))
        }
    }
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

fn print_help(session: &PqliteSession) {
    println!("PartiQL REPL — available commands:");
    println!("  .help     Show this help message");
    println!("  .quit     Exit the REPL (alias: .exit)");
    println!("  .exit     Exit the REPL (alias: .quit)");
    println!();
    println!("To run a query, type any PartiQL statement ending in ';' and press Enter.");
    println!("Several ';'-separated statements in one entry run in order; the first");
    println!("failing statement stops the rest of that entry.");
    println!("Available table functions:");
    for f in session.table_fns().iter() {
        println!("  {}", f.usage);
    }
    println!();
    println!("Example: SELECT t.a, t.b FROM mem(100, 2) t LIMIT 5;");
}

enum MetaOutcome {
    Handled,
    Quit,
}

fn handle_meta_command(input: &str, session: &PqliteSession) -> MetaOutcome {
    match input {
        ".quit" | ".exit" => MetaOutcome::Quit,
        ".help" => {
            print_help(session);
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

/// What the REPL loop does after an entry.
enum EntryOutcome {
    Continue,
    Quit,
}

/// Handle one completed REPL entry (a meta-command, or one or more
/// `;`-separated statements, possibly spanning several lines). Errors are
/// written to `stderr` and stop the rest of the entry, never the session.
fn handle_entry(
    session: &PqliteSession,
    buffer: &str,
    stdout: &mut dyn Write,
    stderr: &mut dyn Write,
) -> EntryOutcome {
    let trimmed = buffer.trim();
    if trimmed.starts_with('.') {
        return match handle_meta_command(trimmed, session) {
            MetaOutcome::Handled => EntryOutcome::Continue,
            MetaOutcome::Quit => EntryOutcome::Quit,
        };
    }

    // Blank input or a lone `;` normalizes to empty — skip rather than parse.
    if normalize_query(trimmed).is_empty() {
        return EntryOutcome::Continue;
    }

    let result = parse_script(trimmed, session.debug_flags())
        .map_err(|e| e.to_string())
        .and_then(|script| run_script(session, &script, OutputFormat::Text, stdout, stderr));
    if let Err(e) = result {
        let _ = writeln!(stderr, "{}", e);
    }
    EntryOutcome::Continue
}

/// Per-statement errors are reported but never terminate the session.
fn run_repl(debug: DebugFlags, fns: TableFnRegistry, db_path: &std::path::Path) {
    // Hard-fail rather than fall back to in-memory: a silently non-persisting
    // db is worse than refusing to start.
    let session = match PqliteSession::open(db_path, debug) {
        Ok(s) => s.with_table_fns(fns),
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
                let mut stdout = std::io::stdout();
                let mut stderr = std::io::stderr();
                match handle_entry(&session, &buffer, &mut stdout, &mut stderr) {
                    EntryOutcome::Continue => {}
                    EntryOutcome::Quit => break,
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

#[cfg(test)]
mod tests {
    use super::*;

    /// Drive one REPL entry without a tty (reedline needs a real terminal).
    fn entry(buffer: &str) -> (String, String) {
        let session = PqliteSession::open_without_db(DebugFlags::default());
        let (mut out, mut err) = (Vec::new(), Vec::new());
        handle_entry(&session, buffer, &mut out, &mut err);
        (
            String::from_utf8(out).unwrap(),
            String::from_utf8(err).unwrap(),
        )
    }

    fn complete(line: &str) -> bool {
        matches!(PqliteValidator.validate(line), ValidationResult::Complete)
    }

    #[test]
    fn validator_waits_for_terminating_semicolon_across_lines() {
        assert!(!complete("SELECT 1;\nSELECT 2"));
        assert!(complete("SELECT 1;\nSELECT 2;"));
        assert!(complete(".help"));
    }

    #[test]
    fn multi_line_multi_statement_entry_runs_each_statement() {
        let (out, err) = entry("SELECT VALUE 'a;b' FROM << 1 >>;\nSELECT VALUE 2\n  FROM << 1 >>;");
        assert!(out.contains("'a;b'") && out.contains('2'), "stdout: {out}");
        assert!(
            err.contains("Statement 1: (1 rows in")
                && err.contains("Statement 2: (1 rows in")
                && err.contains("Total Timing: (2 rows in"),
            "stderr: {err}"
        );
        assert!(!err.contains("Statement 3"), "stderr: {err}");
    }

    #[test]
    fn error_stops_the_rest_of_the_entry() {
        let (out, err) =
            entry("SELECT VALUE 1 FROM << 1 >>; SELECT * FROM ghost; SELECT VALUE 3 FROM << 1 >>;");
        assert!(out.contains('1') && !out.contains('3'), "stdout: {out}");
        assert!(err.contains("Statement 2: "), "stderr: {err}");
        assert!(!err.contains("Statement 3"), "stderr: {err}");
    }

    #[test]
    fn lone_semicolon_is_skipped() {
        assert_eq!(entry(" ; "), (String::new(), String::new()));
    }
}
