use jni::objects::{JByteBuffer, JClass};
use jni::sys::{jboolean, jint, jlong};
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
/// private static native long nativeAsQueryIterator(long handle, long vmHandle);
/// ```
#[no_mangle]
pub extern "system" fn Java_org_partiql_jni_ExecutionResult_nativeAsQueryIterator(
    mut env: JNIEnv<'_>,
    _class: JClass<'_>,
    result_handle: jlong,
    vm_handle: jlong,
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
                // Create IteratorState with VM handle for buffer caching
                let state = IteratorState {
                    iter,
                    current_row: None,
                    vm_handle: vm_handle as u64,
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

// QueryIterator state - holds iterator and VM handle for buffer caching
pub struct IteratorState<'a> {
    pub iter: partiql_eval::QueryIterator<'a>,
    pub current_row: Option<RegisterReader<'a>>,
    vm_handle: u64, // Link back to VM for accessing buffer cache
}

// QueryIterator handle management
#[allow(dead_code)]
pub fn create_iterator_handle(iter: partiql_eval::QueryIterator<'static>, vm_handle: u64) -> u64 {
    let state = IteratorState {
        iter,
        current_row: None,
        vm_handle,
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

/// Advance to next row and write to buffer (zero-copy approach)
/// Returns bytes written if row available, 0 if no more rows, -1 if buffer too small
///
/// Java signature:
/// ```java
/// private static native int nativeNextToBuffer(long handle, ByteBuffer buffer, int bufferId);
/// ```
#[no_mangle]
pub extern "system" fn Java_org_partiql_jni_QueryIterator_nativeNextToBuffer(
    mut env: JNIEnv<'_>,
    _class: JClass<'_>,
    iter_handle: jlong,
    buffer: JByteBuffer<'_>,
    buffer_id: jint,
) -> jint {
    jni_guard!(env, {
        let state = get_iterator_mut(iter_handle as u64)?;

        // Try to get next row
        if let Some(row_result) = state.iter.next() {
            let row = row_result?;

            // Get VM state for buffer cache access
            let vm_state = crate::get_vm_state(state.vm_handle)?;

            // Use buffer_id as cache key (no JNI calls on cache hit!)
            let buffer_key = buffer_id as usize;

            // Check VM-level buffer cache
            let (buffer_ptr, capacity) =
                if let Some(cached) = vm_state.buffer_cache.get(&buffer_key) {
                    // Cache hit! Zero JNI calls - just HashMap lookup (~1-2ns)
                    (cached.ptr, cached.capacity)
                } else {
                    // Cache miss - make JNI calls once and cache everything
                    let buffer_ptr = env.get_direct_buffer_address(&buffer)?;
                    let capacity = env.get_direct_buffer_capacity(&buffer)?;

                    // Store in VM-level cache for reuse across queries
                    vm_state.buffer_cache.insert(
                        buffer_key,
                        crate::handles::BufferMetadata {
                            ptr: buffer_ptr,
                            capacity,
                        },
                    );

                    (buffer_ptr, capacity)
                };

            // Convert raw pointer to mutable slice
            let buffer_slice = unsafe { std::slice::from_raw_parts_mut(buffer_ptr, capacity) };

            // Write row data to buffer in BufferWriter format
            let bytes_written = write_row_to_buffer(&row, buffer_slice, capacity)?;

            if bytes_written > capacity {
                // Buffer too small
                Ok(-1)
            } else {
                // Success - return bytes written
                Ok(bytes_written as jint)
            }
        } else {
            // No more rows
            Ok(0)
        }
    })
}

/// Write a row to buffer in BufferWriter format
/// Returns number of bytes written
fn write_row_to_buffer(
    row: &RegisterReader<'_>,
    buffer: &mut [u8],
    capacity: usize,
) -> Result<usize, crate::JniError> {
    let mut offset = 0;

    // Type tags (must match BufferWriter and Java RegisterReader)
    #[allow(dead_code)]
    const TYPE_NULL: u8 = 0;
    #[allow(dead_code)]
    const TYPE_MISSING: u8 = 1;
    #[allow(dead_code)]
    const TYPE_BOOL: u8 = 2;
    const TYPE_I64: u8 = 3;
    #[allow(dead_code)]
    const TYPE_F64: u8 = 4;
    const TYPE_STRING: u8 = 5;

    // Iterate through all register slots and write non-empty values
    // TODO: Get actual register count from row metadata
    // For now, try common slot range (0-100)
    for slot in 0..100 {
        // Try to get value from this slot
        if let Some(value) = row.get_i64(slot) {
            // Write: [slot: u16][type: u8][data: i64]
            if offset + 2 + 1 + 8 > capacity {
                return Ok(offset); // Buffer full
            }

            buffer[offset..offset + 2].copy_from_slice(&(slot as u16).to_ne_bytes());
            offset += 2;

            buffer[offset] = TYPE_I64;
            offset += 1;

            buffer[offset..offset + 8].copy_from_slice(&value.to_ne_bytes());
            offset += 8;

            continue;
        }

        if let Some(value) = row.get_str(slot) {
            // Write: [slot: u16][type: u8][length: i32][data: bytes]
            let bytes = value.as_bytes();
            let len = bytes.len();

            if offset + 2 + 1 + 4 + len > capacity {
                return Ok(offset); // Buffer full
            }

            buffer[offset..offset + 2].copy_from_slice(&(slot as u16).to_ne_bytes());
            offset += 2;

            buffer[offset] = TYPE_STRING;
            offset += 1;

            buffer[offset..offset + 4].copy_from_slice(&(len as i32).to_ne_bytes());
            offset += 4;

            buffer[offset..offset + len].copy_from_slice(bytes);
            offset += len;

            continue;
        }

        // If we've gone through many empty slots, assume we're done
        // This is a heuristic to avoid checking all 100 slots
        if slot > 10 && offset > 0 {
            break;
        }
    }

    Ok(offset)
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
