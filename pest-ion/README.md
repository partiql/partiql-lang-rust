# Pest to Ion

[![Crate](https://img.shields.io/crates/v/pest-ion.svg)](https://crates.io/crates/pest-ion)
[![Docs](https://docs.rs/pest-ion/badge.svg)](https://docs.rs/pest-ion)
[![License](https://img.shields.io/hexpm/l/plug.svg)](https://github.com/partiql/partiql-lang-rust/blob/main/LICENSE)
[![CI Build](https://github.com/partiql/partiql-lang-rust/workflows/CI%20Build/badge.svg)](https://github.com/partiql/partiql-lang-rust/actions?query=workflow%3A%22CI+Build%22)
[![codecov](https://codecov.io/gh/partiql/partiql-lang-rust/branch/main/graph/badge.svg?token=PDCNQZPVBD)](https://codecov.io/gh/partiql/partiql-lang-rust)

This is a simple tool and library for converting [Pest] grammars to [Ion] data format.

The motivation for this is to make a portable way to introspect [Pest] grammars in other tools
as a data format versus having to provide bespoke parsers for the Pest syntax in other platforms.

[Pest]: https://pest.rs/
[Ion]: https://amzn.github.io/ion-docs/