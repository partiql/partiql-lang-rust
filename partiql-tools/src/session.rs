//! Library-callable PartiQL session for pqlite.
//!
//! `session.run` returns a `RunOutcome` — either a `QueryHandle` for a live
//! query, or a completed non-query `StatementOutcome`. Session does not write
//! to stdout/stderr; callers render via the `render` module.

mod bootstrap;
mod debug;
mod exec;
mod ion_output;
mod naming;
mod outcome;
mod planner;
mod render;
mod value;

use std::path::Path;
use std::sync::Arc;

pub use crate::session::debug::DebugFlags;
pub use crate::session::exec::{QueryFooter, QueryHandle, RunOutcome};
pub use crate::session::naming::normalize_query;
pub use crate::session::outcome::{DebugCapture, OutputFormat, StatementOutcome, StatementTiming};
pub use crate::session::render::{
    flush_debug, render_outcome_ion, render_outcome_text, render_query_footer_text,
    render_query_ion, render_query_text,
};

/// The unit of work `PqliteSession::run` dispatches on. Rendering choices
/// (output format, timing footer, etc.) are the caller's — session only cares
/// about the SQL.
#[derive(Debug)]
pub enum Commands {
    Exec {
        /// The mandatory PartiQL query string to run.
        query: String,
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

    /// Parse + lower + compile the statement and return a `RunOutcome`. For a
    /// query, the caller drives the returned `QueryHandle`; for a write
    /// statement, the outcome is already final.
    ///
    /// On error, the compile-time debug capture is returned alongside so the
    /// caller can flush it (see `DebugCapture` doc).
    pub fn run(
        &self,
        command: &Commands,
    ) -> (Result<RunOutcome, Box<dyn std::error::Error>>, DebugCapture) {
        match command {
            Commands::Exec { query } => exec::run(self.db.as_ref(), &self.debug, query),
        }
    }

    pub fn debug_flags(&self) -> &DebugFlags {
        &self.debug
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    /// End-to-end: a lower error must still flush its captured AST.
    #[test]
    fn ast_capture_flushed_on_lower_error() {
        let session = PqliteSession::open_without_db(DebugFlags {
            ast: true,
            plan: false,
            program: false,
        });
        let cmd = Commands::Exec {
            query: "SELECT * FROM does_not_exist_table".to_string(),
        };
        let (result, on_error_capture) = session.run(&cmd);
        assert!(result.is_err(), "lower/exec must fail");
        let mut stderr = Vec::new();
        flush_debug(&on_error_capture, &mut stderr).unwrap();
        let text = String::from_utf8_lossy(&stderr);
        assert!(
            text.contains("[AST]"),
            "captured AST must be present on error; stderr was: {text}"
        );
    }

    /// Streaming Ion query: envelope on stdout, debug on stderr (before rows),
    /// and many writes rather than one (streaming, not buffered).
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
        let cmd = Commands::Exec { query: sql };

        let (result, _err_capture) = session.run(&cmd);
        let outcome = result.expect("run must succeed");
        match outcome {
            RunOutcome::Query(mut handle) => {
                let mut stderr = Vec::new();
                flush_debug(&handle.take_debug(), &mut stderr).unwrap();
                let mut out = CountingWriter {
                    calls: 0,
                    buf: Vec::new(),
                };
                let ((), footer) = handle
                    .drain(|rows, shape| render_query_ion(rows, shape, &mut out))
                    .expect("render must succeed");
                assert_eq!(footer.row_count, 50, "row count should equal source rows");
                let body = String::from_utf8_lossy(&out.buf);
                assert!(
                    body.contains("$partiql_bag") || body.contains("$bag"),
                    "expected an Ion envelope on stdout, got: {body}"
                );
                let stderr = String::from_utf8_lossy(&stderr);
                assert!(
                    stderr.contains("[AST]"),
                    "AST must flush before rows; stderr was: {stderr}"
                );
                assert!(
                    out.calls > 5,
                    "streaming should issue many write() calls, got {} (bytes: {})",
                    out.calls,
                    out.buf.len()
                );
            }
            RunOutcome::Statement(_) => panic!("expected Query"),
        }
    }

    /// Parse-error path returns no capture: parsing happens before the AST
    /// capture step, so on parse failure we have nothing to flush.
    #[test]
    fn no_debug_capture_on_parse_error() {
        let session = PqliteSession::open_without_db(DebugFlags {
            ast: true,
            plan: true,
            program: true,
        });
        let cmd = Commands::Exec {
            query: "SELECT * FROM x WHERE".to_string(),
        };
        let (result, capture) = session.run(&cmd);
        assert!(result.is_err());
        assert!(
            capture.ast.is_none() && capture.plan.is_none() && capture.program.is_none(),
            "no capture should be built on parse error"
        );
    }
}
