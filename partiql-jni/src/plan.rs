use jni::objects::JClass;
use jni::sys::jlong;
use jni::JNIEnv;

use crate::{jni_guard_void, remove_plan_handle};

/// Close the CompiledPlan and release resources
///
/// Java signature:
/// ```java
/// private static native void nativeClose(long handle);
/// ```
#[no_mangle]
pub extern "system" fn Java_org_partiql_jni_CompiledPlan_nativeClose(
    mut env: JNIEnv<'_>,
    _class: JClass<'_>,
    plan_handle: jlong,
) {
    jni_guard_void!(env, {
        remove_plan_handle(plan_handle as u64);
        Ok(())
    })
}
