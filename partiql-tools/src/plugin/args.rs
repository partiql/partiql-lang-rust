//! Table-function arguments: register values → `PqliteArg`.
//!
//! Scalars cross directly. Containers and decimals are rendered as Ion text so
//! plugins can parse them with any Ion library and never see PartiQL types.

use std::fmt::Write as _;

use partiql_vm::value::{RegisterReader, ValueType, ValueView};
use partiql_vm::{EngineError, Result};

use super::ffi::{self, PqliteArg, PqliteArgValue, PqliteStr};

/// An argument copied out of the register bank (borrowed strings there are
/// only valid during `TableFunction::create`).
#[derive(Debug, Clone, PartialEq)]
pub(crate) enum OwnedArg {
    Null,
    Missing,
    Bool(bool),
    Int(i64),
    Float(f64),
    Str(String),
    Bytes(Vec<u8>),
    IonText(String),
}

impl OwnedArg {
    /// Borrowing FFI view; valid while `self` is alive and unmoved.
    pub(crate) fn as_ffi(&self) -> PqliteArg {
        let (kind, v) = match self {
            OwnedArg::Null => (ffi::PQLITE_ARG_NULL, PqliteArgValue { i: 0 }),
            OwnedArg::Missing => (ffi::PQLITE_ARG_MISSING, PqliteArgValue { i: 0 }),
            OwnedArg::Bool(b) => (ffi::PQLITE_ARG_BOOL, PqliteArgValue { b: *b as i32 }),
            OwnedArg::Int(i) => (ffi::PQLITE_ARG_INT, PqliteArgValue { i: *i }),
            OwnedArg::Float(f) => (ffi::PQLITE_ARG_FLOAT, PqliteArgValue { f: *f }),
            OwnedArg::Str(s) => (
                ffi::PQLITE_ARG_STRING,
                PqliteArgValue {
                    s: PqliteStr::new(s),
                },
            ),
            OwnedArg::Bytes(b) => (
                ffi::PQLITE_ARG_BYTES,
                PqliteArgValue {
                    s: PqliteStr::bytes(b),
                },
            ),
            OwnedArg::IonText(s) => (
                ffi::PQLITE_ARG_ION_TEXT,
                PqliteArgValue {
                    s: PqliteStr::new(s),
                },
            ),
        };
        PqliteArg { kind, v }
    }
}

pub(crate) fn read_args(reader: &RegisterReader<'_>, slots: &[u16]) -> Result<Vec<OwnedArg>> {
    slots
        .iter()
        .map(|&slot| {
            let mut view = reader.get_value_view(slot as usize).ok_or_else(|| {
                EngineError::ReaderError(format!("argument slot {slot} out of bounds"))
            })?;
            read_arg(&mut view)
        })
        .collect()
}

fn read_arg(view: &mut ValueView<'_>) -> Result<OwnedArg> {
    Ok(match view.get_type() {
        ValueType::Missing => OwnedArg::Missing,
        ValueType::Null => OwnedArg::Null,
        ValueType::Bool => OwnedArg::Bool(view.get_bool()?),
        ValueType::Integer => OwnedArg::Int(view.get_i64()?),
        ValueType::Float => OwnedArg::Float(view.get_f64()?),
        ValueType::String => OwnedArg::Str(view.get_str()?.to_string()),
        ValueType::Bytes => OwnedArg::Bytes(view.get_bytes()?.to_vec()),
        ValueType::Decimal | ValueType::Tuple | ValueType::List | ValueType::Bag => {
            let mut out = String::new();
            write_ion(view, &mut out)?;
            OwnedArg::IonText(out)
        }
    })
}

/// Render the value under the cursor as Ion text. Bags become `$bag::[...]`
/// and MISSING `$missing::null`, matching PartiQL's Ion encoding.
fn write_ion(view: &mut ValueView<'_>, out: &mut String) -> Result<()> {
    match view.get_type() {
        ValueType::Missing => out.push_str("$missing::null"),
        ValueType::Null => out.push_str("null"),
        ValueType::Bool => out.push_str(if view.get_bool()? { "true" } else { "false" }),
        ValueType::Integer => {
            let _ = write!(out, "{}", view.get_i64()?);
        }
        ValueType::Float => write_ion_float(view.get_f64()?, out),
        ValueType::Decimal => {
            let d = view.get_decimal()?;
            // `1.50` is an Ion decimal; integral values need an explicit `d0`.
            let s = d.to_string();
            out.push_str(&s);
            if !s.contains('.') {
                out.push_str("d0");
            }
        }
        ValueType::String => write_ion_string(view.get_str()?, out),
        ValueType::Bytes => write_ion_blob(view.get_bytes()?, out),
        ty @ (ValueType::Tuple | ValueType::List | ValueType::Bag) => {
            let (open, close) = match ty {
                ValueType::Tuple => ("{", "}"),
                ValueType::List => ("[", "]"),
                _ => ("$bag::[", "]"),
            };
            out.push_str(open);
            // `step_in` rejects empty containers, which is the only way to
            // tell an empty one apart from here.
            if view.step_in().is_ok() {
                let mut first = true;
                loop {
                    if !first {
                        out.push_str(", ");
                    }
                    first = false;
                    if ty == ValueType::Tuple {
                        write_ion_string(view.get_field_name()?, out);
                        out.push_str(": ");
                    }
                    write_ion(view, out)?;
                    if !view.advance()? {
                        break;
                    }
                }
                view.step_out()?;
            }
            out.push_str(close);
        }
    }
    Ok(())
}

fn write_ion_float(f: f64, out: &mut String) {
    if f.is_nan() {
        out.push_str("nan");
    } else if f.is_infinite() {
        out.push_str(if f > 0.0 { "+inf" } else { "-inf" });
    } else {
        // `{:e}` always carries an exponent, which makes it an Ion float.
        let _ = write!(out, "{f:e}");
    }
}

fn write_ion_string(s: &str, out: &mut String) {
    out.push('"');
    for c in s.chars() {
        match c {
            '"' => out.push_str("\\\""),
            '\\' => out.push_str("\\\\"),
            '\n' => out.push_str("\\n"),
            '\r' => out.push_str("\\r"),
            '\t' => out.push_str("\\t"),
            c if (c as u32) < 0x20 => {
                let _ = write!(out, "\\x{:02x}", c as u32);
            }
            c => out.push(c),
        }
    }
    out.push('"');
}

fn write_ion_blob(b: &[u8], out: &mut String) {
    const ALPHABET: &[u8; 64] = b"ABCDEFGHIJKLMNOPQRSTUVWXYZabcdefghijklmnopqrstuvwxyz0123456789+/";
    out.push_str("{{");
    for chunk in b.chunks(3) {
        let n = chunk.len();
        let v = (chunk[0] as u32) << 16
            | (*chunk.get(1).unwrap_or(&0) as u32) << 8
            | *chunk.get(2).unwrap_or(&0) as u32;
        for i in 0..4 {
            if i <= n {
                out.push(ALPHABET[(v >> (18 - 6 * i) & 0x3f) as usize] as char);
            } else {
                out.push('=');
            }
        }
    }
    out.push_str("}}");
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn blob_is_base64() {
        let mut s = String::new();
        write_ion_blob(b"hello", &mut s);
        assert_eq!(s, "{{aGVsbG8=}}");
        s.clear();
        write_ion_blob(b"", &mut s);
        assert_eq!(s, "{{}}");
    }

    #[test]
    fn floats_and_strings() {
        let mut s = String::new();
        write_ion_float(1.5, &mut s);
        write_ion_string("a\"b\n", &mut s);
        assert_eq!(s, "1.5e0\"a\\\"b\\n\"");
    }
}
