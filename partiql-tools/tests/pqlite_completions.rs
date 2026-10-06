//! Tests for `pqlite complete`, the shell-completion subcommand provided by
//! `clap_autocomplete`. Drives the built binary as a subprocess and checks that
//! every supported shell gets a script and that the clap `ValueHint`s /
//! possible values on pqlite's arguments make it into the generated output.

use rstest::rstest;
use std::process::Command;

const PQLITE: &str = env!("CARGO_BIN_EXE_pqlite");

/// Run `pqlite complete --shell <shell>` and return its stdout. On unix the
/// `--print` flag forces stdout instead of installing into system directories;
/// on other platforms the flag doesn't exist and stdout is the only mode.
fn run_complete(shell: &str) -> String {
    let mut cmd = Command::new(PQLITE);
    cmd.arg("complete").arg("--shell").arg(shell);
    if cfg!(unix) {
        cmd.arg("--print");
    }
    let out = cmd.output().expect("failed to spawn pqlite");
    assert!(
        out.status.success(),
        "completion generation for {shell} should succeed; stderr: {}",
        String::from_utf8_lossy(&out.stderr)
    );
    String::from_utf8(out.stdout).expect("completion script must be UTF-8")
}

#[rstest]
#[case("bash")]
#[case("zsh")]
#[case("fish")]
#[case("powershell")]
#[case("pwsh")]
#[case("elvish")]
fn completions_generated_for_every_shell(#[case] shell: &str) {
    let script = run_complete(shell);
    for word in ["pqlite", "open", "exec", "db", "format", "debug"] {
        assert!(
            script.contains(word),
            "{shell} completion script should mention {word:?}; got:\n{script}"
        );
    }
}

#[test]
fn zsh_completions_carry_value_hints() {
    let script = run_complete("zsh");
    // ValueHint::FilePath becomes zsh's `_files` completer, for both the
    // positional `open <db>` and `exec --db`.
    assert!(
        script.contains(":db -- Path to the database file"),
        "{script}"
    );
    assert!(script.contains(":DB:_files"), "{script}");
    // ValueHint::Other on the query suppresses file completion (empty action).
    assert!(
        script.contains("':query -- The PartiQL query string to run:'"),
        "{script}"
    );
    // value_enum / PossibleValuesParser surface as literal alternatives.
    assert!(script.contains(":FORMAT:(text ion)"), "{script}");
    assert!(script.contains(":DEBUG:(ast plan program *)"), "{script}");
    assert!(
        script.contains("(bash zsh fish powershell pwsh elvish)"),
        "{script}"
    );
}

#[test]
fn fish_completions_carry_value_hints() {
    let script = run_complete("fish");
    // ValueHint::FilePath on `--db` becomes fish's `-F` (force file completion).
    assert!(
        script.contains("-l db -d 'Path to the database file. Omit for db-free queries' -r -F"),
        "{script}"
    );
    assert!(script.contains("-l format"), "{script}");
    assert!(script.contains("ion\\t''"), "{script}");
}

#[test]
fn complete_rejects_unknown_shell() {
    let out = Command::new(PQLITE)
        .args(["complete", "--shell", "tcsh"])
        .output()
        .expect("failed to spawn pqlite");
    assert!(!out.status.success(), "unknown shell must be rejected");
    assert!(
        String::from_utf8_lossy(&out.stderr).contains("invalid value 'tcsh'"),
        "stderr: {}",
        String::from_utf8_lossy(&out.stderr)
    );
}
