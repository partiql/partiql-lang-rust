use dashmap::DashMap;
use jni::objects::{JClass, JObject, JString};
use jni::sys::jlong;
use jni::JNIEnv;
use once_cell::sync::Lazy;
use std::sync::Arc;

use partiql_common::catalog::CatalogId;
use partiql_eval::{CompilationContext, ExecutionContext};

use crate::catalog_bridge;
use crate::error::JniError;
use crate::{jni_guard, jni_guard_void};

// Global handle storage for contexts
// Both contexts are mutable and stored directly (not in Arc) to allow adding catalogs
static COMPILATION_CONTEXT_HANDLES: Lazy<DashMap<u64, CompilationContext>> =
    Lazy::new(DashMap::new);

static EXECUTION_CONTEXT_HANDLES: Lazy<DashMap<u64, ExecutionContext>> = Lazy::new(DashMap::new);

// CompilationContext handle operations
pub fn create_compilation_context_handle(context: CompilationContext) -> u64 {
    let handle = crate::handles::next_handle();
    COMPILATION_CONTEXT_HANDLES.insert(handle, context);
    handle
}

pub fn get_compilation_context_mut(
    handle: u64,
) -> Result<dashmap::mapref::one::RefMut<'static, u64, CompilationContext>, JniError> {
    COMPILATION_CONTEXT_HANDLES
        .get_mut(&handle)
        .ok_or(JniError::InvalidHandle)
}

pub fn get_compilation_context(
    handle: u64,
) -> Result<dashmap::mapref::one::Ref<'static, u64, CompilationContext>, JniError> {
    COMPILATION_CONTEXT_HANDLES
        .get(&handle)
        .ok_or(JniError::InvalidHandle)
}

pub fn remove_compilation_context_handle(handle: u64) -> Option<CompilationContext> {
    COMPILATION_CONTEXT_HANDLES
        .remove(&handle)
        .map(|(_, ctx)| ctx)
}

// ExecutionContext handle operations
pub fn create_execution_context_handle(context: ExecutionContext) -> u64 {
    let handle = crate::handles::next_handle();
    EXECUTION_CONTEXT_HANDLES.insert(handle, context);
    handle
}

pub fn get_execution_context_mut(
    handle: u64,
) -> Result<dashmap::mapref::one::RefMut<'static, u64, ExecutionContext>, JniError> {
    EXECUTION_CONTEXT_HANDLES
        .get_mut(&handle)
        .ok_or(JniError::InvalidHandle)
}

pub fn get_execution_context(
    handle: u64,
) -> Result<dashmap::mapref::one::Ref<'static, u64, ExecutionContext>, JniError> {
    EXECUTION_CONTEXT_HANDLES
        .get(&handle)
        .ok_or(JniError::InvalidHandle)
}

pub fn remove_execution_context_handle(handle: u64) -> Option<ExecutionContext> {
    EXECUTION_CONTEXT_HANDLES
        .remove(&handle)
        .map(|(_, ctx)| ctx)
}

// JNI exports for CompilationContext

/// Create a new CompilationContext
///
/// Java signature:
/// ```java
/// private static native long nativeNew();
/// ```
#[no_mangle]
pub extern "system" fn Java_org_partiql_jni_CompilationContext_nativeNew(
    mut env: JNIEnv<'_>,
    _class: JClass<'_>,
) -> jlong {
    jni_guard!(env, {
        let context = CompilationContext::default();
        Ok(create_compilation_context_handle(context) as jlong)
    })
}

/// Add a catalog to the CompilationContext
///
/// Java signature:
/// ```java
/// private static native long nativeAddCatalog(long handle, String name, CompilationCatalog catalog);
/// ```
#[no_mangle]
pub extern "system" fn Java_org_partiql_jni_CompilationContext_nativeAddCatalog(
    mut env: JNIEnv<'_>,
    _class: JClass<'_>,
    handle: jlong,
    name: JString<'_>,
    catalog: JObject<'_>,
) -> jlong {
    jni_guard!(env, {
        // Check if catalog is null
        if catalog.is_null() {
            return Err(JniError::from(partiql_eval::EngineError::IllegalState(
                "CompilationCatalog cannot be null".to_string(),
            )));
        }

        // Get mutable access to context
        let mut comp_context = get_compilation_context_mut(handle as u64)?;

        let catalog_name: String = env.get_string(&name)?.into();

        // Create GlobalRef for the Java catalog
        let global_ref = env.new_global_ref(&catalog)?;

        // Get JavaVM
        let vm = Arc::new(env.get_java_vm()?);

        // Create JavaCompilationCatalog wrapper with interior mutability
        let java_catalog = Arc::new(catalog_bridge::JavaCompilationCatalog::new());

        // Add catalog to context and get the actual catalog_id
        let catalog_id = comp_context.add_catalog(catalog_name, java_catalog.clone());

        // Now set the catalog_id in the JavaCompilationCatalog
        java_catalog.set_catalog_id(catalog_id);

        // Register the GlobalRef with the catalog_id for callbacks
        catalog_bridge::register_compilation_catalog(catalog_id, global_ref, vm);

        // Convert CatalogId to jlong
        Ok(u64::from(catalog_id) as jlong)
    })
}

/// Close the CompilationContext and release resources
///
/// Java signature:
/// ```java
/// private static native void nativeClose(long handle);
/// ```
#[no_mangle]
pub extern "system" fn Java_org_partiql_jni_CompilationContext_nativeClose(
    mut env: JNIEnv<'_>,
    _class: JClass<'_>,
    handle: jlong,
) {
    jni_guard_void!(env, {
        remove_compilation_context_handle(handle as u64);
        Ok(())
    })
}

// JNI exports for ExecutionContext

/// Create a new ExecutionContext
///
/// Java signature:
/// ```java
/// private static native long nativeNew();
/// ```
#[no_mangle]
pub extern "system" fn Java_org_partiql_jni_ExecutionContext_nativeNew(
    mut env: JNIEnv<'_>,
    _class: JClass<'_>,
) -> jlong {
    jni_guard!(env, {
        let context = ExecutionContext::default();
        Ok(create_execution_context_handle(context) as jlong)
    })
}

/// Add a catalog to the ExecutionContext
///
/// Java signature:
/// ```java
/// private static native void nativeAddCatalog(long handle, long catalogId, ExecutionCatalog catalog);
/// ```
#[no_mangle]
pub extern "system" fn Java_org_partiql_jni_ExecutionContext_nativeAddCatalog(
    mut env: JNIEnv<'_>,
    _class: JClass<'_>,
    handle: jlong,
    catalog_id: jlong,
    catalog: JObject<'_>,
) {
    jni_guard_void!(env, {
        // Check if catalog is null
        if catalog.is_null() {
            return Err(JniError::from(partiql_eval::EngineError::IllegalState(
                "ExecutionCatalog cannot be null".to_string(),
            )));
        }

        // Get mutable access to the ExecutionContext
        let mut exec_context = get_execution_context_mut(handle as u64)?;

        // Create GlobalRef for the Java ExecutionCatalog
        let global_ref = env.new_global_ref(&catalog)?;

        // Get JavaVM
        let vm = Arc::new(env.get_java_vm()?);

        // Convert catalog_id to typed CatalogId
        let catalog_id_typed = CatalogId::from(catalog_id as u64);

        // Create JavaExecutionCatalog wrapper
        let java_exec_catalog: Arc<dyn partiql_eval::ExecutionCatalog> =
            Arc::new(catalog_bridge::JavaExecutionCatalog::new(catalog_id_typed));

        // Add catalog to ExecutionContext
        exec_context.add_catalog(catalog_id_typed, java_exec_catalog);

        // Register the GlobalRef for callbacks
        catalog_bridge::register_execution_catalog(catalog_id_typed, global_ref, vm);

        Ok(())
    })
}

/// Add a buffered catalog to the ExecutionContext
///
/// Java signature:
/// ```java
/// private static native void nativeAddBufferedCatalog(long handle, long catalogId, long entryId, java.nio.ByteBuffer buffer, int bufferSize);
/// ```
#[no_mangle]
pub extern "system" fn Java_org_partiql_jni_ExecutionContext_nativeAddBufferedCatalog(
    mut env: JNIEnv<'_>,
    _class: JClass<'_>,
    handle: jlong,
    catalog_id: jlong,
    entry_id: jlong,
    buffer: JObject<'_>,
    buffer_size: jni::sys::jint,
) {
    jni_guard_void!(env, {
        // Check if buffer is null
        if buffer.is_null() {
            return Err(JniError::from(partiql_eval::EngineError::IllegalState(
                "ByteBuffer cannot be null".to_string(),
            )));
        }

        // Get mutable access to the ExecutionContext
        let mut exec_context = get_execution_context_mut(handle as u64)?;

        // Convert ByteBuffer to Vec<u8> by copying the data
        let byte_buffer = buffer.into();

        // Get direct buffer address
        let buffer_addr = env.get_direct_buffer_address(&byte_buffer)?;

        // Use the provided buffer_size (already passed from Java to avoid extra JNI call)
        let buffer_size = buffer_size as usize;

        // Copy buffer data to owned Vec<u8> using the actual data size
        let buffer_data = unsafe { std::slice::from_raw_parts(buffer_addr, buffer_size) }.to_vec();

        // Convert catalog_id and entry_id to typed IDs
        let catalog_id_typed = CatalogId::from(catalog_id as u64);
        let entry_id_typed = partiql_common::catalog::EntryId::from(entry_id as u64);

        // Create JavaBufferedExecutionCatalog
        let buffered_catalog: Arc<dyn partiql_eval::ExecutionCatalog> = Arc::new(
            catalog_bridge::JavaBufferedExecutionCatalog::new(entry_id_typed, buffer_data),
        );

        // Add catalog to ExecutionContext
        exec_context.add_catalog(catalog_id_typed, buffered_catalog);

        Ok(())
    })
}

/// Close the ExecutionContext and release resources
///
/// Java signature:
/// ```java
/// private static native void nativeClose(long handle);
/// ```
#[no_mangle]
pub extern "system" fn Java_org_partiql_jni_ExecutionContext_nativeClose(
    mut env: JNIEnv<'_>,
    _class: JClass<'_>,
    handle: jlong,
) {
    jni_guard_void!(env, {
        remove_execution_context_handle(handle as u64);
        Ok(())
    })
}
