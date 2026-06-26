//! Custom tagged-union row serialization for the CTAS write path.
//!
//! Operates on the VM's RegisterReader + RowShape surface, no `partiql_value::Value`
//! materialization. The CTAS arm in `pqlite.rs` calls `serialize_row` and passes
//! the encoded bytes to `TableWriter::push_row` — zero per-row allocations after
//! warmup.
//!
//! Wire format:
//!   ROW          = tagged_value
//!   tagged_value = tag(u8) payload
//!   payload(TAG_TUPLE) = field_count(u32 LE) (name_len(u32 LE) name_utf8 tagged_value){N}
//!
//! All multi-byte VALUE bytes are little-endian. Row-id KEYS in LMDB stay
//! big-endian: storage encodes them as `row_id.to_be_bytes()` under a
//! `heed::types::Bytes` key codec so the B+tree's lexicographic key order
//! matches numeric order with zero per-row key allocations.

use partiql_eval::value::{FieldName, RegisterReader, RowShape, ValueType};

/// Sanity cap on `field_count` consumed by the reader. Real CTAS rows top
/// out at hundreds; 1024 is ample headroom while catching a flipped byte
/// producing a huge u32 before `Vec::with_capacity` allocates gigabytes.
pub const MAX_FIELDS_PER_ROW: u32 = 1024;

/// Sanity cap on `name_len` consumed by the reader. PartiQL identifiers are
/// typically <100 bytes; 1 MiB catches corruption while staying well above
/// any real-world name. UTF-8 byte length, not char count.
pub const MAX_NAME_LEN_BYTES: u32 = 1024 * 1024;

// Tag constants. `pub` so the reader and encoder share one source of truth.
pub const TAG_INTEGER: u8 = 0x00;
pub const TAG_DECIMAL: u8 = 0x01;
pub const TAG_FLOAT: u8 = 0x02;
pub const TAG_TUPLE: u8 = 0x03;
// 0x04 = TAG_BAG     — reserved; rejected.
pub const TAG_STRING: u8 = 0x05;
pub const TAG_NULL: u8 = 0x06;
// 0x07 = TAG_MISSING — reserved; rejected.
// 0x08 = TAG_BOOL    — reserved; rejected.
// 0x09 = TAG_BYTES   — reserved; rejected.

/// Errors raised by [`serialize_row`]. `pub` because the binary in `src/bin/`
/// is a separate crate from this library.
#[derive(Debug)]
pub enum SerializeError {
    /// An unsupported type or shape encountered mid-stream: containers,
    /// dynamic column names, MISSING, BOOL, BYTES. The string identifies
    /// the offending column or top-level value.
    Unsupported(String),
}

impl std::fmt::Display for SerializeError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            SerializeError::Unsupported(m) => write!(f, "unsupported: {m}"),
        }
    }
}

impl std::error::Error for SerializeError {}

/// Serialize one VM row into `buf` as a tagged-union byte sequence.
///
/// `buf` is cleared on entry. On success, `buf` contains a single
/// `tagged_value`: one tag byte followed by its payload. A top-level tuple
/// is TAG_TUPLE + struct body; a top-level scalar is its scalar tag +
/// value bytes.
///
/// All integer fields are little-endian. The row-id key in LMDB stays
/// big-endian under `heed::types::Bytes`. Only VALUE bytes are LE.
///
/// # Endianness invariant
///
/// Every multi-byte field is written `to_le_bytes()`. The test parser in
/// `partiql-tools/tests/pqlite_cli.rs::parse_row` uses `from_le_bytes()` to
/// match. If you add a new tag, write it `to_le_bytes()` here AND read it
/// `from_le_bytes()` there — an LE/BE asymmetry compiles cleanly and
/// silently corrupts every persisted row.
pub fn serialize_row(
    row: &RegisterReader<'_>,
    row_shape: &RowShape,
    buf: &mut Vec<u8>,
) -> Result<(), SerializeError> {
    buf.clear();
    match row_shape {
        RowShape::Struct(fields) => write_tuple(row, fields, buf),
        RowShape::Register(slot, _) => write_value(row, *slot, None, buf),
    }
}

fn write_tuple(
    row: &RegisterReader<'_>,
    fields: &[partiql_eval::value::FieldShape],
    buf: &mut Vec<u8>,
) -> Result<(), SerializeError> {
    if fields.len() > MAX_FIELDS_PER_ROW as usize {
        return Err(SerializeError::Unsupported(format!(
            "tuple has {} fields, exceeds MAX_FIELDS_PER_ROW ({})",
            fields.len(),
            MAX_FIELDS_PER_ROW
        )));
    }
    buf.push(TAG_TUPLE);
    // Safe: the cap above bounds fields.len() to MAX_FIELDS_PER_ROW (u32).
    let field_count: u32 = fields.len() as u32;
    buf.extend_from_slice(&field_count.to_le_bytes());
    for (col, field) in fields.iter().enumerate() {
        let name = match &field.name {
            FieldName::Static(s) => s.as_str(),
            FieldName::Register(_) => {
                return Err(SerializeError::Unsupported(format!(
                    "field {col}: dynamic field names are not supported yet"
                )));
            }
        };
        if name.len() > MAX_NAME_LEN_BYTES as usize {
            return Err(SerializeError::Unsupported(format!(
                "field {col}: name length {} bytes exceeds MAX_NAME_LEN_BYTES ({})",
                name.len(),
                MAX_NAME_LEN_BYTES
            )));
        }
        // Safe: the cap above bounds name.len() to MAX_NAME_LEN_BYTES (u32).
        let name_len: u32 = name.len() as u32;
        buf.extend_from_slice(&name_len.to_le_bytes());
        buf.extend_from_slice(name.as_bytes());
        match &field.value {
            RowShape::Register(idx, _) => write_value(row, *idx, Some(name), buf)?,
            RowShape::Struct(nested_fields) => write_tuple(row, nested_fields, buf)?,
        }
    }
    Ok(())
}

/// Write `tag + payload` for the value at register `slot`. Unsupported types
/// return `Err`; the caller propagates and the in-flight wtxn rolls back.
fn write_value(
    row: &RegisterReader<'_>,
    slot: usize,
    field_name: Option<&str>,
    buf: &mut Vec<u8>,
) -> Result<(), SerializeError> {
    let view = row.get_value_view(slot).expect("register slot from shape");
    match view.get_type() {
        ValueType::Null => {
            buf.push(TAG_NULL);
        }
        ValueType::Integer => {
            buf.push(TAG_INTEGER);
            buf.extend_from_slice(&view.get_i64().expect("i64 view").to_le_bytes());
        }
        ValueType::Float => {
            buf.push(TAG_FLOAT);
            buf.extend_from_slice(&view.get_f64().expect("f64 view").to_le_bytes());
        }
        ValueType::Decimal => {
            buf.push(TAG_DECIMAL);
            let d = view.get_decimal().expect("decimal view");
            // rust_decimal: scale() is u32 in 0..=28; the i32 cast is lossless.
            let scale_i32: i32 = d.scale() as i32;
            buf.extend_from_slice(&scale_i32.to_le_bytes());
            buf.extend_from_slice(&d.mantissa().to_le_bytes());
        }
        ValueType::String => {
            buf.push(TAG_STRING);
            let s = view.get_str().expect("string view");
            let sb = s.as_bytes();
            let str_len: u32 = sb.len().try_into().expect("str_len exceeds u32::MAX");
            buf.extend_from_slice(&str_len.to_le_bytes());
            buf.extend_from_slice(sb);
        }
        ty @ (ValueType::Bool
        | ValueType::Bytes
        | ValueType::Missing
        | ValueType::Tuple
        | ValueType::List
        | ValueType::Bag) => {
            let prefix = match field_name {
                Some(name) => format!("field '{name}'"),
                None => "top-level value".to_string(),
            };
            return Err(SerializeError::Unsupported(format!(
                "{prefix}: {ty:?} is not yet supported"
            )));
        }
    }
    Ok(())
}
