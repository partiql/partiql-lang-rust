//! Custom tagged-union row serialization for the CTAS write path.
//!
//! Operates on the VM's RegisterReader + RowShape surface, no `partiql_value::Value`
//! materialization. The CTAS arm in `pqlite.rs` calls `serialize_row` and passes
//! the encoded bytes to `TableWriter::push_row` — zero per-row allocations after
//! warmup.
//!
//! Wire format:
//!   ROW   = ver(u8=0x01) tagged_value
//!   tagged_value = tag(u8) payload
//!   payload(TAG_STRUCT) = field_count(u32 LE) (name_len(u32 LE) name_utf8 tagged_value){N}
//!
//! All multi-byte VALUE bytes are little-endian. Row-id KEYS in LMDB stay
//! big-endian: storage encodes them as `row_id.to_be_bytes()` under a
//! `heed::types::Bytes` key codec so the B+tree's lexicographic key order
//! matches numeric order with zero per-row key allocations.

use partiql_eval::value::{FieldName, RegisterReader, RowShape, ValueType};

/// Format-version byte gating the entire wire grammar (not just per-tag
/// interpretation). Readers must refuse unknown versions. Currently 0x01:
/// `struct_body = field_count + (name_len + name + tagged_value){N}`.
pub const FORMAT_VERSION: u8 = 0x01;

/// Maximum columns per row. Real CTAS rows top out at hundreds; 1024 is
/// ample headroom while catching corruption (a flipped byte producing a
/// huge u32 field_count) before `Vec::with_capacity` allocates gigabytes.
pub const MAX_FIELDS_PER_ROW: u32 = 1024;

/// Maximum byte length of a column name. PartiQL identifiers are typically
/// <100 bytes; 1 MiB catches corruption while staying well above any
/// real-world name. The cap applies to UTF-8 byte length, not char count.
pub const MAX_NAME_LEN_BYTES: u32 = 1024 * 1024;

// Tag constants. `pub` so the reader and encoder share one source of truth.
pub const TAG_INTEGER: u8 = 0x00;
pub const TAG_DECIMAL: u8 = 0x01;
pub const TAG_FLOAT: u8 = 0x02;
pub const TAG_STRUCT: u8 = 0x03;
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
    /// An unsupported type or shape: containers, dynamic column names,
    /// MISSING, BOOL, BYTES, nested struct values, empty column names. The
    /// string identifies the offending column or top-level value.
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
/// `buf` is cleared on entry. On success, `buf` contains:
///   * `FORMAT_VERSION` (1 byte)
///   * top-level tag (1 byte: TAG_STRUCT for struct rows, or a scalar tag)
///   * payload for that tag (struct_body for TAG_STRUCT, value bytes for scalars)
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
        RowShape::Struct(fields) => write_struct_row(row, fields, buf),
        RowShape::Register(slot, _) => write_scalar_row(row, *slot, buf),
    }
}

fn write_struct_row(
    row: &RegisterReader<'_>,
    fields: &[partiql_eval::value::FieldShape],
    buf: &mut Vec<u8>,
) -> Result<(), SerializeError> {
    if fields.len() > MAX_FIELDS_PER_ROW as usize {
        return Err(SerializeError::Unsupported(format!(
            "row has {} columns, exceeds MAX_FIELDS_PER_ROW ({})",
            fields.len(),
            MAX_FIELDS_PER_ROW
        )));
    }
    // Pre-flight: validate every column before pushing any payload bytes.
    for (col, field) in fields.iter().enumerate() {
        let name = match &field.name {
            FieldName::Static(s) if !s.is_empty() => s.as_str(),
            FieldName::Static(_) => {
                return Err(SerializeError::Unsupported(format!(
                    "column {col}: empty column name not supported"
                )));
            }
            FieldName::Register(_) => {
                return Err(SerializeError::Unsupported(format!(
                    "column {col}: dynamic column names are not supported yet"
                )));
            }
        };
        if name.len() > MAX_NAME_LEN_BYTES as usize {
            return Err(SerializeError::Unsupported(format!(
                "column {col}: name length {} bytes exceeds MAX_NAME_LEN_BYTES ({})",
                name.len(),
                MAX_NAME_LEN_BYTES
            )));
        }
        // Duplicate-name check: bounded by MAX_FIELDS_PER_ROW so the O(K²)
        // is acceptable.
        for prior in &fields[..col] {
            if let FieldName::Static(prior_name) = &prior.name {
                if prior_name == name {
                    return Err(SerializeError::Unsupported(format!(
                        "column {col}: duplicate column name '{name}' (already used)"
                    )));
                }
            }
        }
        let slot = match &field.value {
            RowShape::Register(idx, _) => *idx,
            RowShape::Struct(_) => {
                return Err(SerializeError::Unsupported(format!(
                    "column '{name}': nested struct values are not supported yet"
                )));
            }
        };
        let view = row.get_value_view(slot).expect("register slot from shape");
        validate_value_type(view.get_type(), Some(name))?;
    }

    // Pre-flight passed: write the row.
    buf.push(FORMAT_VERSION);
    buf.push(TAG_STRUCT);
    let field_count: u32 = fields
        .len()
        .try_into()
        .expect("field_count exceeds u32::MAX");
    buf.extend_from_slice(&field_count.to_le_bytes());
    for field in fields.iter() {
        let name = match &field.name {
            FieldName::Static(s) => s.as_str(),
            FieldName::Register(_) => unreachable!("pre-flight rejects FieldName::Register"),
        };
        let slot = match &field.value {
            RowShape::Register(idx, _) => *idx,
            RowShape::Struct(_) => unreachable!("pre-flight rejects nested-struct field values"),
        };
        let name_bytes = name.as_bytes();
        let name_len: u32 = name_bytes
            .len()
            .try_into()
            .expect("name_len exceeds u32::MAX");
        buf.extend_from_slice(&name_len.to_le_bytes());
        buf.extend_from_slice(name_bytes);
        write_tagged_value(row, slot, buf);
    }
    Ok(())
}

fn write_scalar_row(
    row: &RegisterReader<'_>,
    slot: usize,
    buf: &mut Vec<u8>,
) -> Result<(), SerializeError> {
    let view = row
        .get_value_view(slot)
        .expect("register slot from shape must exist in the row");
    validate_value_type(view.get_type(), None)?;
    buf.push(FORMAT_VERSION);
    write_tagged_value(row, slot, buf);
    Ok(())
}

/// Reject unsupported types. `col_name` is `Some(name)` for a struct-field
/// walk and `None` for a top-level scalar row; the error wording branches
/// accordingly.
fn validate_value_type(ty: ValueType, col_name: Option<&str>) -> Result<(), SerializeError> {
    let prefix = match col_name {
        Some(name) => format!("column '{name}'"),
        None => "top-level value".to_string(),
    };
    match ty {
        ValueType::Null
        | ValueType::Integer
        | ValueType::Float
        | ValueType::Decimal
        | ValueType::String => Ok(()),
        ValueType::Bool => Err(SerializeError::Unsupported(format!(
            "{prefix}: Bool is not yet supported"
        ))),
        ValueType::Bytes => Err(SerializeError::Unsupported(format!(
            "{prefix}: Bytes is not yet supported"
        ))),
        ValueType::Missing => Err(SerializeError::Unsupported(format!(
            "{prefix}: MISSING is not yet supported"
        ))),
        ValueType::Tuple | ValueType::List | ValueType::Bag => Err(SerializeError::Unsupported(
            format!("{prefix}: container type ({ty:?}) is not yet supported"),
        )),
    }
}

/// Write `tag + payload` for the value at register `slot`. The caller is
/// responsible for having pre-flight-validated the type.
fn write_tagged_value(row: &RegisterReader<'_>, slot: usize, buf: &mut Vec<u8>) {
    let view = row
        .get_value_view(slot)
        .expect("register slot from shape must exist in the row");
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
        ValueType::Bool
        | ValueType::Bytes
        | ValueType::Missing
        | ValueType::Tuple
        | ValueType::List
        | ValueType::Bag => unreachable!("pre-flight rejects all reserved/container types"),
    }
}
