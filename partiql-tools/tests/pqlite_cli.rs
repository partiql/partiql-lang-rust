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

// Wire-format parser (hand-written, mirrors row_codec spec).

use partiql_tools::row_codec::{
    FORMAT_VERSION, MAX_FIELDS_PER_ROW, MAX_NAME_LEN_BYTES, TAG_DECIMAL, TAG_FLOAT, TAG_INTEGER,
    TAG_NULL, TAG_STRING, TAG_STRUCT,
};

#[derive(Debug, PartialEq)]
enum ParsedValue {
    Integer(i64),
    Decimal { scale: i32, mantissa: i128 },
    Float(f64),
    String(String),
    Null,
}

#[derive(Debug)]
struct ParsedRow {
    payload: ParsedPayload,
}

#[derive(Debug)]
enum ParsedPayload {
    /// Top-level TAG_STRUCT: a struct of named fields.
    Struct(Vec<(String, ParsedValue)>),
    /// Top-level scalar tag (Integer/Decimal/Float/String/Null).
    Scalar(ParsedValue),
}

/// Mirror of the wire-format spec. Returns `ParsedRow` or panics with a
/// pointed message; this is a test helper, not a production reader.
///
/// Imports `FORMAT_VERSION`, `TAG_*`, `MAX_FIELDS_PER_ROW`, and
/// `MAX_NAME_LEN_BYTES` from `partiql_tools::row_codec` so a tag-byte or
/// cap change in the encoder cannot drift here.
///
/// ENDIANNESS INVARIANT: every multi-byte field is little-endian, matching
/// `row_codec::serialize_row`'s `to_le_bytes()` calls. A new tag must be
/// written `to_le_bytes()` there AND read `from_le_bytes()` here — an
/// asymmetry compiles cleanly and silently corrupts every persisted row.
fn parse_row(bytes: &[u8]) -> ParsedRow {
    fn take<'a>(i: &mut usize, n: usize, bytes: &'a [u8]) -> &'a [u8] {
        assert!(
            *i + n <= bytes.len(),
            "ran off the end at offset {i} taking {n}"
        );
        let s = &bytes[*i..*i + n];
        *i += n;
        s
    }
    fn take_u32(i: &mut usize, bytes: &[u8]) -> u32 {
        u32::from_le_bytes(take(i, 4, bytes).try_into().unwrap())
    }
    fn take_i32(i: &mut usize, bytes: &[u8]) -> i32 {
        i32::from_le_bytes(take(i, 4, bytes).try_into().unwrap())
    }
    fn take_i64(i: &mut usize, bytes: &[u8]) -> i64 {
        i64::from_le_bytes(take(i, 8, bytes).try_into().unwrap())
    }
    fn take_f64(i: &mut usize, bytes: &[u8]) -> f64 {
        f64::from_le_bytes(take(i, 8, bytes).try_into().unwrap())
    }
    fn take_i128(i: &mut usize, bytes: &[u8]) -> i128 {
        i128::from_le_bytes(take(i, 16, bytes).try_into().unwrap())
    }
    fn take_value(i: &mut usize, bytes: &[u8], tag: u8) -> ParsedValue {
        match tag {
            t if t == TAG_INTEGER => ParsedValue::Integer(take_i64(i, bytes)),
            t if t == TAG_DECIMAL => {
                let scale = take_i32(i, bytes);
                let mantissa = take_i128(i, bytes);
                ParsedValue::Decimal { scale, mantissa }
            }
            t if t == TAG_FLOAT => ParsedValue::Float(take_f64(i, bytes)),
            t if t == TAG_STRING => {
                let len = take_u32(i, bytes) as usize;
                ParsedValue::String(
                    std::str::from_utf8(take(i, len, bytes))
                        .expect("string must be valid UTF-8")
                        .to_string(),
                )
            }
            t if t == TAG_NULL => ParsedValue::Null,
            other => panic!("unknown tag {other:#04x} at offset {}", *i - 1),
        }
    }

    let mut i = 0usize;
    assert_eq!(
        take(&mut i, 1, bytes)[0],
        FORMAT_VERSION,
        "format version must be FORMAT_VERSION ({FORMAT_VERSION:#04x})"
    );
    let top_tag = take(&mut i, 1, bytes)[0];
    let payload = if top_tag == TAG_STRUCT {
        let field_count = take_u32(&mut i, bytes);
        assert!(
            field_count <= MAX_FIELDS_PER_ROW,
            "sanity: field_count={field_count} exceeds MAX_FIELDS_PER_ROW ({MAX_FIELDS_PER_ROW})"
        );
        let mut fields = Vec::with_capacity(field_count as usize);
        for _ in 0..field_count {
            let name_len = take_u32(&mut i, bytes);
            assert!(
                name_len <= MAX_NAME_LEN_BYTES,
                "sanity: name_len={name_len} exceeds MAX_NAME_LEN_BYTES ({MAX_NAME_LEN_BYTES})"
            );
            let name = std::str::from_utf8(take(&mut i, name_len as usize, bytes))
                .expect("field name must be valid UTF-8")
                .to_string();
            let tag = take(&mut i, 1, bytes)[0];
            fields.push((name, take_value(&mut i, bytes, tag)));
        }
        ParsedPayload::Struct(fields)
    } else {
        ParsedPayload::Scalar(take_value(&mut i, bytes, top_tag))
    };
    assert_eq!(
        i,
        bytes.len(),
        "parser left {} trailing bytes",
        bytes.len() - i
    );
    ParsedRow { payload }
}

/// Read every row in table `t` from the .pqlite file at `db_path`, returning
/// ParsedRow values in row_id order.
fn read_all_rows(db_path: &std::path::Path, table: &str) -> Vec<ParsedRow> {
    let env = unsafe {
        heed::EnvOpenOptions::new()
            .map_size(1024 * 1024 * 1024)
            .max_dbs(128)
            .flags(heed::EnvFlags::NO_SUB_DIR)
            .open(db_path)
            .expect("re-open env")
    };
    let rtxn = env.read_txn().expect("read txn");
    // Key codec matches storage: BE u64 bytes under `heed::types::Bytes`
    // (storage switched from `U64<BigEndian>` to `Bytes` to drop the per-put
    // Vec allocation; BE byte order preserves numeric key ordering).
    let rowdb: heed::Database<heed::types::Bytes, heed::types::Bytes> = env
        .open_database(&rtxn, Some(table))
        .expect("open row db")
        .expect("row db must exist");
    let mut out = Vec::new();
    for entry in rowdb.iter(&rtxn).expect("iter rows") {
        let (_row_id, payload) = entry.expect("row entry");
        out.push(parse_row(payload));
    }
    out
}

#[test]
#[cfg_attr(windows, ignore)]
fn ctas_persists_tagged_union_rows() {
    // mem(3,2) yields i64 rows with column `a` taking 0, 1, 2.
    let dir = tempfile::tempdir().unwrap();
    let db_path = dir.path().join("rt.pqlite");
    let (ok, _stdout, stderr) = run_exec(
        "CREATE TABLE t AS (SELECT t.a FROM mem(3,2) t)",
        Some(&db_path),
    );
    assert!(ok, "CTAS should succeed; stderr: {stderr}");
    assert!(db_path.is_file());

    let rows = read_all_rows(&db_path, "t");
    assert_eq!(rows.len(), 3, "expected 3 rows; got: {rows:?}");
    for (idx, row) in rows.iter().enumerate() {
        let fields = match &row.payload {
            ParsedPayload::Struct(f) => f,
            other => panic!("row {idx}: expected Struct, got {other:?}"),
        };
        assert_eq!(fields.len(), 1, "row {idx}: expected 1 field");
        let (name, value) = &fields[0];
        assert_eq!(name, "a", "row {idx}: field name should be `a`");
        match value {
            ParsedValue::Integer(v) => {
                assert_eq!(*v as usize, idx, "row {idx}: value should equal row index");
            }
            other => panic!("row {idx}: expected Integer, got {other:?}"),
        }
    }
}

#[test]
#[cfg_attr(windows, ignore)]
fn ctas_oversize_row_round_trips_through_lmdb_overflow_pages() {
    // Guard the two paths that mem(3,2)'s small rows don't exercise:
    //   1) the reusable Vec<u8> in pqlite's CTAS arm grows past its 4 KiB
    //      initial capacity (Vec doubles its allocation)
    //   2) LMDB routes the value onto overflow pages (heed wraps mdb_put
    //      which handles this transparently)
    //
    // We synthesize the oversize row via a 16 KiB string LITERAL embedded in
    // the SQL projection, joined with mem(1,2) to satisfy the planner's
    // FROM-clause requirement. This avoids adding a test-only table
    // function to the production binary — the literal is materialized once
    // by the parser and lives only for the duration of this CTAS.
    const PAYLOAD_SIZE: usize = 16 * 1024;
    let big_string: String = "x".repeat(PAYLOAD_SIZE);
    let query = format!(
        "CREATE TABLE t AS (SELECT '{}' AS s FROM mem(1,2) m)",
        big_string
    );

    let dir = tempfile::tempdir().unwrap();
    let db_path = dir.path().join("oversize.pqlite");
    let (ok, _stdout, stderr) = run_exec(&query, Some(&db_path));
    assert!(ok, "CTAS should succeed; stderr: {stderr}");

    let rows = read_all_rows(&db_path, "t");
    assert_eq!(rows.len(), 1, "expected exactly 1 row; got: {}", rows.len());
    let fields = match &rows[0].payload {
        ParsedPayload::Struct(f) => f,
        other => panic!("expected Struct payload, got {other:?}"),
    };
    assert_eq!(fields.len(), 1, "row should have exactly one field");
    let (name, value) = &fields[0];
    assert_eq!(name, "s", "field name should be `s`");
    match value {
        ParsedValue::String(s) => {
            assert_eq!(
                s.len(),
                PAYLOAD_SIZE,
                "string must be exactly {PAYLOAD_SIZE} bytes; got len={}",
                s.len()
            );
            assert!(
                s.chars().all(|c| c == 'x'),
                "every char should be 'x' (corruption check)"
            );
        }
        other => panic!("expected String, got {other:?}"),
    }
}

#[test]
fn parser_adversarial_truncation_walk() {
    // Hand-crafted valid 1-column row mirroring row_codec's wire format:
    //   version(0x01), struct_tag(0x03),
    //   field_count(1 u32 LE), name_len(1 u32 LE), name "a",
    //   tag 0x00 (Integer), value 42 (i64 LE). 20 bytes total.
    let valid_bytes: [u8; 20] = [
        FORMAT_VERSION,
        TAG_STRUCT,
        0x01,
        0x00,
        0x00,
        0x00, // field_count = 1
        0x01,
        0x00,
        0x00,
        0x00, // name_len = 1
        b'a', // name
        TAG_INTEGER,
        0x2a,
        0x00,
        0x00,
        0x00,
        0x00,
        0x00,
        0x00,
        0x00, // i64 LE = 42
    ];

    // Sanity: the full payload parses cleanly. If this fails, the loop below
    // is testing the wrong baseline.
    let row = parse_row(&valid_bytes);
    let fields = match &row.payload {
        ParsedPayload::Struct(f) => f,
        other => panic!("expected Struct payload, got {other:?}"),
    };
    assert_eq!(fields.len(), 1);
    assert_eq!(fields[0].0, "a");
    assert_eq!(fields[0].1, ParsedValue::Integer(42));

    // Truncate at every offset 0..20. Every truncation MUST panic — never
    // return a half-parsed ParsedRow, never read OOB. catch_unwind verifies
    // the parser exits via a panic, not via a normal return.
    for len in 0..valid_bytes.len() {
        let truncated = &valid_bytes[..len];
        let result = std::panic::catch_unwind(|| parse_row(truncated));
        assert!(
            result.is_err(),
            "parser returned without panicking on truncation len={len}: must never return half-parsed bytes",
        );
    }
}

/// Extract the message string from a `catch_unwind` payload. Panic messages
/// land as either `&'static str` or `String` depending on whether the panic
/// site used `panic!("literal")` or formatted via `assert!(..., "msg")`.
fn panic_message(err: Box<dyn std::any::Any + Send>) -> String {
    if let Some(s) = err.downcast_ref::<String>() {
        return s.clone();
    }
    if let Some(s) = err.downcast_ref::<&'static str>() {
        return (*s).to_string();
    }
    String::from("<non-string panic payload>")
}

#[test]
fn parser_sanity_cap_field_count_trips_before_allocation() {
    // version + struct_tag + field_count = MAX_FIELDS_PER_ROW + 1 (= 1025,
    // LE: 0x01 0x04 0x00 0x00). The cap rejects values > MAX_FIELDS_PER_ROW.
    //
    // The assertion pins the panic SOURCE to the sanity cap by matching the
    // message text. A naive `result.is_err()` would silently pass if the cap
    // were deleted (the downstream `take()` bounds-check would fire instead
    // on this exact 6-byte payload). Matching `"sanity:"` and `"field_count"`
    // makes cap removal, cap weakening, and cap-position regressions all
    // surface a different message ("ran off the end ...") and fail this test.
    let payload: [u8; 6] = [FORMAT_VERSION, TAG_STRUCT, 0x01, 0x04, 0x00, 0x00];
    assert_eq!(
        u32::from_le_bytes([0x01, 0x04, 0x00, 0x00]),
        MAX_FIELDS_PER_ROW + 1,
        "fixture must encode exactly MAX_FIELDS_PER_ROW + 1",
    );
    let result = std::panic::catch_unwind(|| parse_row(&payload));
    let msg =
        panic_message(result.expect_err("sanity cap must reject field_count > MAX_FIELDS_PER_ROW"));
    assert!(
        msg.contains("sanity:") && msg.contains("field_count"),
        "expected the field_count sanity-cap panic; got: {msg}",
    );
}

#[test]
fn parser_sanity_cap_name_len_trips_before_allocation() {
    // version + struct_tag + field_count=1 + name_len = MAX_NAME_LEN_BYTES + 1
    // (= 1_048_577, LE: 0x01 0x00 0x10 0x00). The cap rejects name_len values
    // > MAX_NAME_LEN_BYTES before attempting to read the name bytes.
    //
    // As with the field_count sibling, the assertion pins the panic SOURCE
    // to the sanity cap by matching the message. Without this, removing or
    // weakening the cap would still leave `take()`'s OOB check to fire,
    // silently passing the test while shipping a real OOM regression.
    let payload: [u8; 10] = [
        FORMAT_VERSION,
        TAG_STRUCT,
        0x01,
        0x00,
        0x00,
        0x00,
        0x01,
        0x00,
        0x10,
        0x00,
    ];
    assert_eq!(
        u32::from_le_bytes([0x01, 0x00, 0x10, 0x00]),
        MAX_NAME_LEN_BYTES + 1,
        "fixture must encode exactly MAX_NAME_LEN_BYTES + 1",
    );
    let result = std::panic::catch_unwind(|| parse_row(&payload));
    let msg =
        panic_message(result.expect_err("sanity cap must reject name_len > MAX_NAME_LEN_BYTES"));
    assert!(
        msg.contains("sanity:") && msg.contains("name_len"),
        "expected the name_len sanity-cap panic; got: {msg}",
    );
}

#[test]
#[cfg_attr(windows, ignore)]
fn ctas_scalar_and_string_boundaries() {
    // Boundary values that exercise the LE byte-packing edges:
    //   * i64::MAX (9_223_372_036_854_775_807) — every i64 bit position used
    //   * negative Float / Decimal literal — sign bit + non-trivial mantissa
    //   * 0.0000 — zero mantissa with non-zero scale (proves scale survives)
    //   * empty string — proves length-prefix handles len=0 correctly
    //
    // We do NOT include i64::MIN — the PartiQL parser rejects the literal
    // `-9223372036854775808` because it tokenizes as unary-minus applied to
    // `9223372036854775808` (= i64::MAX + 1) which overflows i64. That is a
    // parser-level constraint, not an encoder one; out of scope here.
    let dir = tempfile::tempdir().unwrap();
    let db_path = dir.path().join("boundaries.pqlite");
    let (ok, _stdout, stderr) = run_exec(
        "CREATE TABLE t AS (            SELECT                 9223372036854775807 AS max_int,                -12345.6789 AS neg_float,                0.0000 AS zero_dec,                '' AS empty_str             FROM mem(1,1) m         )",
        Some(&db_path),
    );
    assert!(ok, "boundary CTAS failed; stderr: {stderr}");

    let rows = read_all_rows(&db_path, "t");
    assert_eq!(rows.len(), 1);
    let struct_fields = match rows.into_iter().next().unwrap().payload {
        ParsedPayload::Struct(f) => f,
        other => panic!("expected Struct payload, got {other:?}"),
    };
    let fields: std::collections::HashMap<String, ParsedValue> =
        struct_fields.into_iter().collect();

    // i64::MAX must round-trip exactly — proves the high bit of the i64 LE
    // encoding survives the to_le_bytes/from_le_bytes round trip.
    assert_eq!(
        fields.get("max_int"),
        Some(&ParsedValue::Integer(i64::MAX)),
        "i64::MAX must round-trip exactly",
    );

    // PartiQL may type `-12345.6789` as either Float or Decimal depending on
    // planner literal-typing rules. Accept either; the numeric value must
    // round-trip in both representations.
    let neg = fields.get("neg_float").expect("neg_float column missing");
    match neg {
        ParsedValue::Float(v) => assert!(
            (*v - (-12345.6789_f64)).abs() < 1e-9,
            "negative float should round-trip: got {v}",
        ),
        ParsedValue::Decimal { scale, mantissa } => {
            let reconstructed = (*mantissa as f64) / 10f64.powi(*scale);
            assert!(
                (reconstructed - (-12345.6789_f64)).abs() < 1e-9,
                "negative decimal should round-trip: mantissa={mantissa} scale={scale}",
            );
        }
        other => panic!("expected Float or Decimal for -12345.6789, got {other:?}"),
    }

    // 0.0000 must preserve scale even with mantissa=0 — load-bearing
    // assertion for our 20-byte decimal encoding (4-byte i32 scale +
    // 16-byte i128 mantissa). A bug that dropped scale would still pass an
    // `== 0` check but corrupt non-zero decimals.
    match fields.get("zero_dec").expect("zero_dec column missing") {
        ParsedValue::Decimal { scale, mantissa } => {
            assert_eq!(*mantissa, 0, "0.0000 mantissa must be exactly 0");
            assert!(
                *scale >= 1,
                "0.0000 must preserve a non-zero scale, got {scale}"
            );
        }
        other => panic!("expected Decimal for 0.0000, got {other:?}"),
    }

    // Empty string proves the u32 length prefix handles len=0 cleanly.
    assert_eq!(
        fields.get("empty_str"),
        Some(&ParsedValue::String(String::new())),
        "empty string must round-trip with len=0 in the u32 length prefix",
    );
}

#[test]
#[cfg_attr(windows, ignore)]
fn ctas_rejects_bool_at_preflight_with_pointed_message() {
    // The preflight pass must reject a Bool column before any bytes are
    // written. Verifies both the rejection and the error wording.
    let dir = tempfile::tempdir().unwrap();
    let db_path = dir.path().join("bool.pqlite");
    let (ok, _stdout, stderr) = run_exec(
        "CREATE TABLE t AS (SELECT true AS b FROM mem(1,1) m)",
        Some(&db_path),
    );
    assert!(!ok, "Bool column must trip preflight; stderr: {stderr}");
    assert!(
        stderr.contains("column 'b'") && stderr.contains("Bool"),
        "error must name the column and the rejected type; got: {stderr}",
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
#[cfg_attr(windows, ignore)]
fn ctas_persists_bare_scalar_rows() {
    // Top-level shape is RowShape::Register(_) — a bag of scalars, not a
    // bag of structs. Encoder writes ver + TAG_INTEGER + i64 LE per row.
    let dir = tempfile::tempdir().unwrap();
    let db_path = dir.path().join("scalar.pqlite");
    let (ok, _stdout, stderr) = run_exec(
        "CREATE TABLE t AS (SELECT VALUE t.a FROM mem(3,2) t)",
        Some(&db_path),
    );
    assert!(ok, "bare-scalar CTAS should succeed; stderr: {stderr}");
    let rows = read_all_rows(&db_path, "t");
    assert_eq!(rows.len(), 3, "expected 3 scalar rows; got: {rows:?}");
    for (idx, row) in rows.iter().enumerate() {
        match &row.payload {
            ParsedPayload::Scalar(ParsedValue::Integer(v)) => {
                assert_eq!(*v as usize, idx, "row {idx}: value should equal row index");
            }
            other => panic!("row {idx}: expected scalar Integer, got {other:?}"),
        }
    }
}
