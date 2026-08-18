use std::borrow::Cow;
use std::io::Write;

use partiql_tools::session::{
    flush_debug, normalize_query, render_outcome_ion, render_outcome_text,
    render_query_footer_text, render_query_ion, render_query_text, Commands, DebugFlags,
    OutputFormat, PqliteSession, RunOutcome,
};

use clap::{Parser, Subcommand};
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
///   SELECT t.name FROM read('data.json') t;
#[derive(Parser)]
#[command(name = "pqlite", version = VERSION)]
struct Cli {
    /// Print debug info for pipeline stages. Accepts: ast, plan, program, or * for all.
    #[arg(long, global = true, value_delimiter = ',')]
    debug: Vec<String>,

    #[command(subcommand)]
    command: CliCommand,
}

#[derive(Subcommand)]
enum CliCommand {
    /// Open a database file and start the interactive REPL.
    Open {
        /// Path to the database file. The parent directory must already exist.
        db: std::path::PathBuf,
    },
    /// Execute a single query immediately, optionally against a database file.
    Exec {
        /// The PartiQL query string to run.
        query: String,
        /// Path to the database file. Omit for db-free queries.
        #[arg(long)]
        db: Option<std::path::PathBuf>,
        /// Output format: `text` (default) or `ion`.
        #[arg(long, default_value_t = OutputFormat::Text, value_enum)]
        format: OutputFormat,
    },
}

fn main() {
    let cli = Cli::parse();
    let debug = DebugFlags::from_args(&cli.debug);

    match cli.command {
        CliCommand::Exec { query, db, format } => {
            // Reject empty query before opening the database so a `--db <path>`
            // invocation with a blank query does not create the file on disk.
            if normalize_query(&query).is_empty() {
                eprintln!("Error: empty query");
                std::process::exit(1);
            }
            let cmd = Commands::Exec { query };
            let session = match db.as_deref() {
                Some(p) => match PqliteSession::open(p, debug) {
                    Ok(s) => s,
                    Err(e) => {
                        eprintln!("Error: could not open database at {}: {}", p.display(), e);
                        std::process::exit(1);
                    }
                },
                None => PqliteSession::open_without_db(debug),
            };
            let mut stdout = std::io::stdout();
            let mut stderr = std::io::stderr();
            if let Err(e) = dispatch(&session, &cmd, format, &mut stdout, &mut stderr) {
                eprintln!("{}", e);
                std::process::exit(1);
            }
        }
        CliCommand::Open { db } => run_repl(debug, &db),
    }
}

/// Render a session outcome to `stdout`/`stderr`. Debug capture flushes to
/// stderr BEFORE any query rows land on stdout so the ordering matches every
/// `--debug` test expectation.
fn dispatch(
    session: &PqliteSession,
    cmd: &Commands,
    format: OutputFormat,
    stdout: &mut dyn Write,
    stderr: &mut dyn Write,
) -> Result<(), Box<dyn std::error::Error>> {
    let (result, on_error_capture) = session.run(cmd);
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
            })?;
            // Ion mode is silent on stderr for successful queries; the caller
            // pipes stdout to an Ion parser and expects no interleaved lines.
            if !matches!(format, OutputFormat::Ion) {
                render_query_footer_text(&footer, stderr)?;
            }
            Ok(())
        }
        RunOutcome::Statement(outcome) => {
            flush_debug(outcome.debug(), stderr)?;
            match format {
                OutputFormat::Ion => render_outcome_ion(&outcome, stdout)?,
                OutputFormat::Text => render_outcome_text(&outcome, stderr)?,
            }
            Ok(())
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

fn print_help() {
    println!("PartiQL REPL — available commands:");
    println!("  .help     Show this help message");
    println!("  .quit     Exit the REPL (alias: .exit)");
    println!("  .exit     Exit the REPL (alias: .quit)");
    println!();
    println!("To run a query, type any PartiQL statement and press Enter.");
    println!("Available table functions:");
    println!("  mem(rows, cols)       — sequential integer data");
    println!("  read(path)            — read Ion or JSON from a file");
    println!("  stdin()               — read Ion or JSON from stdin");
    println!("  exec(command)         — run `sh -c <command>`, read its stdout");
    println!("  curl(url)             — HTTP GET, read the response body");
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

/// Per-statement errors are reported but never terminate the session.
fn run_repl(debug: DebugFlags, db_path: &std::path::Path) {
    // Hard-fail rather than fall back to in-memory: a silently non-persisting
    // db is worse than refusing to start.
    let session = match PqliteSession::open(db_path, debug) {
        Ok(s) => s,
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

                let mut stdout = std::io::stdout();
                let mut stderr = std::io::stderr();
                let cmd = Commands::Exec {
                    query: query.to_string(),
                };
                if let Err(e) =
                    dispatch(&session, &cmd, OutputFormat::Text, &mut stdout, &mut stderr)
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
