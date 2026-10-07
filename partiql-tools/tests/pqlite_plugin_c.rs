//! End-to-end tests for the plugin C ABI.
//!
//! `tests/plugins/c_plugin.c` is a plugin written in plain C99 against
//! `include/pqlite_plugin.h` alone: no Rust and no Arrow library on the plugin
//! side. Each test run compiles it with the system C compiler (`$CC`, default
//! `cc`) into a shared library and loads it into the built `pqlite` binary
//! with `--load`, so the whole path is exercised: `dlopen`, init, argument
//! passing, the Arrow C stream, error reporting and release.
//!
//! Plugin support is unstable, so these run only when built with
//! `RUSTFLAGS="--cfg pqlite_unstable_plugins"`.
#![cfg(all(pqlite_unstable_plugins, unix))]

use std::path::{Path, PathBuf};
use std::process::Command;
use std::sync::OnceLock;

const PQLITE: &str = env!("CARGO_BIN_EXE_pqlite");

/// Compile the C plugin once per test binary and return its path.
fn plugin() -> &'static Path {
    static SO: OnceLock<(tempfile::TempDir, PathBuf)> = OnceLock::new();
    &SO.get_or_init(|| {
        let manifest = Path::new(env!("CARGO_MANIFEST_DIR"));
        let dir = tempfile::tempdir().expect("tempdir");
        let so = dir.path().join("libc_plugin.so");
        let cc = std::env::var("CC").unwrap_or_else(|_| "cc".to_string());
        let out = Command::new(&cc)
            .args(["-std=c99", "-D_POSIX_C_SOURCE=200809L"])
            .args(["-Wall", "-Wextra", "-Wpedantic", "-Werror"])
            .args(["-shared", "-fPIC"])
            .arg("-I")
            .arg(manifest.join("include"))
            .arg(manifest.join("tests/plugins/c_plugin.c"))
            .arg("-o")
            .arg(&so)
            .output()
            .unwrap_or_else(|e| panic!("failed to run C compiler '{cc}': {e}"));
        assert!(
            out.status.success(),
            "compiling the C plugin failed:\n{}",
            String::from_utf8_lossy(&out.stderr)
        );
        (dir, so)
    })
    .1
}

struct Output {
    ok: bool,
    stdout: String,
    stderr: String,
}

/// Run `pqlite --load <c plugin> [extra...] exec --format ion <query>`.
fn exec_with(extra: &[&str], env: &[(&str, &str)], query: &str) -> Output {
    let mut cmd = Command::new(PQLITE);
    cmd.arg("--load").arg(plugin());
    cmd.args(extra);
    cmd.args(["exec", "--format", "ion", query]);
    cmd.env_remove("PQLITE_PLUGIN_LOG");
    for (k, v) in env {
        cmd.env(k, v);
    }
    let out = cmd.output().expect("failed to spawn pqlite");
    Output {
        ok: out.status.success(),
        stdout: String::from_utf8_lossy(&out.stdout).trim().to_string(),
        stderr: String::from_utf8_lossy(&out.stderr).into_owned(),
    }
}

/// Run a query that must succeed and return its Ion output.
fn query(sql: &str) -> String {
    let out = exec_with(&[], &[], sql);
    assert!(out.ok, "query failed: {sql}\nstderr: {}", out.stderr);
    out.stdout
}

/// Run a query that must fail and return its stderr.
fn query_err(sql: &str) -> String {
    let out = exec_with(&[], &[], sql);
    assert!(!out.ok, "query should fail: {sql}\nstdout: {}", out.stdout);
    out.stderr
}

#[test]
fn whole_rows_across_several_batches() {
    // 6 rows in batches of 4: one full batch, one partial.
    assert_eq!(
        query("SELECT * FROM c_seq(6, 4) AS t"),
        r#"{rows: $bag::[{id: 0, name: "row0"}, {id: 1, name: "row1"}, {id: 2, name: "row2"}, {id: 3, name: "row3"}, {id: 4, name: "row4"}, {id: 5, name: "row5"}]}"#
    );
}

#[test]
fn projection_and_filter_over_static_schema() {
    assert_eq!(
        query("SELECT t.name FROM c_seq(10) AS t WHERE t.id > 7"),
        r#"{rows: $bag::[{name: "row8"}, {name: "row9"}]}"#
    );
}

#[test]
fn aggregation_over_many_batches() {
    assert_eq!(
        query("SELECT COUNT(*) AS n, SUM(t.id) AS total FROM c_seq(1000, 7) AS t"),
        "{rows: $bag::[{n: 1000, total: 499500}]}"
    );
}

#[test]
fn empty_stream_and_early_close() {
    assert_eq!(query("SELECT * FROM c_seq(0) AS t"), "{rows: $bag::[]}");
    // LIMIT stops reading and releases the stream before it is exhausted.
    assert_eq!(
        query("SELECT t.id FROM c_seq(100, 3) AS t LIMIT 2"),
        "{rows: $bag::[{id: 0}, {id: 1}]}"
    );
}

#[test]
fn arguments_cross_as_scalars_or_ion_text() {
    assert_eq!(
        query("SELECT * FROM c_args(1, 'x', [1, {'a': 2}], 2.5) AS t"),
        r#"{rows: $bag::[{kind: "int", text: "1"}, {kind: "string", text: "x"}, {kind: "ion_text", text: "[1, {\"a\": 2}]"}, {kind: "ion_text", text: "2.5"}]}"#
    );
    assert_eq!(
        query("SELECT * FROM c_args(true, null, MISSING, <<1>>) AS t"),
        r#"{rows: $bag::[{kind: "bool", text: "true"}, {kind: "null", text: ""}, {kind: "missing", text: ""}, {kind: "ion_text", text: "$bag::[1]"}]}"#
    );
}

#[test]
fn plugin_options_and_logging_reach_the_plugin() {
    let out = exec_with(
        &["--plugin-opt", "c.alpha=1", "--plugin-opt", "c.beta=two"],
        &[("PQLITE_PLUGIN_LOG", "info")],
        "SELECT * FROM c_config() AS t",
    );
    assert!(out.ok, "stderr: {}", out.stderr);
    assert_eq!(
        out.stdout,
        r#"{rows: $bag::[{key: "c.alpha", value: "1"}, {key: "c.beta", value: "two"}]}"#
    );
    assert!(
        out.stderr.contains("INFO: loaded with 2 option(s)"),
        "stderr: {}",
        out.stderr
    );
}

#[test]
fn loading_prints_the_unstable_warning() {
    let out = exec_with(&[], &[], "SELECT * FROM c_seq(1) AS t");
    assert!(out.ok);
    assert!(out.stderr.contains("plugin support is unstable"));
}

#[test]
fn plugin_errors_are_reported() {
    assert!(query_err("SELECT * FROM c_fail() AS t").contains("c_fail: open failed on purpose"));
    assert!(query_err("SELECT * FROM c_fail_next() AS t")
        .contains("c_fail_next: batch failed on purpose"));
    assert!(query_err("SELECT * FROM c_seq(-1) AS t")
        .contains("c_seq: n must be a non-negative integer"));
}

#[test]
fn load_errors_are_reported() {
    let so = plugin().to_str().unwrap();
    let dup = exec_with(&["--load", so], &[], "SELECT 1");
    assert!(!dup.ok);
    assert!(
        dup.stderr
            .contains("table function 'c_seq' is already registered"),
        "stderr: {}",
        dup.stderr
    );

    let missing = Command::new(PQLITE)
        .args(["--load", "/nonexistent/libnothing.so", "exec", "SELECT 1"])
        .output()
        .expect("failed to spawn pqlite");
    assert!(!missing.status.success());
    assert!(String::from_utf8_lossy(&missing.stderr).contains("could not load plugin"));
}
