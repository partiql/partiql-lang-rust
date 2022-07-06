# Changelog

All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.0.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [Unreleased]

### Added

- Lexer & Parser for the majority of PartiQL query capabilities
- AST for the currently parsed subset of PartiQL
- Tracking of locations in source text for ASTs and Errors
- Conformance tests via test generation from [partiql-tests](https://github.com/partiql/partiql-tests/)
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


