use std::path::Path;
use std::sync::Mutex;

use ion_rs::element::Element;
use ion_rs::{IonData, IonType};
use partiql_tools::session::{Commands, DebugFlags, OutputFormat, PqliteSession};

use crate::pqlite_e2e::loader::{load_test_case, LoadError};

/// Bound live LMDB envs to one at a time. Each case opens a 1 GiB-map_size env
/// in a tempdir; many-core hosts otherwise hit EINVAL on env-open when the
/// OS-imposed mmap cap is exceeded (Linux: `vm.max_map_count`; macOS: OS-imposed
/// cap).
static CASE_LOCK: Mutex<()> = Mutex::new(());

#[derive(Debug)]
pub enum CaseError {
    Load(LoadError),
    OpenDb(String),
    Exec {
        step_index: usize,
        sql: String,
        source: String,
    },
    ExpectedErrorButSucceeded {
        step_index: usize,
        sql: String,
        expected: String,
    },
    Mismatch {
        step_index: usize,
        sql: String,
        detail: String,
    },
}

impl std::fmt::Display for CaseError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        use CaseError::*;
        match self {
            Load(e) => write!(f, "failed to load test case: {e}"),
            OpenDb(e) => write!(f, "failed to open session db: {e}"),
            Exec {
                step_index,
                sql,
                source,
            } => write!(
                f,
                "step {}: `{}` failed: {source}",
                step_index + 1,
                sql
            ),
            ExpectedErrorButSucceeded {
                step_index,
                sql,
                expected,
            } => write!(
                f,
                "step {}: `{}` was expected to fail with an error containing {:?}, but it succeeded",
                step_index + 1,
                sql,
                expected
            ),
            Mismatch {
                step_index,
                sql,
                detail,
            } => write!(
                f,
                "step {}: `{}` produced an unexpected result: {detail}",
                step_index + 1,
                sql
            ),
        }
    }
}

impl std::error::Error for CaseError {}

/// `IonData` equality is strict per `IonEq` but order-sensitive on
/// `$bag::[...]` — a multiset matcher would treat bags as unordered.
pub fn run_case(path: &Path) -> Result<(), CaseError> {
    // Recover from poison so one panicked case doesn't cascade.
    let _guard = CASE_LOCK.lock().unwrap_or_else(|e| e.into_inner());

    let steps = load_test_case(path).map_err(CaseError::Load)?;
    let dir = tempfile::tempdir().map_err(|e| CaseError::OpenDb(format!("{e}")))?;
    let dbp = dir.path().join("t.pqlite");
    let session = PqliteSession::open(&dbp, DebugFlags::default())
        .map_err(|e| CaseError::OpenDb(format!("{e}")))?;

    for (i, step) in steps.iter().enumerate() {
        if step.skip {
            // Preserves step numbering so a `skip::` before a real step does
            // not shift the visible index. Never touches the session.
            eprintln!("skipped step {}: {}", i + 1, step.sql);
            continue;
        }
        let expected_err_substr: Option<&str> = step
            .expect
            .as_ref()
            .and_then(|e| e.as_struct())
            .and_then(|s| s.get("error"))
            .and_then(|el| el.as_string());

        let mut out = Vec::<u8>::new();
        let mut err = Vec::<u8>::new();
        let cmd = Commands::Exec {
            query: step.sql.clone(),
            format: OutputFormat::Ion,
        };
        let result = session.run(&cmd, &mut out, &mut err);

        match (result, expected_err_substr) {
            (Err(e), Some(substr)) => {
                let msg = format!("{e}");
                if !msg.contains(substr) {
                    return Err(CaseError::Exec {
                        step_index: i,
                        sql: step.sql.clone(),
                        source: format!("expected error containing {substr:?}, got: {msg}"),
                    });
                }
            }
            (Ok(()), Some(substr)) => {
                return Err(CaseError::ExpectedErrorButSucceeded {
                    step_index: i,
                    sql: step.sql.clone(),
                    expected: substr.to_string(),
                });
            }
            (Err(e), None) => {
                return Err(CaseError::Exec {
                    step_index: i,
                    sql: step.sql.clone(),
                    source: format!("{e}"),
                });
            }
            (Ok(()), None) => {
                let Some(expect) = &step.expect else { continue };
                let elements = Element::read_all(&out).map_err(|e| CaseError::Exec {
                    step_index: i,
                    sql: step.sql.clone(),
                    source: format!(
                        "captured ion output did not reparse: {e}; bytes={:?}",
                        String::from_utf8_lossy(&out)
                    ),
                })?;
                if elements.len() != 1 {
                    return Err(CaseError::Exec {
                        step_index: i,
                        sql: step.sql.clone(),
                        source: format!(
                            "expected exactly one top-level ion value, got {}",
                            elements.len()
                        ),
                    });
                }
                let actual = &elements[0];
                if actual.ion_type() != IonType::Struct {
                    return Err(CaseError::Exec {
                        step_index: i,
                        sql: step.sql.clone(),
                        source: format!(
                            "expected top-level struct envelope, got {}",
                            actual.ion_type()
                        ),
                    });
                }
                if IonData::from(actual.clone()) != IonData::from(expect.clone()) {
                    return Err(CaseError::Mismatch {
                        step_index: i,
                        sql: step.sql.clone(),
                        detail: format!("expected {expect}, got {actual}"),
                    });
                }
            }
        }
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    fn write_case(name: &str, body: &str) -> (tempfile::TempDir, std::path::PathBuf) {
        let dir = tempfile::tempdir().unwrap();
        let path = dir.path().join(name);
        std::fs::write(&path, body).unwrap();
        (dir, path)
    }

    #[test]
    fn expected_error_but_succeeded() {
        let (_dir, path) = write_case(
            "succeeded.test.ion",
            r#"
            { sql: "CREATE TABLE t", expect: { error: "Parse error:" } }
            "#,
        );
        let err = run_case(&path).unwrap_err();
        assert!(matches!(err, CaseError::ExpectedErrorButSucceeded { .. }));
    }

    #[test]
    fn control_char_string_round_trips_through_ion_format() {
        let (_dir, path) = write_case(
            "ctl.test.ion",
            "{ sql: \"SELECT * FROM << {'s': 'x\\x1by'} >>\", expect: { rows: $bag::[ { s: \"x\\x1by\" } ] } }\n",
        );
        run_case(&path).unwrap();
    }

    #[test]
    fn skipped_step_is_not_executed_and_does_not_fail_case() {
        // If the SELECT were executed it would error (ghost table); if the
        // assertion were checked it would fail (never matches). Skipped =
        // neither. The subsequent real step must still pass so the case
        // exercises the ordering guarantee (skipped steps count for indexing
        // but do not touch the session).
        let (_dir, path) = write_case(
            "skipped.test.ion",
            r#"
            skip::{ sql: "SELECT * FROM nonexistent_table", expect: { error: "must match" } }
            { sql: "SELECT * FROM << {'a': 1} >>", expect: { rows: $bag::[ { a: 1 } ] } }
            "#,
        );
        run_case(&path).unwrap();
    }
}
