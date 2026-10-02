//! `#[repr(C)]` mirror of `include/pqlite_plugin.h`. Keep the two in sync;
//! the header is the source of truth for the contract.

use std::ffi::{c_char, c_void};

use arrow_array::ffi_stream::FFI_ArrowArrayStream;
use arrow_schema::ffi::FFI_ArrowSchema;

pub const PQLITE_PLUGIN_ABI_VERSION: u32 = 1;
pub const PQLITE_PLUGIN_INIT_SYMBOL: &[u8] = b"pqlite_plugin_init\0";

#[repr(C)]
#[derive(Clone, Copy)]
pub struct PqliteStr {
    pub ptr: *const c_char,
    pub len: usize,
}

impl PqliteStr {
    pub fn new(s: &str) -> Self {
        PqliteStr {
            ptr: s.as_ptr().cast(),
            len: s.len(),
        }
    }

    pub fn bytes(b: &[u8]) -> Self {
        PqliteStr {
            ptr: b.as_ptr().cast(),
            len: b.len(),
        }
    }

    /// # Safety
    /// `ptr` must point at `len` readable bytes for the returned lifetime.
    pub unsafe fn as_bytes<'a>(&self) -> &'a [u8] {
        if self.len == 0 || self.ptr.is_null() {
            &[]
        } else {
            std::slice::from_raw_parts(self.ptr.cast(), self.len)
        }
    }

    /// Lossy UTF-8 copy.
    ///
    /// # Safety
    /// As for `as_bytes`.
    pub unsafe fn to_string_lossy(&self) -> String {
        String::from_utf8_lossy(self.as_bytes()).into_owned()
    }
}

#[repr(C)]
pub struct PqliteKeyValue {
    pub key: PqliteStr,
    pub value: PqliteStr,
}

pub const PQLITE_LOG_ERROR: i32 = 1;
pub const PQLITE_LOG_WARN: i32 = 2;
pub const PQLITE_LOG_INFO: i32 = 3;
pub const PQLITE_LOG_DEBUG: i32 = 4;
pub const PQLITE_LOG_TRACE: i32 = 5;

pub type LogFn =
    unsafe extern "C" fn(host_data: *mut c_void, level: i32, target: PqliteStr, message: PqliteStr);

#[repr(C)]
pub struct PqliteHostV1 {
    pub struct_size: u32,
    pub abi_version: u32,
    pub host_name: PqliteStr,
    pub host_version: PqliteStr,
    pub host_data: *mut c_void,
    pub max_log_level: i32,
    pub log: Option<LogFn>,
    pub config: *const PqliteKeyValue,
    pub n_config: usize,
}

pub const PQLITE_ARG_NULL: i32 = 0;
pub const PQLITE_ARG_MISSING: i32 = 1;
pub const PQLITE_ARG_BOOL: i32 = 2;
pub const PQLITE_ARG_INT: i32 = 3;
pub const PQLITE_ARG_FLOAT: i32 = 4;
pub const PQLITE_ARG_STRING: i32 = 5;
pub const PQLITE_ARG_BYTES: i32 = 6;
pub const PQLITE_ARG_ION_TEXT: i32 = 7;

#[repr(C)]
#[derive(Clone, Copy)]
pub union PqliteArgValue {
    pub b: i32,
    pub i: i64,
    pub f: f64,
    pub s: PqliteStr,
}

#[repr(C)]
#[derive(Clone, Copy)]
pub struct PqliteArg {
    pub kind: i32,
    pub v: PqliteArgValue,
}

pub type StaticSchemaFn =
    unsafe extern "C" fn(plugin_data: *mut c_void, fn_index: u32, out: *mut FFI_ArrowSchema) -> i32;

#[repr(C)]
pub struct PqliteTableFnDef {
    pub struct_size: u32,
    pub name: PqliteStr,
    pub usage: PqliteStr,
    pub min_args: u32,
    pub max_args: u32,
    pub static_schema: Option<StaticSchemaFn>,
}

#[repr(C)]
pub struct PqliteOpenRequest {
    pub struct_size: u32,
    pub fn_index: u32,
    pub args: *const PqliteArg,
    pub n_args: usize,
    pub whole_row: i32,
    pub fields: *const PqliteStr,
    pub n_fields: usize,
    pub cancelled: *const u32,
}

pub type OpenFn = unsafe extern "C" fn(
    plugin_data: *mut c_void,
    req: *const PqliteOpenRequest,
    out: *mut FFI_ArrowArrayStream,
    err: *mut *mut c_char,
) -> i32;

pub type FreeStringFn = unsafe extern "C" fn(s: *mut c_char);

#[repr(C)]
pub struct PqlitePluginV1 {
    pub struct_size: u32,
    pub abi_version: u32,
    pub plugin_name: PqliteStr,
    pub plugin_version: PqliteStr,
    pub plugin_data: *mut c_void,
    pub n_functions: usize,
    pub functions: *const PqliteTableFnDef,
    pub open: Option<OpenFn>,
    pub free_string: Option<FreeStringFn>,
}

pub type PluginInitFn =
    unsafe extern "C" fn(host: *const PqliteHostV1, out: *mut *const PqlitePluginV1) -> i32;
