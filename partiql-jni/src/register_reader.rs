use jni::objects::{JClass, JObject};
use jni::sys::{jint, jlong, jobject};
use jni::JNIEnv;

use crate::jni_guard;
use crate::result::get_iterator_mut;

/// Get i64 value from current row at specified column
///
/// Java signature:
/// ```java
/// private static native Long nativeGetI64(long iteratorHandle, int col);
/// ```
#[no_mangle]
pub extern "system" fn Java_org_partiql_jni_RegisterReader_nativeGetI64(
    mut env: JNIEnv<'_>,
    _class: JClass<'_>,
    iterator_handle: jlong,
    col: jint,
) -> jobject {
    jni_guard!(env, {
        let state = get_iterator_mut(iterator_handle as u64)?;

        if let Some(ref row) = state.current_row {
            // Use RegisterReader's get_i64 method
            if let Some(value) = row.get_i64(col as usize) {
                // Create Java Long object
                let long_class = env.find_class("java/lang/Long")?;
                let long_obj =
                    env.new_object(long_class, "(J)V", &[jni::objects::JValue::Long(value)])?;
                Ok(long_obj.into_raw())
            } else {
                // Column doesn't contain an i64, return null
                Ok(std::ptr::null_mut())
            }
        } else {
            // No current row
            Ok(std::ptr::null_mut())
        }
    })
}

/// Get string value from current row at specified column
///
/// Java signature:
/// ```java
/// private static native String nativeGetStr(long iteratorHandle, int col);
/// ```
#[no_mangle]
pub extern "system" fn Java_org_partiql_jni_RegisterReader_nativeGetStr(
    mut env: JNIEnv<'_>,
    _class: JClass<'_>,
    iterator_handle: jlong,
    col: jint,
) -> jobject {
    jni_guard!(env, {
        let state = get_iterator_mut(iterator_handle as u64)?;

        if let Some(ref row) = state.current_row {
            // Use RegisterReader's get_str method
            if let Some(value) = row.get_str(col as usize) {
                // Create Java String
                let java_str = env.new_string(value)?;
                Ok(java_str.into_raw())
            } else {
                // Column doesn't contain a string, return null
                Ok(std::ptr::null_mut())
            }
        } else {
            // No current row
            Ok(std::ptr::null_mut())
        }
    })
}

/// Get generic value from current row at specified column
///
/// Java signature:
/// ```java
/// private static native Value nativeGetValue(long iteratorHandle, int col);
/// ```
#[no_mangle]
pub extern "system" fn Java_org_partiql_jni_RegisterReader_nativeGetValue(
    mut env: JNIEnv<'_>,
    _class: JClass<'_>,
    iterator_handle: jlong,
    col: jint,
) -> jobject {
    jni_guard!(env, {
        let state = get_iterator_mut(iterator_handle as u64)?;

        if let Some(ref row) = state.current_row {
            // Use RegisterReader's get_value method
            let value_owned = row.get_value(col as usize);

            // TODO: Convert ValueOwned to Java Value object
            // For now, create a simple Value wrapper
            let _value = value_owned; // Consume to avoid unused warning

            // Create Java Value object (placeholder)
            let value_class = env.find_class("org/partiql/jni/Value")?;
            let null_obj = JObject::null();
            let value_obj = env.new_object(
                value_class,
                "(Ljava/lang/Object;)V",
                &[jni::objects::JValue::Object(&null_obj)],
            )?;
            Ok(value_obj.into_raw())
        } else {
            // No current row, return null
            Ok(std::ptr::null_mut())
        }
    })
}
