//! Multi-statement script parsing.

use std::time::{Duration, Instant};

use partiql_ast::ast;
use partiql_parser::Parsed;

use crate::common::parse_statements;
use crate::session::debug::DebugFlags;
use crate::session::outcome::DebugCapture;

/// A parsed `;`-separated script, ready to run statement by statement via
/// `PqliteSession::run_statement`.
pub struct Script<'a> {
    parsed: Parsed<'a>,
    /// Time to parse the whole script; statements are not parsed separately.
    pub parse_time: Duration,
    /// `--debug ast` capture of the whole script, flushed before it runs.
    pub debug: DebugCapture,
}

impl Script<'_> {
    pub fn statements(&self) -> &[ast::AstNode<ast::Statement>] {
        &self.parsed.statements
    }
}

/// Parse a `;`-separated script with the PartiQL parser
/// (`Parser::parse_statements`), not by scanning for `;`, so a `;` inside a
/// string literal, quoted identifier, or comment never splits a statement. A
/// trailing `;` is optional; empty or all-comment input yields no statements.
/// A syntax error anywhere rejects the whole script, so no statement runs.
pub fn parse_script<'a>(
    text: &'a str,
    debug: &DebugFlags,
) -> Result<Script<'a>, Box<dyn std::error::Error>> {
    let parse_start = Instant::now();
    let parsed = parse_statements(text).map_err(|e| format!("Parse error: {:?}", e))?;
    let parse_time = parse_start.elapsed();

    let mut capture = DebugCapture::default();
    if debug.ast {
        capture.ast = Some(format!("[AST] {parsed:?}"));
    }
    Ok(Script {
        parsed,
        parse_time,
        debug: capture,
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    fn count(s: &str) -> usize {
        parse_script(s, &DebugFlags::default())
            .unwrap_or_else(|e| panic!("parse failed: {e}"))
            .statements()
            .len()
    }

    #[test]
    fn single_statement_with_and_without_trailing_semicolon() {
        assert_eq!(count("SELECT 1"), 1);
        assert_eq!(count("  SELECT 1 ;  "), 1);
    }

    #[test]
    fn multiple_statements() {
        assert_eq!(count("SELECT 1; SELECT 2;\nSELECT 3"), 3);
    }

    #[test]
    fn semicolons_in_strings_identifiers_and_comments_do_not_split() {
        assert_eq!(
            count("SELECT 'a;b' FROM t; SELECT \"x;y\" FROM t -- c;d\n; /* e;f */ SELECT 3"),
            3
        );
    }

    #[test]
    fn empty_and_comment_only_input_yields_nothing() {
        assert_eq!(count(""), 0);
        assert_eq!(count("  \n "), 0);
        assert_eq!(count("-- nothing here\n"), 0);
    }

    #[test]
    fn syntax_error_anywhere_rejects_the_script() {
        let err = parse_script("SELECT 1; SELECT FROM; SELECT 3", &DebugFlags::default())
            .err()
            .expect("must fail");
        assert!(err.to_string().starts_with("Parse error"), "got: {err}");
    }
}
