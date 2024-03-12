# Changelog

All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.0.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [Unreleased]
### Changed
- Adds quotes to the attributes of PartiQL tuple's debug output so it can be read and transformed using Kotlin `partiql-cli`
- Adds u8, u16, u32, u64, and u128 support to partiql_value::Value::from(type)
- [breaking] Changes the interface to `EvalPlan` to accept an `EvalContext`
- [breaking] Changes `EvaluationError` to not implement `Clone` 
- [breaking] Changes the structure of `EvalPlan`

### Added
- Add `partiql-extension-visualize` for visualizing AST and logical plan
- Add a `SessionContext` containing both a system-level and a user-level context object usable by expression evaluation 

### Fixed
- Fixed `ORDER BY`'s ability to see into projection aliases
- Fixed errors in `BaseTableExpr`s get added to the evaluation context
- Fixed certain errors surfacing in Permissive evaluation mode, when they should only be present in Strict mode

## [0.6.0] - 2023-10-31
### Changed
- *BREAKING:* partiql-value: `BindingsName` changed to hold `Cow<str>` rather than  `String`
- *BREAKING:* partiql-eval: Construction of expression evaluators changed to separate binding from evaluation of expression. & implement strict eval
- *BREAKING:* partiql-value: `Value` trait's `is_null_or_missing` renamed to `is_absent`
- *BREAKING:* partiql-value: `Value` trait's `coerce_to_tuple`, `coerece_to_bag`, and `coerce_to_list` methods renamed to `coerce_into_tuple`, `coerece_into_bag`, and `coerece_into_list`.
- *BREAKING:* partiql-value: `Tuple`'s `pairs` and `into_pairs` changed to return concrete `Iterator` types.
- *BREAKING:* partiql-eval: `EvaluatorPlanner` construction now takes an `EvaluationMode` parameter.
- *BREAKING:* partiql-eval: `like_to_re_pattern` is no longer public.
- *BREAKING:* partiql-value: Box Decimals in `Value` to assure `Value` fits in 16 bytes.
- *BREAKING:* partiql-logical-planner: moves `NameResolver` to `partiql-ast-passes`
- *BREAKING:* partiql-value: removes `partiql` from value macro_rules; e.g. `partiql_bag` renames to `bag`.
- *BREAKING:* partiql-ast: changed modeling of `Query` and `SetExpr` nodes to support `ORDER BY`, `LIMIT`, `OFFSET` in children of set operators
  - Affects the AST and visitor
- *BREAKING:* partiql-ast: rename of `SetExpr` to `BagOpExpr` and `SetOp` to `BagOp`
  - Affects the AST and visitor
- *BREAKING:* partiql-parser: `Parsed` struct's `ast` field is now an `ast::AstNode<ast::TopLevelQuery>`
- *BREAKING:* partiql-eval: `Evaluable` trait's `update_input` fn now also takes in an `EvalContext`
- *BREAKING:* partiql-logical: changed modeling of `Project` `exprs` to be a `Vec<(String, ValueExpr)>` rather than a `HashMap<String, ValueExpr>` to support multiple project items with the same alias
- *BREAKING:* partiql-logical: changed modeling of `VarRef` to include a `VarRefType` to indicate whether to do a local vs global binding lookup

### Added
- Strict mode evaluation partial support added.
- Add interface for `STRICT` mode evalution to `EvaluatorPlanner`.
- Add ability for partiql-extension-ion extension encoding/decoding of `Value` to/from Ion `Element`
- Add `partiql-types` crate that includes data models for PartiQL Types.
- Add `partiql_ast_passes::static_typer` for type annotating the AST.
- Add ability to parse `ORDER BY`, `LIMIT`, `OFFSET` in children of set operators
- Add `OUTER` bag operator (`OUTER UNION`, `OUTER INTERSECT`, `OUTER EXCEPT`) implementation
- Add experimental `partiql_logical_planner::typer` for typing PartiQL queries with the initial support for simple SFW queries with `SELECT` and `FROM` clauses only with no operators, JOINs, etc.
- Add `NullSortedValue` to specify ordering null or missing values `partiql_value::Value`s before or after all other values
- Implements the aggregation functions `ANY`, `SOME`, `EVERY` and their `COLL_` versions
- Add `COUNT(*)` implementation
- Add `to_vec` method to `List` and `Bag` to convert to a `Vec`

### Fixed
- Fixes parsing of multiple consecutive path wildcards (e.g. `a[*][*][*]`), unpivot (e.g. `a.*.*.*`), and path expressions (e.g. `a[1 + 2][3 + 4][5 + 6]`)—previously these would not parse correctly.
- partiql-parser set quantifier for bag operators fixed to `DISTINCT`
- partiql-parser set quantifier for bag operators fixed to be `DISTINCT` when unspecified
- partiql-logical-planner add error for when a `HAVING` is included without `GROUP BY`
- Fixes variable resolution lookup order and excessive lookups
- Fixes variable resolution of some ORDER BY variables
- Fixes nested list/bag/tuple type ordering for when `ASC NULLS LAST` and `DESC NULLS FIRST` are specified
- partiql-value fix deep equality of list, bags, and tuples
- Fixes bug when using multiple aggregations without a `GROUP BY`
- Performance improvements to grouping/evaluation

## [0.5.0] - 2023-06-06
### Changed
- *BREAKING:* partiql-eval: `evaluate` on `Evaluable` returns a `Value` rather than an `Option<Value>`
- *BREAKING:* partiql-ast: changes the modeling of Bag/List/Tuple literals
### Added
- Ability to add and view errors during evaluation with partiql-eval's `EvalContext`
- AST sub-trees representing literal values are lowered to `Value`s during planning
### Fixed

## [0.4.1] - 2023-05-25
### Changed
- partiql-extension-ion-functions : Made `IonExtension` `pub`
### Added
### Fixed

## [0.4.0] - 2023-05-24
### Changed
- *BREAKING:* partiql-eval: modifies visibility of types implementing `EvalExpr` and `Evaluable`
- *BREAKING:* removed `from_ion` method on `Value`
- *BREAKING:* partiql-ast: `visit` fn returns a `partiql-ast::Recurse` type to indicate if visitation of children nodes should continue
- *BREAKING:* partiql-logical-planner: modifies `lower(parsed: &Parsed)` to return a Result type of `Result<logical::LogicalPlan<logical::BindingsOp>, LoweringError>` rather than a `logical::LogicalPlan<logical::BindingsOp>`
- *BREAKING:* partiql-eval: modifies `compile(&mut self, plan: &LogicalPlan<BindingsOp>)` to return a Result type of `Result<EvalPlan, PlanErr>` rather than an `EvalPlan`
  - This is part of an effort to replace `panic`s with `Result`s
- *BREAKING:* partiql-logical-planner: Adds a `LogicalPlanner` to encapsulate the `lower` method
- *BREAKING:* partiql-eval: Adds a `EvaluatorPlanner` now requires a `Catalog` to be supplied at initialization
- *BREAKING:* partiql-logical-planner: `CallDef` and related types moved to partiql-catalog
### Added
- Implements built-in function `EXTRACT`
- Add `partiql-extension-ion` extension for encoding/decoding `Value` to/from Ion data
- Add `partiql-extension-ion-functions` extension which contains an extension function for reading from an Ion file
- Add `partiql-catalog` including an experimental `Catalog` interface and implementation
- Implements the `COLL_*` functions -- `COLL_AVG`, `COLL_COUNT`, `COLL_MAX`, `COLL_MIN`, `COLL_SUM`
- Adds AST to logical plan lowering for `IN` expressions
### Fixed
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

### Fixed
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

[Unreleased]: https://github.com/partiql/partiql-lang-rust/compare/v0.6.0...HEAD
[0.6.0]: https://github.com/partiql/partiql-lang-rust/releases/tag/v0.6.0
[0.5.0]: https://github.com/partiql/partiql-lang-rust/releases/tag/v0.5.0
[0.4.1]: https://github.com/partiql/partiql-lang-rust/releases/tag/v0.4.1
[0.4.0]: https://github.com/partiql/partiql-lang-rust/releases/tag/v0.4.0
[0.3.0]: https://github.com/partiql/partiql-lang-rust/releases/tag/v0.3.0
[0.2.0]: https://github.com/partiql/partiql-lang-rust/releases/tag/v0.2.0
[0.1.0]: https://github.com/partiql/partiql-lang-rust/releases/tag/v0.1.0
