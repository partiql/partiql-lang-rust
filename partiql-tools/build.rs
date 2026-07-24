use std::fs;
use std::path::PathBuf;
use std::process::Command;

/// Capture the short git commit SHA at build time and expose it to the crate
/// as the `PQLITE_GIT_SHA` compile-time environment variable (read via
/// `env!("PQLITE_GIT_SHA")`). Falls back to `"unknown"` when git is
/// unavailable (e.g. building from a source tarball outside a repository).
fn main() {
    let git_sha = run_git(&["rev-parse", "--short", "HEAD"]).unwrap_or_else(|| "unknown".into());
    println!("cargo:rustc-env=PQLITE_GIT_SHA={git_sha}");

    // Re-run this script when HEAD (and the ref it points to) changes, so the
    // embedded SHA stays current across commits/checkouts without a manual
    // `cargo clean`.
    if let Some(git_dir) = run_git(&["rev-parse", "--absolute-git-dir"]).map(PathBuf::from) {
        let head = git_dir.join("HEAD");
        if head.exists() {
            println!("cargo:rerun-if-changed={}", head.display());

            // If HEAD is a symbolic ref (e.g. `ref: refs/heads/main`), also
            // watch the loose ref file it resolves to — that's what actually
            // changes on a normal commit.
            if let Ok(contents) = fs::read_to_string(&head) {
                if let Some(ref_path) = contents.strip_prefix("ref:") {
                    let resolved = git_dir.join(ref_path.trim());
                    if resolved.exists() {
                        println!("cargo:rerun-if-changed={}", resolved.display());
                    }
                }
            }
        }

        // Packed refs are the fallback location when the loose ref is absent.
        let packed = git_dir.join("packed-refs");
        if packed.exists() {
            println!("cargo:rerun-if-changed={}", packed.display());
        }
    }

    // Re-discover `.test.ion` fixtures when the cases directory changes.
    println!("cargo:rerun-if-changed=tests/pqlite/cases");
}

/// Run a git command, returning trimmed stdout on success, or `None` on any
/// failure (git missing, non-zero exit, or non-UTF8 output).
fn run_git(args: &[&str]) -> Option<String> {
    let output = Command::new("git").args(args).output().ok()?;
    if !output.status.success() {
        return None;
    }
    let s = String::from_utf8(output.stdout).ok()?;
    let s = s.trim().to_string();
    if s.is_empty() {
        None
    } else {
        Some(s)
    }
}
