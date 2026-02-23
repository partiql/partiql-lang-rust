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
use jni::JavaVM;
use once_cell::sync::Lazy;
use std::sync::Arc;

use partiql_common::catalog::{CatalogId, EntryId};
use partiql_eval::source::{
    BufferStability, CatalogScans, DataSource, DataSourceHandle, DataSourceMetadata, PhysicalType,
    ScanId, ScanLayout, ScanSource, ScanSourceType,
};
use partiql_eval::{CompilationCatalog, CompiledPlan, EngineError, ExecutionCatalog, Result};
use partiql_value::BindingsName;
use std::collections::HashMap;

use std::sync::RwLock;

/// Global storage for JavaVM pointer
///
/// Initialized once when library loads, used for thread attachment
static JAVA_VM: Lazy<RwLock<Option<Arc<JavaVM>>>> = Lazy::new(|| RwLock::new(None));

/// Initialize the JavaVM pointer for catalog callbacks
///
/// Must be called once during library initialization
pub fn init_java_vm(vm: JavaVM) {
    if let Ok(mut guard) = JAVA_VM.write() {
        *guard = Some(Arc::new(vm));
    }
}

/// Global storage for Java CompilationCatalog references
///
/// Maps catalog_id to (GlobalRef, JavaVM)
static COMPILATION_CATALOGS: Lazy<DashMap<CatalogId, (GlobalRef, Arc<JavaVM>)>> =
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

/// Remove a CompilationCatalog registration
pub fn unregister_compilation_catalog(catalog_id: CatalogId) {
    COMPILATION_CATALOGS.remove(&catalog_id);
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

/// Rust DataSource that reads directly from a pre-populated buffer
///
/// This DataSource provides optimal performance by reading data directly
/// from memory without any JNI callbacks. All data is in the buffer upfront.
///
/// # Wire Format
///
/// The buffer contains a sequence of rows, each terminated by `MARKER_ROW_END`.
/// Within each row, slot entries are: `[slot: u16][Value]`.
///
/// Values can be:
/// - Scalars: `[type_tag: u8][data]`
/// - Containers: `[TYPE_TUPLE|TYPE_LIST|TYPE_BAG] content* MARKER_CONTAINER_END`
///
/// Tuple fields within a container: `MARKER_FIELD_NAME [len: u32] [utf8_bytes] [Value]`
struct BufferDataSource {
    buffer: Vec<u8>,
    offset: usize,
    layout: ScanLayout,
}

// Type tags (must match Java BufferWriter/BufferValueWriter)
const TYPE_NULL: u8 = 0;
const TYPE_MISSING: u8 = 1;
const TYPE_BOOL: u8 = 2;
const TYPE_I64: u8 = 3;
const TYPE_F64: u8 = 4;
const TYPE_STRING: u8 = 5;
const TYPE_TUPLE: u8 = 6;
const TYPE_LIST: u8 = 7;
const TYPE_BAG: u8 = 8;

// Structural markers
const MARKER_FIELD_NAME: u8 = 0x10;
const MARKER_CONTAINER_END: u8 = 0x11;
const MARKER_ROW_END: u8 = 0x12;

impl BufferDataSource {
    fn new(buffer: Vec<u8>, layout: ScanLayout) -> Self {
        BufferDataSource {
            buffer,
            offset: 0,
            layout,
        }
    }

    /// Read a single byte, advancing offset
    #[inline]
    fn read_u8(&mut self) -> Result<u8> {
        if self.offset >= self.buffer.len() {
            return Err(EngineError::IllegalState(
                "Buffer underflow reading u8".to_string(),
            ));
        }
        let v = self.buffer[self.offset];
        self.offset += 1;
        Ok(v)
    }

    /// Peek at the next byte without advancing offset
    #[inline]
    fn peek_u8(&self) -> Option<u8> {
        if self.offset < self.buffer.len() {
            Some(self.buffer[self.offset])
        } else {
            None
        }
    }

    /// Read a u16 from native-endian bytes
    #[inline]
    fn read_u16(&mut self) -> Result<u16> {
        if self.offset + 2 > self.buffer.len() {
            return Err(EngineError::IllegalState(
                "Buffer underflow reading u16".to_string(),
            ));
        }
        let bytes = [self.buffer[self.offset], self.buffer[self.offset + 1]];
        self.offset += 2;
        Ok(u16::from_ne_bytes(bytes))
    }

    /// Read an i32 from native-endian bytes
    #[inline]
    fn read_i32(&mut self) -> Result<i32> {
        if self.offset + 4 > self.buffer.len() {
            return Err(EngineError::IllegalState(
                "Buffer underflow reading i32".to_string(),
            ));
        }
        let bytes: [u8; 4] = self.buffer[self.offset..self.offset + 4]
            .try_into()
            .map_err(|_| EngineError::IllegalState("Failed to read i32".to_string()))?;
        self.offset += 4;
        Ok(i32::from_ne_bytes(bytes))
    }

    /// Read an i64 from native-endian bytes
    #[inline]
    fn read_i64(&mut self) -> Result<i64> {
        if self.offset + 8 > self.buffer.len() {
            return Err(EngineError::IllegalState(
                "Buffer underflow reading i64".to_string(),
            ));
        }
        let bytes: [u8; 8] = self.buffer[self.offset..self.offset + 8]
            .try_into()
            .map_err(|_| EngineError::IllegalState("Failed to read i64".to_string()))?;
        self.offset += 8;
        Ok(i64::from_ne_bytes(bytes))
    }

    /// Read an f64 from native-endian bytes
    #[inline]
    fn read_f64(&mut self) -> Result<f64> {
        if self.offset + 8 > self.buffer.len() {
            return Err(EngineError::IllegalState(
                "Buffer underflow reading f64".to_string(),
            ));
        }
        let bytes: [u8; 8] = self.buffer[self.offset..self.offset + 8]
            .try_into()
            .map_err(|_| EngineError::IllegalState("Failed to read f64".to_string()))?;
        self.offset += 8;
        Ok(f64::from_ne_bytes(bytes))
    }

    /// Read a length-prefixed UTF-8 string (len: i32, data: [u8; len])
    #[inline]
    fn read_string(&mut self) -> Result<&str> {
        let len = self.read_i32()? as usize;
        if self.offset + len > self.buffer.len() {
            return Err(EngineError::IllegalState(format!(
                "Buffer underflow reading string data: need {} bytes, only {} remaining",
                len,
                self.buffer.len() - self.offset
            )));
        }
        let str_bytes = &self.buffer[self.offset..self.offset + len];
        let s = std::str::from_utf8(str_bytes)
            .map_err(|e| EngineError::IllegalState(format!("Invalid UTF-8 in string: {}", e)))?;
        self.offset += len;
        Ok(s)
    }

    /// Map a buffer slot index to the target register slot using layout projections
    fn map_slot(&self, buffer_slot: u16) -> u16 {
        if !self.layout.projections.is_empty() {
            self.layout
                .projections
                .iter()
                .find_map(|proj| {
                    match &proj.source.source_type {
                        ScanSourceType::ColumnIndex(col_idx) => {
                            if *col_idx == buffer_slot as usize {
                                return Some(proj.target_slot);
                            }
                        }
                        ScanSourceType::FieldPath(_) => {
                            if (buffer_slot as usize) < self.layout.projections.len() {
                                return self
                                    .layout
                                    .projections
                                    .get(buffer_slot as usize)
                                    .map(|p| p.target_slot);
                            }
                        }
                        ScanSourceType::WholeValue => {}
                    }
                    None
                })
                .unwrap_or(buffer_slot)
        } else {
            buffer_slot
        }
    }

    /// Decode one row from the buffer and write to registers.
    ///
    /// Reads slot entries until MARKER_ROW_END or end-of-buffer.
    fn decode_and_write_row(
        &mut self,
        writer: &mut partiql_eval::source::RegisterWriter<'_, '_>,
    ) -> Result<bool> {
        let mut has_fields = false;

        while let Some(next) = self.peek_u8() {
            // Check for row-end marker
            if next == MARKER_ROW_END {
                self.offset += 1; // consume marker
                break;
            }

            // Read slot header: [slot: u16]
            let buffer_slot = self.read_u16()?;
            let target_slot = self.map_slot(buffer_slot);

            // Peek at type tag
            let type_tag = self.peek_u8().ok_or_else(|| {
                EngineError::IllegalState(
                    "Buffer underflow: expected type tag after slot".to_string(),
                )
            })?;

            // Dispatch: scalar vs container
            match type_tag {
                TYPE_TUPLE | TYPE_LIST | TYPE_BAG => {
                    // Complex value — use ValueWriter
                    let mut vw = writer.value_writer(target_slot)?;
                    self.decode_value_into_writer(&mut vw)?;
                    vw.finish()?;
                }
                _ => {
                    // Scalar value — decode and write directly
                    self.decode_scalar(writer, target_slot)?;
                }
            }

            has_fields = true;
        }

        Ok(has_fields)
    }

    /// Decode a scalar value from the buffer and write to the register writer.
    fn decode_scalar(
        &mut self,
        writer: &mut partiql_eval::source::RegisterWriter<'_, '_>,
        target_slot: u16,
    ) -> Result<()> {
        let type_tag = self.read_u8()?;
        match type_tag {
            TYPE_NULL => {
                writer.write_null(target_slot)?;
            }
            TYPE_MISSING => {
                writer.write_missing(target_slot)?;
            }
            TYPE_BOOL => {
                let value = self.read_u8()? != 0;
                writer.write_bool(target_slot, value)?;
            }
            TYPE_I64 => {
                let value = self.read_i64()?;
                writer.write_i64(target_slot, value)?;
            }
            TYPE_F64 => {
                let value = self.read_f64()?;
                writer.write_f64(target_slot, value)?;
            }
            TYPE_STRING => {
                let s = self.read_string()?;
                // TODO: Fix memory leak - strings need to be allocated in arena
                // For now, leak the string (same as RegisterWriter)
                let leaked_str: &'static str = Box::leak(s.to_string().into_boxed_str());
                writer.write_str(target_slot, leaked_str)?;
            }
            _ => {
                return Err(EngineError::IllegalState(format!(
                    "Unknown scalar type tag: {} at offset {}",
                    type_tag,
                    self.offset - 1
                )));
            }
        }
        Ok(())
    }

    /// Recursively decode a value (scalar or container) into a ValueWriter.
    ///
    /// This is used for container elements and nested structures.
    fn decode_value_into_writer(
        &mut self,
        vw: &mut partiql_eval::source::ValueWriter<'_, '_>,
    ) -> Result<()> {
        let type_tag = self.read_u8()?;
        match type_tag {
            TYPE_NULL => {
                vw.put_null()?;
            }
            TYPE_MISSING => {
                vw.put_missing()?;
            }
            TYPE_BOOL => {
                let value = self.read_u8()? != 0;
                vw.put_bool(value)?;
            }
            TYPE_I64 => {
                let value = self.read_i64()?;
                vw.put_i64(value)?;
            }
            TYPE_F64 => {
                let value = self.read_f64()?;
                vw.put_f64(value)?;
            }
            TYPE_STRING => {
                let s = self.read_string()?;
                let leaked_str: &'static str = Box::leak(s.to_string().into_boxed_str());
                vw.put_str(leaked_str)?;
            }
            TYPE_TUPLE => {
                vw.step_in_tuple()?;
                self.decode_tuple_contents(vw)?;
                vw.step_out()?;
            }
            TYPE_LIST => {
                vw.step_in_list()?;
                self.decode_sequence_contents(vw)?;
                vw.step_out()?;
            }
            TYPE_BAG => {
                vw.step_in_bag()?;
                self.decode_sequence_contents(vw)?;
                vw.step_out()?;
            }
            _ => {
                return Err(EngineError::IllegalState(format!(
                    "Unknown type tag in value: {} at offset {}",
                    type_tag,
                    self.offset - 1
                )));
            }
        }
        Ok(())
    }

    /// Decode tuple contents: (MARKER_FIELD_NAME len bytes Value)* MARKER_CONTAINER_END
    fn decode_tuple_contents(
        &mut self,
        vw: &mut partiql_eval::source::ValueWriter<'_, '_>,
    ) -> Result<()> {
        loop {
            let next = match self.peek_u8() {
                Some(b) => b,
                None => {
                    return Err(EngineError::IllegalState(
                        "Buffer underflow: expected field name or container end in tuple"
                            .to_string(),
                    ));
                }
            };

            if next == MARKER_CONTAINER_END {
                self.offset += 1; // consume marker
                return Ok(());
            }

            if next != MARKER_FIELD_NAME {
                return Err(EngineError::IllegalState(format!(
                    "Expected MARKER_FIELD_NAME (0x10) or MARKER_CONTAINER_END (0x11) in tuple, got 0x{:02x} at offset {}",
                    next, self.offset
                )));
            }

            // Consume MARKER_FIELD_NAME
            self.offset += 1;

            // Read field name
            let field_name = self.read_string()?;
            let leaked_name: &'static str = Box::leak(field_name.to_string().into_boxed_str());
            vw.put_field_name(leaked_name)?;

            // Read the field value (recursively)
            self.decode_value_into_writer(vw)?;
        }
    }

    /// Decode list/bag contents: Value* MARKER_CONTAINER_END
    fn decode_sequence_contents(
        &mut self,
        vw: &mut partiql_eval::source::ValueWriter<'_, '_>,
    ) -> Result<()> {
        loop {
            let next = match self.peek_u8() {
                Some(b) => b,
                None => {
                    return Err(EngineError::IllegalState(
                        "Buffer underflow: expected value or container end in list/bag".to_string(),
                    ));
                }
            };

            if next == MARKER_CONTAINER_END {
                self.offset += 1; // consume marker
                return Ok(());
            }

            // Read the next element value (recursively)
            self.decode_value_into_writer(vw)?;
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

        // Decode one row from buffer and write to registers
        self.decode_and_write_row(writer)
    }

    fn close(&mut self) -> Result<()> {
        // Nothing to clean up
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

impl DataSourceMetadata for JavaDataSourceConfig {
    fn buffer_stability(&self) -> BufferStability {
        // Attach to JVM
        let mut env = match self.vm.attach_current_thread() {
            Ok(env) => env,
            Err(_) => {
                // If JVM attachment fails, return conservative default
                return BufferStability::UntilNext;
            }
        };

        // Call Java method: BufferStability getBufferStability()
        let stability_obj = match env.call_method(
            self.config_ref.as_obj(),
            "getBufferStability",
            "()Lorg/partiql/jni/BufferStability;",
            &[],
        ) {
            Ok(result) => match result.l() {
                Ok(obj) => obj,
                Err(_) => return BufferStability::UntilNext,
            },
            Err(_) => return BufferStability::UntilNext,
        };

        // Check which stability value it is
        let until_close_class = match env.find_class("org/partiql/jni/BufferStability$UntilClose") {
            Ok(c) => c,
            Err(_) => return BufferStability::UntilNext,
        };

        if env
            .is_instance_of(&stability_obj, until_close_class)
            .unwrap_or(false)
        {
            BufferStability::UntilClose
        } else {
            BufferStability::UntilNext
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
            // Default to Dynamic type for Java sources (type is determined at runtime)
            return Some(ScanSource::column(
                column_index as usize,
                PhysicalType::Dynamic,
            ));
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
            // Default to Dynamic type for Java sources (type is determined at runtime)
            return Some(ScanSource::field(path_str, PhysicalType::Dynamic));
        }

        // Check if it's a WholeValue
        let whole_value_class = env
            .find_class("org/partiql/jni/ScanSource$WholeValue")
            .ok()?;
        if env.is_instance_of(&source_obj, whole_value_class).ok()? {
            return Some(ScanSource::whole_value());
        }

        // Unknown ScanSource type
        None
    }
}
