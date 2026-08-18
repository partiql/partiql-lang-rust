//! End-to-end tests for `read`, `stdin`, `exec`, `curl`.

use std::io::Write;
use std::process::{Command, Stdio};

const PQLITE: &str = env!("CARGO_BIN_EXE_pqlite");

fn run_exec(query: &str) -> (bool, String, String) {
    let out = Command::new(PQLITE)
        .arg("exec")
        .arg(query)
        .output()
        .expect("failed to spawn pqlite");
    (
        out.status.success(),
        String::from_utf8_lossy(&out.stdout).into_owned(),
        String::from_utf8_lossy(&out.stderr).into_owned(),
    )
}

fn run_with_stdin(query: &str, input: &[u8]) -> (bool, String, String) {
    let mut child = Command::new(PQLITE)
        .arg("exec")
        .arg(query)
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()
        .expect("failed to spawn pqlite");
    child
        .stdin
        .as_mut()
        .expect("child stdin")
        .write_all(input)
        .expect("write stdin");
    let out = child.wait_with_output().expect("wait pqlite");
    (
        out.status.success(),
        String::from_utf8_lossy(&out.stdout).into_owned(),
        String::from_utf8_lossy(&out.stderr).into_owned(),
    )
}

#[test]
fn stdin_reads_jsonl() {
    let (ok, stdout, stderr) = run_with_stdin("SELECT * FROM stdin()", b"{\"a\": 1}\n{\"a\": 2}\n");
    assert!(ok, "stderr: {stderr}");
    assert!(stdout.contains("{ 'a': 1 }"), "got: {stdout}");
    assert!(stdout.contains("{ 'a': 2 }"), "got: {stdout}");
}

#[test]
fn stdin_reads_nested_json() {
    let input = b"{\"id\": 1, \"author\": {\"login\": \"alice\"}, \"tags\": [1, 2]}\n";
    let (ok, stdout, stderr) = run_with_stdin("SELECT * FROM stdin()", input);
    assert!(ok, "stderr: {stderr}");
    assert!(
        stdout.contains("'author': { 'login': 'alice' }"),
        "got: {stdout}"
    );
    assert!(stdout.contains("'tags': [1, 2]"), "got: {stdout}");
}

#[test]
fn stdin_unwraps_top_level_array() {
    let (ok, stdout, stderr) = run_with_stdin(
        "SELECT * FROM stdin()",
        b"[{\"a\": 1}, {\"a\": 2}, {\"a\": 3}]",
    );
    assert!(ok, "stderr: {stderr}");
    assert!(stderr.contains("3 rows"), "got: {stderr}");
    assert!(stdout.contains("{ 'a': 1 }"), "got: {stdout}");
    assert!(stdout.contains("{ 'a': 3 }"), "got: {stdout}");
}

#[test]
fn read_decodes_json_file() {
    let dir = tempfile::tempdir().unwrap();
    let path = dir.path().join("rows.json");
    std::fs::write(&path, b"[{\"a\": 1}, {\"a\": 2}]").unwrap();
    let (ok, stdout, stderr) = run_exec(&format!("SELECT * FROM read('{}')", path.display()));
    assert!(ok, "stderr: {stderr}");
    assert!(stdout.contains("{ 'a': 1 }"), "got: {stdout}");
    assert!(stdout.contains("{ 'a': 2 }"), "got: {stdout}");
}

#[test]
fn read_decodes_ion_file() {
    let dir = tempfile::tempdir().unwrap();
    let path = dir.path().join("rows.ion");
    std::fs::write(&path, b"{a: 1}\n{a: 2}\n").unwrap();
    let (ok, stdout, stderr) = run_exec(&format!("SELECT * FROM read('{}')", path.display()));
    assert!(ok, "stderr: {stderr}");
    assert!(stdout.contains("{ 'a': 1 }"), "got: {stdout}");
    assert!(stdout.contains("{ 'a': 2 }"), "got: {stdout}");
}

#[test]
fn read_projects_nested_object_field() {
    let dir = tempfile::tempdir().unwrap();
    let path = dir.path().join("rows.json");
    std::fs::write(
        &path,
        b"[{\"number\": 1, \"state\": \"open\", \"author\": {\"login\": \"alice\"}}]",
    )
    .unwrap();
    let (ok, stdout, stderr) = run_exec(&format!(
        "SELECT number, state, author FROM read('{}')",
        path.display()
    ));
    assert!(ok, "stderr: {stderr}");
    assert!(
        stdout.contains("'author': { 'login': 'alice' }"),
        "got: {stdout}"
    );
}

#[test]
fn exec_reads_subprocess_stdout() {
    let (ok, stdout, stderr) = run_exec("SELECT * FROM exec('echo {\"n\": 7}')");
    assert!(ok, "stderr: {stderr}");
    assert!(stdout.contains("'n': 7"), "got: {stdout}");
}

#[test]
fn exec_surfaces_command_failure() {
    let (ok, _stdout, stderr) = run_exec("SELECT * FROM exec('sh -c \"echo boom 1>&2; exit 1\"')");
    assert!(!ok, "should fail; stderr: {stderr}");
    assert!(
        stderr.contains("exec") && stderr.contains("boom"),
        "got: {stderr}"
    );
}

#[test]
fn create_table_from_stdin_persists_across_processes() {
    let dir = tempfile::tempdir().unwrap();
    let dbp = dir.path().join("t.pqlite");

    let mut child = Command::new(PQLITE)
        .arg("exec")
        .arg("--db")
        .arg(&dbp)
        .arg("CREATE TABLE issues AS (SELECT * FROM stdin())")
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()
        .expect("spawn CTAS");
    child
        .stdin
        .as_mut()
        .unwrap()
        .write_all(
            b"[{\"n\": 1, \"a\": {\"login\": \"alice\"}}, {\"n\": 2, \"a\": {\"login\": \"bob\"}}]",
        )
        .unwrap();
    let out = child.wait_with_output().unwrap();
    assert!(
        out.status.success(),
        "CTAS failed; stderr: {}",
        String::from_utf8_lossy(&out.stderr)
    );

    let out = Command::new(PQLITE)
        .arg("exec")
        .arg("--db")
        .arg(&dbp)
        .arg("SELECT * FROM issues")
        .output()
        .expect("spawn SELECT");
    assert!(
        out.status.success(),
        "reopen SELECT failed; stderr: {}",
        String::from_utf8_lossy(&out.stderr)
    );
    let stdout = String::from_utf8_lossy(&out.stdout);
    assert!(
        stdout.contains("'a': { 'login': 'alice' }"),
        "got: {stdout}"
    );
    assert!(stdout.contains("'a': { 'login': 'bob' }"), "got: {stdout}");
}
