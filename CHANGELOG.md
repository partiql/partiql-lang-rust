# Changelog

All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.0.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [Unreleased]
### Changed

### Added
- Implements `LIMIT` and `OFFSET` operators in evaluator
- Adds some benchmarks for parsing, compiling, planning, & evaluation
- Implements `PIVOT` operator in evaluator
- `serde` feature to `partiql-value` and `partiql-logical` with `Serialize` and `Deserialize` traits.
- Adds `Display` for `LogicalPlan`
- Expose `partiql_value::parse_ion` as a public API.

### Fixes

## [0.2.0] - 2023-01-10
### Changed
- *BREAKING:* Refactors the AST
  - Removes Location from the AST, replacing with a 'node id' that gives the AST node identity; the id can be used to retrieve Location
  - Removes redundancies and extraneous nesting
  - Refactor some AST nodes (including `FROM`, `WHERE`, and `HAVING` clauses) for better visitation
  - Refactor `FromSource` to not wrap in `AstNode`

### Added
- Adds end-to-end PartiQL query evaluation with the following supported features
  - SELECT-FROM-WHERE
  - LATERAL LEFT, INNER, CROSS JOINs
  - UNPIVOT
  - SELECT VALUE
  - Query expressions
  - List, Bag, Tuple constructors
  - Path expressions (wildcard & unpivot path are not yet supported)
  - Subquery (supported in logical and eval plan; not yet in AST to plan conversion)
  - DISTINCT
  - Variable references
  - Literals
  - Arithmetic operators (+, -, *, /, %)
  - Logical operators (AND, OR, NOT)
  - Equality operators (= , !=)
  - Comparison operators (<, >, <=, >=)
  - IS [NOT] MISSING, IS [NOT] NULL
  - IN
  - BETWEEN
  - LIKE
  - Searched and simple case expressions
  - COALESCE and NULLIF
  - CONCAT
  - And the following functions
    - LOWER
    - UPPER
    - CHARACTER_LENGTH
    - LTRIM
    - BTRIM
    - RTRIM
    - SUBSTRING
    - EXISTS
- Adds `Visit` and `Visitor` traits for visiting AST
- Add AST node `Visit` impls via `proc_macro`s
- Adds PartiQL `Value`, an in-memory representation of PartiQL values
  - Supports PartiQL values other than `DATE`, `TIME`, s-expressions
  - Supports basic arithmetic, logical, equality, and comparison operators
  - Supports partiql parsing of Ion into `Value`
- Defines logical plan and evaluation DAG
- AST lowering to logical plan with name resolution
- `partiql-conformance-tests` support for parsing and running evaluation tests from `partiql-tests`

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

[Unreleased]: https://github.com/partiql/partiql-lang-rust/compare/v0.2.0...HEAD
[0.1.0]: https://github.com/partiql/partiql-lang-rust/releases/tag/v0.1.0
[0.2.0]: https://github.com/partiql/partiql-lang-rust/releases/tag/v0.2.0
