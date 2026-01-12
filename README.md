# PartiQL Rust

[![Crate](https://img.shields.io/crates/v/partiql.svg)](https://crates.io/crates/partiql)
[![Docs](https://docs.rs/partiql/badge.svg)](https://docs.rs/partiql)
[![License](https://img.shields.io/hexpm/l/plug.svg)](https://github.com/partiql/partiql-lang-rust/blob/main/LICENSE)
[![CI Build](https://github.com/partiql/partiql-lang-rust/workflows/CI%20Build/badge.svg)](https://github.com/partiql/partiql-lang-rust/actions?query=workflow%3A%22CI+Build%22)
[![codecov](https://codecov.io/gh/partiql/partiql-lang-rust/branch/main/graph/badge.svg?token=PDCNQZPVBD)](https://codecov.io/gh/partiql/partiql-lang-rust)

This is a collection of crates to provide Rust support for the [PartiQL][partiql] query language.

***The crates in this repository are considered experimental, under active/early development,
and APIs are subject to change.***

This project uses [workspaces][workspaces] to manage the crates in this repository.  The `partiql` crate is intended
to be the crate that exports all the relevant `partiql-*` sub-crate functionality.  It is factored in this way
to make applications needing only some sub-component of the PartiQL implementation possible (e.g. an application
that only requires the PartiQL parser can depend on `partiql-parser` directly).

## Development
This project uses a [git submodule](https://git-scm.com/book/en/v2/Git-Tools-Submodules) to pull in 
[partiql-tests](https://github.com/partiql/partiql-tests). The easiest way to pull everything in is to clone the 
repository recursively:

```bash
$ git clone --recursive https://github.com/partiql/partiql-lang-rust.git
```

You can also initialize the submodules as follows:

```bash
$ git submodule update --init --recursive
```

### Local CI Checks
To run the same checks that GitHub Actions CI runs locally, you can use the provided Makefile:

```bash
# Run all core CI checks (build, test, format, clippy, security)
make ci-check

# Or run individual checks
make build        # Build the workspace
make test         # Run tests
make fmt          # Check code formatting
make clippy       # Run clippy lints
make deny         # Run cargo-deny security/license checks
make conformance  # Run conformance tests (slower)
make coverage     # Generate code coverage report
make help         # Show all available targets
```

The `ci-check` target runs the essential checks that must pass for CI, equivalent to:
```bash
cargo build --workspace && cargo test --workspace && cargo fmt --all -- --check && cargo clippy --all-features --workspace -- -D warnings && cargo deny check advisories && cargo deny check bans licenses sources
```

**Note:** The `deny` target requires [cargo-deny](https://github.com/EmbarkStudios/cargo-deny) to be installed:
```bash
cargo install cargo-deny
```

## Running the conformance tests
Running `cargo test` from the `partiql-lang-rust` root will not run the conformance tests by default.

To run all the tests (including conformance tests), you will need to run `cargo test` with the "conformance_test" `--features` flag:

```shell
cargo test --features "conformance_test"
```

Or to run just the conformance tests:

```shell
cargo test --package partiql-conformance-tests --features "conformance_test"
```

More details on running individual tests can be found in the `partiql-conformance-tests` crate [README](partiql-conformance-tests/README.md).

## Security

See [CONTRIBUTING](CONTRIBUTING.md#security-issue-notifications) for more information.

## License

This project is licensed under the Apache-2.0 License.

[partiql]: https://partiql.org/
[workspaces]: https://doc.rust-lang.org/stable/cargo/reference/workspaces.html
[crates]: https://crates.io/policies