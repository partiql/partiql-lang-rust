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
6. **partiql-eval** — Two evaluation backends:
   - **Legacy tree-walker** (`eval/` module) — interprets logical plan nodes via `Evaluable` trait.
   - **Bytecode VM** (`engine/` module) — compiles logical plan to flat `Inst` bytecode, executes in a register-based VM with cursors. This is the actively developed path (`partiql-vm` branch).

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
- `partiql-hybrid` — CLI runner for bytecode engine with data source options (mem/ion/rand).
- `partiql-legacy` — CLI runner for legacy evaluator.
- `partiql-benchmarks` — Benchmark harness.
- `partiql-profile` — Profiling harness.

## pqlite: testing & debugging

`pqlite` is the fastest way to poke at the bytecode engine end-to-end. It has two subcommands: `open` (interactive REPL against a db file) and `exec` (single-shot, script-friendly).

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

## Bytecode VM (engine module)

The new engine in `partiql-eval/src/engine/`:
- `compiler.rs` — Walks the `LogicalPlan` graph and emits bytecode (`Inst` stream) + metadata.
- `expr.rs` — Defines `Inst` (typed register-machine instructions) and `Expr` (expression tree for fallback).
- `plan.rs` — `CompiledPlan` struct holding the program, scan metadata, cursor info, constants.
- `arena.rs` — Slot-based register arena for VM execution.
- `value/` — `ValueRef`, `ValueOwned`, `Shape` (row schema known at compile time).
- `source/` — Data source abstraction (`DataSourceImpl`, `RegisterWriter`, `ScanLayout`).

## Commit & PR Workflow

- Always run `make ci-check` before committing.
- Commit messages follow conventional commits: `type(scope): description`
  - Types: `feat`, `fix`, `refactor`, `test`, `docs`, `chore`, `perf`
  - Scope is the crate or module affected, e.g. `feat(eval): add OFFSET support`

## Conventions

- MSRV: Rust 1.86.0
- All crates use `#![deny(rust_2018_idioms)]` and `#![deny(clippy::all)]`.
- The logical plan is a DAG: nodes are `OpId`, edges are flows `(src, dst, branch)`.
- `EvaluationMode::Permissive` vs `Strict` controls runtime error handling (coerce-to-MISSING vs error).
- Snapshot tests use `insta` (run `cargo insta review` to update snapshots).
- The conformance test suite lives in a git submodule at `partiql-conformance-tests/partiql-tests/`. Initialize with `git submodule update --init --recursive`.
