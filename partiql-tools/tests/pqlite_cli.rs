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
    // through the encode → LMDB → decode → format pipeline. Nested tuples
    // (i.e. a column whose VALUE is itself a tuple) are not yet supported
    // by the encoder — the inline `{ 'k': v }` literal lowers to
    // ValueType::Tuple, which `write_value` rejects — so the nested case
    // is intentionally out of scope here. See PR5-deferred-items.
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
fn ctas_rejects_container_rolls_back_wtxn() {
    // Repoint target: migrate this to a cap-violation trigger (oversize
    // string/bytes or too-many-elements) once List/Bag are supported, since the
    // container triggers used here become valid then.
    //
    // The encoder must reject a still-unsupported container column mid-stream.
    // Err propagates so the wtxn rolls back; the env file may persist but the
    // catalog stays clean.
    let dir = tempfile::tempdir().unwrap();
    let db_path = dir.path().join("list.pqlite");
    let (ok, _stdout, stderr) = run_exec(
        "CREATE TABLE t AS (SELECT [1, 2] AS xs FROM mem(1,1) m)",
        Some(&db_path),
    );
    assert!(!ok, "List column must be rejected; stderr: {stderr}");
    assert!(
        stderr.contains("List") && stderr.contains("not yet supported"),
        "error must name the rejected container type; got: {stderr}",
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
