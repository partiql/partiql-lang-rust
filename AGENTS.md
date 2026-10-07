# AGENTS.md

This file provides guidance to Claude Code (claude.ai/code) when working with code in this repository.

## Project Overview

Rust implementation of the [PartiQL](https://partiql.org/) query language. This is a Cargo workspace with ~25 crates, where `partiql` is the top-level crate re-exporting sub-crate functionality.

## Build & Test Commands

```bash
# Build everything
cargo build --workspace

# Run all tests (excludes conformance tests)
cargo test --workspace

# Run a single test
cargo test -p partiql-eval test_name

# Run conformance tests (requires git submodule init)
cargo test --package partiql-conformance-tests --features "conformance_test"

# Lint
cargo clippy --all-features --workspace -- -D warnings

# Format check
cargo fmt --all -- --check

# All CI checks at once
make ci-check
```

## Architecture

The compilation pipeline flows through these crates in order:

1. **partiql-parser** — Lexer + parser producing a concrete syntax tree. Uses `insta` for snapshot testing.
2. **partiql-ast** — AST data structures and visitor traits. Includes a proc-macro crate (`partiql-ast-macros`).
3. **partiql-ast-passes** — AST transformations (name resolution).
4. **partiql-logical-planner** — Lowers AST to a logical plan. Contains the type checker (`typer.rs`) and built-in function registry (`builtins.rs`).
5. **partiql-logical** — Graph-based logical plan representation. Nodes are `BindingsOp` (relational) and leaves are `ValueExpr` (scalar).
6. **Evaluation** — Two backends, in separate crates:
   - **partiql-eval** — Legacy tree-walker; interprets logical plan nodes via the `Evaluable` trait. Uses the planner's default `VarRefResolution::Dynamic`.
   - **partiql-vm** — Bytecode VM (unpublished); compiles the logical plan to flat `Inst` bytecode and executes it in a register-based VM with cursors. This is the actively developed path. Requires plans lowered with `VarRefResolution::Static` (`LogicalPlanner::with_var_resolution`).

### Key supporting crates

- **partiql-value** — Runtime value types (`Value`, `Bag`, `List`, `Tuple`, `DateTime`).
- **partiql-types** — Static type system.
- **partiql-catalog** — Catalog trait for schema/function resolution, used by both planner and eval.
- **partiql-common** — Shared utilities (syntax locations, `ObjectId`).
- **partiql-conformance-tests** — Test runner for the [partiql-tests](https://github.com/partiql/partiql-tests) submodule (feature-gated).

### Extensions (`extension/`)

Plugin crates for Ion, CSV, DDL, and additional scalar functions. Each registers with the catalog system.

### Binaries (`partiql-tools`)

- `pqlite` — Interactive REPL / one-shot exec runner for the bytecode engine backed by an LMDB store. See "pqlite: testing & debugging" below.

## pqlite: testing & debugging

`pqlite` is the fastest way to poke at the bytecode engine end-to-end. It has two main subcommands: `open` (interactive REPL against a db file) and `exec` (single-shot, script-friendly). `pqlite completions <bash|zsh|fish|powershell|elvish>` prints a shell-completion script (install steps in `partiql-tools/PQLITE.md`).

### One-shot execution

```bash
# db-free query — no file is created
cargo run --bin pqlite -- exec "SELECT t.a FROM mem(3, 2) t"

# persistent db — parent dir must exist; the file is created on first CREATE TABLE
cargo run --bin pqlite -- exec --db /tmp/scratch.pqlite "CREATE TABLE users"
cargo run --bin pqlite -- exec --db /tmp/scratch.pqlite \
    "INSERT INTO users SELECT * FROM << {'id':1,'name':'alice'} >>"
cargo run --bin pqlite -- exec --db /tmp/scratch.pqlite "SELECT * FROM users"

# Ion output — pipe stdout to another Ion consumer; stderr is silent on success
cargo run --bin pqlite -- exec --format ion "SELECT * FROM << {'a': 1}, {'a': 2} >>"
```

Query rows and Ion output go to **stdout**; the timing footer, debug dumps, and errors go to **stderr**. That split is load-bearing for scripting — `2>/dev/null` gets you clean data.

Built-in table functions available inside queries: `mem(rows, cols)` (sequential ints), `rand(rows, cols)` (random ints), `scan_ion(path)` (read an Ion file).

### `--debug` pipeline dumps

`--debug` is a global flag (goes before the subcommand). Values are comma-separated; each value prints one stage of the compile pipeline to stderr before the result:

| Flag | Emits |
|---|---|
| `--debug ast` | Parsed AST (`[AST] Parsed { ... }`) |
| `--debug plan` | Logical plan graph (`[Plan] LogicalPlan { nodes, edges }`) |
| `--debug program` | Compiled bytecode: `CompiledPlan` with slot count, registers, cursors, constants, and the `Inst` stream |
| `--debug '*'` | All three |
| `--debug ast,plan` | Any comma-separated subset |

```bash
cargo run --bin pqlite -- --debug program exec "SELECT t.a FROM mem(3, 2) t"
cargo run --bin pqlite -- --debug ast,plan exec "SELECT 1"
```

Inside the REPL, pass `--debug` at launch (`pqlite --debug plan open db.pqlite`); every subsequent statement prints its stage dump.

### End-to-end test harness

Golden-file tests live under `partiql-tools/tests/pqlite/cases/**/*.test.ion`. Each `.test.ion` file is a sequence of Ion structs of the form `{ sql: "...", expect: { rows: $bag::[...] } }`. `expect` supports `rows`, `affected_rows`, `created_table`, or `error: "substring"`; annotate a struct with `skip::` to keep a known-broken case in tree. The `partiql_tools::pqlite_e2e::case` test runs them via `rstest` file globbing.

```bash
# Run every fixture (each file is its own test case)
cargo test -p partiql-tools --test pqlite_e2e

# Run one fixture by name-substring
cargo test -p partiql-tools --test pqlite_e2e select_projection

# CLI-contract tests (spawn the built binary as a subprocess)
cargo test -p partiql-tools --test pqlite_cli
```

When adding a new engine feature, the usual loop is: write a `.test.ion` under `cases/query/`, run it with `cargo test -p partiql-tools --test pqlite_e2e <name>`, and when it fails inspect the pipeline with `pqlite --debug '*' exec "<your sql>"` to see where the compile pipeline diverges from expectation.

## Bytecode VM (`partiql-vm`)

The VM lives in `partiql-vm/src/`:
- `compiler.rs` — Walks the `LogicalPlan` graph and emits bytecode (`Inst` stream) + metadata.
- `expr.rs` — Defines `Inst` (typed register-machine instructions) and `Expr` (expression tree for fallback).
- `plan.rs` — `CompiledPlan` struct holding the program, scan metadata, cursor info, constants.
- `arena.rs` — Slot-based register arena for VM execution.
- `value/` — `ValueRef`, `ValueOwned`, `Shape` (row schema known at compile time).
- `source/` — Data source abstraction (`DataSourceImpl`, `RegisterWriter`, `ScanLayout`).

## Commit & PR Workflow

- Always run `make ci-check` before committing.
- Commit messages follow conventional commits: `type(scope): description`
  - Types: `feat`, `fix`, `refactor`, `test`, `docs`, `chore`, `perf`, `ci`
  - Scope is the crate or module affected, e.g. `feat(eval): add OFFSET support`

### Before opening or updating a PR

1. **Local checks.** `make ci-check` (build, test, fmt, clippy, cargo-deny), or individually:
   ```bash
   cargo fmt --all -- --check
   cargo clippy --all-features --workspace -- -D warnings
   cargo test --workspace
   ```
   For pqlite plugin work, also run the clippy and test commands with `RUSTFLAGS='--cfg pqlite_unstable_plugins'`.
2. **Local CI with [`act`](https://github.com/nektos/act)** for the jobs your change affects (`ci_build_test.yml` ignores `**.md`, `docs/**`, `LICENSE`, `NOTICE`, so docs-only changes need none). `-W` is required (`build` also exists in `nightly.yml`), and `-P` maps the custom runner label to a Docker image:
   ```bash
   W=".github/workflows/ci_build_test.yml"; P="partiql-lang-rust_ubuntu-24.04_4-core=catthehacker/ubuntu:act-latest"
   act pull_request -W "$W" -P "$P" -j build      # lint, cargo-deny, build, test
   act pull_request -W "$W" -P "$P" -j coverage   # only if coverage setup changes
   act push -W "$W" -P "$P" -j conformance-report --artifact-server-path "$(mktemp -d)"  # conformance changes
   ```
   `act` copies the working tree instead of cloning, so run `git submodule update --init --recursive` first. act can't skip steps, so `coverage` always ends with a failed `Codecov Upload` locally; treat it as passing if `Cargo Test w/ Coverage` succeeds. Run `conformance-report` with `push`: its `pull_request`-only steps download the base report and comment on the PR, which can't work locally.
3. **AI review before pushing.** Run the Copilot CLI review against the branch diff and address its findings:
   ```bash
   copilot -p "/review the changes on this branch vs origin/main" -s --allow-tool 'shell(git:*)'
   ```
   After opening the PR, also request a Copilot review on it: `gh pr edit <number> --add-reviewer @copilot`.

### PR title and description

- PRs are squash-merged, so the **PR title becomes the commit message**. It MUST be a conventional commit: `type(scope): description` (scope optional), imperative, lowercase after the colon, no trailing period. E.g. `fix(vm): handle null fields in scan_ion`.
- Keep the description extremely concise and human-readable, filling the template in `.github/PULL_REQUEST_TEMPLATE.md` (keep its Apache 2.0 license line):
  - **What and why**
  - **How to use** (if applicable)
  - **Testing**
- Keep the title and description current whenever the PR changes (rebases, review fixes, scope changes): `gh pr edit <number> --title "..." --body-file <file>`.

## Conventions

- MSRV: Rust 1.86.0
- All crates use `#![deny(rust_2018_idioms)]` and `#![deny(clippy::all)]`.
- The logical plan is a DAG: nodes are `OpId`, edges are flows `(src, dst, branch)`.
- `EvaluationMode::Permissive` vs `Strict` controls runtime error handling (coerce-to-MISSING vs error).
- Snapshot tests use `insta` (run `cargo insta review` to update snapshots).
- The conformance test suite lives in a git submodule at `partiql-conformance-tests/partiql-tests/`. Initialize with `git submodule update --init --recursive`.
