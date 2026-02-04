#![deny(rust_2018_idioms)]
#![deny(clippy::all)]

mod catalog_bridge;
mod compiler;
mod context;
mod conversion;
mod error;
mod handles;
mod plan;
mod register_reader;
mod register_writer;
mod result;
mod vm;

// Re-export for internal use
pub(crate) use error::*;
pub(crate) use handles::*;
pub(crate) use result::create_result_handle;

/// JNI_OnLoad - called when library is loaded
///
/// Initializes JavaVM pointer for catalog callbacks
#[no_mangle]
pub extern "system" fn JNI_OnLoad(
    vm: jni::JavaVM,
    _reserved: *mut std::ffi::c_void,
) -> jni::sys::jint {
    // Initialize JavaVM for catalog callbacks
    catalog_bridge::init_java_vm(vm);

    // Return JNI version
    jni::sys::JNI_VERSION_1_6
}
