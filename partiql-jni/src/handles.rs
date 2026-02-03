use dashmap::DashMap;
use once_cell::sync::Lazy;
use std::sync::Arc;

use partiql_eval::{CompiledPlan, PartiQLVM};

use crate::error::JniError;

// Global handle storage for CompiledPlan (which is Send + Sync)
#[allow(dead_code)]
static PLAN_HANDLES: Lazy<DashMap<u64, Arc<CompiledPlan>>> = Lazy::new(DashMap::new);

// VM Handle operations - PartiQLVM is single-threaded, so we use raw pointers
// Java manages the lifetime by calling nativeClose()
pub fn create_vm_handle(vm: PartiQLVM) -> u64 {
    let boxed = Box::new(vm);
    Box::into_raw(boxed) as u64
}

pub fn get_vm(handle: u64) -> Result<&'static mut PartiQLVM, JniError> {
    if handle == 0 {
        return Err(JniError::InvalidHandle);
    }
    // Safety: Handle is a valid pointer created by create_vm_handle
    // and Java ensures exclusive access per thread
    unsafe {
        let ptr = handle as *mut PartiQLVM;
        ptr.as_mut().ok_or(JniError::InvalidHandle)
    }
}

pub fn remove_vm_handle(handle: u64) {
    if handle != 0 {
        // Safety: Reconstruct Box to properly drop the VM
        unsafe {
            let _ = Box::from_raw(handle as *mut PartiQLVM);
        }
    }
}

// CompiledPlan Handle operations - using raw pointer for consistency
pub fn create_plan_handle(plan: CompiledPlan) -> u64 {
    let arc_plan = Arc::new(plan);
    Arc::into_raw(arc_plan) as u64
}

pub fn get_plan(handle: u64) -> Result<Arc<CompiledPlan>, JniError> {
    if handle == 0 {
        return Err(JniError::InvalidHandle);
    }
    // Safety: Handle is a valid pointer created by create_plan_handle
    // Arc is cloned to increment reference count
    unsafe {
        let ptr = handle as *const CompiledPlan;
        let arc = Arc::from_raw(ptr);
        let cloned = arc.clone();
        // Prevent dropping the original Arc
        let _ = Arc::into_raw(arc);
        Ok(cloned)
    }
}

pub fn remove_plan_handle(handle: u64) {
    if handle != 0 {
        // Safety: Reconstruct Arc to properly drop the plan
        unsafe {
            let _ = Arc::from_raw(handle as *const CompiledPlan);
        }
    }
}
