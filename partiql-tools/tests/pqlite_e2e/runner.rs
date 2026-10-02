use std::path::Path;
use std::sync::Mutex;

use ion_rs::element::Element;
use ion_rs::types::IntAccess;
use ion_rs::{IonData, IonType};
use partiql_tools::session::{
    render_outcome_ion, render_query_ion, Commands, DebugFlags, PqliteSession, RunOutcome,
    StatementOutcome,
};

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

        let cmd = Commands::Exec {
            query: step.sql.clone(),
        };
        let (result, _err_capture) = session.run(&cmd);

        let outcome = match (result, expected_err_substr) {
            (Err(e), Some(substr)) => {
                let msg = format!("{e}");
                if !msg.contains(substr) {
                    return Err(CaseError::Exec {
                        step_index: i,
                        sql: step.sql.clone(),
                        source: format!("expected error containing {substr:?}, got: {msg}"),
                    });
                }
                continue;
            }
            (Err(e), None) => {
                return Err(CaseError::Exec {
                    step_index: i,
                    sql: step.sql.clone(),
                    source: format!("{e}"),
                });
            }
            (Ok(o), Some(substr)) => {
                // `run` succeeded, but Query rows can still error mid-iteration
                // so drain before declaring the expected-error missed.
                match drain_outcome(o) {
                    Err(e) => {
                        let msg = format!("{e}");
                        if !msg.contains(substr) {
                            return Err(CaseError::Exec {
                                step_index: i,
                                sql: step.sql.clone(),
                                source: format!("expected error containing {substr:?}, got: {msg}"),
                            });
                        }
                        continue;
                    }
                    Ok(_) => {
                        return Err(CaseError::ExpectedErrorButSucceeded {
                            step_index: i,
                            sql: step.sql.clone(),
                            expected: substr.to_string(),
                        });
                    }
                }
            }
            (Ok(o), None) => o,
        };

        let Some(expect) = &step.expect else {
            // No expectation: just drain to detect a late error.
            drain_outcome(outcome).map_err(|e| CaseError::Exec {
                step_index: i,
                sql: step.sql.clone(),
                source: format!("{e}"),
            })?;
            continue;
        };

        match outcome {
            RunOutcome::Query(handle) => {
                let mut buf = Vec::<u8>::new();
                handle
                    .drain(|rows, shape| render_query_ion(rows, shape, &mut buf))
                    .map_err(|e| CaseError::Exec {
                        step_index: i,
                        sql: step.sql.clone(),
                        source: format!("{e}"),
                    })?;
                match_ion_envelope(&buf, expect).map_err(|detail| CaseError::Mismatch {
                    step_index: i,
                    sql: step.sql.clone(),
                    detail,
                })?;
            }
            RunOutcome::Statement(outcome) => {
                // Structural check first: catches typos, unexpected fields,
                // and the semantic (canonical_key + row count) mismatch.
                match_statement_outcome(&outcome, expect).map_err(|detail| {
                    CaseError::Mismatch {
                        step_index: i,
                        sql: step.sql.clone(),
                        detail,
                    }
                })?;
                // Render + reparse to keep the shared Ion envelope emitter
                // under test coverage; the harness would otherwise never
                // exercise render_outcome_ion for CTAS/INSERT/CREATE.
                let mut buf = Vec::<u8>::new();
                render_outcome_ion(&outcome, &mut buf).map_err(|e| CaseError::Exec {
                    step_index: i,
                    sql: step.sql.clone(),
                    source: format!("render_outcome_ion: {e}"),
                })?;
                match_ion_envelope(&buf, expect).map_err(|detail| CaseError::Mismatch {
                    step_index: i,
                    sql: step.sql.clone(),
                    detail,
                })?;
            }
        }
    }
    Ok(())
}

/// Parse `buf` as one top-level Ion struct and compare against `expect` under
/// `IonData` equality — struct fields match as a multiset, but bag/list order
/// is significant.
fn match_ion_envelope(buf: &[u8], expect: &Element) -> Result<(), String> {
    let elements = Element::read_all(buf).map_err(|e| {
        format!(
            "captured ion output did not reparse: {e}; bytes={:?}",
            String::from_utf8_lossy(buf)
        )
    })?;
    if elements.len() != 1 {
        return Err(format!(
            "expected exactly one top-level ion value, got {}",
            elements.len()
        ));
    }
    let actual = &elements[0];
    if actual.ion_type() != IonType::Struct {
        return Err(format!(
            "expected top-level struct envelope, got {}",
            actual.ion_type()
        ));
    }
    if IonData::from(actual.clone()) != IonData::from(expect.clone()) {
        return Err(format!("expected {expect}, got {actual}"));
    }
    Ok(())
}

/// Drain a `RunOutcome` for side-effect-only cases (no expectation): pull rows
/// off a query iterator so late errors surface, and drop write outcomes on the
/// floor.
fn drain_outcome(outcome: RunOutcome) -> Result<(), Box<dyn std::error::Error>> {
    match outcome {
        RunOutcome::Query(handle) => handle
            .drain(|rows, _shape| {
                for row in rows {
                    row.map_err(|e| format!("Execution error: {:?}", e))?;
                }
                Ok(())
            })
            .map(|((), _footer)| ()),
        RunOutcome::Statement(_) => Ok(()),
    }
}

/// Compare a `StatementOutcome::{CreateTableAs, InsertInto, CreateTable}`
/// against the fixture's `expect` envelope. Compares structurally against the
/// outcome fields — NO byte round-trip through Ion.
///
/// Strict on unknown fields: an expect struct with extra keys (e.g. both
/// `affected_rows` and `created_table`) fails, matching the old `IonData`
/// equality behavior.
fn match_statement_outcome(outcome: &StatementOutcome, expect: &Element) -> Result<(), String> {
    let expect_struct = expect.as_struct().ok_or_else(|| {
        format!(
            "expected top-level struct in expect, got {}",
            expect.ion_type()
        )
    })?;
    match outcome {
        StatementOutcome::InsertInto { rows, .. } => {
            expect_struct_has_only(expect_struct, &["affected_rows"])?;
            let expected_rows = expect_struct
                .get("affected_rows")
                .and_then(|el| el.as_i64())
                .ok_or_else(|| {
                    format!("expected `affected_rows: <int>` in expect for INSERT; got {expect}")
                })?;
            if expected_rows as u64 != *rows {
                return Err(format!(
                    "affected_rows mismatch: expected {expected_rows}, got {rows}"
                ));
            }
            Ok(())
        }
        StatementOutcome::CreateTableAs {
            canonical_key,
            rows,
            ..
        } => match_created_table(expect_struct, canonical_key, *rows),
        StatementOutcome::CreateTable { canonical_key, .. } => {
            match_created_table(expect_struct, canonical_key, 0)
        }
    }
}

fn expect_struct_has_only(s: &ion_rs::element::Struct, allowed: &[&str]) -> Result<(), String> {
    for (name, _) in s.fields() {
        let n = name.text().unwrap_or("");
        if !allowed.contains(&n) {
            return Err(format!(
                "unexpected field `{n}` in expect (allowed: {allowed:?})"
            ));
        }
    }
    Ok(())
}

fn match_created_table(
    expect_struct: &ion_rs::element::Struct,
    canonical_key: &str,
    rows: u64,
) -> Result<(), String> {
    expect_struct_has_only(expect_struct, &["created_table"])?;
    let inner = expect_struct
        .get("created_table")
        .and_then(|el| el.as_struct())
        .ok_or_else(|| "expected `created_table: {..}` in expect".to_string())?;
    expect_struct_has_only(inner, &["name", "rows"])?;
    let name_list = inner
        .get("name")
        .and_then(|el| el.as_sequence())
        .ok_or_else(|| "created_table.name must be a list".to_string())?;
    // Reject any non-string element (catches `name: ["orders", 123]`) and any
    // wrong length.
    let mut names: Vec<&str> = Vec::new();
    for el in name_list.elements() {
        let s = el.as_string().ok_or_else(|| {
            format!(
                "created_table.name elements must be strings; got {}",
                el.ion_type()
            )
        })?;
        names.push(s);
    }
    if names.len() != 1 || names[0] != canonical_key {
        return Err(format!(
            "created_table.name mismatch: expected [{:?}], got {:?}",
            canonical_key, names
        ));
    }
    let expected_rows = inner
        .get("rows")
        .and_then(|el| el.as_i64())
        .ok_or_else(|| "created_table.rows must be an int".to_string())?;
    if expected_rows as u64 != rows {
        return Err(format!(
            "created_table.rows mismatch: expected {expected_rows}, got {rows}"
        ));
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

    /// The old runner compared the full Ion envelope under `IonData` equality,
    /// so a fixture accidentally naming BOTH `affected_rows` AND `created_table`
    /// would fail. The structured `match_statement_outcome` path must preserve
    /// that: extra fields = mismatch, not silent pass.
    #[test]
    fn extra_field_in_insert_expect_fails() {
        let (_dir, path) = write_case(
            "extra.test.ion",
            r#"
            { sql: "CREATE TABLE t AS (SELECT m.a FROM mem(1,1) m)", expect: { created_table: { name: ["t"], rows: 1 } } }
            { sql: "INSERT INTO t SELECT m.a FROM mem(2,1) m", expect: { affected_rows: 2, created_table: { name: ["t"], rows: 1 } } }
            "#,
        );
        let err = run_case(&path).unwrap_err();
        match err {
            CaseError::Mismatch { detail, .. } => {
                assert!(
                    detail.contains("unexpected field"),
                    "expected an unexpected-field mismatch, got: {detail}"
                );
            }
            other => panic!("expected Mismatch, got: {other:?}"),
        }
    }

    /// Non-string entries in `created_table.name` must fail (the fixture author
    /// typo'd a bare int); a filter_map would silently ignore them.
    #[test]
    fn non_string_name_element_fails() {
        let (_dir, path) = write_case(
            "bad_name.test.ion",
            r#"
            { sql: "CREATE TABLE t AS (SELECT m.a FROM mem(1,1) m)", expect: { created_table: { name: ["t", 123], rows: 1 } } }
            "#,
        );
        let err = run_case(&path).unwrap_err();
        match err {
            CaseError::Mismatch { detail, .. } => {
                assert!(
                    detail.contains("name elements must be strings"),
                    "expected a string-element mismatch, got: {detail}"
                );
            }
            other => panic!("expected Mismatch, got: {other:?}"),
        }
    }

    /// A stray field inside `created_table` (e.g. `columns:`) must fail; keeps
    /// fixture authors from silently drifting the schema.
    #[test]
    fn extra_field_inside_created_table_fails() {
        let (_dir, path) = write_case(
            "extra_inner.test.ion",
            r#"
            { sql: "CREATE TABLE t AS (SELECT m.a FROM mem(1,1) m)", expect: { created_table: { name: ["t"], rows: 1, columns: 1 } } }
            "#,
        );
        let err = run_case(&path).unwrap_err();
        match err {
            CaseError::Mismatch { detail, .. } => {
                assert!(
                    detail.contains("unexpected field `columns`"),
                    "expected an unexpected-field mismatch, got: {detail}"
                );
            }
            other => panic!("expected Mismatch, got: {other:?}"),
        }
    }
}
