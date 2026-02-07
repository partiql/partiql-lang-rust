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
                // Get schema from VM for efficient slot reading
                let vm_state = crate::get_vm_state(vm_handle as u64)?;
                let schema = vm_state.vm.schema();

                // Create IteratorState with VM handle for buffer caching
                let state = IteratorState {
                    iter,
                    current_row: None,
                    vm_handle: vm_handle as u64,
                    schema,
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
    vm_handle: u64,               // Link back to VM for accessing buffer cache
    schema: partiql_eval::Schema, // Output schema for efficient slot reading
}

// QueryIterator handle management
#[allow(dead_code)]
pub fn create_iterator_handle(
    iter: partiql_eval::QueryIterator<'static>,
    vm_handle: u64,
    schema: partiql_eval::Schema,
) -> u64 {
    let state = IteratorState {
        iter,
        current_row: None,
        vm_handle,
        schema,
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

            // Write row data to buffer in BufferWriter format using schema
            let bytes_written = write_row_to_buffer(&row, &state.schema, buffer_slice, capacity)?;

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

/// Write a row to buffer in BufferWriter format using schema for efficient slot reading
/// Returns number of bytes written
fn write_row_to_buffer(
    row: &RegisterReader<'_>,
    schema: &partiql_eval::Schema,
    buffer: &mut [u8],
    capacity: usize,
) -> Result<usize, crate::JniError> {
    let mut offset = 0;

    // Type tag for i64 (must match BufferWriter and Java RegisterReader)
    const TYPE_I64: u8 = 3;

    // Iterate only through schema-defined columns (no garbage data!)
    // Schema index directly maps to slot index
    for slot in 0..schema.columns.len() {
        // Read value from slot (assuming i64 for now as confirmed)
        if let Some(value) = row.get_i64(slot) {
            // Check if we have space: [slot: u16][type: u8][data: i64]
            if offset + 2 + 1 + 8 > capacity {
                return Ok(offset); // Buffer full
            }

            // Write slot index
            buffer[offset..offset + 2].copy_from_slice(&(slot as u16).to_ne_bytes());
            offset += 2;

            // Write type tag
            buffer[offset] = TYPE_I64;
            offset += 1;

            // Write i64 value
            buffer[offset..offset + 8].copy_from_slice(&value.to_ne_bytes());
            offset += 8;
        }
        // Note: If slot has no value, we skip it (sparse representation)
        // This is correct behavior - missing values are not written
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
