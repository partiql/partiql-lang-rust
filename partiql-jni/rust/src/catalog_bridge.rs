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
    BufferStability, CatalogScans, DataSource, DataSourceConfig, DataSourceHandle,
    ScanCapabilities, ScanId, ScanLayout, ScanSource,
};
use partiql_eval::{CompilationCatalog, CompiledPlan, EngineError, ExecutionCatalog, Result};
use partiql_value::BindingsName;
use std::collections::HashMap;

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
        let class_ref = env
            .new_global_ref(&writer_class)
            .map_err(|e| EngineError::IllegalState(format!("Failed to create GlobalRef: {}", e)))?;

        // Get constructor method ID: (J)V
        let constructor_id = env
            .get_method_id(writer_class, "<init>", "(J)V")
            .map_err(|e| EngineError::IllegalState(format!("Failed to get constructor ID: {}", e)))?
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
            let java_path = env.new_string(path_str).map_err(|e| {
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
///
/// TODO: This type will be deleted once we migrate to a pure-Rust catalog system.
/// It exists for backward compatibility with Java-based catalog implementations.
#[allow(dead_code)]
pub struct JavaExecutionCatalog {
    catalog_id: CatalogId,
    /// Mapping from ScanId to (EntryId, ScanLayout) built during prepare()
    scan_mappings: HashMap<ScanId, (EntryId, ScanLayout)>,
}

#[allow(dead_code)]
impl JavaExecutionCatalog {
    pub fn new(catalog_id: CatalogId) -> Self {
        JavaExecutionCatalog {
            catalog_id,
            scan_mappings: HashMap::new(),
        }
    }

    /// Prepare the catalog by inspecting the CompiledPlan
    pub fn prepare(&mut self, compiled: &CompiledPlan) {
        self.scan_mappings.clear();
        for (scan_id, scan_meta) in compiled.scans() {
            self.scan_mappings.insert(
                scan_id,
                (scan_meta.object_id.entry_id(), scan_meta.layout.clone()),
            );
        }
    }
}

/// Rust ExecutionCatalog that reads from a pre-populated DirectByteBuffer
///
/// This catalog provides optimal performance by eliminating Rust-to-Java callbacks
/// during query execution. All data is pre-populated in the buffer at registration time.
pub struct JavaBufferedExecutionCatalog {
    entry_id: EntryId,
    buffer: Vec<u8>, // Owned copy of the buffer data
    /// Mapping from ScanId to (EntryId, ScanLayout) built during prepare()
    scan_mappings: HashMap<ScanId, (EntryId, ScanLayout)>,
}

impl JavaBufferedExecutionCatalog {
    /// Create a new buffered catalog from a DirectByteBuffer
    pub fn new(entry_id: EntryId, buffer: Vec<u8>) -> Self {
        JavaBufferedExecutionCatalog {
            entry_id,
            buffer,
            scan_mappings: HashMap::new(),
        }
    }

    /// Prepare the catalog by inspecting the CompiledPlan
    #[allow(dead_code)] // Part of public API, called by customers
    pub fn prepare(&mut self, compiled: &CompiledPlan) {
        self.scan_mappings.clear();
        for (scan_id, scan_meta) in compiled.scans() {
            self.scan_mappings.insert(
                scan_id,
                (scan_meta.object_id.entry_id(), scan_meta.layout.clone()),
            );
        }
    }
}

impl ExecutionCatalog for JavaBufferedExecutionCatalog {
    fn prepare(&mut self, scans: &CatalogScans) {
        self.scan_mappings.clear();
        for (scan_id, entry_id, layout) in scans.iter() {
            self.scan_mappings
                .insert(scan_id, (entry_id, layout.clone()));
        }
    }

    fn create(&self, scan_id: ScanId) -> Result<Box<dyn DataSource>> {
        // Look up the scan mapping
        let (entry_id, layout) = self.scan_mappings.get(&scan_id).ok_or_else(|| {
            EngineError::IllegalState(format!(
                "ScanId {:?} not found in catalog mappings. Did you call prepare()?",
                scan_id
            ))
        })?;

        // Verify entry_id matches (currently only support single entry ID)
        if *entry_id != self.entry_id {
            return Err(EngineError::IllegalState(format!(
                "Entry ID mismatch: expected {}, got {}. TODO: Support multiple entry IDs",
                u64::from(self.entry_id),
                u64::from(*entry_id)
            )));
        }

        // Create BufferDataSource that reads from our buffer with the layout
        Ok(Box::new(BufferDataSource::new(
            self.buffer.clone(),
            layout.clone(),
        )))
    }
}

impl ExecutionCatalog for JavaExecutionCatalog {
    fn prepare(&mut self, scans: &CatalogScans) {
        self.scan_mappings.clear();
        for (scan_id, entry_id, layout) in scans.iter() {
            self.scan_mappings
                .insert(scan_id, (entry_id, layout.clone()));
        }
    }

    fn create(&self, scan_id: ScanId) -> Result<Box<dyn DataSource>> {
        // Look up the scan mapping
        let (entry_id, layout) = self.scan_mappings.get(&scan_id).ok_or_else(|| {
            EngineError::IllegalState(format!(
                "ScanId {:?} not found in catalog mappings. Did you call prepare()?",
                scan_id
            ))
        })?;

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
        let entry_id_value = u64::from(*entry_id) as i64;

        // Convert Rust ScanLayout to Java ScanLayout
        let layout_obj = convert_scan_layout_to_java(&mut env, layout)?;

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
            layout.clone(),
        )))
    }
}

/// Rust DataSource that reads directly from a pre-populated buffer
///
/// This DataSource provides optimal performance by reading data directly
/// from memory without any JNI callbacks. All data is in the buffer upfront.
struct BufferDataSource {
    buffer: Vec<u8>,
    offset: usize,
    layout: ScanLayout,
}

impl BufferDataSource {
    fn new(buffer: Vec<u8>, layout: ScanLayout) -> Self {
        BufferDataSource {
            buffer,
            offset: 0,
            layout,
        }
    }
}

impl DataSource for BufferDataSource {
    fn open(&mut self) -> Result<()> {
        // Reset offset to start of buffer
        self.offset = 0;
        Ok(())
    }

    fn next_row(
        &mut self,
        writer: &mut partiql_eval::source::RegisterWriter<'_, '_>,
    ) -> Result<bool> {
        // Check if we've reached the end
        if self.offset >= self.buffer.len() {
            return Ok(false);
        }

        // Decode values from buffer and write to registers
        // Returns true if row was written, false if no more rows
        self.decode_and_write_row(writer)
    }

    fn close(&mut self) -> Result<()> {
        // Nothing to clean up
        Ok(())
    }
}

impl BufferDataSource {
    /// Decode one row from buffer and write to registers
    ///
    /// Buffer format (same as RegisterWriter):
    /// For each field: [slot: u16][type_tag: u8][data: variable]
    fn decode_and_write_row(
        &mut self,
        writer: &mut partiql_eval::source::RegisterWriter<'_, '_>,
    ) -> Result<bool> {
        // Type tags (must match RegisterWriter/BufferWriter)
        const TYPE_NULL: u8 = 0;
        const TYPE_MISSING: u8 = 1;
        const TYPE_BOOL: u8 = 2;
        const TYPE_I64: u8 = 3;
        const TYPE_F64: u8 = 4;
        const TYPE_STRING: u8 = 5;

        // Check if we have at least 3 bytes for a field (slot + type)
        if self.offset + 3 > self.buffer.len() {
            // No more complete fields
            return Ok(false);
        }

        // Track if we've written any field in this row
        let mut has_fields = false;

        // Read fields until we run out of buffer or detect row boundary
        // For now, we'll read all remaining fields as a single row
        // TODO: Add proper row boundaries in buffer format
        while self.offset + 3 <= self.buffer.len() {
            // Read slot from buffer (2 bytes) - this is the source column index
            let slot_bytes = [self.buffer[self.offset], self.buffer[self.offset + 1]];
            let buffer_slot = u16::from_ne_bytes(slot_bytes);
            self.offset += 2;

            // Map buffer slot to target slot using layout projections
            // For FieldPath: buffer_slot corresponds to index in projections list
            // For ColumnIndex: buffer_slot corresponds to the column index
            let target_slot = if !self.layout.projections.is_empty() {
                // Try to find matching projection
                self.layout
                    .projections
                    .iter()
                    .find_map(|proj| {
                        match &proj.source {
                            ScanSource::ColumnIndex(col_idx) => {
                                if *col_idx == buffer_slot as usize {
                                    return Some(proj.target_slot);
                                }
                            }
                            ScanSource::FieldPath(_) => {
                                // For FieldPath: buffer slots are sequential (0, 1, 2, ...)
                                // Map buffer_slot to the corresponding projection by index
                                if (buffer_slot as usize) < self.layout.projections.len() {
                                    return self
                                        .layout
                                        .projections
                                        .get(buffer_slot as usize)
                                        .map(|p| p.target_slot);
                                }
                            }
                            _ => {}
                        }
                        None
                    })
                    .unwrap_or(buffer_slot) // Fallback to buffer slot if no mapping found
            } else {
                buffer_slot // No projections, use buffer slot directly
            };

            // Read type tag (1 byte)
            let type_tag = self.buffer[self.offset];
            self.offset += 1;

            // Decode and write based on type, using target_slot
            match type_tag {
                TYPE_NULL => {
                    writer.put_null(target_slot)?;
                    has_fields = true;
                }
                TYPE_MISSING => {
                    writer.put_missing(target_slot)?;
                    has_fields = true;
                }
                TYPE_BOOL => {
                    if self.offset >= self.buffer.len() {
                        return Err(EngineError::IllegalState(
                            "Buffer underflow reading bool".to_string(),
                        ));
                    }
                    let value = self.buffer[self.offset] != 0;
                    self.offset += 1;
                    writer.put_bool(target_slot, value)?;
                    has_fields = true;
                }
                TYPE_I64 => {
                    if self.offset + 8 > self.buffer.len() {
                        return Err(EngineError::IllegalState(
                            "Buffer underflow reading i64".to_string(),
                        ));
                    }
                    let bytes: [u8; 8] = self.buffer[self.offset..self.offset + 8]
                        .try_into()
                        .map_err(|_| EngineError::IllegalState("Failed to read i64".to_string()))?;
                    let value = i64::from_ne_bytes(bytes);
                    self.offset += 8;
                    writer.put_i64(target_slot, value)?;
                    has_fields = true;
                }
                TYPE_F64 => {
                    if self.offset + 8 > self.buffer.len() {
                        return Err(EngineError::IllegalState(
                            "Buffer underflow reading f64".to_string(),
                        ));
                    }
                    let bytes: [u8; 8] = self.buffer[self.offset..self.offset + 8]
                        .try_into()
                        .map_err(|_| EngineError::IllegalState("Failed to read f64".to_string()))?;
                    let value = f64::from_ne_bytes(bytes);
                    self.offset += 8;
                    writer.put_f64(target_slot, value)?;
                    has_fields = true;
                }
                TYPE_STRING => {
                    // Read length prefix (4 bytes)
                    if self.offset + 4 > self.buffer.len() {
                        return Err(EngineError::IllegalState(
                            "Buffer underflow reading string length".to_string(),
                        ));
                    }
                    let len_bytes: [u8; 4] = self.buffer[self.offset..self.offset + 4]
                        .try_into()
                        .map_err(|_| {
                        EngineError::IllegalState("Failed to read string length".to_string())
                    })?;
                    let len = i32::from_ne_bytes(len_bytes) as usize;
                    self.offset += 4;

                    // Read string data
                    if self.offset + len > self.buffer.len() {
                        return Err(EngineError::IllegalState(format!(
                            "Buffer underflow reading string data: need {} bytes, only {} remaining",
                            len,
                            self.buffer.len() - self.offset
                        )));
                    }
                    let str_bytes = &self.buffer[self.offset..self.offset + len];
                    let s = std::str::from_utf8(str_bytes).map_err(|e| {
                        EngineError::IllegalState(format!("Invalid UTF-8 in string: {}", e))
                    })?;

                    // TODO: Fix memory leak - strings need to be allocated in arena
                    // For now, leak the string (same as RegisterWriter)
                    let leaked_str: &'static str = Box::leak(s.to_string().into_boxed_str());
                    writer.put_str(target_slot, leaked_str)?;
                    self.offset += len;
                    has_fields = true;
                }
                _ => {
                    return Err(EngineError::IllegalState(format!(
                        "Unknown type tag: {}",
                        type_tag
                    )));
                }
            }

            // For now, treat each complete set of fields as one row
            // Break after reading fields to return one row at a time
            // TODO: Add explicit row boundaries in buffer format
            if has_fields {
                break;
            }
        }

        Ok(has_fields)
    }
}

/// Rust DataSource that delegates to Java via JNI
///
/// Wraps a Java DataSource object and implements the Rust DataSource trait.
/// Handles JNI callbacks for open(), next_row(), and close().
struct JavaDataSource {
    data_source_ref: GlobalRef,
    vm: Arc<JavaVM>,
    writer_obj: Option<GlobalRef>, // Cached RegisterWriter object for reuse
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
        if let Some(writer_obj) = &self.writer_obj {
            // Subsequent calls: update the handle in existing object
            env.set_field(
                writer_obj.as_obj(),
                "nativeHandle",
                "J",
                jni::objects::JValue::Long(writer_handle),
            )
            .map_err(|e| {
                EngineError::IllegalState(format!("Failed to update nativeHandle: {}", e))
            })?;
        } else {
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
        }

        let writer_obj = self
            .writer_obj
            .as_ref()
            .expect("writer_obj must be Some after initialization");

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
        let has_next = result.z().map_err(|e| {
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

        // Check if it's a ColumnIndex
        let column_index_class = env
            .find_class("org/partiql/jni/ScanSource$ColumnIndex")
            .ok()?;
        if env.is_instance_of(&source_obj, column_index_class).ok()? {
            let column_index = env
                .call_method(&source_obj, "getIndex", "()I", &[])
                .ok()?
                .i()
                .ok()?;
            return Some(ScanSource::ColumnIndex(column_index as usize));
        }

        // Check if it's a FieldPath
        let field_path_class = env
            .find_class("org/partiql/jni/ScanSource$FieldPath")
            .ok()?;
        if env.is_instance_of(&source_obj, field_path_class).ok()? {
            let path_jstring = env
                .call_method(&source_obj, "getPath", "()Ljava/lang/String;", &[])
                .ok()?
                .l()
                .ok()?;
            let path_str: String = env.get_string(&path_jstring.into()).ok()?.into();
            // Leak the string to create a 'static reference (same pattern used elsewhere)
            let leaked_path: &'static str = Box::leak(path_str.into_boxed_str());
            return Some(ScanSource::FieldPath(leaked_path.into()));
        }

        // Unknown ScanSource type
        None
    }
}
