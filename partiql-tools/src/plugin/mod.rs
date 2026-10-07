//! Table-function plugins loaded from C-ABI shared libraries.
//!
//! The ABI is specified in `include/pqlite_plugin.h`; `ffi` mirrors it. A
//! plugin exports `pqlite_plugin_init`, receives a `PqliteHostV1` context
//! (logging, config), and returns a vtable of table functions. `load_plugin`
//! turns each function into a `TableFnDef` for the session's
//! `TableFnRegistry`. Libraries are never unloaded.

mod args;
pub mod ffi;
mod source;

use std::ffi::c_void;
use std::path::Path;
use std::sync::{Arc, OnceLock};

use crate::table_fns::{TableFnDef, TableFnRegistry};
use ffi::{PluginInitFn, PqliteHostV1, PqliteKeyValue, PqlitePluginV1, PqliteStr};

/// A plugin whose `pqlite_plugin_init` succeeded. Lives for the process.
pub struct LoadedPlugin {
    pub name: String,
    pub version: String,
    /// Our copy of the plugin's vtable (see `ffi::read_prefixed`).
    vtable: PqlitePluginV1,
}

// SAFETY: the header requires `open` and the stream callbacks to be callable
// from any thread, and the vtable is immutable after init.
unsafe impl Send for LoadedPlugin {}
unsafe impl Sync for LoadedPlugin {}

/// Summary of a loaded plugin, for `.help` and diagnostics.
pub struct PluginInfo {
    pub name: String,
    pub version: String,
    pub functions: Vec<String>,
}

/// Load the shared library at `path`, initialise it with `config`, and add its
/// table functions to `registry`.
pub fn load_plugin(
    path: &Path,
    config: &[(String, String)],
    registry: &mut TableFnRegistry,
) -> Result<PluginInfo, String> {
    let label = path
        .file_name()
        .map(|n| n.to_string_lossy().into_owned())
        .unwrap_or_else(|| path.display().to_string());
    // SAFETY: loading a plugin runs its initialisers; plugins are trusted
    // native code by contract (see header).
    let lib = unsafe { libloading::Library::new(path) }
        .map_err(|e| format!("could not load plugin {}: {e}", path.display()))?;
    let init: PluginInitFn = unsafe {
        *lib.get::<PluginInitFn>(ffi::PQLITE_PLUGIN_INIT_SYMBOL)
            .map_err(|e| format!("{}: missing pqlite_plugin_init: {e}", path.display()))?
    };
    // Never dlclose: Arrow release callbacks and the vtable point into it.
    std::mem::forget(lib);
    // SAFETY: `init` has the signature the header declares.
    unsafe { load_from_init(&label, init, config, registry) }
}

/// Initialise a plugin from its init function and register its functions.
/// `load_plugin` without the `dlopen`; also used by tests to exercise the
/// ABI in-process.
///
/// # Safety
/// `init` must implement `pqlite_plugin_init` per `pqlite_plugin.h`.
pub unsafe fn load_from_init(
    label: &str,
    init: PluginInitFn,
    config: &[(String, String)],
    registry: &mut TableFnRegistry,
) -> Result<PluginInfo, String> {
    // Boxed until init succeeds; after that the plugin may keep `host`, so
    // both are leaked for the process (see header).
    let host_state = Box::new(HostState {
        label: label.to_string(),
        name: OnceLock::new(),
    });
    let kvs: Vec<PqliteKeyValue> = config
        .iter()
        .map(|(k, v)| PqliteKeyValue {
            key: PqliteStr::new(k),
            value: PqliteStr::new(v),
        })
        .collect();
    let mut host = Box::new(PqliteHostV1 {
        struct_size: std::mem::size_of::<PqliteHostV1>() as u32,
        abi_version: ffi::PQLITE_PLUGIN_ABI_VERSION,
        host_name: PqliteStr::new("pqlite"),
        host_version: PqliteStr::new(HOST_VERSION),
        host_data: &*host_state as *const HostState as *mut c_void,
        max_log_level: max_log_level(),
        log: Some(host_log),
        config: kvs.as_ptr(),
        n_config: kvs.len(),
    });

    let mut out: *const PqlitePluginV1 = std::ptr::null();
    let rc = init(&*host, &mut out);
    // Config is only borrowed for the duration of init.
    host.config = std::ptr::null();
    host.n_config = 0;
    drop(kvs);
    if rc != 0 {
        return Err(format!("{label}: pqlite_plugin_init failed ({rc})"));
    }
    // From here on the plugin is initialised and may hold on to `host` (and
    // call `log` from its own threads), so it must stay valid for the process
    // even if the checks below reject the plugin. The cost is two small
    // allocations per plugin that loads.
    let host_state: &'static HostState = Box::leak(host_state);
    Box::leak(host);
    if out.is_null() {
        return Err(format!("{label}: pqlite_plugin_init returned no vtable"));
    }
    // `struct_size` and `abi_version` lead every vtable version; check the
    // version before trusting the rest of the layout.
    let abi_version = std::ptr::addr_of!((*out).abi_version).read();
    if abi_version != ffi::PQLITE_PLUGIN_ABI_VERSION {
        return Err(format!(
            "{label}: plugin ABI version {abi_version} does not match host ABI version {}",
            ffi::PQLITE_PLUGIN_ABI_VERSION
        ));
    }
    let (vt, _) = ffi::read_prefixed(out).map_err(|e| format!("{label}: plugin vtable: {e}"))?;
    if vt.open.is_none() || vt.free_string.is_none() {
        return Err(format!(
            "{label}: plugin vtable is missing open/free_string"
        ));
    }

    let name = utf8(vt.plugin_name, label, "plugin_name")?;
    let version = utf8(vt.plugin_version, label, "plugin_version")?;
    let _ = host_state.name.set(name.clone());
    let defs = fn_defs(&vt).map_err(|e| format!("{label}: {e}"))?;
    let plugin = Arc::new(LoadedPlugin {
        name: name.clone(),
        version: version.clone(),
        vtable: vt,
    });
    // Validate everything before registering anything, so a bad plugin
    // leaves the registry untouched.
    let mut staged: Vec<(String, usize, ffi::PqliteTableFnDef, String, _)> =
        Vec::with_capacity(defs.len());
    for (i, def) in defs.into_iter().enumerate() {
        let fn_name = utf8(def.name, label, "function name")?;
        if fn_name.is_empty() {
            return Err(format!("{label}: function #{i} has an empty name"));
        }
        if def.min_args > def.max_args || def.max_args > MAX_ARGS {
            return Err(format!(
                "{label}: {fn_name}: invalid arity {}..={} (max {MAX_ARGS})",
                def.min_args, def.max_args
            ));
        }
        if registry.get(&fn_name).is_some()
            || staged
                .iter()
                .any(|(name, ..)| name.eq_ignore_ascii_case(&fn_name))
        {
            return Err(format!(
                "{label}: table function '{fn_name}' is already registered"
            ));
        }
        let usage = utf8(def.usage, label, "usage")?;
        let static_schema = source::read_static_schema(&plugin, i as u32, &def)
            .map_err(|e| format!("{label}: {fn_name}: {e}"))?;
        staged.push((fn_name, i, def, usage, static_schema));
    }
    let mut functions = Vec::with_capacity(staged.len());
    for (fn_name, i, def, usage, static_schema) in staged {
        // CallDef names must be 'static; plugins are never unloaded. Leaked
        // only once the whole plugin has validated.
        let name: &'static str = Box::leak(fn_name.into_boxed_str());
        functions.push(name.to_string());
        registry.add(TableFnDef {
            name,
            arities: def.min_args as usize..=def.max_args as usize,
            metadata: Arc::new(source::PluginMetadata {
                static_schema: static_schema.clone(),
            }),
            factory: Arc::new(source::PluginTableFn {
                plugin: Arc::clone(&plugin),
                fn_index: i as u32,
                name,
                static_schema,
            }),
            usage: if usage.is_empty() {
                name.to_string()
            } else {
                usage
            },
        })?;
    }
    Ok(PluginInfo {
        name,
        version,
        functions,
    })
}

/// Copies of the plugin's function definitions. The array is strided by the
/// plugin's `struct_size` (see header), which may differ from ours.
unsafe fn fn_defs(vt: &PqlitePluginV1) -> Result<Vec<ffi::PqliteTableFnDef>, String> {
    if vt.n_functions == 0 {
        return Ok(Vec::new());
    }
    if vt.functions.is_null() {
        return Err("plugin declares functions but none are given".to_string());
    }
    let (first, stride) =
        ffi::read_prefixed(vt.functions).map_err(|e| format!("function #0: {e}"))?;
    // Alignment is a power of two (`is_multiple_of` is above MSRV).
    if stride & (std::mem::align_of::<ffi::PqliteTableFnDef>() - 1) != 0 {
        return Err(format!("function definition size {stride} is misaligned"));
    }
    let base = vt.functions.cast::<u8>();
    let mut defs = Vec::with_capacity(vt.n_functions);
    defs.push(first);
    for i in 1..vt.n_functions {
        let ptr = base.add(i * stride).cast::<ffi::PqliteTableFnDef>();
        let (def, size) = ffi::read_prefixed(ptr).map_err(|e| format!("function #{i}: {e}"))?;
        if size != stride {
            return Err(format!(
                "function #{i} has struct_size {size}, expected {stride}"
            ));
        }
        defs.push(def);
    }
    Ok(defs)
}

/// Each arity becomes its own planner overload; keep that list small.
const MAX_ARGS: u32 = 16;

const HOST_VERSION: &str = env!("CARGO_PKG_VERSION");

unsafe fn utf8(s: PqliteStr, label: &str, what: &str) -> Result<String, String> {
    std::str::from_utf8(s.as_bytes())
        .map(str::to_string)
        .map_err(|_| format!("{label}: {what} is not valid UTF-8"))
}

struct HostState {
    /// Library file name; used until the plugin reports its own name.
    label: String,
    name: OnceLock<String>,
}

/// `PQLITE_PLUGIN_LOG` = off | error | warn | info | debug | trace (default
/// warn). Read once per process.
fn max_log_level() -> i32 {
    static LEVEL: OnceLock<i32> = OnceLock::new();
    *LEVEL.get_or_init(|| {
        match std::env::var("PQLITE_PLUGIN_LOG")
            .unwrap_or_default()
            .to_ascii_lowercase()
            .as_str()
        {
            "off" | "none" => 0,
            "error" => ffi::PQLITE_LOG_ERROR,
            "" | "warn" => ffi::PQLITE_LOG_WARN,
            "info" => ffi::PQLITE_LOG_INFO,
            "debug" => ffi::PQLITE_LOG_DEBUG,
            "trace" => ffi::PQLITE_LOG_TRACE,
            other => {
                eprintln!(
                    "pqlite: ignoring PQLITE_PLUGIN_LOG={other:?} \
                     (expected off|error|warn|info|debug|trace); using warn"
                );
                ffi::PQLITE_LOG_WARN
            }
        }
    })
}

unsafe extern "C" fn host_log(
    host_data: *mut c_void,
    level: i32,
    target: PqliteStr,
    message: PqliteStr,
) {
    let _ = std::panic::catch_unwind(|| {
        if level > max_log_level() {
            return;
        }
        let state = &*(host_data as *const HostState);
        let who = state.name.get().unwrap_or(&state.label);
        let level = match level {
            ffi::PQLITE_LOG_ERROR => "ERROR",
            ffi::PQLITE_LOG_WARN => "WARN",
            ffi::PQLITE_LOG_INFO => "INFO",
            ffi::PQLITE_LOG_DEBUG => "DEBUG",
            _ => "TRACE",
        };
        let target = target.to_string_lossy();
        let sep = if target.is_empty() { "" } else { "/" };
        eprintln!(
            "[{who}{sep}{target}] {level}: {}",
            message.to_string_lossy()
        );
    });
}

#[cfg(test)]
mod tests;
