//! JNI exports for RegisterWriter callbacks
//!
//! These functions allow Java DataSource implementations to write values
//! back to the Rust RegisterWriter during iteration.
//!
//! The buffered API (nativeFlush) provides optimal performance by transferring
//! all field values in a single JNI call via a DirectByteBuffer.

use jni::objects::{JClass, JObject};
use jni::sys::{jboolean, jdouble, jint, jlong};
use jni::JNIEnv;
use partiql_eval::source::RegisterWriter;
use partiql_eval::EngineError;

use crate::jni_guard_void;

// Type tags matching RegisterWriter.java
const TYPE_NULL: u8 = 0;
const TYPE_MISSING: u8 = 1;
const TYPE_BOOL: u8 = 2;
const TYPE_I64: u8 = 3;
const TYPE_F64: u8 = 4;
const TYPE_STRING: u8 = 5;

/// Write a NULL value to a register
///
/// Java signature:
/// ```java
/// public static native void nativeWriteNull(long writerHandle, int slot);
/// ```
#[no_mangle]
pub extern "system" fn Java_org_partiql_jni_RegisterWriter_nativeWriteNull(
    mut env: JNIEnv<'_>,
    _class: JClass<'_>,
    writer_handle: jlong,
    slot: jint,
) {
    jni_guard_void!(env, {
        let writer = unsafe { &mut *(writer_handle as *mut RegisterWriter<'_, '_>) };
        writer.put_null(slot as u16)?;
        Ok(())
    })
}

/// Write a boolean value to a register
///
/// Java signature:
/// ```java
/// public static native void nativeWriteBoolean(long writerHandle, int slot, boolean value);
/// ```
#[no_mangle]
pub extern "system" fn Java_org_partiql_jni_RegisterWriter_nativeWriteBoolean(
    mut env: JNIEnv<'_>,
    _class: JClass<'_>,
    writer_handle: jlong,
    slot: jint,
    value: jboolean,
) {
    jni_guard_void!(env, {
        let writer = unsafe { &mut *(writer_handle as *mut RegisterWriter<'_, '_>) };
        writer.put_bool(slot as u16, value != 0)?;
        Ok(())
    })
}

/// Write an integer value to a register
///
/// Java signature:
/// ```java
/// public static native void nativeWriteLong(long writerHandle, int slot, long value);
/// ```
#[no_mangle]
pub extern "system" fn Java_org_partiql_jni_RegisterWriter_nativeWriteLong(
    mut env: JNIEnv<'_>,
    _class: JClass<'_>,
    writer_handle: jlong,
    slot: jint,
    value: jlong,
) {
    jni_guard_void!(env, {
        let writer = unsafe { &mut *(writer_handle as *mut RegisterWriter<'_, '_>) };
        writer.put_i64(slot as u16, value)?;
        Ok(())
    })
}

/// Write a double value to a register
///
/// Java signature:
/// ```java
/// public static native void nativeWriteDouble(long writerHandle, int slot, double value);
/// ```
#[no_mangle]
pub extern "system" fn Java_org_partiql_jni_RegisterWriter_nativeWriteDouble(
    mut env: JNIEnv<'_>,
    _class: JClass<'_>,
    writer_handle: jlong,
    slot: jint,
    value: jdouble,
) {
    jni_guard_void!(env, {
        let writer = unsafe { &mut *(writer_handle as *mut RegisterWriter<'_, '_>) };
        writer.put_f64(slot as u16, value)?;
        Ok(())
    })
}

/// Write a string value to a register (LEGACY - use nativeFlush instead)
///
/// Note: The string data must remain valid. Since we're converting from Java String to Rust String,
/// we need to leak the string to ensure it has 'static lifetime that satisfies RegisterWriter's 'a lifetime.
/// This is safe because the string is only used until the next row is fetched (BufferStability::UntilNext).
///
/// Java signature:
/// ```java
/// public static native void nativeWriteString(long writerHandle, int slot, String value);
/// ```
#[no_mangle]
pub extern "system" fn Java_org_partiql_jni_RegisterWriter_nativeWriteString(
    mut env: JNIEnv<'_>,
    _class: JClass<'_>,
    writer_handle: jlong,
    slot: jint,
    value: JObject<'_>,
) {
    jni_guard_void!(env, {
        let writer = unsafe { &mut *(writer_handle as *mut RegisterWriter<'_, '_>) };

        let jstring = value.into();
        let java_str = env.get_string(&jstring)?;
        let rust_str: String = java_str.into();

        // Leak the string to get a 'static lifetime
        // This is safe because:
        // 1. The DataSource has BufferStability::UntilNext
        // 2. The string will only be used until the next row
        // 3. The VM will not hold references past the row iteration
        let leaked_str: &'static str = Box::leak(rust_str.into_boxed_str());

        writer.put_str(slot as u16, leaked_str)?;
        Ok(())
    })
}

/// Flush buffered writes from ByteBuffer to registers (OPTIMIZED API)
///
/// Decodes a slot-tagged buffer format and writes all values to registers
/// in a single JNI call, eliminating per-field JNI overhead.
///
/// Buffer format: [slot: u16][type_tag: u8][data: variable]...
///
/// Java signature:
/// ```java
/// private static native void nativeFlush(long writerHandle, ByteBuffer buffer, int limit);
/// ```
#[no_mangle]
pub extern "system" fn Java_org_partiql_jni_RegisterWriter_nativeFlush(
    mut env: JNIEnv<'_>,
    _class: JClass<'_>,
    writer_handle: jlong,
    buffer: JObject<'_>,
    limit: jint,
) {
    jni_guard_void!(env, {
        let writer = unsafe { &mut *(writer_handle as *mut RegisterWriter<'_, '_>) };

        // Convert JObject to JByteBuffer
        let byte_buffer = buffer.into();
        
        // Get direct buffer address (zero-copy!)
        let buffer_addr = env.get_direct_buffer_address(&byte_buffer)?;

        // Read buffer as byte slice using the limit passed from Java
        let buffer_slice = unsafe { std::slice::from_raw_parts(buffer_addr, limit as usize) };

        // Decode and write to registers
        decode_buffer_to_registers(buffer_slice, writer)?;

        Ok(())
    })
}

/// Decode slot-tagged buffer format and write to registers
///
/// Buffer format for each field: [slot: u16][type_tag: u8][data: variable]
fn decode_buffer_to_registers(
    buffer: &[u8],
    writer: &mut RegisterWriter<'_, '_>,
) -> Result<(), EngineError> {
    let mut offset = 0;

    while offset < buffer.len() {
        // Read slot (2 bytes)
        if offset + 2 > buffer.len() {
            return Err(EngineError::IllegalState(
                "Buffer underflow reading slot".to_string(),
            ));
        }
        let slot = u16::from_ne_bytes([buffer[offset], buffer[offset + 1]]);
        offset += 2;

        // Read type tag (1 byte)
        if offset >= buffer.len() {
            return Err(EngineError::IllegalState(
                "Buffer underflow reading type tag".to_string(),
            ));
        }
        let type_tag = buffer[offset];
        offset += 1;

        // Decode value based on type
        match type_tag {
            TYPE_NULL => {
                writer.put_null(slot)?;
            }
            TYPE_MISSING => {
                writer.put_missing(slot)?;
            }
            TYPE_BOOL => {
                if offset >= buffer.len() {
                    return Err(EngineError::IllegalState(
                        "Buffer underflow reading bool".to_string(),
                    ));
                }
                let value = buffer[offset] != 0;
                writer.put_bool(slot, value)?;
                offset += 1;
            }
            TYPE_I64 => {
                if offset + 8 > buffer.len() {
                    return Err(EngineError::IllegalState(
                        "Buffer underflow reading i64".to_string(),
                    ));
                }
                let bytes: [u8; 8] = buffer[offset..offset + 8]
                    .try_into()
                    .map_err(|_| EngineError::IllegalState("Failed to read i64".to_string()))?;
                let value = i64::from_ne_bytes(bytes);
                writer.put_i64(slot, value)?;
                offset += 8;
            }
            TYPE_F64 => {
                if offset + 8 > buffer.len() {
                    return Err(EngineError::IllegalState(
                        "Buffer underflow reading f64".to_string(),
                    ));
                }
                let bytes: [u8; 8] = buffer[offset..offset + 8]
                    .try_into()
                    .map_err(|_| EngineError::IllegalState("Failed to read f64".to_string()))?;
                let value = f64::from_ne_bytes(bytes);
                writer.put_f64(slot, value)?;
                offset += 8;
            }
            TYPE_STRING => {
                // Read length prefix (4 bytes)
                if offset + 4 > buffer.len() {
                    return Err(EngineError::IllegalState(
                        "Buffer underflow reading string length".to_string(),
                    ));
                }
                let len_bytes: [u8; 4] = buffer[offset..offset + 4]
                    .try_into()
                    .map_err(|_| {
                        EngineError::IllegalState("Failed to read string length".to_string())
                    })?;
                let len = i32::from_ne_bytes(len_bytes) as usize;
                offset += 4;

                // Read string data
                if offset + len > buffer.len() {
                    return Err(EngineError::IllegalState(format!(
                        "Buffer underflow reading string data: need {} bytes, only {} remaining",
                        len,
                        buffer.len() - offset
                    )));
                }
                let str_bytes = &buffer[offset..offset + len];
                let s = std::str::from_utf8(str_bytes).map_err(|e| {
                    EngineError::IllegalState(format!("Invalid UTF-8 in string: {}", e))
                })?;

                // TODO: Fix memory leak - strings need to be allocated in arena
                // For now, leak the string (same as legacy API)
                let leaked_str: &'static str = Box::leak(s.to_string().into_boxed_str());
                writer.put_str(slot, leaked_str)?;
                offset += len;
            }
            _ => {
                return Err(EngineError::IllegalState(format!(
                    "Unknown type tag: {}",
                    type_tag
                )));
            }
        }
    }

    Ok(())
}
