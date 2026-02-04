//! JNI exports for RegisterWriter callbacks
//!
//! These functions allow Java DataSource implementations to write values
//! back to the Rust RegisterWriter during iteration.

use jni::objects::{JClass, JObject};
use jni::sys::{jboolean, jdouble, jint, jlong};
use jni::JNIEnv;
use partiql_eval::source::RegisterWriter;

use crate::jni_guard_void;

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

/// Write a string value to a register
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
