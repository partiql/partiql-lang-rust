# PartiQL Rust REPL

PoC for a REPL. It should be considered experimental, subject to change, etc.

In its current state, it largely exists to test parser interface & types from the perspective of an external application. 
Probably the the mietter::Diagnostic stuff should be refactored and moved to the main parser crate.

The REPL currently accepts no commands, assuming any/all input is a PartiQL query, which it will attempt to parse. Parse errors are pretty printed to the output.

Features:
- Syntax highlighting of query input
- User-friendly error reporting
- Readling/editing
- `CTRL-D`/`CTRL-C` to quit.

# TODO

- Use central location for syntax files rather than embedded in this crate
- Add github issue link for Internal Compiler Errors
- Better interaction model
  - commands
  - more robust editing
  - etc.