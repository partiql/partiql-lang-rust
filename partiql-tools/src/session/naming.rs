//! Query-and-name normalization helpers. Pure functions; no I/O.

use partiql_value::BindingsName;

/// Trim outer whitespace and strip a single trailing `;` so the REPL can use
/// `;` as a completion signal without it leaking into the PartiQL grammar.
/// TODO: drop this once the parser accepts `;` natively.
pub fn normalize_query(input: &str) -> &str {
    let trimmed = input.trim();
    trimmed.strip_suffix(';').unwrap_or(trimmed).trim_end()
}

/// SQL-idiomatic rendering: quoted identifiers re-quoted, bare ones bare.
pub fn format_table_name(name: &BindingsName<'_>) -> String {
    match name {
        BindingsName::CaseSensitive(s) => format!("\"{}\"", s),
        BindingsName::CaseInsensitive(s) => s.as_ref().to_string(),
    }
}

/// Bare identifiers fold to ASCII lowercase; quoted identifiers are verbatim.
/// Non-ASCII bare identifiers would diverge from the engine's UniCase fold,
/// but the catalog does not accept them today.
pub(super) fn canonical_table_key(name: &BindingsName<'_>) -> String {
    match name {
        BindingsName::CaseInsensitive(s) => s.to_lowercase(),
        BindingsName::CaseSensitive(s) => s.to_string(),
    }
}
