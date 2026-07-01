//! Tagged-union row serialization and deserialization for pqlite.
//!
//! Wire format:
//!   ROW          = tagged_value
//!   tagged_value = tag(u8) payload
//!   payload(TAG_TUPLE) = field_count(u32 LE) (name_len(u32 LE) name_utf8 tagged_value){N}
//!
//! All multi-byte VALUE bytes are little-endian. Row-id KEYS in LMDB stay
//! big-endian (encoded as `row_id.to_be_bytes()` under `heed::types::Bytes`)
//! so the B+tree's lexicographic order matches numeric order.

use partiql_eval::value::{FieldName, RegisterReader, RowShape, ValueType};

/// Cap on `field_count` to bound `Vec::with_capacity` against a flipped byte.
pub const MAX_FIELDS_PER_ROW: u32 = 1024;

/// Cap on `name_len` (UTF-8 byte length) to bound allocations on corruption.
pub const MAX_NAME_LEN_BYTES: u32 = 1024 * 1024;

/// Cap on `bytes` payload length to bound allocations on corruption.
pub const MAX_BYTES_LEN: u32 = 1024 * 1024;

/// Cap on `string` payload length (UTF-8 bytes) to bound allocations on corruption.
pub const MAX_STRING_LEN: u32 = 1024 * 1024;

/// Cap on container nesting depth to bound stack use when decoding a
/// deeply nested (possibly adversarial) on-disk payload.
///
/// Safety invariant: the encoder rejects at a depth no deeper than the decoder
/// on every path (encode is the stricter side), so any row the encoder produces
/// is always decodable.
pub const MAX_RECURSION_DEPTH: u32 = 128;

// Tag taxonomy: scalars 0x00-0x07, containers 0x08-0x0A. This split is
// load-bearing: deserialize_row_into tells a container-rooted row from a
// scalar-rooted one with a single `tag >= TAG_TUPLE` test, so no scalar
// tag may sit at or above TAG_TUPLE.
pub const TAG_NULL: u8 = 0x00;
pub const TAG_MISSING: u8 = 0x01;
pub const TAG_BOOL: u8 = 0x02;
pub const TAG_INTEGER: u8 = 0x03;
pub const TAG_FLOAT: u8 = 0x04;
pub const TAG_DECIMAL: u8 = 0x05;
pub const TAG_STRING: u8 = 0x06;
pub const TAG_BYTES: u8 = 0x07;
pub const TAG_TUPLE: u8 = 0x08;
pub const TAG_LIST: u8 = 0x09;
pub const TAG_BAG: u8 = 0x0A;

#[derive(Debug)]
pub enum SerializeError {
    /// Unsupported type or shape: containers (Tuple/List/Bag) or dynamic
    /// field names. The string identifies the offending field or top-level
    /// value.
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

/// Serialize one VM row into `buf` as a tagged-union byte sequence. `buf`
/// is cleared on entry.
///
/// # Endianness invariant
///
/// Every multi-byte VALUE field is written `to_le_bytes()` and the symmetric
/// decoder reads `from_le_bytes()`. An LE/BE asymmetry compiles cleanly and
/// silently corrupts every persisted row.
pub fn serialize_row(
    row: &RegisterReader<'_>,
    row_shape: &RowShape,
    buf: &mut Vec<u8>,
) -> Result<(), SerializeError> {
    buf.clear();
    match row_shape {
        RowShape::Struct(fields) => write_tuple(row, fields, 0, buf),
        RowShape::Register(slot, _) => write_value_at_root(row, *slot, buf),
    }
}

fn write_value_at_root(
    row: &RegisterReader<'_>,
    slot: usize,
    buf: &mut Vec<u8>,
) -> Result<(), SerializeError> {
    // Top-level scalar rows land as a bare tag + payload with no tuple frame.
    // `field_name = None` selects the "top-level value" error prefix.
    write_value(row, slot, None, buf)
}

fn write_tuple(
    row: &RegisterReader<'_>,
    fields: &[partiql_eval::value::FieldShape],
    depth: u32,
    buf: &mut Vec<u8>,
) -> Result<(), SerializeError> {
    if depth > MAX_RECURSION_DEPTH {
        return Err(SerializeError::Unsupported(format!(
            "nesting depth {depth} exceeds MAX_RECURSION_DEPTH ({MAX_RECURSION_DEPTH})"
        )));
    }
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
            RowShape::Struct(nested_fields) => write_tuple(row, nested_fields, depth + 1, buf)?,
        }
    }
    Ok(())
}

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
            if sb.len() > MAX_STRING_LEN as usize {
                let prefix = match field_name {
                    Some(name) => format!("field '{name}'"),
                    None => "top-level value".to_string(),
                };
                return Err(SerializeError::Unsupported(format!(
                    "{prefix}: string length {} bytes exceeds MAX_STRING_LEN ({})",
                    sb.len(),
                    MAX_STRING_LEN
                )));
            }
            // Safe: the cap above bounds sb.len() to MAX_STRING_LEN (u32).
            let str_len: u32 = sb.len() as u32;
            buf.extend_from_slice(&str_len.to_le_bytes());
            buf.extend_from_slice(sb);
        }
        ValueType::Bool => {
            buf.push(TAG_BOOL);
            let b = view.get_bool().expect("bool view");
            buf.push(if b { 0x01 } else { 0x00 });
        }
        ValueType::Missing => {
            buf.push(TAG_MISSING);
        }
        ValueType::Bytes => {
            buf.push(TAG_BYTES);
            let b = view.get_bytes().expect("bytes view");
            if b.len() > MAX_BYTES_LEN as usize {
                let prefix = match field_name {
                    Some(name) => format!("field '{name}'"),
                    None => "top-level value".to_string(),
                };
                return Err(SerializeError::Unsupported(format!(
                    "{prefix}: bytes length {} exceeds MAX_BYTES_LEN ({})",
                    b.len(),
                    MAX_BYTES_LEN
                )));
            }
            // Safe: the cap above bounds b.len() to MAX_BYTES_LEN (u32).
            let byte_len: u32 = b.len() as u32;
            buf.extend_from_slice(&byte_len.to_le_bytes());
            buf.extend_from_slice(b);
        }
        ty @ (ValueType::Tuple | ValueType::List | ValueType::Bag) => {
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

#[derive(Debug)]
pub enum DeserializeError {
    Truncated,
    UnknownTag(u8),
    InvalidUtf8,
    NameTooLong(u32),
    FieldCountTooLarge(u32),
    InvalidBool(u8),
    BytesTooLong(u32),
    StringTooLong(u32),
    DepthExceeded(u32),
    /// Reserved tag or `ValueWriter` failure; symmetric to encoder rejection.
    Unsupported(String),
}

impl std::fmt::Display for DeserializeError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            DeserializeError::Truncated => write!(f, "truncated row payload"),
            DeserializeError::UnknownTag(t) => write!(f, "unknown tag byte: 0x{t:02x}"),
            DeserializeError::InvalidUtf8 => write!(f, "field name is not valid UTF-8"),
            DeserializeError::NameTooLong(n) => {
                write!(f, "field name length {n} exceeds {MAX_NAME_LEN_BYTES}")
            }
            DeserializeError::FieldCountTooLarge(n) => {
                write!(f, "tuple field count {n} exceeds {MAX_FIELDS_PER_ROW}")
            }
            DeserializeError::InvalidBool(b) => write!(f, "invalid bool payload: 0x{b:02x}"),
            DeserializeError::BytesTooLong(len) => {
                write!(f, "bytes length {len} exceeds MAX_BYTES_LEN")
            }
            DeserializeError::StringTooLong(len) => {
                write!(f, "string length {len} exceeds MAX_STRING_LEN")
            }
            DeserializeError::DepthExceeded(d) => {
                write!(f, "nesting depth {d} exceeds MAX_RECURSION_DEPTH")
            }
            DeserializeError::Unsupported(m) => write!(f, "unsupported: {m}"),
        }
    }
}

impl std::error::Error for DeserializeError {}

/// Decode a scalar-rooted row directly into `target_slot` via `RegisterWriter`.
///
/// # Safety
///
/// Same contract as [`deserialize_row_into`]: `bytes` must outlive every
/// arena reset that touches `target_slot`. Reachable only when the row's
/// first byte is a scalar tag (`< TAG_TUPLE`); container roots use the
/// `ValueWriter` frame-stack path in `deserialize_row_into`.
unsafe fn decode_top_scalar_into(
    bytes: &[u8],
    writer: &mut partiql_eval::source::RegisterWriter<'_, '_>,
    target_slot: u16,
) -> Result<(), DeserializeError> {
    let mut cursor = 0usize;
    let tag = take_byte(bytes, &mut cursor)?;
    // Safety: see this fn's # Safety block; contract is delegated to the helper.
    let scalar = unsafe { decode_scalar_payload(tag, bytes, &mut cursor)? };
    match scalar {
        Scalar::Null => writer.write_null(target_slot).map_err(io_err)?,
        Scalar::Missing => writer.write_missing(target_slot).map_err(io_err)?,
        Scalar::Bool(v) => writer.write_bool(target_slot, v).map_err(io_err)?,
        Scalar::I64(v) => writer.write_i64(target_slot, v).map_err(io_err)?,
        Scalar::F64(v) => writer.write_f64(target_slot, v).map_err(io_err)?,
        Scalar::Decimal(d) => writer.write_decimal(target_slot, d).map_err(io_err)?,
        Scalar::Str(s) => writer.write_str(target_slot, s).map_err(io_err)?,
        Scalar::Bytes(b) => writer.write_bytes(target_slot, b).map_err(io_err)?,
    }
    if cursor != bytes.len() {
        return Err(DeserializeError::Unsupported(format!(
            "trailing bytes after scalar-root row: {} of {} consumed",
            cursor,
            bytes.len()
        )));
    }
    Ok(())
}

/// Decode one row of the tagged-union wire format into `writer`'s `target_slot`.
///
/// # Safety
///
/// `bytes` must outlive every arena reset that touches `target_slot`'s
/// register. In practice the caller (a `DataSource::next_row`
/// implementation) must hold the backing row buffer for the duration
/// of the scan. Violating this precondition causes use-after-free of
/// the string and byte slices written into the engine's arena via the
/// lifetime extensions inside `decode_scalar_payload` and `decode_tagged_into`.
///
/// String UTF-8 validity is checked inline by `take_str` as each
/// string is read; no pre-pass is required.
pub unsafe fn deserialize_row_into(
    bytes: &[u8],
    writer: &mut partiql_eval::source::RegisterWriter<'_, '_>,
    target_slot: u16,
) -> Result<(), DeserializeError> {
    if bytes.is_empty() {
        return Err(DeserializeError::Truncated);
    }
    // Container-rooted rows start with a tag >= TAG_TUPLE (0x08) and use the
    // ValueWriter frame-stack path. Scalar-rooted rows write straight to the
    // register (a bare scalar has no frame for ValueWriter::put_*).
    if bytes[0] >= TAG_TUPLE {
        let mut cursor = 0usize;
        let mut vw = writer.value_writer(target_slot).map_err(|e| {
            DeserializeError::Unsupported(format!("value_writer({target_slot}): {e}"))
        })?;
        decode_tagged_into(&mut vw, bytes, &mut cursor, 0)?;
        if cursor != bytes.len() {
            return Err(DeserializeError::Unsupported(format!(
                "trailing bytes after row decode: {} of {} consumed",
                cursor,
                bytes.len()
            )));
        }
        vw.finish()
            .map_err(|e| DeserializeError::Unsupported(format!("finish: {e}")))?;
    } else {
        // Safety: same contract as this function's # Safety block.
        unsafe { decode_top_scalar_into(bytes, writer, target_slot)? };
    }
    Ok(())
}

/// A decoded scalar value. `Str`/`Bytes` carry arena-lifetime borrows
/// laundered from the source buffer.
enum Scalar<'a> {
    Null,
    Missing,
    Bool(bool),
    I64(i64),
    F64(f64),
    Decimal(rust_decimal::Decimal),
    Str(&'a str),
    Bytes(&'a [u8]),
}

/// Decode the payload for a single scalar `tag`, advancing `cursor`. The
/// single source of truth for scalar reads shared by the register-root path
/// (`decode_top_scalar_into`) and the `ValueWriter` frame path
/// (`decode_tagged_into`): the two must decode byte-identically forever, so
/// the reads live here once. An unknown scalar tag yields `UnknownTag`.
///
/// # Safety
///
/// The `Str`/`Bytes` variants carry borrows extended to an unbounded arena
/// lifetime. See [`deserialize_row_into`]'s `# Safety` block: `bytes` must
/// outlive every arena reset that touches the register the value is written
/// into.
unsafe fn decode_scalar_payload<'a>(
    tag: u8,
    bytes: &[u8],
    cursor: &mut usize,
) -> Result<Scalar<'a>, DeserializeError> {
    let scalar = match tag {
        TAG_NULL => Scalar::Null,
        TAG_MISSING => Scalar::Missing,
        TAG_BOOL => {
            let b = take_byte(bytes, cursor)?;
            match b {
                0x00 => Scalar::Bool(false),
                0x01 => Scalar::Bool(true),
                _ => return Err(DeserializeError::InvalidBool(b)),
            }
        }
        TAG_INTEGER => Scalar::I64(i64::from_le_bytes(take_array::<8>(bytes, cursor)?)),
        TAG_FLOAT => Scalar::F64(f64::from_le_bytes(take_array::<8>(bytes, cursor)?)),
        TAG_DECIMAL => {
            let scale = i32::from_le_bytes(take_array::<4>(bytes, cursor)?);
            let mantissa = i128::from_le_bytes(take_array::<16>(bytes, cursor)?);
            let scale_u32 = u32::try_from(scale).map_err(|_| {
                DeserializeError::Unsupported(format!("negative decimal scale {scale}"))
            })?;
            let d = rust_decimal::Decimal::try_from_i128_with_scale(mantissa, scale_u32)
                .map_err(|e| DeserializeError::Unsupported(format!("invalid decimal: {e}")))?;
            Scalar::Decimal(d)
        }
        TAG_STRING => {
            let len = u32::from_le_bytes(take_array::<4>(bytes, cursor)?);
            if len > MAX_STRING_LEN {
                return Err(DeserializeError::StringTooLong(len));
            }
            let s = take_str(bytes, cursor, len)?;
            // Safety: see `deserialize_row_into`'s # Safety block.
            let s_ext: &'a str = unsafe { extend_to_arena_lifetime(s) };
            Scalar::Str(s_ext)
        }
        TAG_BYTES => {
            let len = u32::from_le_bytes(take_array::<4>(bytes, cursor)?);
            if len > MAX_BYTES_LEN {
                return Err(DeserializeError::BytesTooLong(len));
            }
            let slice = take_bytes(bytes, cursor, len)?;
            // Safety: see `deserialize_row_into`'s # Safety block.
            let b_ext: &'a [u8] = unsafe { extend_bytes_to_arena_lifetime(slice) };
            Scalar::Bytes(b_ext)
        }
        t => return Err(DeserializeError::UnknownTag(t)),
    };
    Ok(scalar)
}

fn decode_tagged_into(
    vw: &mut partiql_eval::source::ValueWriter<'_, '_>,
    bytes: &[u8],
    cursor: &mut usize,
    depth: u32,
) -> Result<(), DeserializeError> {
    if depth > MAX_RECURSION_DEPTH {
        return Err(DeserializeError::DepthExceeded(depth));
    }
    let tag = take_byte(bytes, cursor)?;
    match tag {
        TAG_TUPLE => {
            let field_count = u32::from_le_bytes(take_array::<4>(bytes, cursor)?);
            if field_count > MAX_FIELDS_PER_ROW {
                return Err(DeserializeError::FieldCountTooLarge(field_count));
            }
            vw.step_in_tuple().map_err(io_err)?;
            for _ in 0..field_count {
                let name_len = u32::from_le_bytes(take_array::<4>(bytes, cursor)?);
                if name_len > MAX_NAME_LEN_BYTES {
                    return Err(DeserializeError::NameTooLong(name_len));
                }
                let name = take_str(bytes, cursor, name_len)?;
                // Safety: see `deserialize_row_into`'s # Safety block.
                let name_ext: &str = unsafe { extend_to_arena_lifetime(name) };
                vw.put_field_name(name_ext).map_err(io_err)?;
                decode_tagged_into(vw, bytes, cursor, depth + 1)?;
            }
            vw.step_out().map_err(io_err)?;
        }
        _ => {
            // Safety: decode_tagged_into is only reached from deserialize_row_into,
            // which upholds the # Safety contract; delegated to the helper.
            let scalar = unsafe { decode_scalar_payload(tag, bytes, cursor)? };
            match scalar {
                Scalar::Null => vw.put_null().map_err(io_err)?,
                Scalar::Missing => vw.put_missing().map_err(io_err)?,
                Scalar::Bool(v) => vw.put_bool(v).map_err(io_err)?,
                Scalar::I64(v) => vw.put_i64(v).map_err(io_err)?,
                Scalar::F64(v) => vw.put_f64(v).map_err(io_err)?,
                Scalar::Decimal(d) => vw.put_decimal(d).map_err(io_err)?,
                Scalar::Str(s) => vw.put_str(s).map_err(io_err)?,
                Scalar::Bytes(b) => vw.put_bytes(b).map_err(io_err)?,
            }
        }
    }
    Ok(())
}

#[inline]
fn take_byte(bytes: &[u8], cursor: &mut usize) -> Result<u8, DeserializeError> {
    let b = *bytes.get(*cursor).ok_or(DeserializeError::Truncated)?;
    *cursor += 1;
    Ok(b)
}

#[inline]
fn take_array<const N: usize>(
    bytes: &[u8],
    cursor: &mut usize,
) -> Result<[u8; N], DeserializeError> {
    let end = cursor.checked_add(N).ok_or(DeserializeError::Truncated)?;
    let slice = bytes.get(*cursor..end).ok_or(DeserializeError::Truncated)?;
    let arr = <[u8; N]>::try_from(slice).expect("slice is exactly N bytes");
    *cursor = end;
    Ok(arr)
}

#[inline]
fn take_str<'b>(
    bytes: &'b [u8],
    cursor: &mut usize,
    len: u32,
) -> Result<&'b str, DeserializeError> {
    let len = len as usize;
    let end = cursor.checked_add(len).ok_or(DeserializeError::Truncated)?;
    let slice = bytes.get(*cursor..end).ok_or(DeserializeError::Truncated)?;
    let s = std::str::from_utf8(slice).map_err(|_| DeserializeError::InvalidUtf8)?;
    *cursor = end;
    Ok(s)
}

#[inline]
fn take_bytes<'b>(
    bytes: &'b [u8],
    cursor: &mut usize,
    len: u32,
) -> Result<&'b [u8], DeserializeError> {
    let end = cursor
        .checked_add(len as usize)
        .ok_or(DeserializeError::Truncated)?;
    let slice = bytes.get(*cursor..end).ok_or(DeserializeError::Truncated)?;
    *cursor = end;
    Ok(slice)
}

/// Extends `s`'s lifetime to match the arena's. Caller must ensure the
/// backing buffer outlives every arena reset that touches the register
/// the string is written into — see [`deserialize_row_into`]'s
/// `# Safety` contract for the call-site invariant.
///
/// The output lifetime `'out` is unrelated to the input lifetime to
/// permit the arena-lifetime borrow `put_str` / `put_field_name` require.
///
/// # Safety
///
/// See [`deserialize_row_into`].
#[inline]
unsafe fn extend_to_arena_lifetime<'out>(s: &str) -> &'out str {
    // SAFETY: caller upholds the precondition documented on
    // deserialize_row_into.
    unsafe { &*(s as *const str) }
}

/// Extends `b`'s lifetime to match the arena's. Sibling of
/// [`extend_to_arena_lifetime`] for `&[u8]`: the caller must uphold the same
/// backing-buffer-outlives-arena-reset invariant.
///
/// The output lifetime `'out` is unrelated to the input lifetime to permit
/// the arena-lifetime borrow `put_bytes` requires.
///
/// # Safety
///
/// See [`deserialize_row_into`].
#[inline]
unsafe fn extend_bytes_to_arena_lifetime<'out>(b: &[u8]) -> &'out [u8] {
    // SAFETY: caller upholds the precondition documented on
    // deserialize_row_into.
    unsafe { &*(b as *const [u8]) }
}

#[inline]
fn io_err(e: partiql_eval::EngineError) -> DeserializeError {
    DeserializeError::Unsupported(format!("writer error: {e}"))
}

// `RegisterWriter::new` and `ValueRef` are `pub(crate)` in `partiql-eval`, so
// round-trip unit tests must run as integration tests. Tests below cover the
// byte-reader helpers and the `DeserializeError` surface.
#[cfg(test)]
mod deserialize_tests {
    use super::*;

    #[test]
    fn take_byte_advances_cursor_on_success() {
        let mut cursor = 0usize;
        let b = take_byte(&[0x42, 0xAA], &mut cursor).unwrap();
        assert_eq!(b, 0x42);
        assert_eq!(cursor, 1);
    }

    #[test]
    fn take_byte_truncated_on_empty() {
        let mut cursor = 0usize;
        let res = take_byte(&[], &mut cursor);
        assert!(matches!(res, Err(DeserializeError::Truncated)));
        assert_eq!(cursor, 0, "cursor must not advance on error");
    }

    #[test]
    fn take_array_reads_le_i64() {
        // 42 as little-endian i64
        let bytes = 42i64.to_le_bytes();
        let mut cursor = 0usize;
        let arr = take_array::<8>(&bytes, &mut cursor).unwrap();
        assert_eq!(i64::from_le_bytes(arr), 42);
        assert_eq!(cursor, 8);
    }

    #[test]
    fn take_array_truncated_when_short() {
        let bytes = [0x01, 0x02, 0x03];
        let mut cursor = 0usize;
        let res = take_array::<8>(&bytes, &mut cursor);
        assert!(matches!(res, Err(DeserializeError::Truncated)));
    }

    #[test]
    fn take_str_returns_utf8_slice() {
        let bytes = b"hello";
        let mut cursor = 0usize;
        let s = take_str(bytes, &mut cursor, 5).unwrap();
        assert_eq!(s, "hello");
        assert_eq!(cursor, 5);
    }

    #[test]
    fn take_str_invalid_utf8_returns_invalid_utf8() {
        // 0xFF is not valid UTF-8 leading byte.
        let bytes = [0xFFu8];
        let mut cursor = 0usize;
        let res = take_str(&bytes, &mut cursor, 1);
        assert!(matches!(res, Err(DeserializeError::InvalidUtf8)));
    }

    #[test]
    fn take_str_truncated_when_shorter_than_len() {
        let bytes = b"hi";
        let mut cursor = 0usize;
        let res = take_str(bytes, &mut cursor, 10);
        assert!(matches!(res, Err(DeserializeError::Truncated)));
    }

    #[test]
    fn display_renders_each_variant() {
        assert_eq!(
            format!("{}", DeserializeError::Truncated),
            "truncated row payload"
        );
        assert_eq!(
            format!("{}", DeserializeError::UnknownTag(0xFE)),
            "unknown tag byte: 0xfe"
        );
        assert_eq!(
            format!("{}", DeserializeError::InvalidUtf8),
            "field name is not valid UTF-8"
        );
        assert!(
            format!("{}", DeserializeError::NameTooLong(99)).starts_with("field name length 99")
        );
        assert!(format!("{}", DeserializeError::FieldCountTooLarge(2048))
            .starts_with("tuple field count 2048"));
        assert_eq!(
            format!("{}", DeserializeError::Unsupported("x".to_string())),
            "unsupported: x"
        );
    }
}
