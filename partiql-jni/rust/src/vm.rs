use jni::objects::JClass;
use jni::sys::jlong;
use jni::JNIEnv;

use partiql_eval::PartiQLVM;

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
    exec_context_handle: jlong,
) -> jlong {
    jni_guard!(env, {
        let plan = get_plan(plan_handle as u64)?;

        // Get ExecutionContext from handle (wrapped in Arc)
        let exec_context = crate::context::get_execution_context(exec_context_handle as u64)?;

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

/// Update the ExecutionContext for an existing VM
///
/// Java signature:
/// ```java
/// private static native void nativeSetContext(long vmHandle, long executionContextHandle)
///     throws PartiQLException;
/// ```
#[no_mangle]
pub extern "system" fn Java_org_partiql_jni_PartiQLVM_nativeSetContext(
    mut env: JNIEnv<'_>,
    _class: JClass<'_>,
    vm_handle: jlong,
    exec_context_handle: jlong,
) {
    jni_guard_void!(env, {
        let vm = get_vm(vm_handle as u64)?;
        let exec_context = crate::context::get_execution_context(exec_context_handle as u64)?;
        vm.set_context(&exec_context)?;
        Ok(())
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
