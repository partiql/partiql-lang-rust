use jni::objects::{GlobalRef, JClass, JObject};
use jni::sys::{jint, jlong, jmethodID, jobject};
use jni::JNIEnv;
use once_cell::sync::Lazy;
use std::sync::Mutex;

use crate::error::JniError;
use crate::jni_guard;
use crate::result::get_iterator_mut;

/// Cached Long class and method IDs
///
/// Cached globally to avoid expensive find_class calls on every getI64() invocation
static LONG_CLASS_CACHE: Lazy<Mutex<Option<LongClassCache>>> = Lazy::new(|| Mutex::new(None));

struct LongClassCache {
    class_ref: GlobalRef,
    constructor_id: jmethodID,
}

unsafe impl Send for LongClassCache {}
unsafe impl Sync for LongClassCache {}

/// Initialize Long class cache
///
/// Should be called once during initialization to cache class and method references
fn init_long_class_cache(env: &mut jni::JNIEnv<'_>) -> Result<(), JniError> {
    let mut cache_guard = LONG_CLASS_CACHE.lock().map_err(|_| {
        JniError::Jni(jni::errors::Error::JniCall(jni::errors::JniError::Other(
            -1,
        )))
    })?;

    if cache_guard.is_none() {
        // Find Long class
        let long_class = env.find_class("java/lang/Long")?;

        // Create global reference
        let class_ref = env.new_global_ref(&long_class)?;

        // Get constructor method ID: (J)V
        let constructor_id = env.get_method_id(long_class, "<init>", "(J)V")?.into_raw();

        *cache_guard = Some(LongClassCache {
            class_ref,
            constructor_id,
        });
    }

    Ok(())
}

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
        // Initialize cache if needed (first call only)
        init_long_class_cache(&mut env)?;

        let state = get_iterator_mut(iterator_handle as u64)?;

        if let Some(ref row) = state.current_row {
            // Use RegisterReader's get_i64 method
            if let Some(value) = row.get_i64(col as usize) {
                // Use cached class reference and constructor
                let cache_guard = LONG_CLASS_CACHE.lock().map_err(|_| {
                    JniError::Jni(jni::errors::Error::JniCall(jni::errors::JniError::Other(
                        -1,
                    )))
                })?;

                let cache = cache_guard.as_ref().ok_or({
                    JniError::Jni(jni::errors::Error::JniCall(jni::errors::JniError::Other(
                        -1,
                    )))
                })?;

                // Create Java Long object using cached class and method
                let long_obj = unsafe {
                    let method_id = jni::objects::JMethodID::from_raw(cache.constructor_id);
                    let args = [jni::sys::jvalue { j: value }];
                    env.new_object_unchecked(&cache.class_ref, method_id, &args)?
                };

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

            // Convert ValueOwned to Java Value object
            // Note: Full implementation deferred - use get_i64/get_str for specific types
            let _value = value_owned; // Consume to avoid unused warning

            // Create Java Value object
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
