//! Catalog callback bridge for JNI integration
//!
//! This module implements Rust CompilationCatalog and ExecutionCatalog traits
//! that delegate to Java implementations via JNI callbacks.
//!
//! # Architecture
//!
//! 1. Java catalog objects are stored as JNI GlobalRefs
//! 2. JavaVM pointer enables thread attachment for callbacks
//! 3. Rust wrapper implements catalog traits and calls Java methods
//! 4. Thread-safe access via Arc and proper JNI attachment

use dashmap::DashMap;
use jni::objects::GlobalRef;
use jni::sys::jmethodID;
use jni::JavaVM;
use once_cell::sync::Lazy;
use std::sync::{Arc, Mutex};

use partiql_common::catalog::{CatalogId, EntryId};
use partiql_eval::source::{
    BufferStability, DataSource, DataSourceConfig, DataSourceHandle, ScanCapabilities, ScanLayout,
    ScanSource,
};
use partiql_eval::{CompilationCatalog, EngineError, ExecutionCatalog, Result};
use partiql_value::BindingsName;

use std::sync::RwLock;

/// Global storage for JavaVM pointer
///
/// Initialized once when library loads, used for thread attachment
static JAVA_VM: Lazy<RwLock<Option<Arc<JavaVM>>>> = Lazy::new(|| RwLock::new(None));

/// Cached RegisterWriter class and method IDs
///
/// These are cached globally to avoid expensive find_class/get_method_id calls
/// on every nextRow() invocation (saves ~1-2μs per row)
static REGISTER_WRITER_CACHE: Lazy<Mutex<Option<RegisterWriterCache>>> =
    Lazy::new(|| Mutex::new(None));

struct RegisterWriterCache {
    class_ref: GlobalRef,
    constructor_id: jmethodID,
}

unsafe impl Send for RegisterWriterCache {}
unsafe impl Sync for RegisterWriterCache {}

/// Initialize the JavaVM pointer for catalog callbacks
///
/// Must be called once during library initialization
pub fn init_java_vm(vm: JavaVM) {
    if let Ok(mut guard) = JAVA_VM.write() {
        *guard = Some(Arc::new(vm));
    }
}

/// Initialize RegisterWriter class cache
///
/// Should be called once during initialization to cache class and method references
fn init_register_writer_cache(env: &mut jni::JNIEnv<'_>) -> Result<()> {
    let mut cache_guard = REGISTER_WRITER_CACHE
        .lock()
        .map_err(|e| EngineError::IllegalState(format!("Failed to lock cache: {}", e)))?;

    if cache_guard.is_none() {
        // Find RegisterWriter class
        let writer_class = env
            .find_class("org/partiql/jni/RegisterWriter")
            .map_err(|e| {
                EngineError::IllegalState(format!("Failed to find RegisterWriter class: {}", e))
            })?;

        // Create global reference
        let class_ref = env.new_global_ref(&writer_class).map_err(|e| {
            EngineError::IllegalState(format!("Failed to create GlobalRef: {}", e))
        })?;

        // Get constructor method ID: (J)V
        let constructor_id = env
            .get_method_id(writer_class, "<init>", "(J)V")
            .map_err(|e| {
                EngineError::IllegalState(format!("Failed to get constructor ID: {}", e))
            })?
            .into_raw();

        *cache_guard = Some(RegisterWriterCache {
            class_ref,
            constructor_id,
        });
    }

    Ok(())
}

/// Global storage for Java CompilationCatalog references
///
/// Maps catalog_id to (GlobalRef, JavaVM)
static COMPILATION_CATALOGS: Lazy<DashMap<CatalogId, (GlobalRef, Arc<JavaVM>)>> =
    Lazy::new(DashMap::new);

/// Global storage for Java ExecutionCatalog references
///
/// Maps catalog_id to (GlobalRef, JavaVM)
static EXECUTION_CATALOGS: Lazy<DashMap<CatalogId, (GlobalRef, Arc<JavaVM>)>> =
    Lazy::new(DashMap::new);

/// Register a Java CompilationCatalog for callbacks
///
/// Stores a GlobalRef that will be used for get_table() callbacks
pub fn register_compilation_catalog(
    catalog_id: CatalogId,
    catalog_ref: GlobalRef,
    vm: Arc<JavaVM>,
) {
    COMPILATION_CATALOGS.insert(catalog_id, (catalog_ref, vm));
}

/// Register a Java ExecutionCatalog for callbacks
///
/// Stores a GlobalRef that will be used for create() callbacks
pub fn register_execution_catalog(catalog_id: CatalogId, catalog_ref: GlobalRef, vm: Arc<JavaVM>) {
    EXECUTION_CATALOGS.insert(catalog_id, (catalog_ref, vm));
}

/// Remove a CompilationCatalog registration
#[allow(dead_code)]
pub fn unregister_compilation_catalog(catalog_id: CatalogId) {
    COMPILATION_CATALOGS.remove(&catalog_id);
}

/// Remove an ExecutionCatalog registration
#[allow(dead_code)]
pub fn unregister_execution_catalog(catalog_id: CatalogId) {
    EXECUTION_CATALOGS.remove(&catalog_id);
}

/// Convert Rust ScanLayout to Java ScanLayout object
fn convert_scan_layout_to_java<'a>(
    env: &mut jni::JNIEnv<'a>,
    layout: &ScanLayout,
) -> Result<jni::objects::JObject<'a>> {
    // Create ArrayList<ScanProjection> for projections
    let array_list_class = env
        .find_class("java/util/ArrayList")
        .map_err(|e| EngineError::IllegalState(format!("Failed to find ArrayList class: {}", e)))?;
    let proj_list = env
        .new_object(array_list_class, "()V", &[])
        .map_err(|e| EngineError::IllegalState(format!("Failed to create ArrayList: {}", e)))?;

    // Convert each Rust projection to Java
    for proj in layout.projections.iter() {
        let java_proj = convert_scan_projection_to_java(env, proj)?;
        env.call_method(
            &proj_list,
            "add",
            "(Ljava/lang/Object;)Z",
            &[jni::objects::JValue::Object(&java_proj)],
        )
        .map_err(|e| {
            EngineError::IllegalState(format!("Failed to add projection to list: {}", e))
        })?;
    }

    // Create Java ScanLayout with the projection list
    let layout_class = env.find_class("org/partiql/jni/ScanLayout").map_err(|e| {
        EngineError::IllegalState(format!("Failed to find ScanLayout class: {}", e))
    })?;

    let layout_obj = env
        .new_object(
            layout_class,
            "(Ljava/util/List;)V",
            &[jni::objects::JValue::Object(&proj_list)],
        )
        .map_err(|e| EngineError::IllegalState(format!("Failed to create ScanLayout: {}", e)))?;

    Ok(layout_obj)
}

/// Convert Rust ScanProjection to Java ScanProjection object
fn convert_scan_projection_to_java<'a>(
    env: &mut jni::JNIEnv<'a>,
    proj: &partiql_eval::source::ScanProjection,
) -> Result<jni::objects::JObject<'a>> {
    // Convert ScanSource
    let java_source = convert_scan_source_to_java(env, &proj.source)?;

    // Get target slot
    let target_slot = proj.target_slot as i32;

    // Create TypeHint.ANY
    let type_hint_class = env
        .find_class("org/partiql/jni/TypeHint")
        .map_err(|e| EngineError::IllegalState(format!("Failed to find TypeHint class: {}", e)))?;
    let type_hint_any = env
        .get_static_field(type_hint_class, "ANY", "Lorg/partiql/jni/TypeHint;")
        .map_err(|e| EngineError::IllegalState(format!("Failed to get TypeHint.ANY: {}", e)))?
        .l()
        .map_err(|e| EngineError::IllegalState(format!("Failed to convert TypeHint.ANY: {}", e)))?;

    // Create ScanProjection
    let proj_class = env
        .find_class("org/partiql/jni/ScanProjection")
        .map_err(|e| {
            EngineError::IllegalState(format!("Failed to find ScanProjection class: {}", e))
        })?;

    let proj_obj = env
        .new_object(
            proj_class,
            "(Lorg/partiql/jni/ScanSource;ILorg/partiql/jni/TypeHint;)V",
            &[
                jni::objects::JValue::Object(&java_source),
                jni::objects::JValue::Int(target_slot),
                jni::objects::JValue::Object(&type_hint_any),
            ],
        )
        .map_err(|e| {
            EngineError::IllegalState(format!("Failed to create ScanProjection: {}", e))
        })?;

    Ok(proj_obj)
}

/// Convert Rust ScanSource to Java ScanSource object
fn convert_scan_source_to_java<'a>(
    env: &mut jni::JNIEnv<'a>,
    source: &ScanSource,
) -> Result<jni::objects::JObject<'a>> {
    match source {
        ScanSource::ColumnIndex(idx) => {
            // Create ScanSource.ColumnIndex
            let source_class = env
                .find_class("org/partiql/jni/ScanSource$ColumnIndex")
                .map_err(|e| {
                    EngineError::IllegalState(format!(
                        "Failed to find ScanSource.ColumnIndex class: {}",
                        e
                    ))
                })?;

            let source_obj = env
                .new_object(
                    source_class,
                    "(I)V",
                    &[jni::objects::JValue::Int(*idx as i32)],
                )
                .map_err(|e| {
                    EngineError::IllegalState(format!(
                        "Failed to create ScanSource.ColumnIndex: {}",
                        e
                    ))
                })?;

            Ok(source_obj)
        }
        ScanSource::FieldPath(path) => {
            // Create Java string for the field path
            let path_str: &str = path.as_ref();
            let java_path = env
                .new_string(path_str)
                .map_err(|e| {
                    EngineError::IllegalState(format!("Failed to create Java string: {}", e))
                })?;

            // Create ScanSource.FieldPath
            let source_class = env
                .find_class("org/partiql/jni/ScanSource$FieldPath")
                .map_err(|e| {
                    EngineError::IllegalState(format!(
                        "Failed to find ScanSource.FieldPath class: {}",
                        e
                    ))
                })?;

            let source_obj = env
                .new_object(
                    source_class,
                    "(Ljava/lang/String;)V",
                    &[jni::objects::JValue::Object(&java_path)],
                )
                .map_err(|e| {
                    EngineError::IllegalState(format!(
                        "Failed to create ScanSource.FieldPath: {}",
                        e
                    ))
                })?;

            Ok(source_obj)
        }
        _ => {
            // Other ScanSource types not yet supported
            Err(EngineError::IllegalState(format!(
                "Unsupported ScanSource type: {:?}",
                source
            )))
        }
    }
}

/// Rust CompilationCatalog that delegates to Java via JNI
pub struct JavaCompilationCatalog {
    catalog_id: std::sync::RwLock<Option<CatalogId>>,
}

impl JavaCompilationCatalog {
    pub fn new() -> Self {
        JavaCompilationCatalog {
            catalog_id: std::sync::RwLock::new(None),
        }
    }

    pub fn set_catalog_id(&self, catalog_id: CatalogId) {
        if let Ok(mut guard) = self.catalog_id.write() {
            *guard = Some(catalog_id);
        }
    }

    fn get_catalog_id(&self) -> Option<CatalogId> {
        self.catalog_id.read().ok().and_then(|guard| *guard)
    }
}

impl CompilationCatalog for JavaCompilationCatalog {
    fn get_table(&self, path: &[BindingsName<'_>]) -> Option<DataSourceHandle> {
        // Get the catalog ID
        let catalog_id = self.get_catalog_id()?;

        // Get the registered catalog
        let entry = COMPILATION_CATALOGS.get(&catalog_id)?;
        let (catalog_ref, vm) = entry.value();

        // Attach to JVM for this thread
        let mut env = vm.attach_current_thread().ok()?;

        // Create ArrayList<BindingsName> for Java
        let array_list_class = env.find_class("java/util/ArrayList").ok()?;
        let list = env.new_object(array_list_class, "()V", &[]).ok()?;

        // Convert path to Java BindingsName objects and add to list
        for name in path.iter() {
            let (name_str, is_case_sensitive) = match name {
                BindingsName::CaseSensitive(s) => (s.as_ref(), true),
                BindingsName::CaseInsensitive(s) => (s.as_ref(), false),
            };

            let java_name_str = env.new_string(name_str).ok()?;

            // Create BindingsName object
            let bindings_name_class = env.find_class("org/partiql/jni/BindingsName").ok()?;
            let bindings_name = if is_case_sensitive {
                // Call static delimited(String) method
                env.call_static_method(
                    bindings_name_class,
                    "delimited",
                    "(Ljava/lang/String;)Lorg/partiql/jni/BindingsName;",
                    &[jni::objects::JValue::Object(&java_name_str)],
                )
                .ok()?
                .l()
                .ok()?
            } else {
                // Call static undelimited(String) method
                env.call_static_method(
                    bindings_name_class,
                    "undelimited",
                    "(Ljava/lang/String;)Lorg/partiql/jni/BindingsName;",
                    &[jni::objects::JValue::Object(&java_name_str)],
                )
                .ok()?
                .l()
                .ok()?
            };

            // Add to list
            env.call_method(
                &list,
                "add",
                "(Ljava/lang/Object;)Z",
                &[jni::objects::JValue::Object(&bindings_name)],
            )
            .ok()?;
        }

        // Call Java method: DataSourceHandle getTable(List<BindingsName> path)
        let result = env
            .call_method(
                catalog_ref.as_obj(),
                "getTable",
                "(Ljava/util/List;)Lorg/partiql/jni/DataSourceHandle;",
                &[jni::objects::JValue::Object(&list)],
            )
            .ok()?;

        let handle_obj = result.l().ok()?;
        if handle_obj.is_null() {
            return None;
        }

        // Extract entry_id from the Java DataSourceHandle
        let entry_id_value = env
            .call_method(&handle_obj, "getEntryId", "()J", &[])
            .ok()?
            .j()
            .ok()?;

        // Get the DataSourceConfig from the handle
        let config_obj = env
            .call_method(
                &handle_obj,
                "getConfig",
                "()Lorg/partiql/jni/DataSourceConfig;",
                &[],
            )
            .ok()?
            .l()
            .ok()?;

        // Create GlobalRef for the config
        let config_ref = env.new_global_ref(config_obj).ok()?;

        // Create JavaDataSourceConfig wrapper
        let config = Arc::new(JavaDataSourceConfig::new(config_ref, vm.clone()));

        Some(DataSourceHandle::new(
            EntryId::from(entry_id_value as u64),
            config,
        ))
    }
}

/// Rust ExecutionCatalog that delegates to Java via JNI
pub struct JavaExecutionCatalog {
    catalog_id: CatalogId,
}

impl JavaExecutionCatalog {
    pub fn new(catalog_id: CatalogId) -> Self {
        JavaExecutionCatalog { catalog_id }
    }
}

impl ExecutionCatalog for JavaExecutionCatalog {
    fn create(&self, entry_id: EntryId, layout: ScanLayout) -> Result<Box<dyn DataSource>> {
        // Get the registered catalog
        let entry = EXECUTION_CATALOGS
            .get(&self.catalog_id)
            .ok_or_else(|| EngineError::IllegalState("Catalog not registered".to_string()))?;
        let (catalog_ref, vm) = entry.value();

        // Attach to JVM for this thread
        let mut env = vm
            .attach_current_thread()
            .map_err(|e| EngineError::IllegalState(format!("Failed to attach to JVM: {}", e)))?;

        // Convert EntryId to jlong
        let entry_id_value = u64::from(entry_id) as i64;

        // Convert Rust ScanLayout to Java ScanLayout
        let layout_obj = convert_scan_layout_to_java(&mut env, &layout)?;

        // Call Java method: DataSource create(long entryId, ScanLayout layout)
        let result = env
            .call_method(
                catalog_ref.as_obj(),
                "create",
                "(JLorg/partiql/jni/ScanLayout;)Lorg/partiql/jni/DataSource;",
                &[
                    jni::objects::JValue::Long(entry_id_value),
                    jni::objects::JValue::Object(&layout_obj),
                ],
            )
            .map_err(|e| EngineError::IllegalState(format!("Failed to call create(): {}", e)))?;

        let data_source_obj = result.l().map_err(|e| {
            EngineError::IllegalState(format!("Failed to get DataSource object: {}", e))
        })?;

        if data_source_obj.is_null() {
            return Err(EngineError::IllegalState(
                "Java ExecutionCatalog.create() returned null".to_string(),
            ));
        }

        // Create GlobalRef for the DataSource
        let data_source_ref = env
            .new_global_ref(data_source_obj)
            .map_err(|e| EngineError::IllegalState(format!("Failed to create GlobalRef: {}", e)))?;

        // Wrap in JavaDataSource
        Ok(Box::new(JavaDataSource::new(
            data_source_ref,
            vm.clone(),
            layout,
        )))
    }
}

/// Rust DataSource that delegates to Java via JNI
///
/// Wraps a Java DataSource object and implements the Rust DataSource trait.
/// Handles JNI callbacks for open(), next_row(), and close().
struct JavaDataSource {
    data_source_ref: GlobalRef,
    vm: Arc<JavaVM>,
    writer_obj: Option<GlobalRef>,  // Cached RegisterWriter object for reuse
}

impl JavaDataSource {
    fn new(data_source_ref: GlobalRef, vm: Arc<JavaVM>, _layout: ScanLayout) -> Self {
        JavaDataSource {
            data_source_ref,
            vm,
            writer_obj: None,
        }
    }
}

impl DataSource for JavaDataSource {
    fn open(&mut self) -> Result<()> {
        let mut env = self
            .vm
            .attach_current_thread()
            .map_err(|e| EngineError::IllegalState(format!("Failed to attach to JVM: {}", e)))?;

        // Call Java method: void open()
        env.call_method(self.data_source_ref.as_obj(), "open", "()V", &[])
            .map_err(|e| EngineError::IllegalState(format!("Failed to call open(): {}", e)))?;

        Ok(())
    }

    fn next_row(
        &mut self,
        writer: &mut partiql_eval::source::RegisterWriter<'_, '_>,
    ) -> Result<bool> {
        
        // 1. Thread attachment
        let mut env = self
            .vm
            .attach_current_thread()
            .map_err(|e| EngineError::IllegalState(format!("Failed to attach to JVM: {}", e)))?;

        // 2. Initialize cache if needed (first call only)
        init_register_writer_cache(&mut env)?;

        // 3. Create a handle for the RegisterWriter
        let writer_ptr = writer as *mut partiql_eval::source::RegisterWriter<'_, '_>;
        let writer_handle = writer_ptr as i64;

        // 4. Create or reuse RegisterWriter object
        if self.writer_obj.is_none() {
            // First call: create the object
            let cache_guard = REGISTER_WRITER_CACHE
                .lock()
                .map_err(|e| EngineError::IllegalState(format!("Failed to lock cache: {}", e)))?;

            let cache = cache_guard
                .as_ref()
                .ok_or_else(|| EngineError::IllegalState("Cache not initialized".to_string()))?;

            let writer_obj = unsafe {
                let method_id = jni::objects::JMethodID::from_raw(cache.constructor_id);
                let args = [jni::sys::jvalue { j: writer_handle }];
                env.new_object_unchecked(&cache.class_ref, method_id, &args)
                    .map_err(|e| {
                        EngineError::IllegalState(format!("Failed to create RegisterWriter: {}", e))
                    })?
            };

            // Store as global reference for reuse
            let global_ref = env.new_global_ref(writer_obj).map_err(|e| {
                EngineError::IllegalState(format!("Failed to create GlobalRef: {}", e))
            })?;

            self.writer_obj = Some(global_ref);
        } else {
            // Subsequent calls: update the handle in existing object
            let writer_obj = self.writer_obj.as_ref().unwrap();
            env.set_field(
                writer_obj.as_obj(),
                "nativeHandle",
                "J",
                jni::objects::JValue::Long(writer_handle),
            )
            .map_err(|e| {
                EngineError::IllegalState(format!("Failed to update nativeHandle: {}", e))
            })?;
        }

        let writer_obj = self.writer_obj.as_ref().unwrap();

        // 5. Call Java method: boolean nextRow(RegisterWriter writer)
        let result = env
            .call_method(
                self.data_source_ref.as_obj(),
                "nextRow",
                "(Lorg/partiql/jni/RegisterWriter;)Z",
                &[jni::objects::JValue::Object(writer_obj.as_obj())],
            )
            .map_err(|e| EngineError::IllegalState(format!("Failed to call nextRow(): {}", e)))?;

        // 6. Extract result
        let has_next = result.z()
            .map_err(|e| {
                EngineError::IllegalState(format!("Failed to get boolean result: {}", e))
            })?;


        Ok(has_next)
    }

    fn close(&mut self) -> Result<()> {
        let mut env = self
            .vm
            .attach_current_thread()
            .map_err(|e| EngineError::IllegalState(format!("Failed to attach to JVM: {}", e)))?;

        // Call Java method: void close()
        env.call_method(self.data_source_ref.as_obj(), "close", "()V", &[])
            .map_err(|e| EngineError::IllegalState(format!("Failed to call close(): {}", e)))?;

        Ok(())
    }
}

/// DataSourceConfig that delegates to Java via JNI
///
/// Provides compile-time metadata for data sources by calling back to Java DataSourceConfig.
struct JavaDataSourceConfig {
    config_ref: GlobalRef,
    vm: Arc<JavaVM>,
}

impl JavaDataSourceConfig {
    fn new(config_ref: GlobalRef, vm: Arc<JavaVM>) -> Self {
        JavaDataSourceConfig { config_ref, vm }
    }
}

impl DataSourceConfig for JavaDataSourceConfig {
    fn caps(&self) -> ScanCapabilities {
        // Attach to JVM
        let mut env = match self.vm.attach_current_thread() {
            Ok(env) => env,
            Err(_) => {
                // If JVM attachment fails, return conservative defaults
                return ScanCapabilities {
                    stability: BufferStability::UntilNext,
                    can_project: false,
                    can_return_opaque: false,
                };
            }
        };

        // Call Java method: ScanCapabilities getCaps()
        let caps_obj = match env.call_method(
            self.config_ref.as_obj(),
            "getCaps",
            "()Lorg/partiql/jni/ScanCapabilities;",
            &[],
        ) {
            Ok(result) => match result.l() {
                Ok(obj) => obj,
                Err(_) => {
                    return ScanCapabilities {
                        stability: BufferStability::UntilNext,
                        can_project: false,
                        can_return_opaque: false,
                    }
                }
            },
            Err(_) => {
                return ScanCapabilities {
                    stability: BufferStability::UntilNext,
                    can_project: false,
                    can_return_opaque: false,
                }
            }
        };

        // Extract capabilities from Java object
        let can_project = env
            .call_method(&caps_obj, "canProject", "()Z", &[])
            .and_then(|v| v.z())
            .unwrap_or(false);

        let can_return_opaque = env
            .call_method(&caps_obj, "canReturnOpaque", "()Z", &[])
            .and_then(|v| v.z())
            .unwrap_or(false);

        // For now, always use UntilNext stability (most conservative)
        ScanCapabilities {
            stability: BufferStability::UntilNext,
            can_project,
            can_return_opaque,
        }
    }

    fn resolve(&self, field_name: &str) -> Option<ScanSource> {
        // Attach to JVM
        let mut env = self.vm.attach_current_thread().ok()?;

        // Convert field name to Java string
        let java_field_name = env.new_string(field_name).ok()?;

        // Call Java method: ScanSource resolve(String fieldName)
        let result = env
            .call_method(
                self.config_ref.as_obj(),
                "resolve",
                "(Ljava/lang/String;)Lorg/partiql/jni/ScanSource;",
                &[jni::objects::JValue::Object(&java_field_name)],
            )
            .ok()?;

        let source_obj = result.l().ok()?;
        if source_obj.is_null() {
            return None;
        }

        // Extract column index from ScanSource
        let column_index = env
            .call_method(&source_obj, "getIndex", "()I", &[])
            .ok()?
            .i()
            .ok()?;

        Some(ScanSource::ColumnIndex(column_index as usize))
    }
}
