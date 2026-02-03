use jni::objects::JClass;
use jni::sys::{jlong, jobject};
use jni::JNIEnv;

use partiql_eval::{ExecutionContext, PartiQLVM};

use crate::{create_vm_handle, get_plan, get_vm, jni_guard, jni_guard_void, remove_vm_handle};

/// Create a new PartiQLVM from a CompiledPlan
///
/// Java signature:
/// ```java
/// private static native long nativeNew(long compiledPlanHandle, long executionContextHandle)
///     throws PartiQLException;
/// ```
#[no_mangle]
pub extern "system" fn Java_org_partiql_jni_PartiQLVM_nativeNew(
    mut env: JNIEnv<'_>,
    _class: JClass<'_>,
    plan_handle: jlong,
    _exec_context_handle: jlong,
) -> jlong {
    jni_guard!(env, {
        let plan = get_plan(plan_handle as u64)?;

        // TODO: Get ExecutionContext from handle
        // For now, create a default one
        let exec_context = ExecutionContext::default();

        let vm = PartiQLVM::new((*plan).clone(), &exec_context)?;
        Ok(create_vm_handle(vm) as jlong)
    })
}

/// Execute the currently loaded plan
///
/// Java signature:
/// ```java
/// private static native long nativeExecute(long handle) throws PartiQLException;
/// ```
#[no_mangle]
pub extern "system" fn Java_org_partiql_jni_PartiQLVM_nativeExecute(
    mut env: JNIEnv<'_>,
    _class: JClass<'_>,
    vm_handle: jlong,
) -> jlong {
    jni_guard!(env, {
        let vm = get_vm(vm_handle as u64)?;
        let result = vm.execute()?;

        // Create ExecutionResult handle and return it
        Ok(crate::create_result_handle(result) as jlong)
    })
}

/// Close the VM and release resources
///
/// Java signature:
/// ```java
/// private static native void nativeClose(long handle);
/// ```
#[no_mangle]
pub extern "system" fn Java_org_partiql_jni_PartiQLVM_nativeClose(
    mut env: JNIEnv<'_>,
    _class: JClass<'_>,
    vm_handle: jlong,
) {
    jni_guard_void!(env, {
        remove_vm_handle(vm_handle as u64);
        Ok(())
    })
}

/// Get the schema for this VM's query
///
/// Java signature:
/// ```java
/// private static native jobject nativeSchema(long handle) throws PartiQLException;
/// ```
#[no_mangle]
pub extern "system" fn Java_org_partiql_jni_PartiQLVM_nativeSchema(
    mut env: JNIEnv<'_>,
    _class: JClass<'_>,
    vm_handle: jlong,
) -> jobject {
    jni_guard!(env, {
        let vm = get_vm(vm_handle as u64)?;
        let schema = vm.schema();

        // TODO: Convert Schema to Java Schema object
        let _ = schema;
        Err(crate::error::JniError::EngineError(
            partiql_eval::EngineError::NotImplemented,
        ))
    })
}
