# PartiQL Rust CLI
PoC for a CLI & REPL. It should be considered experimental, subject to change, etc.

In its current state, it largely exists to test parser interface & types from the perspective of an external application.
Probably the the mietter::Diagnostic stuff should be refactored and moved to the main parser crate.

## CLI Commands

- **`help`** : print the CLI's help message and supported commands
- **`repl`** : launches the [REPL](##REPL)
- **`ast -T<format> "<query>"`**: outputs a rendered version of the parsed AST  ([see Visualization](##Visualizations)):
  - **`<format>`**:
    - **`json`** : pretty-print to stdout in a json dump
    - **`dot`** : pretty-print to stdout in [Graphviz][Graphviz] [dot][GvDot] format
    - **`svg`** : print to stdout a [Graphviz][Graphviz] rendered svg xml document
    - **`png`** : print to stdout a [Graphviz][Graphviz] rendered png bitmap
    - **`display`** : display a [Graphviz][Graphviz] rendered png bitmap directly in supported terminals
  - **`query`** : the PartiQL query text

## REPL

The REPL currently assumes most of the input line is a PartiQL query, which it will attempt to parse.
- For an invalid query, errors are pretty printed to the output.
- For a valid query,
  - with no prefix, `Parse OK!` is printed to the output
  - if prefixed by `\ast`, a rendered AST tree image is printed to the output ([see Visualization](##Visualizations))

Features:
- Syntax highlighting of query input
- User-friendly error reporting
- Readling/editing
- `CTRL-D`/`CTRL-C` to quit.

# Visualizations

In order to use any of the [Graphviz][Graphviz]-based visualizations, you will need the graphviz libraries
installed on your machine (e.g. `brew install graphviz` or similar).

# TODO

See [REPL-tagged issues](https://github.com/partiql/partiql-lang-rust/issues?q=is%3Aissue+is%3Aopen+%5BREPL%5D)

- Use central location for syntax files rather than embedded in this crate
- Better interaction model
  - commands
  - more robust editing
  - etc.


[Graphviz]: https://graphviz.org/
[GvDot]: https://graphviz.org/doc/info/lang.html