//! Custom tagged-union row serialization for the CTAS write path.
//!
//! Operates on the VM's [`RegisterReader`] + [`RowShape`] surface (not on
//! `partiql_value::Value`). The CTAS arm in `pqlite.rs` calls
//! [`serialize_row`] inside the driver closure passed to
//! `HeedDB::create_table_from_rows`, then hands the resulting bytes to
//! storage via `push_row(&buf)` — zero per-row allocations after warmup.
//!
//! Wire format:
//!   ROW = ver(u8=0x01) struct_tag(u8=0x03) struct_body
//!   struct_body = field_count(u32 LE)
//!                 ( name_len(u32 LE) name_utf8 tagged_value ){field_count}
//!   tagged_value = tag(u8) payload
//!
//! VALUE bytes are little-endian. The row-id KEY in LMDB stays big-endian:
//! storage encodes it as `row_id.to_be_bytes()` and stores it under a
//! `heed::types::Bytes` key codec, so the B+tree's lexicographic key order
//! matches numeric order with zero per-row key allocations.

use partiql_eval::value::{FieldName, RegisterReader, RowShape, ValueType};

/// Format-version byte. PR 3 = 0x01. The byte gates the ENTIRE struct_body
/// grammar, not just per-tag interpretation. Under 0x01, struct_body is
/// `field_count + (name_len + name + tagged_value){N}`. A future PR that
/// adds a schema registry will bump to 0x02 and drop per-row field names;
/// readers refuse unknown versions.
pub const FORMAT_VERSION: u8 = 0x01;

/// Maximum columns per row. Real CTAS rows top out at hundreds; 1024 is
/// ample headroom while catching corruption (a flipped byte producing a
/// huge u32 field_count) before `Vec::with_capacity` allocates gigabytes.
pub const MAX_FIELDS_PER_ROW: u32 = 1024;

/// Maximum byte length of a column name. PartiQL identifiers are typically
/// <100 bytes; 1 MiB catches corruption while staying well above any
/// real-world name. The cap applies to UTF-8 byte length, not char count.
pub const MAX_NAME_LEN_BYTES: u32 = 1024 * 1024;

// Tag constants. `pub` so PR 4's reader imports the same source of truth
// as the encoder — see the LE/BE invariant in the function docstring; the
// same drift risk applies to tag-byte numerals.
pub const TAG_INTEGER: u8 = 0x00;
pub const TAG_DECIMAL: u8 = 0x01;
pub const TAG_FLOAT: u8 = 0x02;
pub const TAG_STRUCT: u8 = 0x03;
// 0x04 = TAG_BAG — reserved; PR 3 rejects, encoder write path lands later.
pub const TAG_STRING: u8 = 0x05;
pub const TAG_NULL: u8 = 0x06;
// 0x07 = TAG_MISSING — reserved; PR 3 rejects.
// 0x08 = TAG_BOOL    — reserved; PR 3 rejects.
// 0x09 = TAG_BYTES   — reserved; PR 3 rejects.

/// Errors raised by [`serialize_row`].
///
/// One variant by design: PR 3 lumps "type not supported yet" and "shape not
/// supported yet" under `Unsupported(String)` because every consumer flattens
/// them through `Display` at the CTAS-arm boundary. Richer discrimination
/// lands when a non-pqlite consumer needs it. `storage.rs` never sees this
/// type.
///
/// `pub` (not `pub(crate)`) because the `pqlite` binary in `src/bin/` is a
/// separate crate from the library; same for [`serialize_row`] and the
/// `row_codec` module declaration in `lib.rs`.
#[derive(Debug)]
pub enum SerializeError {
    /// A type or shape PR 3 doesn't implement (containers, dynamic column
    /// names, MISSING, BOOL, BYTES, nested struct values, empty column
    /// names, bare-scalar rows). The string names the offending column.
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
///   * `TAG_STRUCT` (1 byte)
///   * struct body: u32 LE field_count, then per field
///     (u32 LE name_len + UTF-8 name + tag byte + value payload)
///
/// All integer fields are little-endian. The row-id key in LMDB stays
/// big-endian: storage writes it as `row_id.to_be_bytes()` under a
/// `heed::types::Bytes` key codec. Only VALUE bytes are LE.
///
/// # Rejection contract
///
/// Walks the row shape and peeks `view.get_type()` for every column BEFORE
/// pushing any bytes. Returns `Err(Unsupported)` immediately if any column
/// trips the reject set (Tuple, List, Bag, Missing, Bool, Bytes) or if the
/// shape is structurally unsupported (top-level scalar, dynamic field name,
/// nested struct value, empty field name). `buf` is left empty (already
/// cleared) on any rejection.
///
/// # Endianness invariant
///
/// Every multi-byte field (`field_count`, `name_len`, `i64`, `f64`, `i32`
/// scale, `i128` mantissa, `u32` string length) is written via
/// `to_le_bytes()`. The corresponding test parser in
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

    // PR 3 supports exactly RowShape::Struct at the top of a row. A bare
    // scalar at the top has no column name and is rejected with an
    // actionable message; the user can wrap the projection (`AS <name>`).
    let fields = match row_shape {
        RowShape::Struct(fs) => fs,
        RowShape::Register(_, _) => {
            return Err(SerializeError::Unsupported(
                "CTAS source must produce a struct; got a bare scalar (add an AS alias)".into(),
            ));
        }
    };

    // Row-shape cap: catches a corrupt/oversize column count before the
    // per-field walk allocates anything proportional to the count.
    if fields.len() > MAX_FIELDS_PER_ROW as usize {
        return Err(SerializeError::Unsupported(format!(
            "row has {} columns, exceeds MAX_FIELDS_PER_ROW ({})",
            fields.len(),
            MAX_FIELDS_PER_ROW
        )));
    }

    // ── Pre-flight: validate the whole row before writing any bytes ─────────
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
                    "column {col}: dynamic name (FieldName::Register) not supported in PR 3"
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
        // Duplicate-name check: PR 4's reader and PR 5's schema registry
        // both need column names to be a unique key. Quadratic in column
        // count, but the count is bounded by MAX_FIELDS_PER_ROW above.
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
                    "column '{name}': nested struct values not supported in PR 3"
                )));
            }
        };
        // The slot index was emitted by the same compiler pass that built
        // this register file; a None here is a planner/VM invariant
        // violation, not a recoverable runtime condition.
        let view = row
            .get_value_view(slot)
            .expect("register slot from shape must exist in the row");
        match view.get_type() {
            ValueType::Null
            | ValueType::Integer
            | ValueType::Float
            | ValueType::Decimal
            | ValueType::String => {}
            ValueType::Bool => {
                return Err(SerializeError::Unsupported(format!(
                    "column '{name}': Bool — tag reserved, write path lands in a later PR"
                )));
            }
            ValueType::Bytes => {
                return Err(SerializeError::Unsupported(format!(
                    "column '{name}': Bytes — tag reserved, write path lands in a later PR"
                )));
            }
            ValueType::Missing => {
                return Err(SerializeError::Unsupported(format!(
                    "column '{name}': MISSING wire encoding deferred to the deserializer PR"
                )));
            }
            ValueType::Tuple | ValueType::List | ValueType::Bag => {
                return Err(SerializeError::Unsupported(format!(
                    "column '{name}': container types ({:?}) land in PR 5",
                    view.get_type()
                )));
            }
        }
    }

    // ── Pre-flight passed: write the row ────────────────────────────────────
    buf.push(FORMAT_VERSION);
    buf.push(TAG_STRUCT);
    // Lengths and counts use `try_into().expect(...)` rather than `as u32`
    // throughout serialize_row: silent truncation on >u32::MAX inputs would
    // corrupt the row format (the reader would parse the truncated length
    // and read garbage thereafter). Panic with a pointed message instead;
    // this layer has no recovery anyway — the row's already materialized in
    // VM registers.
    let field_count: u32 = fields.len().try_into().expect(
        "row has more than u32::MAX columns — wire format requires field_count fits in u32",
    );
    buf.extend_from_slice(&field_count.to_le_bytes());

    for field in fields.iter() {
        // SAFETY of unwrap: pre-flight validated FieldName::Static(non-empty)
        // and RowShape::Register for every field.
        let name = match &field.name {
            FieldName::Static(s) => s.as_str(),
            FieldName::Register(_) => unreachable!("pre-flight rejects FieldName::Register"),
        };
        let slot = match &field.value {
            RowShape::Register(idx, _) => *idx,
            RowShape::Struct(_) => unreachable!("pre-flight rejects nested-struct field values"),
        };

        // Write the field name: u32 LE length + raw UTF-8 (no terminator).
        let name_bytes = name.as_bytes();
        let name_len: u32 = name_bytes
            .len()
            .try_into()
            .expect("column name longer than u32::MAX bytes");
        buf.extend_from_slice(&name_len.to_le_bytes());
        buf.extend_from_slice(name_bytes);

        let view = row
            .get_value_view(slot)
            .expect("register slot from shape must exist in the row");
        match view.get_type() {
            ValueType::Null => {
                buf.push(TAG_NULL);
            }
            ValueType::Integer => {
                buf.push(TAG_INTEGER);
                buf.extend_from_slice(&view.get_i64().unwrap().to_le_bytes());
            }
            ValueType::Float => {
                buf.push(TAG_FLOAT);
                buf.extend_from_slice(&view.get_f64().unwrap().to_le_bytes());
            }
            ValueType::Decimal => {
                buf.push(TAG_DECIMAL);
                let d = view.get_decimal().unwrap();
                // rust_decimal: scale() -> u32 with documented range 0..=28
                // across all 1.x; mantissa() -> i128 (widened from native
                // 96-bit; upper 32 bits sign-extended). Wire format per the
                // spec: 4-byte i32 LE scale + 16-byte i128 LE mantissa.
                // The `as i32` cast is provably safe (scale ≤ 28 fits in i32).
                let scale_i32: i32 = d.scale() as i32;
                buf.extend_from_slice(&scale_i32.to_le_bytes());
                buf.extend_from_slice(&d.mantissa().to_le_bytes());
            }
            ValueType::String => {
                buf.push(TAG_STRING);
                let s = view.get_str().unwrap();
                let sb = s.as_bytes();
                let str_len: u32 = sb
                    .len()
                    .try_into()
                    .expect("string column value longer than u32::MAX bytes — wire format requires length fits in u32");
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

    Ok(())
}
