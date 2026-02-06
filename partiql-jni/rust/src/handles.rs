use dashmap::DashMap;
use once_cell::sync::Lazy;
use std::collections::HashMap;
use std::sync::atomic::{AtomicU64, Ordering};
use std::sync::Arc;

use partiql_eval::{CompiledPlan, PartiQLVM};

use crate::error::JniError;

/// Buffer metadata for caching JNI buffer lookups
#[derive(Clone, Copy)]
pub struct BufferMetadata {
    pub ptr: *mut u8,
    pub capacity: usize,
}

/// VM state wrapper that includes buffer cache
pub struct VMState {
    pub vm: PartiQLVM,
    pub buffer_cache: HashMap<usize, BufferMetadata>,
}

// Global handle counter
static NEXT_HANDLE: AtomicU64 = AtomicU64::new(1);

/// Generate next unique handle ID
pub fn next_handle() -> u64 {
    NEXT_HANDLE.fetch_add(1, Ordering::Relaxed)
}

// Global handle storage for CompiledPlan (which is Send + Sync)
#[allow(dead_code)]
static PLAN_HANDLES: Lazy<DashMap<u64, Arc<CompiledPlan>>> = Lazy::new(DashMap::new);

// VM Handle operations - VMState is single-threaded, so we use raw pointers
// Java manages the lifetime by calling nativeClose()
pub fn create_vm_handle(vm: PartiQLVM) -> u64 {
    let vm_state = VMState {
        vm,
        buffer_cache: HashMap::new(),
    };
    let boxed = Box::new(vm_state);
    Box::into_raw(boxed) as u64
}

pub fn get_vm_state(handle: u64) -> Result<&'static mut VMState, JniError> {
    if handle == 0 {
        return Err(JniError::InvalidHandle);
    }
    // Safety: Handle is a valid pointer created by create_vm_handle
    // and Java ensures exclusive access per thread
    unsafe {
        let ptr = handle as *mut VMState;
        ptr.as_mut().ok_or(JniError::InvalidHandle)
    }
}

pub fn get_vm(handle: u64) -> Result<&'static mut PartiQLVM, JniError> {
    let vm_state = get_vm_state(handle)?;
    Ok(&mut vm_state.vm)
}

pub fn remove_vm_handle(handle: u64) {
    if handle != 0 {
        // Safety: Reconstruct Box to properly drop the VMState
        unsafe {
            let _ = Box::from_raw(handle as *mut VMState);
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
