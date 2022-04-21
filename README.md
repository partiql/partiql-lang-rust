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

Due to the lack of namespacing in [crates.io][crates], we have published `0.0.0` versions for the sub-crates we know
we will need.  A crate with a version `0.1.0` or higher, should have real, albeit potentially very experimental and/or
early implementations.

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

## Security

See [CONTRIBUTING](CONTRIBUTING.md#security-issue-notifications) for more information.

## License

This project is licensed under the Apache-2.0 License.

[partiql]: https://partiql.org/
[workspaces]: https://doc.rust-lang.org/stable/cargo/reference/workspaces.html
[crates]: https://crates.io/policies