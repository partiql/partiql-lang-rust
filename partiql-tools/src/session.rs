//! Library-callable CLI logic for pqlite.

mod bootstrap;
mod debug;
mod exec;
mod ion_output;
mod naming;
mod outcome;
mod planner;
mod value;

use std::io::Write;
use std::path::Path;
use std::sync::Arc;

use clap::Subcommand;

pub use crate::session::debug::DebugFlags;
pub use crate::session::naming::normalize_query;
pub use crate::session::outcome::OutputFormat;

use crate::session::exec::RunResult;
use crate::session::ion_output::write_outcome_ion;
use crate::session::naming::format_table_name;
use crate::session::outcome::{DebugCapture, StatementOutcome};

/// Clap subcommands shared between the CLI and the test harness so both drive
/// `run` through the same dispatch path.
#[derive(Subcommand, Debug)]
pub enum Commands {
    /// Execute a single query immediately
    Exec {
        /// The mandatory PartiQL query string to run
        query: String,
        /// Output format: `text` (default) or `ion`
        #[arg(long, default_value_t = OutputFormat::Text, value_enum)]
        format: OutputFormat,
    },
}

use crate::storage::HeedDB;

/// A stateful pqlite session: an optionally-open database plus debug settings.
pub struct PqliteSession {
    db: Option<Arc<HeedDB>>,
    debug: DebugFlags,
}

impl PqliteSession {
    /// Open the database at `db_path`, bootstrapping it to the current schema.
    pub fn open(db_path: &Path, debug: DebugFlags) -> Result<Self, Box<dyn std::error::Error>> {
        let db = bootstrap::open_and_bootstrap(db_path)?;
        Ok(PqliteSession {
            db: Some(db),
            debug,
        })
    }

    /// A session with no database (lazy-open): only db-free queries succeed.
    pub fn open_without_db(debug: DebugFlags) -> Self {
        PqliteSession { db: None, debug }
    }

    /// Dispatch a parsed clap subcommand; the CLI and test harness share this entry.
    pub fn run(
        &self,
        command: &Commands,
        out: &mut dyn Write,
        err: &mut dyn Write,
    ) -> Result<(), Box<dyn std::error::Error>> {
        match command {
            Commands::Exec { query, format } => self.run_exec(query, *format, out, err),
        }
    }

    /// Body/envelope to `out`, debug + timing to `err`. On Err, any captured
    /// `--debug` output is still flushed to `err`.
    pub fn run_exec(
        &self,
        sql: &str,
        format: OutputFormat,
        out: &mut dyn Write,
        err: &mut dyn Write,
    ) -> Result<(), Box<dyn std::error::Error>> {
        let (result, on_error_capture) =
            exec::run(self.db.as_ref(), &self.debug, sql, format, out, err);
        match result {
            Ok(RunResult::Query { row_count, timing }) => {
                // `--format ion` stays silent on stderr for successful queries.
                if !matches!(format, OutputFormat::Ion) {
                    write_timing_line(
                        err,
                        &format!("{row_count} rows in"),
                        timing.parse,
                        timing.lower,
                        timing.compile,
                        timing.exec,
                    )?;
                }
                Ok(())
            }
            Ok(RunResult::NonQuery(outcome)) => {
                flush_debug(outcome.debug(), err)?;
                if let OutputFormat::Ion = format {
                    write_outcome_ion(&outcome, out)?;
                } else {
                    render_non_query_text(&outcome, err)?;
                }
                Ok(())
            }
            Err(e) => {
                flush_debug(&on_error_capture, err)?;
                Err(e)
            }
        }
    }
}

fn render_non_query_text(outcome: &StatementOutcome, err: &mut dyn Write) -> std::io::Result<()> {
    match outcome {
        StatementOutcome::CreateTableAs {
            table_name,
            rows,
            timing,
            ..
        } => {
            writeln!(
                err,
                "Created table {} ({} rows)",
                format_table_name(table_name),
                rows
            )?;
            write_timing_line(
                err,
                "took",
                timing.parse,
                timing.lower,
                timing.compile,
                timing.exec,
            )?;
        }
        StatementOutcome::InsertInto {
            table_name,
            rows,
            timing,
            ..
        } => {
            writeln!(
                err,
                "Inserted {} rows into {}",
                rows,
                format_table_name(table_name)
            )?;
            write_timing_line(
                err,
                "took",
                timing.parse,
                timing.lower,
                timing.compile,
                timing.exec,
            )?;
        }
        StatementOutcome::CreateTable {
            table_name, timing, ..
        } => {
            writeln!(err, "Created table {}", format_table_name(table_name))?;
            write_timing_line(
                err,
                "took",
                timing.parse,
                timing.lower,
                timing.compile,
                timing.exec,
            )?;
        }
    }
    err.flush()?;
    Ok(())
}

pub(super) fn flush_debug(debug: &DebugCapture, err: &mut dyn Write) -> std::io::Result<()> {
    if let Some(ast) = &debug.ast {
        writeln!(err, "{}", ast)?;
    }
    if let Some(plan) = &debug.plan {
        writeln!(err, "{}", plan)?;
    }
    if let Some(program) = &debug.program {
        writeln!(err, "{}", program)?;
    }
    err.flush()?;
    Ok(())
}

/// `(<prefix> Xms — parse: ..., lower: ..., compile: ..., exec: ...)` line.
fn write_timing_line(
    err: &mut dyn Write,
    prefix: &str,
    parse: std::time::Duration,
    lower: std::time::Duration,
    compile: std::time::Duration,
    exec: std::time::Duration,
) -> std::io::Result<()> {
    let total = parse + lower + compile + exec;
    writeln!(
        err,
        "({} {:.1}ms — parse: {:.1}ms, lower: {:.1}ms, compile: {:.1}ms, exec: {:.1}ms)",
        prefix,
        total.as_secs_f64() * 1000.0,
        parse.as_secs_f64() * 1000.0,
        lower.as_secs_f64() * 1000.0,
        compile.as_secs_f64() * 1000.0,
        exec.as_secs_f64() * 1000.0,
    )
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn ast_capture_flushed_on_lower_error() {
        let session = PqliteSession::open_without_db(DebugFlags {
            ast: true,
            plan: false,
            program: false,
        });
        let mut out = Vec::new();
        let mut err = Vec::new();
        let result = session.run_exec(
            "SELECT * FROM does_not_exist_table",
            OutputFormat::Ion,
            &mut out,
            &mut err,
        );
        assert!(result.is_err(), "lower/exec must fail: {result:?}");
        let stderr = String::from_utf8_lossy(&err);
        assert!(
            stderr.contains("[AST]"),
            "captured AST must be flushed on error; stderr was: {stderr}"
        );
    }

    /// Covers three invariants of the streaming-Ion Query path:
    /// envelope on stdout, `--debug ast` on stderr, and >1 write() call per
    /// large result (streaming, not buffered).
    #[test]
    fn streaming_ion_query_end_to_end() {
        struct CountingWriter {
            calls: usize,
            buf: Vec<u8>,
        }
        impl std::io::Write for CountingWriter {
            fn write(&mut self, buf: &[u8]) -> std::io::Result<usize> {
                self.calls += 1;
                self.buf.extend_from_slice(buf);
                Ok(buf.len())
            }
            fn flush(&mut self) -> std::io::Result<()> {
                Ok(())
            }
        }
        let session = PqliteSession::open_without_db(DebugFlags {
            ast: true,
            plan: false,
            program: false,
        });
        let elems: Vec<String> = (0..50).map(|i| format!("{{'n':{}}}", i)).collect();
        let sql = format!("SELECT VALUE t.n FROM << {} >> t", elems.join(", "));
        let mut out = CountingWriter {
            calls: 0,
            buf: Vec::new(),
        };
        let mut err = Vec::new();
        session
            .run(
                &Commands::Exec {
                    query: sql,
                    format: OutputFormat::Ion,
                },
                &mut out,
                &mut err,
            )
            .unwrap();

        let body = String::from_utf8_lossy(&out.buf);
        assert!(
            body.contains("$partiql_bag") || body.contains("$bag"),
            "expected an Ion envelope on stdout, got: {body}"
        );
        let stderr = String::from_utf8_lossy(&err);
        assert!(
            stderr.contains("[AST]"),
            "AST must flush to stderr even with --format ion; stderr was: {stderr}"
        );
        assert!(
            out.calls > 5,
            "streaming should issue many write() calls, got {} (bytes: {})",
            out.calls,
            out.buf.len()
        );
    }

    #[test]
    fn flush_debug_and_text_paths_flush_explicitly() {
        // A writer that counts flush() calls: if the code paths never flush,
        // this catches it (block-buffered stdout can otherwise outrun stderr
        // ordering with no test-visible signal).
        struct FlushCounter {
            flushes: usize,
            buf: Vec<u8>,
        }
        impl std::io::Write for FlushCounter {
            fn write(&mut self, buf: &[u8]) -> std::io::Result<usize> {
                self.buf.extend_from_slice(buf);
                Ok(buf.len())
            }
            fn flush(&mut self) -> std::io::Result<()> {
                self.flushes += 1;
                Ok(())
            }
        }
        let session = PqliteSession::open_without_db(DebugFlags {
            ast: true,
            plan: false,
            program: false,
        });
        let cmd = Commands::Exec {
            query: "SELECT * FROM << {'a': 1} >>".to_string(),
            format: OutputFormat::Text,
        };
        let mut out = FlushCounter {
            flushes: 0,
            buf: Vec::new(),
        };
        let mut err = FlushCounter {
            flushes: 0,
            buf: Vec::new(),
        };
        session.run(&cmd, &mut out, &mut err).unwrap();
        assert!(
            out.flushes >= 1,
            "text stdout must flush at end of stream_query_text"
        );
        assert!(
            err.flushes >= 1,
            "err path must flush (debug capture + footer)"
        );
    }

    #[test]
    fn no_debug_output_on_parse_error() {
        let session = PqliteSession::open_without_db(DebugFlags {
            ast: true,
            plan: true,
            program: true,
        });
        let mut out = Vec::new();
        let mut err = Vec::new();
        let result = session.run_exec(
            "SELECT * FROM x WHERE",
            OutputFormat::Ion,
            &mut out,
            &mut err,
        );
        assert!(result.is_err());
        assert!(
            err.is_empty(),
            "no capture should exist yet, err was: {:?}",
            String::from_utf8_lossy(&err)
        );
    }
}
