//! Multi-statement script splitting. Pure; no I/O.

use crate::common::parse_statements;

/// Split a `;`-separated script into per-statement source slices, in order.
///
/// Splitting is done by the PartiQL parser (`Parser::parse_statements`), not
/// by scanning for `;`, so a `;` inside a string literal, quoted identifier,
/// or comment never splits a statement. A trailing `;` is optional; empty or
/// all-comment input yields no statements. A syntax error anywhere rejects
/// the whole script, so no statement runs.
///
/// Each slice is re-parsed when it is run (`PqliteSession::run`) so that
/// per-statement parse timing and `--debug ast` output stay per-statement.
pub fn split_statements(script: &str) -> Result<Vec<&str>, Box<dyn std::error::Error>> {
    let parsed = parse_statements(script).map_err(|e| format!("Parse error: {:?}", e))?;
    parsed
        .statements
        .iter()
        .map(|stmt| {
            let loc = parsed
                .locations
                .get(&stmt.id)
                .ok_or("Parse error: statement has no source location")?;
            let (start, end) = (loc.start.0.to_usize(), loc.end.0.to_usize());
            // A statement's span can run over trailing whitespace and comments
            // up to its `;`; trim the whitespace (a trailing comment is harmless).
            script
                .get(start..end)
                .map(str::trim)
                .ok_or_else(|| "Parse error: statement location out of range".into())
        })
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    fn split(s: &str) -> Vec<&str> {
        split_statements(s).unwrap_or_else(|e| panic!("split failed: {e}"))
    }

    #[test]
    fn single_statement_with_and_without_trailing_semicolon() {
        assert_eq!(split("SELECT 1"), vec!["SELECT 1"]);
        assert_eq!(split("  SELECT 1 ;  "), vec!["SELECT 1"]);
    }

    #[test]
    fn multiple_statements_in_order() {
        assert_eq!(
            split("SELECT 1; SELECT 2;\nSELECT 3"),
            vec!["SELECT 1", "SELECT 2", "SELECT 3"]
        );
    }

    #[test]
    fn semicolons_in_strings_identifiers_and_comments_do_not_split() {
        assert_eq!(
            split("SELECT 'a;b' FROM t; SELECT \"x;y\" FROM t -- c;d\n; /* e;f */ SELECT 3"),
            vec![
                "SELECT 'a;b' FROM t",
                "SELECT \"x;y\" FROM t -- c;d",
                "SELECT 3"
            ]
        );
    }

    #[test]
    fn empty_and_comment_only_input_yields_nothing() {
        assert!(split("").is_empty());
        assert!(split("  \n ").is_empty());
        assert!(split("-- nothing here\n").is_empty());
    }

    #[test]
    fn syntax_error_anywhere_rejects_the_script() {
        let err = split_statements("SELECT 1; SELECT FROM; SELECT 3").unwrap_err();
        assert!(err.to_string().starts_with("Parse error"), "got: {err}");
    }
}
