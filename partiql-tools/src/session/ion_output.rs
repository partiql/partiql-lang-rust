//! PartiQL-encoded-Ion envelope for non-query statement outcomes plus the
//! control-char escape used by every Ion emit path. Streaming Query row
//! rendering lives in `session::render::render_query_ion`.

use std::io::{self, Write};

use ion_rs::element::writer::TextKind;
use ion_rs::element::{Element, Sequence, Struct};

use crate::session::outcome::StatementOutcome;
#[cfg(test)]
use crate::session::value::value_to_element;

/// Checked cast: LMDB stores row counts as `u64`; `Element::integer` takes
/// `i64`. A count that exceeds `i64::MAX` is reported as an I/O error rather
/// than silently wrapping into a negative value or being clamped.
fn checked_u64_to_i64(v: u64) -> io::Result<i64> {
    i64::try_from(v).map_err(|_| io::Error::other("row count exceeds i64::MAX"))
}

/// Envelope for a non-query outcome (CTAS / INSERT / CREATE TABLE).
pub(super) fn write_outcome_ion(outcome: &StatementOutcome, out: &mut dyn Write) -> io::Result<()> {
    let element = outcome_to_element(outcome)?;
    let text = element
        .to_text(TextKind::Compact)
        .map_err(|e| io::Error::other(e.to_string()))?;
    let escaped = escape_control_chars_in_strings(&text);
    writeln!(out, "{escaped}")?;
    out.flush()
}

fn outcome_to_element(outcome: &StatementOutcome) -> io::Result<Element> {
    match outcome {
        StatementOutcome::InsertInto { rows, .. } => Ok(Element::from(
            Struct::builder()
                .with_field(
                    "affected_rows",
                    Element::integer(checked_u64_to_i64(*rows)?),
                )
                .build(),
        )),
        StatementOutcome::CreateTable { canonical_key, .. } => {
            created_table_element(canonical_key, 0)
        }
        StatementOutcome::CreateTableAs {
            canonical_key,
            rows,
            ..
        } => created_table_element(canonical_key, *rows),
    }
}

/// Inside `"..."` string literals and `'...'` symbol literals (both can carry
/// raw control chars past ion_rs 0.18.1's writer), replace C0 controls, DEL,
/// and C1 bytes with `\xNN` escapes. Tracks whichever delimiter opened the
/// current region so a `'` inside a `"..."` is data.
pub(crate) fn escape_control_chars_in_strings(text: &str) -> String {
    let mut out = String::with_capacity(text.len());
    let mut delim: Option<char> = None;
    let mut prev_backslash = false;
    for ch in text.chars() {
        match delim {
            None => {
                out.push(ch);
                if ch == '"' || ch == '\'' {
                    delim = Some(ch);
                    prev_backslash = false;
                }
            }
            Some(d) => {
                if prev_backslash {
                    out.push(ch);
                    prev_backslash = false;
                    continue;
                }
                if ch == '\\' {
                    out.push(ch);
                    prev_backslash = true;
                    continue;
                }
                if ch == d {
                    out.push(ch);
                    delim = None;
                    continue;
                }
                let code = ch as u32;
                // Tab/LF/CR are already escaped by the writer; other C0, DEL,
                // and C1 (0x80..=0x9F) must be escaped here.
                let is_c0 = code < 0x20 && code != 0x09 && code != 0x0a && code != 0x0d;
                let is_del_or_c1 = code == 0x7f || (0x80..=0x9f).contains(&code);
                if is_c0 || is_del_or_c1 {
                    use std::fmt::Write as _;
                    let _ = write!(&mut out, "\\x{:02x}", code);
                } else {
                    out.push(ch);
                }
            }
        }
    }
    out
}

/// `{ name: [<canonical_key>], rows: N }` — `name` is a single-element list
/// mirroring `_tables`.
fn created_table_element(canonical_key: &str, rows: u64) -> io::Result<Element> {
    let name = Element::from(
        Sequence::builder()
            .push(Element::string(canonical_key))
            .build_list(),
    );
    let inner = Struct::builder()
        .with_field("name", name)
        .with_field("rows", Element::integer(checked_u64_to_i64(rows)?))
        .build();
    Ok(Element::from(
        Struct::builder().with_field("created_table", inner).build(),
    ))
}

/// Test helper: renders one Value through the same escape pass as streaming.
#[cfg(test)]
pub(crate) fn render_element_escaped(v: &partiql_value::Value) -> String {
    let element = value_to_element(v).expect("test value must convert");
    let text = element
        .to_text(TextKind::Compact)
        .expect("test element must render");
    escape_control_chars_in_strings(&text)
}

#[cfg(test)]
mod tests {
    use super::*;
    use ion_rs::element::Element;
    use partiql_value::Value;

    /// Emit a `{rows: $bag::[<element>]}` envelope by hand for a single value,
    /// exercising the same escape pass the streaming write path uses.
    fn envelope_bag_one(v: &Value) -> Vec<u8> {
        let inner = render_element_escaped(v);
        format!("{{rows: $bag::[{inner}]}}\n").into_bytes()
    }

    #[test]
    fn control_chars_escaped_and_reparseable() {
        // ESC (0x1B) and NUL (0x00) inside a string body.
        let bytes = envelope_bag_one(&Value::String(Box::new("x\u{1b}y\u{00}z".to_string())));
        // Round-trip: the emitted bytes must reparse.
        Element::read_all(&bytes).expect("output must reparse");
    }

    #[test]
    fn control_char_in_field_name_escapes_via_single_quotes() {
        // A field name containing a C0 control forces ion_rs' text writer to
        // single-quote it; the escape scanner must escape symbol bodies the
        // same way as string bodies.
        let mut tuple = partiql_value::Tuple::new();
        tuple.insert("x\u{1b}y", Value::Integer(1));
        let bytes = envelope_bag_one(&Value::Tuple(Box::new(tuple)));
        Element::read_all(&bytes).expect("output with escaped symbol must reparse");
        assert!(!bytes.contains(&0x1b), "raw ESC byte leaked into output");
    }

    #[test]
    fn blob_value_reparses_as_blob() {
        // Query row of type Bytes decodes to Value::Blob; render must emit
        // valid Ion blob syntax, not error.
        let bytes = envelope_bag_one(&Value::Blob(Box::new(vec![0x00, 0x1b, 0xff])));
        let parsed = Element::read_all(&bytes).expect("blob output must reparse");
        assert_eq!(parsed.len(), 1);
        // Envelope is `{ rows: $bag::[ <blob> ] }` — dig in and confirm.
        let rows = parsed[0]
            .as_struct()
            .and_then(|s| s.get("rows"))
            .expect("envelope has rows field");
        let list = rows.as_sequence().expect("rows is a sequence");
        let blob = list.elements().next().expect("one row");
        assert_eq!(blob.ion_type(), ion_rs::IonType::Blob);
    }

    #[test]
    fn ordinary_strings_unchanged_by_escape_pass() {
        let bytes = envelope_bag_one(&Value::String(Box::new("hello world".to_string())));
        let text = std::str::from_utf8(&bytes).unwrap();
        assert!(text.contains("\"hello world\""), "got: {text}");
    }
}
