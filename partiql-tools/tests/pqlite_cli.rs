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
fn fresh_db_bootstraps_and_select_star_from_tables_works() {
    let dir = tempfile::tempdir().unwrap();
    let db = dir.path().join("boot.pqlite");
    let (ok, stdout, stderr) = run_exec("SELECT t.a FROM mem(1,1) t", Some(&db));
    assert!(ok, "stderr: {stderr}");
    assert!(
        !stdout.contains("_tables") && !stderr.contains("Created table _tables"),
        "bootstrap must be silent: out={stdout} err={stderr}"
    );
    let (ok2, stdout2, stderr2) = run_exec("SELECT * FROM _tables", Some(&db));
    assert!(ok2, "stderr: {stderr2}");
    assert!(
        stdout2.contains("_tables"),
        "expected self-entry: {stdout2}"
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
fn select_from_unknown_table_errors_cleanly() {
    let dir = tempfile::tempdir().unwrap();
    let dbp = dir.path().join("unknown.pqlite");

    // CTAS one table so the .pqlite env exists.
    let (ok, _stdout, _stderr) = run_exec(
        "CREATE TABLE real_table AS (SELECT m.a FROM mem(1, 1) m)",
        Some(&dbp),
    );
    assert!(ok, "CTAS setup must succeed");

    // SELECT from a DIFFERENT, non-existent table.
    let (ok, _stdout, stderr) = run_exec("SELECT * FROM ghost_table", Some(&dbp));
    assert!(
        !ok,
        "SELECT from unknown table must fail with non-zero exit"
    );
    assert!(
        stderr.contains("Table 'ghost_table' not found"),
        "stderr should name the missing table; got: {}",
        stderr
    );
}

#[test]
fn select_from_unknown_table_without_db_errors_cleanly() {
    // No --db, so `self.heed` is None inside the catalog. The recorder still
    // captures "ghost" from the bare-name probe and fail-fast fires before
    // VM construction. Exercises the lazy-open path with an unknown table.
    let dir = tempfile::tempdir().unwrap();
    let out = Command::new(PQLITE)
        .arg("exec")
        .arg("SELECT * FROM ghost")
        .current_dir(dir.path())
        .output()
        .expect("failed to spawn pqlite");

    assert!(
        !out.status.success(),
        "SELECT from unknown table (no --db) must fail with non-zero exit"
    );
    let stderr = String::from_utf8_lossy(&out.stderr);
    assert!(
        stderr.contains("Table 'ghost' not found"),
        "stderr should name the missing table; got: {stderr}"
    );

    // Lazy-open contract still holds: no db file created.
    let stray: Vec<_> = std::fs::read_dir(dir.path())
        .unwrap()
        .filter_map(|e| e.ok())
        .map(|e| e.file_name().to_string_lossy().into_owned())
        .collect();
    assert!(
        stray.is_empty(),
        "a failed unknown-table SELECT must create no files, but found: {stray:?}"
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
fn ctas_with_bare_db_filename_creates_file_in_cwd() {
    // Regression: `pqlite --db foo.pqlite` with a BARE filename (no directory
    // component) must create the file in the current directory, like any UNIX
    // tool — not fail with ENOENT. The child runs with its cwd set to a temp
    // dir, so the bare name resolves there.
    let dir = tempfile::tempdir().unwrap();

    let out = Command::new(PQLITE)
        .arg("--db")
        .arg("bare.pqlite") // bare name: parent() is "" — the bug case
        .arg("exec")
        .arg("CREATE TABLE t AS (SELECT t.a FROM mem(2,2) t)")
        .current_dir(dir.path())
        .output()
        .expect("failed to spawn pqlite");

    assert!(
        out.status.success(),
        "bare --db filename should create the db in cwd; stderr: {}",
        String::from_utf8_lossy(&out.stderr)
    );
    assert!(
        dir.path().join("bare.pqlite").is_file(),
        "the bare-named db file should exist in the cwd after CTAS"
    );
}

#[test]
fn ctas_with_missing_parent_dir_errors() {
    // The flip side of the bare-name fix: a path whose parent directory does
    // NOT exist must still error (we never create directories on the user's
    // behalf). This guards against the normalization over-reaching.
    let dir = tempfile::tempdir().unwrap();
    let missing = dir.path().join("does_not_exist").join("x.pqlite");

    let (ok, _, stderr) = run_exec(
        "CREATE TABLE t AS (SELECT t.a FROM mem(1,2) t)",
        Some(&missing),
    );
    assert!(!ok, "CTAS into a missing parent dir must fail");
    assert!(
        stderr.contains("could not open database"),
        "expected an open error for the missing parent, got: {stderr}"
    );
    assert!(
        !missing.exists(),
        "we must not create the file or its parent dir"
    );
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

// End-to-end CTAS-then-SELECT round-trip tests. Each spins up the binary
// twice against the same temp .pqlite file: once to write, once to read.
// Output format expectations (assertion source-of-truth, taken from the
// binary's actual stdout):
//   * SELECT results render as `<<\n  { 'a': 0 },\n  { 'a': 1 } ...\n>>`
//   * String values render single-quoted: `'x'`
//   * NULL renders uppercase: `NULL`
//   * Empty bag renders `<<\n\n>>`

#[test]
fn ctas_then_select_single_i64_column() {
    let dir = tempfile::tempdir().unwrap();
    let dbp = dir.path().join("e2e.pqlite");

    let (ok, _, stderr) = run_exec("CREATE TABLE t AS (SELECT t.a FROM mem(3,2) t)", Some(&dbp));
    assert!(ok, "CTAS should succeed; stderr: {stderr}");

    let (ok, stdout, stderr) = run_exec("SELECT * FROM t", Some(&dbp));
    assert!(ok, "SELECT should succeed; stderr: {stderr}");
    // mem(3,2) yields a=0,1,2 in insertion order.
    assert!(stdout.contains("'a': 0"), "got: {stdout}");
    assert!(stdout.contains("'a': 1"), "got: {stdout}");
    assert!(stdout.contains("'a': 2"), "got: {stdout}");
}

#[test]
fn ctas_then_select_mixed_scalars() {
    let dir = tempfile::tempdir().unwrap();
    let dbp = dir.path().join("mixed.pqlite");

    let (ok, _, stderr) = run_exec(
        "CREATE TABLE t AS (SELECT 1 AS a, 2.5 AS b, 'x' AS c, NULL AS d FROM mem(1,2) m)",
        Some(&dbp),
    );
    assert!(ok, "CTAS should succeed; stderr: {stderr}");

    let (ok, stdout, stderr) = run_exec("SELECT * FROM t", Some(&dbp));
    assert!(ok, "SELECT should succeed; stderr: {stderr}");
    assert!(stdout.contains("'a': 1"), "got: {stdout}");
    assert!(stdout.contains("'b': 2.5"), "got: {stdout}");
    assert!(stdout.contains("'c': 'x'"), "got: {stdout}");
    assert!(stdout.contains("'d': NULL"), "got: {stdout}");
    // The Missing value would be a regression — distinct from explicit NULL.
    assert!(!stdout.contains("MISSING"), "got: {stdout}");
}

#[test]
fn ctas_then_select_preserves_multi_field_struct_shape() {
    // Two-column row guards field-name preservation and column ordering
    // through the encode → LMDB → decode → format pipeline. The nested-tuple
    // case (a column whose VALUE is itself a tuple) is covered separately by
    // ctas_then_select_runtime_tuple and ctas_then_select_nested_tuple_in_list.
    let dir = tempfile::tempdir().unwrap();
    let dbp = dir.path().join("tuple.pqlite");

    let (ok, _, stderr) = run_exec(
        "CREATE TABLE t AS (SELECT 99 AS outer_a, 'inner_a' AS outer_b FROM mem(1,2) m)",
        Some(&dbp),
    );
    assert!(ok, "CTAS should succeed; stderr: {stderr}");

    let (ok, stdout, stderr) = run_exec("SELECT * FROM t", Some(&dbp));
    assert!(ok, "SELECT should succeed; stderr: {stderr}");
    // Both field names survive the round trip.
    assert!(stdout.contains("'outer_a': 99"), "got: {stdout}");
    assert!(stdout.contains("'outer_b': 'inner_a'"), "got: {stdout}");
}

#[test]
fn ctas_zero_rows_then_select_prints_no_rows_no_error() {
    let dir = tempfile::tempdir().unwrap();
    let dbp = dir.path().join("empty.pqlite");

    let (ok, _, stderr) = run_exec(
        "CREATE TABLE t AS (SELECT t.a FROM mem(1,2) t WHERE FALSE)",
        Some(&dbp),
    );
    assert!(ok, "zero-row CTAS should succeed; stderr: {stderr}");

    let (ok, stdout, stderr) = run_exec("SELECT * FROM t", Some(&dbp));
    assert!(
        ok,
        "SELECT against empty table should succeed; stderr: {stderr}"
    );
    // Empty bag: the `<<` opener and `>>` closer surround a blank line, no rows.
    assert!(stdout.contains("<<"), "got: {stdout}");
    assert!(stdout.contains(">>"), "got: {stdout}");
    assert!(
        !stdout.contains("'a':"),
        "expected no row payloads; got: {stdout}"
    );
    // Footer reports 0 rows. Anchor to the opening paren to avoid matching
    // a hypothetical "10 rows" prefix.
    assert!(stderr.contains("(0 rows "), "stderr: {stderr}");
}

#[test]
fn ctas_then_select_10k_rows_completes() {
    let dir = tempfile::tempdir().unwrap();
    let dbp = dir.path().join("big.pqlite");

    let (ok, _, stderr) = run_exec(
        "CREATE TABLE t AS (SELECT t.a FROM mem(10000,2) t)",
        Some(&dbp),
    );
    assert!(ok, "10k-row CTAS should succeed; stderr: {stderr}");

    let (ok, stdout, stderr) = run_exec("SELECT * FROM t", Some(&dbp));
    assert!(ok, "10k-row SELECT should succeed; stderr: {stderr}");
    // The final row (a=9999) must be visible; this catches premature termination.
    assert!(
        stdout.contains("'a': 9999"),
        "expected final row visible; got tail: {:?}",
        &stdout[stdout.len().saturating_sub(200)..]
    );
    // Footer reports 10000 rows. Anchor to the opening paren for tightness.
    assert!(stderr.contains("(10000 rows "), "stderr: {stderr}");
}

/// Streaming chunk-boundary regression: a table whose row count is an EXACT
/// multiple of the internal CHUNK_ROWS (64) exercises the path where a full
/// chunk is followed by an empty trailing chunk that signals end-of-scan. A
/// resume-cursor or exhausted-flag off-by-one here would drop the last row or
/// loop forever. `mem(64,1)` and `mem(128,1)` sit exactly on the 1x and 2x
/// boundaries; every other end-to-end test lands on a partial final chunk.
#[test]
fn ctas_then_select_exact_chunk_multiple_rows() {
    for (n, last) in [(64u32, 63u32), (128, 127)] {
        let dir = tempfile::tempdir().unwrap();
        let dbp = dir.path().join(format!("chunk_{n}.pqlite"));

        let (ok, _, stderr) = run_exec(
            &format!("CREATE TABLE t AS (SELECT t.a FROM mem({n},1) t)"),
            Some(&dbp),
        );
        assert!(ok, "{n}-row CTAS should succeed; stderr: {stderr}");

        let (ok, stdout, stderr) = run_exec("SELECT * FROM t", Some(&dbp));
        assert!(ok, "{n}-row SELECT should succeed; stderr: {stderr}");
        // Exactly N rows — no dropped last row, no phantom extra row.
        assert!(stderr.contains(&format!("({n} rows ")), "stderr: {stderr}");
        // The final row (a = N-1) survives the full-chunk -> empty-chunk handoff.
        assert!(
            stdout.contains(&format!("'a': {last}")),
            "expected final row a={last} visible for n={n}; got tail: {:?}",
            &stdout[stdout.len().saturating_sub(200)..]
        );
    }
}

#[test]
fn two_tables_in_one_db_two_selects_in_one_run() {
    // The pqlite binary does not support multi-statement input (`;`-separated
    // queries are rejected by the parser). Each statement is issued as a
    // separate `pqlite exec` invocation against the same `--db` file.
    let dir = tempfile::tempdir().unwrap();
    let dbp = dir.path().join("two.pqlite");

    let (ok, _, stderr) = run_exec(
        "CREATE TABLE alpha AS (SELECT t.a FROM mem(1,2) t)",
        Some(&dbp),
    );
    assert!(ok, "alpha CTAS should succeed; stderr: {stderr}");

    let (ok, _, stderr) = run_exec(
        "CREATE TABLE beta AS (SELECT t.a FROM mem(2,2) t)",
        Some(&dbp),
    );
    assert!(ok, "beta CTAS should succeed; stderr: {stderr}");

    let (ok, stdout_alpha, stderr) = run_exec("SELECT * FROM alpha", Some(&dbp));
    assert!(ok, "SELECT alpha should succeed; stderr: {stderr}");
    assert!(stdout_alpha.contains("'a': 0"), "alpha got: {stdout_alpha}");

    let (ok, stdout_beta, stderr) = run_exec("SELECT * FROM beta", Some(&dbp));
    assert!(ok, "SELECT beta should succeed; stderr: {stderr}");
    assert!(stdout_beta.contains("'a': 0"), "beta got: {stdout_beta}");
    assert!(stdout_beta.contains("'a': 1"), "beta got: {stdout_beta}");
}

#[test]
fn select_surfaces_truncated_row_error() {
    let dir = tempfile::tempdir().unwrap();
    let dbp = dir.path().join("trunc.pqlite");

    // Seed a valid row at row id 0 so the LMDB env + named db exist.
    let (ok, _, stderr) = run_exec("CREATE TABLE t AS (SELECT t.a FROM mem(1,2) t)", Some(&dbp));
    assert!(ok, "CTAS should succeed; stderr: {stderr}");

    // Inject a bad row at id 1: just the TAG_INTEGER byte, no 8-byte payload.
    // LMDB is single-writer; open the env, inject, and drop before run_exec.
    {
        let db = partiql_tools::storage::HeedDB::open(&dbp).unwrap();
        partiql_tools::test_support::inject_row(
            &db,
            "t",
            1,
            &[partiql_tools::row_codec::TAG_INTEGER],
        );
    }

    let (ok, _, stderr) = run_exec("SELECT * FROM t", Some(&dbp));
    assert!(!ok, "SELECT over a truncated row must fail");
    assert!(
        stderr.contains("row 1 decode failed") && stderr.contains("truncated"),
        "stderr was: {stderr}"
    );
}

#[test]
fn select_surfaces_unknown_tag_error() {
    let dir = tempfile::tempdir().unwrap();
    let dbp = dir.path().join("badtag.pqlite");

    let (ok, _, stderr) = run_exec("CREATE TABLE t AS (SELECT t.a FROM mem(1,2) t)", Some(&dbp));
    assert!(ok, "CTAS should succeed; stderr: {stderr}");

    // 0xFE is unassigned in the tag space.
    // LMDB is single-writer; open the env, inject, and drop before run_exec.
    {
        let db = partiql_tools::storage::HeedDB::open(&dbp).unwrap();
        partiql_tools::test_support::inject_row(&db, "t", 1, &[0xFE]);
    }

    let (ok, _, stderr) = run_exec("SELECT * FROM t", Some(&dbp));
    assert!(!ok, "SELECT over an unknown-tag row must fail");
    assert!(
        stderr.contains("row 1 decode failed") && stderr.contains("unknown tag"),
        "stderr was: {stderr}"
    );
}

#[test]
fn quoted_table_name_preserves_case_through_roundtrip() {
    let dir = tempfile::tempdir().unwrap();
    let dbp = dir.path().join("quoted.pqlite");

    // CTAS with a CASE-SENSITIVE quoted identifier.
    let (ok, _stdout, stderr) = run_exec(
        r#"CREATE TABLE "MyTable" AS (SELECT 42 AS id FROM mem(1, 1) m)"#,
        Some(&dbp),
    );
    assert!(
        ok,
        "CTAS with quoted name should succeed; stderr: {}",
        stderr
    );

    // SELECT with the SAME case-sensitive name should find the table and return the row.
    let (ok, stdout, stderr) = run_exec(r#"SELECT * FROM "MyTable""#, Some(&dbp));
    assert!(
        ok,
        "SELECT with matching quoted name should succeed; stderr: {}",
        stderr
    );
    assert!(
        stdout.contains("42"),
        "expected '42' in output; got stdout: {}",
        stdout
    );
}

#[test]
fn quoted_table_name_is_not_found_via_different_case() {
    let dir = tempfile::tempdir().unwrap();
    let dbp = dir.path().join("case_distinct.pqlite");

    // CTAS with a CASE-SENSITIVE quoted identifier.
    let (ok, _, stderr) = run_exec(
        r#"CREATE TABLE "Foo" AS (SELECT 1 AS x FROM mem(1, 1) m)"#,
        Some(&dbp),
    );
    assert!(ok, "CTAS should succeed; stderr: {}", stderr);

    // SELECT with a BARE identifier (case-insensitive, lowercased to "foo") should NOT find "Foo".
    let (ok, _, stderr) = run_exec(r#"SELECT * FROM foo"#, Some(&dbp));
    assert!(
        !ok,
        "bare 'foo' should not find quoted \"Foo\" — case-sensitive identifiers are distinct"
    );
    assert!(
        stderr.contains("Table 'foo' not found"),
        "expected fail-fast error naming bare 'foo'; got stderr: {}",
        stderr
    );
}

#[test]
fn wire_format_byte_shape_is_stable() {
    // Pin the on-disk byte layout for a 2-column row. Round-trip tests can
    // accidentally compensate for an LE↔BE flip in both encoder and decoder;
    // a bit-for-bit anchor cannot. If this fails, the byte literal is the
    // source of truth — investigate `serialize_row`, do not rewrite the test.
    let dir = tempfile::tempdir().unwrap();
    let dbp = dir.path().join("anchor.pqlite");

    let (ok, _, stderr) = run_exec(
        "CREATE TABLE t AS (SELECT 1 AS a, 'xy' AS b FROM mem(1,2) m)",
        Some(&dbp),
    );
    assert!(ok, "CTAS should succeed; stderr: {stderr}");

    let row0 = {
        let db = partiql_tools::storage::HeedDB::open(&dbp).unwrap();
        partiql_tools::test_support::read_row(&db, "t", 0)
    };

    // Expected wire format for {"a": 1i64, "b": "xy"}:
    //   TAG_TUPLE (0x08)
    //   field_count = 2 (LE u32: 02 00 00 00)
    //   field 0:
    //     name_len = 1 (LE u32: 01 00 00 00)
    //     name = 'a'   (0x61)
    //     TAG_INTEGER (0x03)
    //     i64 value = 1 (LE: 01 00 00 00 00 00 00 00)
    //   field 1:
    //     name_len = 1 (LE u32: 01 00 00 00)
    //     name = 'b'   (0x62)
    //     TAG_STRING (0x06)
    //     str_len = 2  (LE u32: 02 00 00 00)
    //     str bytes = 'x' 'y' (0x78 0x79)
    // name_len (1) deliberately differs from str_len (2) so a regression that
    // swapped the two u32 prefixes would not cancel out.
    let expected: &[u8] = &[
        0x08, // TAG_TUPLE
        0x02, 0x00, 0x00, 0x00, // field_count = 2
        0x01, 0x00, 0x00, 0x00, 0x61, // name "a"
        0x03, // TAG_INTEGER
        0x01, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, // i64 1 LE
        0x01, 0x00, 0x00, 0x00, 0x62, // name "b"
        0x06, // TAG_STRING
        0x02, 0x00, 0x00, 0x00, // str_len = 2
        0x78, 0x79, // "xy"
    ];

    assert_eq!(
        row0, expected,
        "wire format byte shape regressed: expected {expected:02x?}, got {row0:02x?}",
    );
}

#[test]
fn ctas_then_select_bool_column() {
    let dir = tempfile::tempdir().unwrap();
    let dbp = dir.path().join("bool_rt.pqlite");
    // mem's arg order is (rows, cols): mem(2,1) yields two rows a=0,a=1,
    // so `m.a = 0` produces true then false. (Not mem(1,2), which is one row.)
    let (ok, _out, err) = run_exec(
        "CREATE TABLE t AS (SELECT (m.a = 0) AS b FROM mem(2,1) m)",
        Some(&dbp),
    );
    assert!(ok, "CTAS failed: {err}");
    let (ok, out, err) = run_exec("SELECT * FROM t", Some(&dbp));
    assert!(ok, "SELECT failed: {err}");
    assert!(
        out.contains("true"),
        "expected 'true' in output; got: {out}"
    );
    assert!(
        out.contains("false"),
        "expected 'false' in output; got: {out}"
    );
}

#[test]
fn ctas_then_select_missing_column() {
    let dir = tempfile::tempdir().unwrap();
    let dbp = dir.path().join("missing_rt.pqlite");
    // mem's arg order is (rows, cols); mem(2,1) yields two rows.
    let (ok, _out, err) = run_exec(
        "CREATE TABLE t AS (SELECT m.nonexistent AS x FROM mem(2,1) m)",
        Some(&dbp),
    );
    assert!(ok, "CTAS failed: {err}");
    let (ok, out, err) = run_exec("SELECT * FROM t", Some(&dbp));
    assert!(ok, "SELECT failed: {err}");
    assert!(
        out.contains("MISSING"),
        "expected MISSING in output; got: {out}"
    );
}

#[test]
fn invalid_bool_payload_surfaces_error() {
    let dir = tempfile::tempdir().unwrap();
    let dbp = dir.path().join("bad_bool.pqlite");
    let (ok, _out, err) = run_exec(
        "CREATE TABLE t AS (SELECT (m.a = 0) AS b FROM mem(1,1) m)",
        Some(&dbp),
    );
    assert!(ok, "CTAS setup failed: {err}");
    let db = partiql_tools::storage::HeedDB::open(&dbp).unwrap();
    partiql_tools::test_support::inject_row(
        &db,
        "t",
        1,
        &[
            partiql_tools::row_codec::TAG_TUPLE,
            0x01,
            0x00,
            0x00,
            0x00,
            0x01,
            0x00,
            0x00,
            0x00,
            b'b',
            partiql_tools::row_codec::TAG_BOOL,
            0x7F,
        ],
    );
    drop(db);
    let (ok, _out, err) = run_exec("SELECT * FROM t", Some(&dbp));
    assert!(!ok, "expected SELECT to fail on invalid bool payload");
    assert!(
        err.contains("invalid bool") && err.contains("0x7f"),
        "expected the specific bad byte 0x7f to propagate; got: {err}"
    );
}

#[test]
#[cfg_attr(windows, ignore)]
fn ctas_rejects_oversize_tuple_rolls_back_wtxn() {
    // A mid-encode rejection must roll back the wtxn, leaving `_tables` clean.
    // Trigger: a 1025-field runtime tuple trips MAX_FIELDS_PER_ROW. (An oversize
    // string would exceed the OS arg length limit when passed via the CLI.)
    let dir = tempfile::tempdir().unwrap();
    let db_path = dir.path().join("oversize.pqlite");
    let field_count = partiql_tools::row_codec::MAX_FIELDS_PER_ROW + 1;
    let fields: String = (0..field_count)
        .map(|i| format!("'k{i}': {i}"))
        .collect::<Vec<_>>()
        .join(", ");
    let query = format!("CREATE TABLE t AS (SELECT {{ {fields} }} AS obj FROM mem(1,1) m)");
    let (ok, _stdout, stderr) = run_exec(&query, Some(&db_path));
    assert!(
        !ok,
        "oversize tuple column must be rejected; stderr: {stderr}"
    );
    assert!(
        stderr.contains("MAX_FIELDS_PER_ROW"),
        "error must name the field-count cap; got: {stderr}",
    );
    // Verify wtxn rollback by re-opening: the env file may persist on disk,
    // but the catalog must not register `t`.
    let env = unsafe {
        heed::EnvOpenOptions::new()
            .map_size(1024 * 1024 * 1024)
            .max_dbs(128)
            .flags(heed::EnvFlags::NO_SUB_DIR)
            .open(&db_path)
            .expect("re-open env")
    };
    let rtxn = env.read_txn().expect("read txn");
    let catalog: heed::Database<heed::types::Str, heed::types::Bytes> = env
        .open_database(&rtxn, Some("_tables"))
        .expect("open catalog")
        .expect("catalog must exist");
    assert!(
        catalog.get(&rtxn, "t").expect("catalog get").is_none(),
        "row-0 rejection must leave catalog clean even if env file persists"
    );
}

#[test]
fn string_at_1kb_encodes_and_round_trips() {
    let dir = tempfile::tempdir().unwrap();
    let dbp = dir.path().join("kb_str.pqlite");
    let query = format!(
        "CREATE TABLE t AS (SELECT '{}' AS s FROM mem(1,1) m)",
        "x".repeat(1024)
    );
    let (ok, _out, err) = run_exec(&query, Some(&dbp));
    assert!(ok, "1KB string CTAS failed: {err}");
    let (ok, out, err) = run_exec("SELECT * FROM t", Some(&dbp));
    assert!(ok, "SELECT failed: {err}");
    assert!(
        out.contains(&"x".repeat(1024)),
        "1KB string did not round-trip"
    );
}

#[test]
fn ctas_then_select_value_int_rows() {
    let dir = tempfile::tempdir().unwrap();
    let dbp = dir.path().join("sv_int.pqlite");
    let (ok, _out, err) = run_exec(
        "CREATE TABLE t AS (SELECT VALUE m.a * 1000 + 777 FROM mem(3,1) m)",
        Some(&dbp),
    );
    assert!(ok, "CTAS failed: {err}");
    let (ok, out, err) = run_exec("SELECT * FROM t", Some(&dbp));
    assert!(ok, "SELECT failed: {err}");
    // m.a = 0,1,2 -> 777, 1777, 2777 : distinctive, won't collide with framing.
    assert!(out.contains("777"), "expected 777; got: {out}");
    assert!(out.contains("1777"), "expected 1777; got: {out}");
    assert!(out.contains("2777"), "expected 2777; got: {out}");
}

#[test]
fn ctas_then_select_value_string_rows() {
    let dir = tempfile::tempdir().unwrap();
    let dbp = dir.path().join("sv_str.pqlite");
    let (ok, _out, err) = run_exec(
        "CREATE TABLE t AS (SELECT VALUE 'hello' FROM mem(2,1) m)",
        Some(&dbp),
    );
    assert!(ok, "CTAS failed: {err}");
    let (ok, out, err) = run_exec("SELECT * FROM t", Some(&dbp));
    assert!(ok, "SELECT failed: {err}");
    assert!(out.contains("hello"), "expected 'hello'; got: {out}");
}

#[test]
fn ctas_then_select_value_null_distinct_from_missing() {
    let dir = tempfile::tempdir().unwrap();
    let dbp = dir.path().join("sv_null.pqlite");
    let (ok, _out, err) = run_exec(
        "CREATE TABLE t AS (SELECT VALUE NULL FROM mem(1,1) m)",
        Some(&dbp),
    );
    assert!(ok, "CTAS failed: {err}");
    let (ok, out, err) = run_exec("SELECT * FROM t", Some(&dbp));
    assert!(ok, "SELECT failed: {err}");
    assert!(out.contains("NULL"), "expected NULL; got: {out}");
    assert!(
        !out.contains("MISSING"),
        "NULL must not render as MISSING; got: {out}"
    );
}

#[test]
fn top_level_scalar_row_is_bare_tag_on_disk() {
    let dir = tempfile::tempdir().unwrap();
    let dbp = dir.path().join("bare.pqlite");
    let (ok, _out, err) = run_exec(
        "CREATE TABLE t AS (SELECT VALUE m.a FROM mem(1,1) m)",
        Some(&dbp),
    );
    assert!(ok, "CTAS failed: {err}");
    let db = partiql_tools::storage::HeedDB::open(&dbp).unwrap();
    let row0 = partiql_tools::test_support::read_row(&db, "t", 0);
    drop(db);
    // Bare TAG_INTEGER (0x03) + i64 0 LE — NO tuple wrapper (no 0x08 prefix).
    let expected: &[u8] = &[
        0x03, // TAG_INTEGER
        0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, // i64 0 LE
    ];
    assert_eq!(
        row0, expected,
        "top-level scalar must be a bare tag, not tuple-wrapped"
    );
}

#[test]
fn deeply_nested_tuple_payload_within_cap_round_trips() {
    // A tuple nested a few levels deep, crafted as raw bytes, must decode
    // cleanly (well within MAX_RECURSION_DEPTH). This directly exercises the
    // decode_tagged_into recursion without needing the planner to produce a
    // nested static struct (which it does not yet do).
    let dir = tempfile::tempdir().unwrap();
    let dbp = dir.path().join("nest_ok.pqlite");
    // Seed a real table so the table + catalog exist.
    let (ok, _o, e) = run_exec(
        "CREATE TABLE t AS (SELECT 1 AS a FROM mem(1,1) m)",
        Some(&dbp),
    );
    assert!(ok, "setup CTAS failed: {e}");

    // Build a payload: 4 nested single-field tuples, innermost value = i64 42.
    // Each tuple frame: TAG_TUPLE, field_count=1(LE u32), name_len=1(LE u32), 'n', <value>.
    fn wrap(inner: Vec<u8>) -> Vec<u8> {
        let mut v = vec![partiql_tools::row_codec::TAG_TUPLE];
        v.extend_from_slice(&1u32.to_le_bytes()); // field_count
        v.extend_from_slice(&1u32.to_le_bytes()); // name_len
        v.push(b'n');
        v.extend_from_slice(&inner);
        v
    }
    let mut payload = vec![partiql_tools::row_codec::TAG_INTEGER];
    payload.extend_from_slice(&42i64.to_le_bytes());
    for _ in 0..4 {
        payload = wrap(payload);
    }

    let db = partiql_tools::storage::HeedDB::open(&dbp).unwrap();
    partiql_tools::test_support::inject_row(&db, "t", 1, &payload);
    drop(db);

    let (ok, out, err) = run_exec("SELECT * FROM t", Some(&dbp));
    assert!(ok, "SELECT over nested tuple failed: {err}");
    assert!(
        out.contains("42"),
        "nested value 42 must survive round-trip; got: {out}"
    );
}

#[test]
fn over_deep_nested_tuple_payload_is_rejected() {
    let dir = tempfile::tempdir().unwrap();
    let dbp = dir.path().join("nest_deep.pqlite");
    let (ok, _o, e) = run_exec(
        "CREATE TABLE t AS (SELECT 1 AS a FROM mem(1,1) m)",
        Some(&dbp),
    );
    assert!(ok, "setup CTAS failed: {e}");

    fn wrap(inner: Vec<u8>) -> Vec<u8> {
        let mut v = vec![partiql_tools::row_codec::TAG_TUPLE];
        v.extend_from_slice(&1u32.to_le_bytes());
        v.extend_from_slice(&1u32.to_le_bytes());
        v.push(b'n');
        v.extend_from_slice(&inner);
        v
    }
    let mut payload = vec![partiql_tools::row_codec::TAG_INTEGER];
    payload.extend_from_slice(&0i64.to_le_bytes());
    // Nest deeper than the cap. MAX_RECURSION_DEPTH + 5 levels guarantees rejection.
    let levels = partiql_tools::row_codec::MAX_RECURSION_DEPTH + 5;
    for _ in 0..levels {
        payload = wrap(payload);
    }

    let db = partiql_tools::storage::HeedDB::open(&dbp).unwrap();
    partiql_tools::test_support::inject_row(&db, "t", 1, &payload);
    drop(db);

    let (ok, _out, err) = run_exec("SELECT * FROM t", Some(&dbp));
    assert!(!ok, "over-deep nesting must be rejected");
    assert!(
        err.contains("nesting depth") || err.contains("depth"),
        "error must mention depth; got: {err}"
    );
}

#[test]
fn ctas_then_select_list_of_ints() {
    let dir = tempfile::tempdir().unwrap();
    let dbp = dir.path().join("list_ints.pqlite");
    let (ok, _out, err) = run_exec(
        "CREATE TABLE t AS (SELECT [10, 20, 30] AS xs FROM mem(1,1) m)",
        Some(&dbp),
    );
    assert!(ok, "list CTAS should succeed; stderr: {err}");
    let (ok, out, err) = run_exec("SELECT * FROM t", Some(&dbp));
    assert!(ok, "SELECT failed: {err}");
    assert!(out.contains("10"), "expected 10; got: {out}");
    assert!(out.contains("20"), "expected 20; got: {out}");
    assert!(out.contains("30"), "expected 30; got: {out}");
}

#[test]
fn ctas_then_select_bag_of_ints() {
    let dir = tempfile::tempdir().unwrap();
    let dbp = dir.path().join("bag_ints.pqlite");
    // `<< ... >>` is the PartiQL bag-literal syntax and parses cleanly.
    let (ok, _out, err) = run_exec(
        "CREATE TABLE t AS (SELECT << 100, 200 >> AS xs FROM mem(1,1) m)",
        Some(&dbp),
    );
    assert!(ok, "bag CTAS should succeed; stderr: {err}");
    let (ok, out, err) = run_exec("SELECT * FROM t", Some(&dbp));
    assert!(ok, "SELECT failed: {err}");
    assert!(out.contains("100"), "expected 100; got: {out}");
    assert!(out.contains("200"), "expected 200; got: {out}");
}

#[test]
fn ctas_then_select_empty_list() {
    let dir = tempfile::tempdir().unwrap();
    let dbp = dir.path().join("empty_list.pqlite");
    // An empty container is legal: count=0, no elements, no step_in.
    let (ok, _out, err) = run_exec(
        "CREATE TABLE t AS (SELECT [] AS xs FROM mem(1,1) m)",
        Some(&dbp),
    );
    assert!(ok, "empty-list CTAS should succeed; stderr: {err}");
    let (ok, out, err) = run_exec("SELECT * FROM t", Some(&dbp));
    assert!(ok, "SELECT over empty list failed: {err}");
    // Empty list renders as `[]` inside the row tuple.
    assert!(out.contains("'xs': []"), "expected empty list; got: {out}");
}

#[test]
fn ctas_then_select_nested_tuple_in_list() {
    let dir = tempfile::tempdir().unwrap();
    let dbp = dir.path().join("nested.pqlite");
    // A list of runtime tuples exercises the full recursion:
    // list → tuple → scalar, on both encode and decode.
    let (ok, _out, err) = run_exec(
        "CREATE TABLE t AS (SELECT [{ 'a': 1 }, { 'a': 2 }] AS xs FROM mem(1,1) m)",
        Some(&dbp),
    );
    assert!(
        ok,
        "nested-tuple-in-list CTAS should succeed; stderr: {err}"
    );
    let (ok, out, err) = run_exec("SELECT * FROM t", Some(&dbp));
    assert!(ok, "SELECT failed: {err}");
    assert!(out.contains("1"), "expected 1; got: {out}");
    assert!(out.contains("2"), "expected 2; got: {out}");
    // Both tuples preserve their field name through the round trip.
    assert!(out.contains("'a': 1"), "expected 'a': 1; got: {out}");
    assert!(out.contains("'a': 2"), "expected 'a': 2; got: {out}");
}

#[test]
fn ctas_then_select_runtime_tuple() {
    let dir = tempfile::tempdir().unwrap();
    let dbp = dir.path().join("runtime_tuple.pqlite");
    // A runtime `{...}` tuple value flows through write_tuple_via_view
    // (distinct from the static-struct encode path).
    let (ok, _out, err) = run_exec(
        "CREATE TABLE t AS (SELECT { 'k': 42 } AS obj FROM mem(1,1) m)",
        Some(&dbp),
    );
    assert!(ok, "runtime-tuple CTAS should succeed; stderr: {err}");
    let (ok, out, err) = run_exec("SELECT * FROM t", Some(&dbp));
    assert!(ok, "SELECT failed: {err}");
    assert!(out.contains("42"), "expected 42; got: {out}");
    assert!(out.contains("'k': 42"), "expected 'k': 42; got: {out}");
}

#[test]
fn encode_accepts_implies_decode_accepts_at_depth_boundary() {
    // End-to-end depth coverage for the VIEW encode path: a shallow runtime
    // tuple must round-trip. The injected-bytes depth tests
    // (deeply_nested_tuple_payload_within_cap_round_trips /
    // over_deep_nested_tuple_payload_is_rejected) anchor the DECODE boundary;
    // this one proves write_tuple_via_view's own depth accounting produces a
    // decodable row for a query-produced nested tuple.
    let dir = tempfile::tempdir().unwrap();
    let dbp = dir.path().join("depth_e2e.pqlite");
    let (ok, _out, err) = run_exec(
        "CREATE TABLE t AS (SELECT { 'a': { 'b': { 'c': 42 } } } AS r FROM mem(1,1) m)",
        Some(&dbp),
    );
    assert!(
        ok,
        "3-level runtime-tuple CTAS should succeed; stderr: {err}"
    );
    let (ok, out, err) = run_exec("SELECT * FROM t", Some(&dbp));
    assert!(ok, "SELECT over 3-level nested tuple failed: {err}");
    // The innermost value and the full nesting survive the round trip.
    assert!(out.contains("42"), "expected inner value 42; got: {out}");
    assert!(
        out.contains("{ 'r': { 'a': { 'b': { 'c': 42 } } } }"),
        "expected the full 3-level nesting to render; got: {out}"
    );
}

#[test]
fn static_root_deep_runtime_tuple_never_writes_unreadable_row() {
    // Regression: a runtime container nested under a static-struct root
    // (`... AS r`) is charged at a depth no shallower than the decoder (the
    // encoder is the stricter side), so the codec's safety invariant holds — the
    // encoder must NEVER persist a row the decoder then refuses (write-but-can't-
    // read = data loss). Sweep depths straddling MAX_RECURSION_DEPTH (128) and
    // assert the invariant directly: for every depth, we never observe
    // `CTAS ok && SELECT fails`.
    let mut any_round_tripped = false;
    for n in [126usize, 127, 128, 129, 130] {
        let dir = tempfile::tempdir().unwrap();
        let dbp = dir.path().join(format!("depth_{n}.pqlite"));

        // Build an n-deep runtime tuple `{ 'a': { 'a': ... 42 ... } }` under a
        // static-struct root named `r`.
        let mut inner = String::from("42");
        for _ in 0..n {
            inner = format!("{{ 'a': {inner} }}");
        }
        let query = format!("CREATE TABLE t AS (SELECT {inner} AS r FROM mem(1,1) m)");

        let (ctas_ok, _o, ctas_err) = run_exec(&query, Some(&dbp));
        let (select_ok, _so, _se) = if ctas_ok {
            run_exec("SELECT * FROM t", Some(&dbp))
        } else {
            (true, String::new(), String::new()) // no row written, nothing to read
        };

        // The invariant: a persisted row must always be readable back.
        let wrote_unreadable_row = ctas_ok && !select_ok;
        assert!(
            !wrote_unreadable_row,
            "depth {n}: encoder persisted a row the decoder cannot read \
             (write-but-can't-read). CTAS err: {ctas_err}"
        );
        any_round_tripped |= ctas_ok && select_ok;
    }
    // Guard against a vacuous pass: the sweep must include at least one depth
    // the encoder accepts and the decoder reads back, or the invariant above is
    // trivially satisfied by an encoder that rejects everything.
    assert!(
        any_round_tripped,
        "sweep never achieved a write+read round-trip; boundary coverage evaporated"
    );
}

#[test]
fn wire_format_list_byte_shape() {
    // Pin the on-disk byte layout for a single-element int list. Locks the
    // TAG_LIST wire format the way `wire_format_byte_shape_is_stable` locks the
    // tuple/scalar layout. If this fails, the byte literal is the source of
    // truth — investigate the encoder, do not rewrite the test.
    let dir = tempfile::tempdir().unwrap();
    let dbp = dir.path().join("list_anchor.pqlite");

    let (ok, _, stderr) = run_exec(
        "CREATE TABLE t AS (SELECT [1] AS xs FROM mem(1,1) m)",
        Some(&dbp),
    );
    assert!(ok, "CTAS should succeed; stderr: {stderr}");

    let row0 = {
        let db = partiql_tools::storage::HeedDB::open(&dbp).unwrap();
        partiql_tools::test_support::read_row(&db, "t", 0)
    };

    // Expected wire format for the row {"xs": [1i64]}:
    //   TAG_TUPLE (0x08)
    //   field_count = 1 (LE u32: 01 00 00 00)
    //   field 0:
    //     name_len = 2 (LE u32: 02 00 00 00)
    //     name = 'x' 's' (0x78 0x73)
    //     TAG_LIST (0x09)
    //     elem_count = 1 (LE u32: 01 00 00 00)
    //     element 0:
    //       TAG_INTEGER (0x03)
    //       i64 value = 1 (LE: 01 00 00 00 00 00 00 00)
    let expected: &[u8] = &[
        0x08, // TAG_TUPLE
        0x01, 0x00, 0x00, 0x00, // field_count = 1
        0x02, 0x00, 0x00, 0x00, 0x78, 0x73, // name "xs"
        0x09, // TAG_LIST
        0x01, 0x00, 0x00, 0x00, // elem_count = 1
        0x03, // TAG_INTEGER
        0x01, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, // i64 1 LE
    ];

    assert_eq!(
        row0, expected,
        "list wire format byte shape regressed: expected {expected:02x?}, got {row0:02x?}",
    );
}

#[test]
fn over_many_list_elements_rejected() {
    let dir = tempfile::tempdir().unwrap();
    let dbp = dir.path().join("too_many.pqlite");
    let (ok, _o, e) = run_exec(
        "CREATE TABLE t AS (SELECT 1 AS a FROM mem(1,1) m)",
        Some(&dbp),
    );
    assert!(ok, "setup CTAS failed: {e}");

    // Craft a tuple row whose 'xs' list claims one element more than the cap.
    // No element bytes are needed — the cap check trips before the read.
    let mut payload = Vec::new();
    payload.push(partiql_tools::row_codec::TAG_TUPLE);
    payload.extend_from_slice(&1u32.to_le_bytes()); // field_count = 1
    payload.extend_from_slice(&2u32.to_le_bytes()); // name_len = 2
    payload.extend_from_slice(b"xs");
    payload.push(partiql_tools::row_codec::TAG_LIST);
    let bad_count: u32 = partiql_tools::row_codec::MAX_CONTAINER_ELEMENTS + 1;
    payload.extend_from_slice(&bad_count.to_le_bytes());

    let db = partiql_tools::storage::HeedDB::open(&dbp).unwrap();
    partiql_tools::test_support::inject_row(&db, "t", 1, &payload);
    drop(db);

    let (ok, _out, err) = run_exec("SELECT * FROM t", Some(&dbp));
    assert!(!ok, "over-cap element count must be rejected");
    assert!(
        err.contains("element count"),
        "error must mention element count; got: {err}"
    );
}

#[test]
fn oversize_string_payload_surfaces_error() {
    let dir = tempfile::tempdir().unwrap();
    let dbp = dir.path().join("bad_str.pqlite");
    let (ok, _out, err) = run_exec(
        "CREATE TABLE t AS (SELECT 'ok' AS s FROM mem(1,1) m)",
        Some(&dbp),
    );
    assert!(ok, "CTAS setup failed: {err}");
    // Craft a tuple row whose string field claims a payload 1 byte over the cap.
    let mut payload = Vec::new();
    payload.push(partiql_tools::row_codec::TAG_TUPLE);
    payload.extend_from_slice(&1u32.to_le_bytes()); // field_count = 1
    payload.extend_from_slice(&1u32.to_le_bytes()); // name_len = 1
    payload.push(b's');
    payload.push(partiql_tools::row_codec::TAG_STRING);
    let bad_len: u32 = partiql_tools::row_codec::MAX_STRING_LEN + 1;
    payload.extend_from_slice(&bad_len.to_le_bytes());
    // Intentionally NO actual string bytes — the cap check must trip before the read.
    let db = partiql_tools::storage::HeedDB::open(&dbp).unwrap();
    partiql_tools::test_support::inject_row(&db, "t", 1, &payload);
    drop(db);
    let (ok, _out, err) = run_exec("SELECT * FROM t", Some(&dbp));
    assert!(!ok, "expected SELECT to fail on oversize string");
    assert!(err.contains("string length"), "got: {err}");
}

// ---------------------------------------------------------------------------
// Container-root, decimal/float/bytes, empty-bag, deep-mixed-nesting, and
// list/bag-parity coverage. These fill gaps the earlier round-trip tests left:
//   * every prior `SELECT VALUE` test produced a SCALAR root (tag < 0x08);
//     none exercised the `bytes[0] >= TAG_TUPLE` auto-detect on a CONTAINER
//     root (a bare List/Bag row);
//   * no test round-tripped a Decimal, a Float, or a Bytes VALUE;
//   * the empty *list* was covered, the empty *bag* was not;
//   * no single row exercised tuple -> list -> bag -> scalar recursion;
//   * the List byte-anchor existed, the Bag one did not.
//
// Rendering facts pinned from the binary's actual stdout (source of truth):
//   * A SELECT reading a bare container/scalar row (a `RowShape::Register`
//     row on disk) re-wraps it under a synthetic `'_1'` field, e.g. a bare
//     List row renders `{ '_1': [11, 22] }`, a bare Float renders
//     `{ '_1': 2.5 }`.
//   * List renders `[a, b]`; Bag renders `<<a, b>>`; empty Bag renders `<<>>`.
//   * A Bytes value renders as an Ion-style hex blob literal: `x'deadbeef'`.

#[test]
fn ctas_then_select_value_list_root() {
    // A top-level list row (RowShape::Register holding a List) decodes and
    // renders its elements. The bare-on-disk shape is pinned by
    // top_level_list_root_is_bare_list_on_disk.
    let dir = tempfile::tempdir().unwrap();
    let dbp = dir.path().join("list_root.pqlite");
    let (ok, _out, err) = run_exec(
        "CREATE TABLE t AS (SELECT VALUE [m.a * 100 + 11, m.a * 100 + 22] FROM mem(2,1) m)",
        Some(&dbp),
    );
    assert!(ok, "list-root CTAS should succeed; stderr: {err}");

    let (ok, out, err) = run_exec("SELECT * FROM t", Some(&dbp));
    assert!(ok, "SELECT over a list-root table failed: {err}");
    // m.a = 0 -> [11, 22]; m.a = 1 -> [111, 122]. A read-back of a bare row
    // re-wraps it under the synthetic '_1' field, so each row renders
    // `{ '_1': [.., ..] }`.
    assert!(out.contains("[11, 22]"), "expected [11, 22]; got: {out}");
    assert!(
        out.contains("[111, 122]"),
        "expected [111, 122]; got: {out}"
    );
    assert!(
        out.contains("'_1':"),
        "a read-back bare row wraps under '_1'; got: {out}"
    );
}

#[test]
fn top_level_list_root_is_bare_list_on_disk() {
    // Companion byte-anchor to `top_level_scalar_row_is_bare_tag_on_disk`: a
    // top-level CONTAINER value must persist BARE (starting with TAG_LIST,
    // 0x09), NOT tuple-wrapped (0x08). Proves the container-root encode path
    // in `write_value_at_root` does not add a tuple frame.
    let dir = tempfile::tempdir().unwrap();
    let dbp = dir.path().join("list_root_anchor.pqlite");
    let (ok, _out, err) = run_exec(
        "CREATE TABLE t AS (SELECT VALUE [m.a + 55] FROM mem(1,1) m)",
        Some(&dbp),
    );
    assert!(ok, "single-element list-root CTAS failed: {err}");

    let row0 = {
        let db = partiql_tools::storage::HeedDB::open(&dbp).unwrap();
        partiql_tools::test_support::read_row(&db, "t", 0)
    };

    // Expected wire format for the bare row `[55i64]`:
    //   TAG_LIST (0x09)
    //   elem_count = 1 (LE u32: 01 00 00 00)
    //   element 0:
    //     TAG_INTEGER (0x03)
    //     i64 value = 55 (LE: 37 00 00 00 00 00 00 00)
    // Note the FIRST byte is 0x09, not 0x08 — no tuple wrapper.
    let expected: &[u8] = &[
        0x09, // TAG_LIST (container root, bare)
        0x01, 0x00, 0x00, 0x00, // elem_count = 1
        0x03, // TAG_INTEGER
        0x37, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, // i64 55 LE
    ];
    assert_eq!(
        row0, expected,
        "a top-level container must persist bare (TAG_LIST 0x09), not tuple-wrapped: \
         expected {expected:02x?}, got {row0:02x?}",
    );
}

#[test]
fn ctas_then_select_decimal_column() {
    // The literal `3.14` is a DECIMAL in this engine (on-disk tag TAG_DECIMAL
    // 0x05, scale 2, mantissa 314 — confirmed by direct byte inspection), NOT
    // a float. Decimal is exact, so `3.14` must round-trip as exactly `3.14`,
    // with no float drift like `3.1400000001`.
    let dir = tempfile::tempdir().unwrap();
    let dbp = dir.path().join("decimal_col.pqlite");
    let (ok, _out, err) = run_exec(
        "CREATE TABLE t AS (SELECT 3.14 AS d FROM mem(1,1) m)",
        Some(&dbp),
    );
    assert!(ok, "decimal CTAS failed: {err}");

    // Anchor: the value must land in field 'd' as an exact 3.14.
    let row0 = {
        let db = partiql_tools::storage::HeedDB::open(&dbp).unwrap();
        partiql_tools::test_support::read_row(&db, "t", 0)
    };
    // {'d': 3.14dec}: TAG_TUPLE, fc=1, name_len=1, 'd', TAG_DECIMAL, scale=2,
    // mantissa=314 (i128 LE: 3a 01 then 14 zero bytes).
    let expected: &[u8] = &[
        0x08, // TAG_TUPLE
        0x01, 0x00, 0x00, 0x00, // field_count = 1
        0x01, 0x00, 0x00, 0x00, 0x64, // name "d"
        0x05, // TAG_DECIMAL
        0x02, 0x00, 0x00, 0x00, // scale = 2 (i32 LE)
        0x3a, 0x01, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, // mantissa 314 (i128 LE)
        0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
    ];
    assert_eq!(
        row0, expected,
        "decimal 3.14 wire format regressed: expected {expected:02x?}, got {row0:02x?}",
    );

    let (ok, out, err) = run_exec("SELECT * FROM t", Some(&dbp));
    assert!(ok, "SELECT failed: {err}");
    assert!(
        out.contains("'d': 3.14"),
        "decimal must round-trip exactly as 3.14; got: {out}"
    );
    // Exactness guard: no float-style drift.
    assert!(
        !out.contains("3.13") && !out.contains("3.1400"),
        "decimal must be exact, not float-drifted; got: {out}"
    );
}

#[test]
fn ctas_then_select_float_roundtrip() {
    // No SQL literal produces a FLOAT in this engine: `2.5`, `2.5e0`, `25e-1`
    // and `1.5e10` all parse to DECIMAL (on-disk TAG_DECIMAL 0x05 — verified by
    // byte inspection). A genuine FLOAT (TAG_FLOAT 0x04) therefore has to be
    // crafted on disk, then decoded back through SELECT. A bare Float row is a
    // `RowShape::Register` root, so the read-back wraps it under '_1'.
    let dir = tempfile::tempdir().unwrap();
    let dbp = dir.path().join("float_rt.pqlite");
    let (ok, _out, err) = run_exec(
        "CREATE TABLE t AS (SELECT 1 AS a FROM mem(1,1) m)",
        Some(&dbp),
    );
    assert!(ok, "setup CTAS failed: {err}");

    // Bare TAG_FLOAT root: 0x04 + f64(2.5) LE. 2.5 is exactly representable in
    // binary64, so it prints back as `2.5` with no drift.
    let mut payload = vec![partiql_tools::row_codec::TAG_FLOAT];
    payload.extend_from_slice(&2.5f64.to_le_bytes());
    {
        let db = partiql_tools::storage::HeedDB::open(&dbp).unwrap();
        partiql_tools::test_support::inject_row(&db, "t", 1, &payload);
    }

    let (ok, out, err) = run_exec("SELECT * FROM t", Some(&dbp));
    assert!(ok, "SELECT over crafted float failed: {err}");
    assert!(
        out.contains("'_1': 2.5"),
        "crafted float 2.5 must round-trip under '_1'; got: {out}"
    );
    assert!(
        !out.contains("2.50"),
        "float 2.5 must not render with drift/padding; got: {out}"
    );
}

#[test]
fn ctas_then_select_bytes_roundtrip() {
    // Bytes (blob) has no SQL literal reachable through this CLI (CAST is an
    // UnsupportedFunction, and there is no `b'..'` / `{{..}}` literal path
    // that survives lowering here), so a Bytes VALUE is crafted on disk inside
    // a tuple field and decoded back through SELECT. A Bytes value renders as
    // an Ion-style hex blob literal `x'deadbeef'`.
    let dir = tempfile::tempdir().unwrap();
    let dbp = dir.path().join("bytes_rt.pqlite");
    let (ok, _out, err) = run_exec(
        "CREATE TABLE t AS (SELECT 1 AS a FROM mem(1,1) m)",
        Some(&dbp),
    );
    assert!(ok, "setup CTAS failed: {err}");

    // {'b': BYTES 0xDE 0xAD 0xBE 0xEF}
    let raw: [u8; 4] = [0xDE, 0xAD, 0xBE, 0xEF];
    let mut payload = vec![partiql_tools::row_codec::TAG_TUPLE];
    payload.extend_from_slice(&1u32.to_le_bytes()); // field_count = 1
    payload.extend_from_slice(&1u32.to_le_bytes()); // name_len = 1
    payload.push(b'b');
    payload.push(partiql_tools::row_codec::TAG_BYTES);
    payload.extend_from_slice(&(raw.len() as u32).to_le_bytes());
    payload.extend_from_slice(&raw);
    {
        let db = partiql_tools::storage::HeedDB::open(&dbp).unwrap();
        partiql_tools::test_support::inject_row(&db, "t", 1, &payload);
    }

    let (ok, out, err) = run_exec("SELECT * FROM t", Some(&dbp));
    assert!(ok, "SELECT over crafted bytes failed: {err}");
    assert!(
        out.contains("'b': x'deadbeef'"),
        "crafted bytes must render as an Ion hex blob under 'b'; got: {out}"
    );
}

#[test]
fn ctas_then_select_empty_bag() {
    // The empty-*list* case is covered; the empty-*bag* is the gap. An empty
    // container encodes as count=0 with no elements and no step_in, and the
    // empty bag renders `<<>>` inside the row tuple.
    let dir = tempfile::tempdir().unwrap();
    let dbp = dir.path().join("empty_bag.pqlite");
    let (ok, _out, err) = run_exec(
        "CREATE TABLE t AS (SELECT << >> AS xs FROM mem(1,1) m)",
        Some(&dbp),
    );
    assert!(ok, "empty-bag CTAS should succeed; stderr: {err}");
    let (ok, out, err) = run_exec("SELECT * FROM t", Some(&dbp));
    assert!(ok, "SELECT over empty bag failed: {err}");
    assert!(out.contains("'xs': <<>>"), "expected empty bag; got: {out}");
}

#[test]
fn ctas_then_select_three_level_mixed_nesting() {
    // A single row that nests all three container kinds:
    //   tuple 'r' -> tuple 'a' -> list -> bag -> scalar.
    // Exercises tuple/list/bag recursion together on BOTH encode and decode in
    // one row. Renders `{ 'r': { 'a': [<<777, 888>>] } }`.
    let dir = tempfile::tempdir().unwrap();
    let dbp = dir.path().join("mixed3.pqlite");
    let (ok, _out, err) = run_exec(
        "CREATE TABLE t AS (SELECT { 'a': [ << 777, 888 >> ] } AS r FROM mem(1,1) m)",
        Some(&dbp),
    );
    assert!(ok, "three-level-nesting CTAS should succeed; stderr: {err}");
    let (ok, out, err) = run_exec("SELECT * FROM t", Some(&dbp));
    assert!(ok, "SELECT over three-level nesting failed: {err}");
    assert!(out.contains("777"), "expected 777; got: {out}");
    assert!(out.contains("888"), "expected 888; got: {out}");
    // The full structure survives: outer field 'r', inner field 'a', a list of
    // one bag of two ints.
    assert!(out.contains("'r':"), "expected outer field 'r'; got: {out}");
    assert!(out.contains("'a':"), "expected inner field 'a'; got: {out}");
    assert!(
        out.contains("[<<777, 888>>]"),
        "expected list-of-bag structure; got: {out}"
    );
}

#[test]
fn wire_format_bag_byte_shape() {
    // Companion to `wire_format_list_byte_shape`: pin the single-element Bag
    // wire format. The bytes must be identical to the LIST anchor EXCEPT the
    // container tag byte: TAG_BAG (0x0A) where the list has TAG_LIST (0x09).
    let dir = tempfile::tempdir().unwrap();
    let dbp = dir.path().join("bag_anchor.pqlite");
    let (ok, _out, err) = run_exec(
        "CREATE TABLE t AS (SELECT << 1 >> AS xs FROM mem(1,1) m)",
        Some(&dbp),
    );
    assert!(ok, "single-element bag CTAS failed: {err}");

    let row0 = {
        let db = partiql_tools::storage::HeedDB::open(&dbp).unwrap();
        partiql_tools::test_support::read_row(&db, "t", 0)
    };

    // Expected wire format for the row {"xs": <<1i64>>}:
    //   TAG_TUPLE (0x08)
    //   field_count = 1 (LE u32: 01 00 00 00)
    //   field 0:
    //     name_len = 2 (LE u32: 02 00 00 00)
    //     name = 'x' 's' (0x78 0x73)
    //     TAG_BAG (0x0A)          <-- the ONLY difference from the list anchor
    //     elem_count = 1 (LE u32: 01 00 00 00)
    //     element 0:
    //       TAG_INTEGER (0x03)
    //       i64 value = 1 (LE: 01 00 00 00 00 00 00 00)
    let expected: &[u8] = &[
        0x08, // TAG_TUPLE
        0x01, 0x00, 0x00, 0x00, // field_count = 1
        0x02, 0x00, 0x00, 0x00, 0x78, 0x73, // name "xs"
        0x0A, // TAG_BAG (vs 0x09 TAG_LIST for the identical list anchor)
        0x01, 0x00, 0x00, 0x00, // elem_count = 1
        0x03, // TAG_INTEGER
        0x01, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, // i64 1 LE
    ];
    assert_eq!(
        row0, expected,
        "bag wire format byte shape regressed: expected {expected:02x?}, got {row0:02x?}",
    );
    // Cross-check the list/bag-differ-only-by-tag invariant explicitly: the
    // bag anchor equals the list anchor with byte 11 flipped 0x09 -> 0x0A.
    let mut as_list = expected.to_vec();
    assert_eq!(
        as_list[11],
        partiql_tools::row_codec::TAG_BAG,
        "byte 11 is the container tag"
    );
    as_list[11] = partiql_tools::row_codec::TAG_LIST;
    let list_anchor: &[u8] = &[
        0x08, 0x01, 0x00, 0x00, 0x00, 0x02, 0x00, 0x00, 0x00, 0x78, 0x73, 0x09, 0x01, 0x00, 0x00,
        0x00, 0x03, 0x01, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
    ];
    assert_eq!(
        as_list, list_anchor,
        "bag and list wire formats must differ ONLY at the container tag byte"
    );
}

#[test]
fn list_and_bag_decode_same_elements_modulo_tag() {
    // Inject two tuple rows whose bytes are byte-identical except the container
    // tag (TAG_LIST vs TAG_BAG). Both must decode and surface the same
    // elements (7 and 8), proving the decoder treats the two tags as the same
    // shape modulo semantics — list renders `[7, 8]`, bag renders `<<7, 8>>`.
    let dir = tempfile::tempdir().unwrap();
    let dbp = dir.path().join("list_bag_parity.pqlite");
    let (ok, _out, err) = run_exec(
        "CREATE TABLE t AS (SELECT 1 AS a FROM mem(1,1) m)",
        Some(&dbp),
    );
    assert!(ok, "setup CTAS failed: {err}");

    // Build a {"xs": <container>[7, 8]} tuple parametrised by the container tag.
    fn tuple_container(tag: u8) -> Vec<u8> {
        let mut p = vec![partiql_tools::row_codec::TAG_TUPLE];
        p.extend_from_slice(&1u32.to_le_bytes()); // field_count = 1
        p.extend_from_slice(&2u32.to_le_bytes()); // name_len = 2
        p.extend_from_slice(b"xs");
        p.push(tag);
        p.extend_from_slice(&2u32.to_le_bytes()); // elem_count = 2
        for v in [7i64, 8i64] {
            p.push(partiql_tools::row_codec::TAG_INTEGER);
            p.extend_from_slice(&v.to_le_bytes());
        }
        p
    }
    let list_bytes = tuple_container(partiql_tools::row_codec::TAG_LIST);
    let bag_bytes = tuple_container(partiql_tools::row_codec::TAG_BAG);
    // The two payloads differ at exactly one byte: the container tag at index 11.
    assert_eq!(list_bytes.len(), bag_bytes.len());
    let diffs: Vec<usize> = (0..list_bytes.len())
        .filter(|&i| list_bytes[i] != bag_bytes[i])
        .collect();
    assert_eq!(
        diffs,
        vec![11],
        "list and bag payloads must differ only at the container tag byte"
    );

    {
        let db = partiql_tools::storage::HeedDB::open(&dbp).unwrap();
        partiql_tools::test_support::inject_row(&db, "t", 1, &list_bytes);
        partiql_tools::test_support::inject_row(&db, "t", 2, &bag_bytes);
    }

    let (ok, out, err) = run_exec("SELECT * FROM t", Some(&dbp));
    assert!(ok, "SELECT over injected list+bag rows failed: {err}");
    assert!(
        out.contains("'xs': [7, 8]"),
        "expected list row; got: {out}"
    );
    assert!(
        out.contains("'xs': <<7, 8>>"),
        "expected bag row; got: {out}"
    );
}

#[test]
fn ctas_then_select_null_inside_list_distinct_from_missing() {
    // NULL as a container element (distinct from the existing bare-scalar-root
    // NULL/MISSING test `ctas_then_select_value_null_distinct_from_missing`):
    // exercises the null tag inside list recursion. Confirms NULL does not
    // render as MISSING. Renders `{ 'xs': [NULL, 42] }`.
    let dir = tempfile::tempdir().unwrap();
    let dbp = dir.path().join("null_in_list.pqlite");
    let (ok, _out, err) = run_exec(
        "CREATE TABLE t AS (SELECT [NULL, 42] AS xs FROM mem(1,1) m)",
        Some(&dbp),
    );
    assert!(ok, "null-in-list CTAS should succeed; stderr: {err}");
    let (ok, out, err) = run_exec("SELECT * FROM t", Some(&dbp));
    assert!(ok, "SELECT over null-in-list failed: {err}");
    assert!(
        out.contains("[NULL, 42]"),
        "expected [NULL, 42]; got: {out}"
    );
    assert!(
        !out.contains("MISSING"),
        "an explicit NULL element must not render as MISSING; got: {out}"
    );
}

#[test]
fn ctas_from_stored_table_roundtrips() {
    // Disk-to-disk CTAS: the source read txn must close before the write txn opens
    // (else MDB_BAD_DBI).
    let dir = tempfile::tempdir().unwrap();
    let dbp = dir.path().join("d2d.pqlite");

    let (ok, _, err) = run_exec(
        "CREATE TABLE src AS (SELECT m.a FROM mem(3,1) m)",
        Some(&dbp),
    );
    assert!(ok, "seed CTAS should succeed; stderr: {err}");

    let (ok, _, err) = run_exec("CREATE TABLE dest AS (SELECT src.a FROM src)", Some(&dbp));
    assert!(ok, "CTAS from a stored table should succeed; stderr: {err}");

    let (ok, out, err) = run_exec("SELECT * FROM dest", Some(&dbp));
    assert!(ok, "SELECT from dest should succeed; stderr: {err}");
    assert!(out.contains("'a': 0"), "got: {out}");
    assert!(out.contains("'a': 1"), "got: {out}");
    assert!(out.contains("'a': 2"), "got: {out}");
    assert!(
        err.contains("(3 rows "),
        "dest should have 3 rows; stderr: {err}"
    );
}

// End-to-end `INSERT INTO t SELECT ...` tests. INSERT appends onto an existing
// table (CTAS creates a fresh one), so these seed a table via CTAS first, then
// append and re-read. INSERT source is BARE (no wrapping parens), unlike CTAS.

#[test]
fn insert_select_roundtrip_cases() {
    // (setup DDL, insert stmt, values expected present after SELECT, total row count)
    let cases: &[(&str, &str, &[&str], u32)] = &[
        // Append 3 rows onto a seeded 2-row table -> 5 total.
        (
            "CREATE TABLE t AS (SELECT m.a FROM mem(2,1) m)",
            "INSERT INTO t SELECT m.a FROM mem(3,1) m",
            &["0", "1", "2"],
            5,
        ),
        // Append onto a seeded 1-row table -> 3 total.
        (
            "CREATE TABLE t AS (SELECT m.a FROM mem(1,1) m)",
            "INSERT INTO t SELECT m.a FROM mem(2,1) m",
            &["0", "1"],
            3,
        ),
    ];
    for (i, (setup, insert, expected, total)) in cases.iter().enumerate() {
        let dir = tempfile::tempdir().unwrap();
        let dbp = dir.path().join(format!("ins_{i}.pqlite"));
        let (ok, _o, err) = run_exec(setup, Some(&dbp));
        assert!(ok, "case {i} setup failed: {err}");
        let (ok, _o, err) = run_exec(insert, Some(&dbp));
        assert!(ok, "case {i} INSERT failed: {err}");
        assert!(
            err.contains("Inserted"),
            "case {i} missing confirmation: {err}"
        );
        let (ok, out, err) = run_exec("SELECT * FROM t", Some(&dbp));
        assert!(ok, "case {i} SELECT failed: {err}");
        for v in *expected {
            assert!(
                out.contains(&format!("'a': {v}")),
                "case {i} missing {v}: {out}"
            );
        }
        assert!(
            err.contains(&format!("({total} rows ")),
            "case {i} count: {err}"
        );
    }
}

#[test]
fn insert_into_missing_table_errors() {
    let dir = tempfile::tempdir().unwrap();
    let dbp = dir.path().join("m.pqlite");
    // Create an unrelated table so the env/file exists.
    run_exec(
        "CREATE TABLE other AS (SELECT m.a FROM mem(1,1) m)",
        Some(&dbp),
    );
    let (ok, _o, err) = run_exec("INSERT INTO ghost SELECT m.a FROM mem(1,1) m", Some(&dbp));
    assert!(!ok, "INSERT into missing table must fail");
    assert!(err.contains("table not found: ghost"), "got: {err}");
}

#[test]
fn insert_preserves_existing_rows() {
    let dir = tempfile::tempdir().unwrap();
    let dbp = dir.path().join("p.pqlite");
    run_exec("CREATE TABLE t AS (SELECT m.a FROM mem(3,1) m)", Some(&dbp)); // 0,1,2
    let (ok, _o, err) = run_exec("INSERT INTO t SELECT m.a FROM mem(2,1) m", Some(&dbp)); // +0,1
    assert!(ok, "{err}");
    let (ok, out, _e) = run_exec("SELECT * FROM t", Some(&dbp));
    assert!(ok);
    // a=2 only exists in the seeded batch; it must survive the append.
    assert!(
        out.contains("'a': 2"),
        "seeded rows must survive append: {out}"
    );
}

#[test]
fn insert_row_ids_continue_after_max() {
    let dir = tempfile::tempdir().unwrap();
    let dbp = dir.path().join("k.pqlite");
    run_exec("CREATE TABLE t AS (SELECT m.a FROM mem(3,1) m)", Some(&dbp)); // keys 0,1,2
    run_exec("INSERT INTO t SELECT m.a FROM mem(2,1) m", Some(&dbp)); // keys 3,4
    let db = partiql_tools::storage::HeedDB::open(&dbp).unwrap();
    // read_row panics if a key is absent, so reading 3,4 asserts they exist.
    // Appended rows come from mem(2,1) = a:0,1 — byte-identical to the seed's
    // first two rows (mem(3,1) = a:0,1,2), so keys 3,4 must equal keys 0,1.
    let (k0, k1) = (
        partiql_tools::test_support::read_row(&db, "t", 0),
        partiql_tools::test_support::read_row(&db, "t", 1),
    );
    assert_eq!(
        partiql_tools::test_support::read_row(&db, "t", 3),
        k0,
        "appended key 3 must equal the seed's a:0 row"
    );
    assert_eq!(
        partiql_tools::test_support::read_row(&db, "t", 4),
        k1,
        "appended key 4 must equal the seed's a:1 row"
    );
}

#[test]
fn insert_without_db_errors() {
    let (ok, _o, err) = run_exec("INSERT INTO t SELECT m.a FROM mem(1,1) m", None);
    assert!(!ok, "INSERT without --db must fail");
    assert!(err.contains("--db"), "got: {err}");
}

#[test]
fn insert_empty_select_is_noop() {
    let dir = tempfile::tempdir().unwrap();
    let dbp = dir.path().join("e.pqlite");
    run_exec("CREATE TABLE t AS (SELECT m.a FROM mem(2,1) m)", Some(&dbp));
    // WHERE FALSE yields zero source rows (house idiom for an empty result).
    let (ok, _o, err) = run_exec(
        "INSERT INTO t SELECT m.a FROM mem(1,1) m WHERE FALSE",
        Some(&dbp),
    );
    assert!(ok, "empty INSERT should succeed: {err}");
    assert!(err.contains("Inserted 0 rows"), "got: {err}");
    // The pre-existing rows are untouched.
    let (ok, out, _e) = run_exec("SELECT * FROM t", Some(&dbp));
    assert!(ok);
    assert!(
        out.contains("'a': 0") && out.contains("'a': 1"),
        "got: {out}"
    );
}

#[test]
fn insert_from_stored_table() {
    // Disk-to-disk INSERT: source is a stored table (incl. the destination
    // itself), so the source read txn must close before the append write txn
    // opens (else MDB_BAD_DBI).
    let dir = tempfile::tempdir().unwrap();
    let dbp = dir.path().join("d2d_ins.pqlite");
    run_exec(
        "CREATE TABLE src AS (SELECT m.a FROM mem(3,1) m)",
        Some(&dbp),
    );
    run_exec("CREATE TABLE t AS (SELECT m.a FROM mem(2,1) m)", Some(&dbp));

    // Append from another stored table.
    let (ok, _o, err) = run_exec("INSERT INTO t SELECT src.a FROM src", Some(&dbp));
    assert!(
        ok,
        "INSERT from a stored table should succeed; stderr: {err}"
    );
    assert!(err.contains("Inserted 3 rows"), "got: {err}");

    // Self-insert: read and write the same table in one statement.
    let (ok, _o, err) = run_exec("INSERT INTO t SELECT t.a FROM t", Some(&dbp));
    assert!(ok, "self-insert should succeed; stderr: {err}");
    assert!(err.contains("Inserted 5 rows"), "got: {err}");

    let (ok, _out, err) = run_exec("SELECT * FROM t", Some(&dbp));
    assert!(ok);
    assert!(
        err.contains("(10 rows "),
        "expected 10 rows total; stderr: {err}"
    );
}

#[test]
fn insert_into_quoted_case_sensitive_target() {
    // Parity with CTAS's quoted-name roundtrip: a case-sensitive quoted target
    // must resolve to the same physical table for CREATE, INSERT, and SELECT.
    let dir = tempfile::tempdir().unwrap();
    let dbp = dir.path().join("quoted_ins.pqlite");

    let (ok, _o, err) = run_exec(
        r#"CREATE TABLE "MyTable" AS (SELECT m.a FROM mem(2,1) m)"#,
        Some(&dbp),
    );
    assert!(ok, "quoted CTAS should succeed; stderr: {err}");

    let (ok, _o, err) = run_exec(
        r#"INSERT INTO "MyTable" SELECT m.a FROM mem(3,1) m"#,
        Some(&dbp),
    );
    assert!(
        ok,
        "INSERT into quoted target should succeed; stderr: {err}"
    );
    assert!(err.contains("Inserted 3 rows"), "got: {err}");

    let (ok, _out, err) = run_exec(r#"SELECT * FROM "MyTable""#, Some(&dbp));
    assert!(
        ok,
        "SELECT from quoted target should succeed; stderr: {err}"
    );
    assert!(
        err.contains("(5 rows "),
        "expected 5 rows total; stderr: {err}"
    );
}

// ---------------------------------------------------------------------------
// End-to-end bootstrap + system-table (`_tables`) acceptance suite. These drive
// the built binary against a fresh `--db` to verify the startup bootstrap:
// the `_tables` catalog is created on first run, a schema version is stamped,
// user tables register in the catalog, and the system table is read-only.

#[test]
fn select_star_from_tables_shows_self_and_user_tables() {
    let dir = tempfile::tempdir().unwrap();
    let db = dir.path().join("intro.pqlite");
    let (ok, _, err) = run_exec(
        "CREATE TABLE users AS (SELECT t.a FROM mem(1,1) t)",
        Some(&db),
    );
    assert!(ok, "stderr: {err}");
    let (ok2, out, err2) = run_exec("SELECT * FROM _tables", Some(&db));
    assert!(ok2, "stderr: {err2}");
    assert!(
        out.contains("_tables") && out.contains("users"),
        "got: {out}"
    );
}

#[test]
fn bare_create_table_registers_in_catalog() {
    let dir = tempfile::tempdir().unwrap();
    let db = dir.path().join("bare.pqlite");
    let (ok, _, err) = run_exec("CREATE TABLE widgets", Some(&db));
    assert!(ok, "stderr: {err}");
    let (ok2, out, _) = run_exec("SELECT * FROM _tables", Some(&db));
    assert!(
        ok2 && out.contains("widgets"),
        "bare CREATE must register: {out}"
    );
}

#[test]
fn fresh_bootstrap_stamps_version_one() {
    let dir = tempfile::tempdir().unwrap();
    let db_path = dir.path().join("stamp.pqlite");
    assert!(run_exec("SELECT t.a FROM mem(1,1) t", Some(&db_path)).0);
    let db = partiql_tools::storage::HeedDB::open(&db_path).unwrap();
    assert_eq!(db.read_schema_version().unwrap(), 1);
    assert!(db.tables_has_self_entry().unwrap());
}

#[test]
fn second_startup_skips_bootstrap_and_keeps_version() {
    let dir = tempfile::tempdir().unwrap();
    let db_path = dir.path().join("second.pqlite");
    assert!(run_exec("SELECT t.a FROM mem(1,1) t", Some(&db_path)).0);
    let (ok2, out2, err2) = run_exec("SELECT t.a FROM mem(1,1) t", Some(&db_path));
    assert!(ok2, "stderr: {err2}");
    assert!(
        !out2.contains("_tables") && !err2.contains("Created table _tables"),
        "no re-bootstrap"
    );
    let db = partiql_tools::storage::HeedDB::open(&db_path).unwrap();
    assert_eq!(db.read_schema_version().unwrap(), 1);
}

#[test]
fn insert_into_tables_is_rejected() {
    let dir = tempfile::tempdir().unwrap();
    let db = dir.path().join("ins.pqlite");
    let (ok, _, err) = run_exec("INSERT INTO _tables SELECT t.a FROM mem(1,1) t", Some(&db));
    assert!(!ok && err.contains("system table"), "got: {err}");
}

#[test]
fn create_table_as_system_table_is_rejected_after_bootstrap_catalog_intact() {
    // After bootstrap _tables holds its self-entry, so a user CREATE TABLE _tables
    // fails cleanly with a duplicate error — and the catalog stays queryable.
    let dir = tempfile::tempdir().unwrap();
    let db = dir.path().join("dupsys.pqlite");
    let (ok0, _, err0) = run_exec(
        "CREATE TABLE users AS (SELECT t.a FROM mem(1,1) t)",
        Some(&db),
    );
    assert!(ok0, "stderr: {err0}");
    let (ok, _, err) = run_exec(
        "CREATE TABLE _tables AS (SELECT t.a FROM mem(1,1) t)",
        Some(&db),
    );
    assert!(
        !ok && err.contains("already exists"),
        "CREATE TABLE _tables after bootstrap must fail with a duplicate error; err: {err}"
    );
    // Catalog intact: still queryable, still shows the real tables.
    let (ok2, out, err2) = run_exec("SELECT * FROM _tables", Some(&db));
    assert!(
        ok2,
        "catalog must survive the rejected create; stderr: {err2}"
    );
    assert!(
        out.contains("_tables") && out.contains("users"),
        "got: {out}"
    );
}

#[test]
fn create_table_named_schema_version_still_works() {
    let dir = tempfile::tempdir().unwrap();
    let db = dir.path().join("sv.pqlite");
    let (ok, _, err) = run_exec(
        "CREATE TABLE \"schema_version\" AS (SELECT t.a FROM mem(1,1) t)",
        Some(&db),
    );
    assert!(ok, "stderr: {err}");
    let (ok2, out, _) = run_exec("SELECT * FROM _tables", Some(&db));
    assert!(ok2 && out.contains("schema_version"), "got: {out}");
}

#[test]
fn underscore_prefixed_user_table_is_allowed() {
    let dir = tempfile::tempdir().unwrap();
    let db = dir.path().join("under.pqlite");
    assert!(
        run_exec(
            "CREATE TABLE _scratch AS (SELECT t.a FROM mem(1,1) t)",
            Some(&db)
        )
        .0
    );
    let (ok2, out, _) = run_exec("SELECT * FROM _tables", Some(&db));
    assert!(ok2 && out.contains("_scratch"), "got: {out}");
}

#[test]
fn newer_schema_version_is_rejected() {
    let dir = tempfile::tempdir().unwrap();
    let db_path = dir.path().join("newer.pqlite");
    {
        let db = partiql_tools::storage::HeedDB::open(&db_path).unwrap();
        db.set_schema_version(2).unwrap();
    }
    let (ok, _, err) = run_exec("SELECT t.a FROM mem(1,1) t", Some(&db_path));
    assert!(!ok && err.contains("newer than this binary"), "got: {err}");
}

#[test]
fn version_one_without_self_entry_is_rejected() {
    let dir = tempfile::tempdir().unwrap();
    let db_path = dir.path().join("inv.pqlite");
    {
        let db = partiql_tools::storage::HeedDB::open(&db_path).unwrap();
        db.set_schema_version(1).unwrap();
    }
    let (ok, _, err) = run_exec("SELECT t.a FROM mem(1,1) t", Some(&db_path));
    assert!(!ok && err.contains("missing or incomplete"), "got: {err}");
}

#[test]
fn bootstrap_reconciles_when_self_entry_present_but_version_zero() {
    // Crash-after-create, before-stamp: self-entry exists, version still 0.
    // Bootstrap must run CREATE (hitting TableExists) and reconcile, NOT skip.
    let dir = tempfile::tempdir().unwrap();
    let db_path = dir.path().join("recover.pqlite");
    {
        let db = partiql_tools::storage::HeedDB::open(&db_path).unwrap();
        let mut v = Vec::new();
        partiql_tools::row_codec::serialize_name_row(&["_tables"], &mut v).unwrap();
        db.create_table("_tables", &v).unwrap().commit().unwrap(); // version still 0
    }
    let (ok, _, err) = run_exec("SELECT * FROM _tables", Some(&db_path));
    assert!(ok, "recovery must succeed; stderr: {err}");
    let db = partiql_tools::storage::HeedDB::open(&db_path).unwrap();
    assert_eq!(
        db.read_schema_version().unwrap(),
        1,
        "recovery stamps version 1"
    );
}
