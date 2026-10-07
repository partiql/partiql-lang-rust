//! Tests for `pqlite completions <shell>`, which prints a `clap_complete`
//! script to stdout. Drives the built binary as a subprocess and checks that
//! every supported shell gets a script and that the clap `ValueHint`s /
//! possible values on pqlite's arguments make it into the generated output.

use rstest::rstest;
use std::process::Command;

const PQLITE: &str = env!("CARGO_BIN_EXE_pqlite");

/// Run `pqlite completions <shell>` and return its stdout.
fn run_completions(shell: &str) -> String {
    let out = Command::new(PQLITE)
        .args(["completions", shell])
        .output()
        .expect("failed to spawn pqlite");
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
#[case("elvish")]
fn completions_generated_for_every_shell(#[case] shell: &str) {
    let script = run_completions(shell);
    for word in [
        "pqlite",
        "open",
        "exec",
        "completions",
        "db",
        "format",
        "debug",
    ] {
        assert!(
            script.contains(word),
            "{shell} completion script should mention {word:?}; got:\n{script}"
        );
    }
}

#[test]
fn zsh_completions_carry_value_hints() {
    let script = run_completions("zsh");
    // ValueHint::FilePath becomes zsh's `_files` completer, for both the
    // positional `open <db>` and `exec --db`.
    assert!(
        script.contains(":db -- Path to the database file"),
        "{script}"
    );
    assert!(script.contains(":DB:_files"), "{script}");
    // ValueHint::Other on the query suppresses file completion (empty action).
    assert!(
        script
            .contains("':query -- The PartiQL statement(s) to run, e.g. \"SELECT 1; SELECT 2\":'"),
        "{script}"
    );
    // value_enum / PossibleValuesParser surface as literal alternatives.
    assert!(script.contains(":FORMAT:(text ion none)"), "{script}");
    assert!(script.contains(":DEBUG:(ast plan program all)"), "{script}");
    assert!(
        script.contains(
            ":shell -- The shell to generate completions for:(bash elvish fish powershell zsh)"
        ),
        "{script}"
    );
}

/// A bare `*` among the listed values is glob-expanded by the shell, which
/// floods `--debug <TAB>` with the current directory's files.
#[rstest]
#[case("bash")]
#[case("zsh")]
#[case("fish")]
fn completions_do_not_emit_glob_values(#[case] shell: &str) {
    let script = run_completions(shell);
    for needle in ["program *", "*\\t''"] {
        assert!(
            !script.contains(needle),
            "{shell} script contains glob value {needle:?}"
        );
    }
}

#[test]
fn debug_star_alias_still_accepted() {
    let out = Command::new(PQLITE)
        .args(["--debug", "*", "exec", "SELECT t.a FROM mem(1, 1) t"])
        .output()
        .expect("failed to spawn pqlite");
    assert!(
        out.status.success(),
        "stderr: {}",
        String::from_utf8_lossy(&out.stderr)
    );
    let stderr = String::from_utf8_lossy(&out.stderr);
    assert!(
        stderr.contains("[AST]") && stderr.contains("[Plan]"),
        "{stderr}"
    );
}

#[test]
fn fish_completions_carry_value_hints() {
    let script = run_completions("fish");
    // ValueHint::FilePath on `--db` becomes fish's `-F` (force file completion).
    assert!(
        script.contains("-l db -d 'Path to the database file. Omit for db-free queries' -r -F"),
        "{script}"
    );
    assert!(script.contains("-l format"), "{script}");
    assert!(script.contains("ion\\t''"), "{script}");
    assert!(script.contains("none\\t''"), "{script}");
}

#[test]
fn completions_rejects_unknown_shell() {
    let out = Command::new(PQLITE)
        .args(["completions", "tcsh"])
        .output()
        .expect("failed to spawn pqlite");
    assert!(!out.status.success(), "unknown shell must be rejected");
    assert!(
        String::from_utf8_lossy(&out.stderr).contains("invalid value 'tcsh'"),
        "stderr: {}",
        String::from_utf8_lossy(&out.stderr)
    );
}
