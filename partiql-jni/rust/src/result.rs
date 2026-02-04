use jni::objects::JClass;
use jni::sys::{jboolean, jlong};
use jni::JNIEnv;

use partiql_eval::{value::RegisterReader, ExecutionResult};

use crate::{jni_guard, jni_guard_void};

// Store ExecutionResult handles similar to VM/Plan
pub fn create_result_handle(result: ExecutionResult<'static>) -> u64 {
    let boxed = Box::new(result);
    Box::into_raw(boxed) as u64
}

pub fn get_result(handle: u64) -> Result<&'static mut ExecutionResult<'static>, crate::JniError> {
    if handle == 0 {
        return Err(crate::JniError::InvalidHandle);
    }
    unsafe {
        let ptr = handle as *mut ExecutionResult<'static>;
        ptr.as_mut().ok_or(crate::JniError::InvalidHandle)
    }
}

pub fn remove_result_handle(handle: u64) {
    if handle != 0 {
        unsafe {
            let _ = Box::from_raw(handle as *mut ExecutionResult<'static>);
        }
    }
}

/// Check if ExecutionResult is a query result
///
/// Java signature:
/// ```java
/// private static native boolean nativeIsQuery(long handle);
/// ```
#[no_mangle]
pub extern "system" fn Java_org_partiql_jni_ExecutionResult_nativeIsQuery(
    mut env: JNIEnv<'_>,
    _class: JClass<'_>,
    result_handle: jlong,
) -> jboolean {
    jni_guard!(env, {
        let _result = get_result(result_handle as u64)?;
        // ExecutionResult currently only has Query variant
        // When Mutation/Definition variants are added, implement type checking here
        Ok(1 as jboolean)
    })
}

/// Get QueryIterator handle from ExecutionResult
///
/// Java signature:
/// ```java
/// private static native long nativeAsQueryIterator(long handle);
/// ```
#[no_mangle]
pub extern "system" fn Java_org_partiql_jni_ExecutionResult_nativeAsQueryIterator(
    mut env: JNIEnv<'_>,
    _class: JClass<'_>,
    result_handle: jlong,
) -> jlong {
    jni_guard!(env, {
        // Take ownership of the ExecutionResult
        let result = unsafe {
            if result_handle == 0 {
                return Err(crate::JniError::InvalidHandle);
            }
            Box::from_raw(result_handle as *mut partiql_eval::ExecutionResult<'static>)
        };

        // Extract QueryIterator from ExecutionResult
        match *result {
            partiql_eval::ExecutionResult::Query(iter) => {
                // Create IteratorState and return handle
                let state = IteratorState {
                    iter,
                    current_row: None,
                };
                let boxed = Box::new(state);
                Ok(Box::into_raw(boxed) as jlong)
            }
        }
    })
}

/// Close ExecutionResult and release resources
///
/// Java signature:
/// ```java
/// private static native void nativeClose(long handle);
/// ```
#[no_mangle]
pub extern "system" fn Java_org_partiql_jni_ExecutionResult_nativeClose(
    mut env: JNIEnv<'_>,
    _class: JClass<'_>,
    result_handle: jlong,
) {
    jni_guard_void!(env, {
        remove_result_handle(result_handle as u64);
        Ok(())
    })
}

// QueryIterator state - holds iterator and current row
pub struct IteratorState<'a> {
    pub iter: partiql_eval::QueryIterator<'a>,
    pub current_row: Option<RegisterReader<'a>>,
}

// QueryIterator handle management
#[allow(dead_code)]
pub fn create_iterator_handle(iter: partiql_eval::QueryIterator<'static>) -> u64 {
    let state = IteratorState {
        iter,
        current_row: None,
    };
    let boxed = Box::new(state);
    Box::into_raw(boxed) as u64
}

pub(crate) fn get_iterator_mut(
    handle: u64,
) -> Result<&'static mut IteratorState<'static>, crate::JniError> {
    if handle == 0 {
        return Err(crate::JniError::InvalidHandle);
    }
    unsafe {
        let ptr = handle as *mut IteratorState<'static>;
        ptr.as_mut().ok_or(crate::JniError::InvalidHandle)
    }
}

pub fn remove_iterator_handle(handle: u64) {
    if handle != 0 {
        unsafe {
            let _ = Box::from_raw(handle as *mut IteratorState<'static>);
        }
    }
}

/// Check if iterator has more rows
///
/// Java signature:
/// ```java
/// private static native boolean nativeHasNext(long handle);
/// ```
#[no_mangle]
pub extern "system" fn Java_org_partiql_jni_QueryIterator_nativeHasNext(
    mut env: JNIEnv<'_>,
    _class: JClass<'_>,
    iter_handle: jlong,
) -> jboolean {
    jni_guard!(env, {
        let _state = get_iterator_mut(iter_handle as u64)?;
        // Note: QueryIterator doesn't have peek(), so we can't check without consuming
        // For now, always return true and let next() handle empty case
        Ok(1 as jboolean)
    })
}

/// Advance to next row and prepare RegisterReader
/// Returns 1 if there's a row available, 0 if iteration is complete
///
/// Java signature:
/// ```java
/// private static native int nativeNext(long handle);
/// ```
#[no_mangle]
pub extern "system" fn Java_org_partiql_jni_QueryIterator_nativeNext(
    mut env: JNIEnv<'_>,
    _class: JClass<'_>,
    iter_handle: jlong,
) -> jlong {
    jni_guard!(env, {
        let state = get_iterator_mut(iter_handle as u64)?;

        if let Some(row_result) = state.iter.next() {
            let row = row_result?;
            // Store the current row in state so RegisterReader can access it
            state.current_row = Some(row);
            // Return 1 to indicate row is available
            Ok(1 as jlong)
        } else {
            // No more rows
            state.current_row = None;
            Ok(0 as jlong)
        }
    })
}

/// Close iterator and release resources
///
/// Java signature:
/// ```java
/// private static native void nativeClose(long handle);
/// ```
#[no_mangle]
pub extern "system" fn Java_org_partiql_jni_QueryIterator_nativeClose(
    mut env: JNIEnv<'_>,
    _class: JClass<'_>,
    iter_handle: jlong,
) {
    jni_guard_void!(env, {
        remove_iterator_handle(iter_handle as u64);
        Ok(())
    })
}
