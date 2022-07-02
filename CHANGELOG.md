# Changelog

All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.0.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [Unreleased]
### Changed
- *BREAKING:* Refactors the AST
  - Removes Location from the AST, replacing with a 'node id' that gives the AST node identity; the id can be used to retrieve Location
  - Removes redundancies and extraneous nesting

### Added
- Adds the following functionalities to PartiQL Playground:
  - Moves the project to a Node.js project
  - Adds the capability for exporting the playground session on client side to be able to get fetched from another playground windows.
  - Adds a REST API and exposes /parse for parsing the query over http request.
  - Containerization using Docker.
- An experimental (pending [#15](https://github.com/partiql/partiql-docs/issues/15)) embedding of a subset of
    the [GPML (Graph Pattern Matching Language)](https://arxiv.org/abs/2112.06217) graph query into the `FROM` clause,
    supporting. The use within the grammar is based on the assumption of a new graph data type being added to the
    specification of data types within PartiQL, and should be considered experimental until the semantics of the graph
    data type are specified.
- basic and abbreviated node and edge patterns (section 4.1 of the GPML paper)
- concatenated path patterns  (section 4.2 of the GPML paper)
- path variables  (section 4.2 of the GPML paper)
- graph patterns (i.e., comma separated path patterns)  (section 4.3 of the GPML paper)
- parenthesized patterns (section 4.4 of the GPML paper)
- path quantifiers  (section 4.4 of the GPML paper)
- restrictors and selector  (section 5.1 of the GPML paper)
- pre-filters and post-filters (section 5.2 of the GPML paper)

### Fixes
- Fixes the bug with AST graph PAN and ZOOM—before this change the pan and zoom was quite flaky and very hard to work with.
- Fixes the version value for the session and JSON output by ensuring it gets picked from the selected version in the UI.


## [0.1.0] - 2022-08-05
### Added
- Lexer & Parser for the majority of PartiQL query capabilities—see syntax [success](https://github.com/partiql/partiql-tests/tree/main/partiql-tests-data/success/syntax)
  and [fail](https://github.com/partiql/partiql-tests/tree/main/partiql-tests-data/fail/syntax) tests for more details.
- AST for the currently parsed subset of PartiQL
- Tracking of locations in source text for ASTs and Errors
- Parser fuzz tester
- Conformance tests via test generation from [partiql-tests](https://github.com/partiql/partiql-tests/)
- PartiQL Playground proof of concept (POC)
- PartiQL CLI with REPL and query visualization features


[Unreleased]: https://github.com/partiql/partiql-lang-rust/compare/v0.1.0...HEAD
[0.1.0]: https://github.com/partiql/partiql-lang-rust/compare/v0.1.0