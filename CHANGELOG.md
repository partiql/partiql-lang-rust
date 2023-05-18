# Changelog

All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.0.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [Unreleased]
### Changed
- *BREAKING:* partiql-eval: modifies visibility of types implementing `EvalExpr` and `Evaluable`
- *BREAKING:* removed `from_ion` method on `Value`
- *BREAKING:* partiql-ast: `visit` fn returns a `partiql-ast::Recurse` type to indicate if visitation of children nodes should continue
- *BREAKING:* partiql-logical-planner: modifies `lower(parsed: &Parsed)` to return a Result type of `Result<logical::LogicalPlan<logical::BindingsOp>, LoweringError>` rather than a `logical::LogicalPlan<logical::BindingsOp>`
- *BREAKING:* partiql-eval: modifies `compile(&mut self, plan: &LogicalPlan<BindingsOp>)` to return a Result type of `Result<EvalPlan, EvalErr>` rather than an `EvalPlan`

### Added
- Implements built-in function `EXTRACT`
- Add `partiql-extension-ion` extension for encoding/decoding `Value` to/from Ion data
### Fixes
- Fix parsing of `EXTRACT` datetime parts `YEAR`, `TIMEZONE_HOUR`, and `TIMEZONE_MINUTE`
- Fix logical plan to eval plan conversion for `EvalOrderBySortSpec` with arguments `DESC` and `NULLS LAST`
- Fix parsing of `EXTRACT` to allow keywords after the `FROM`

## [0.3.0] - 2023-04-11
### Changed
- `EvalExpr.evaluate` function now returns a [Cow](https://doc.rust-lang.org/std/borrow/enum.Cow.html) of `Value`
- `Evaluable` trait's `get_vars` function returns by ref
- Refactor of `partiql-eval` crate
  - Operators previously implementing `Evaluable` (e.g. `EvalScan`, `EvalFilter`) are under the `eval::evaluable` module
  - Expressions previously implementing `EvalExpr` (e.g. `EvalBinOpExpr`, `EvalLitExpr`) are under the `eval::expr` module
- Refactor `CallAgg` `partiql-ast` node

### Added
- Adds some benchmarks for parsing, compiling, planning, & evaluation
- Implements more built-in functions -- `POSITION`, `OCTET_LEN`, `BIT_LEN`, `ABS`, `MOD`, `CARDINALITY`, `OVERLAY`
- Implements `PIVOT` operator in evaluator
- Implements `LIKE` for non-string, non-literals
- `serde` feature to `partiql-value` and `partiql-logical` with `Serialize` and `Deserialize` traits
- Adds `Display` for `LogicalPlan`
- Expose `partiql_value::parse_ion` as a public API
- Adds some convenience methods on `Value`
  - Add `Extend` implementations for `List` and `Bag`
  - Add methods to iterate a `Tuple`'s values without zipping its names
  - Allow `collect()` into a `Tuple` with any `Into<String>`
- Parse `OUTER UNION`/`INTERSECT`/`EXCEPT`
- Parse `WITH` clause
- Implements `LIMIT` and `OFFSET` operators in evaluator
- `DATE`/`TIME`/`TIMESTAMP` values
- Parse `TABLE <id>` references
- Implements `GROUP BY` operator in evaluator
- Implements `HAVING` operator in evaluator
- Implements `ORDER BY` operator in evaluator
- Implements SQL aggregation functions (`AVG`, `COUNT`, `MAX`, `MIN`, `SUM`) in evaluator

### Fixes
- Some performance improvements from removing extraneous `clone`s and tweaking buffer sizes
- Fix off by one error when checking preconditions to lower join `ON`
- Recognize aggregate fn names in parser
- Pass-through comments when processing special forms
- Make `BY <x>` optional in `GROUP` clause
- Fix `JOIN` parsing by defaulting to `INNER` and allowing elision of keywords
- Allow un-parenthesized subquery as the only argument of a function in parser
- Fix handling of List/Bag/Tuple in keyword argument preprocessing in parser
- Fixes Tuple value duplicate equality and hashing
- Properly skip comments when parsing

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

[Unreleased]: https://github.com/partiql/partiql-lang-rust/compare/v0.3.0...HEAD
[0.3.0]: https://github.com/partiql/partiql-lang-rust/releases/tag/v0.3.0
[0.2.0]: https://github.com/partiql/partiql-lang-rust/releases/tag/v0.2.0
[0.1.0]: https://github.com/partiql/partiql-lang-rust/releases/tag/v0.1.0
