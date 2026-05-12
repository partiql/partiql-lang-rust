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

- `partiql-hybrid` — CLI runner for bytecode engine with data source options (mem/ion/rand).
- `partiql-legacy` — CLI runner for legacy evaluator.
- `partiql-benchmarks` — Benchmark harness.
- `partiql-profile` — Profiling harness.

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
