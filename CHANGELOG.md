# Changelog

All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.0.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [Unreleased]
### Added
- Adds the following functionalities to PartiQL Playground
  - Moves the project to a Node.js project
  - Adds the capability for exporting the playground session on client side to be able to get fetched from another playground windows.
  - Adds a REST API and exposes /parse for parsing the query over http request.
  - Containerization using Docker.

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