//! End-to-end tests for the `pqlite` binary's CLI contract.
//!
//! These drive the built binary as a subprocess (via Cargo's
//! `CARGO_BIN_EXE_pqlite`) so the load-bearing CLI behavior is verified in CI,
//! not just by hand:
//!   * the lazy-open contract — a plain `SELECT` via `exec` needs no `--db` and
//!     creates no file;
//!   * `CREATE TABLE AS` requires `--db` and errors cleanly without it;
//!   * `CREATE TABLE AS` with `--db` creates the table and persists the file;
//!   * a duplicate `CREATE TABLE AS` is rejected.
//!
//! No extra dev-dependencies: just `std::process::Command` and `tempfile`
//! (already a dev-dependency).

use std::process::Command;

/// Path to the freshly built `pqlite` binary, provided by Cargo for integration
/// tests of a binary target in the same package.
const PQLITE: &str = env!("CARGO_BIN_EXE_pqlite");

/// Run `pqlite exec <query>`, optionally with `--db <path>`. Returns
/// (exit_success, stdout, stderr).
fn run_exec(query: &str, db: Option<&std::path::Path>) -> (bool, String, String) {
    let mut cmd = Command::new(PQLITE);
    cmd.arg("exec").arg(query);
    if let Some(path) = db {
        cmd.arg("--db").arg(path);
    }
    let out = cmd.output().expect("failed to spawn pqlite");
    (
        out.status.success(),
        String::from_utf8_lossy(&out.stdout).into_owned(),
        String::from_utf8_lossy(&out.stderr).into_owned(),
    )
}

#[test]
fn plain_select_via_exec_needs_no_db_and_creates_no_file() {
    // Run from inside a temp dir so we can assert nothing was written there.
    // (The `exec` subcommand never builds the REPL history file, so there is
    // nothing to escape cleanup — only the lazy-open db file is at issue here.)
    let dir = tempfile::tempdir().unwrap();

    let out = Command::new(PQLITE)
        .arg("exec")
        .arg("SELECT t.a FROM mem(3,2) t")
        .current_dir(dir.path())
        .output()
        .expect("failed to spawn pqlite");

    assert!(
        out.status.success(),
        "plain SELECT via exec should succeed without --db; stderr: {}",
        String::from_utf8_lossy(&out.stderr)
    );
    let stdout = String::from_utf8_lossy(&out.stdout);
    assert!(
        stdout.contains("'a': 0") && stdout.contains("'a': 2"),
        "expected the 3-row bag on stdout, got: {stdout}"
    );

    // The lazy-open contract: no database file was created anywhere in the cwd.
    let stray: Vec<_> = std::fs::read_dir(dir.path())
        .unwrap()
        .filter_map(|e| e.ok())
        .map(|e| e.file_name().to_string_lossy().into_owned())
        .collect();
    assert!(
        stray.is_empty(),
        "a plain SELECT must create no files, but found: {stray:?}"
    );
}

#[test]
fn ctas_without_db_errors_cleanly() {
    let (ok, stdout, stderr) = run_exec("CREATE TABLE t AS (SELECT t.a FROM mem(1,2) t)", None);

    assert!(!ok, "CTAS without --db must fail");
    assert!(
        stderr.contains("--db") && stderr.contains("CREATE TABLE AS"),
        "expected a --db-required error mentioning CREATE TABLE AS, got: {stderr}"
    );
    assert!(
        stdout.is_empty(),
        "a failed write must print nothing to stdout"
    );
}

#[test]
fn ctas_with_db_creates_table_and_persists_file() {
    let dir = tempfile::tempdir().unwrap();
    let db = dir.path().join("ctas.pqlite");

    let (ok, stdout, stderr) = run_exec(
        "CREATE TABLE widgets AS (SELECT t.a FROM mem(5,2) t)",
        Some(&db),
    );

    assert!(ok, "CTAS with --db should succeed; stderr: {stderr}");
    assert!(
        stdout.is_empty(),
        "a write must leave stdout empty, got: {stdout}"
    );
    assert!(
        stderr.contains("Created table") && stderr.contains("widgets") && stderr.contains("5 rows"),
        "expected the creation confirmation on stderr, got: {stderr}"
    );
    assert!(db.is_file(), "the --db file should exist after CTAS");
}

#[test]
fn duplicate_ctas_is_rejected_and_names_the_table() {
    let dir = tempfile::tempdir().unwrap();
    let db = dir.path().join("dup.pqlite");

    let (ok1, _, _) = run_exec("CREATE TABLE t AS (SELECT t.a FROM mem(2,2) t)", Some(&db));
    assert!(ok1, "first CTAS should succeed");

    let (ok2, _, stderr2) = run_exec("CREATE TABLE t AS (SELECT t.a FROM mem(1,2) t)", Some(&db));
    assert!(!ok2, "second CTAS for the same table must fail");
    // Anchor the table name to the error context. A bare `contains('t')` would
    // be tautological: the word "table" in the message already supplies a 't",
    // so it would pass even if the name were dropped from the error.
    assert!(
        stderr2.contains("already exists: t"),
        "duplicate error should name the table, got: {stderr2}"
    );
}
