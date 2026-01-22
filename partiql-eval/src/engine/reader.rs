use crate::engine::error::{EngineError, Result};
use crate::engine::row::{RowFrame, SlotId, SlotValue};
use crate::engine::value::{value_ref_from_value_in_arena, ValueRef};
use ion_rs_old::data_source::ToIonDataSource;
use ion_rs_old::{IonReader, IonType, ReaderBuilder};
use partiql_value::{BindingsName, Value};
use rustc_hash::FxHashMap;
use std::fs::File;
use std::io::BufReader;

#[derive(Clone, Copy, Debug)]
pub enum BufferStability {
    UntilNext,
    UntilClose,
}

#[derive(Clone, Copy, Debug)]
pub struct ReaderCaps {
    pub stability: BufferStability,
    pub can_project: bool,
    pub can_return_opaque: bool,
}

#[derive(Clone, Debug, Default)]
pub struct ScanLayout {
    pub projections: Vec<ScanProjection>,
}

impl ScanLayout {
    pub fn base_row() -> Self {
        ScanLayout {
            projections: vec![ScanProjection {
                source: ScanSource::BaseRow,
                target_slot: 0,
                type_hint: TypeHint::Any,
            }],
        }
    }

    pub fn is_base_row_only(&self) -> bool {
        self.projections.len() == 1
            && matches!(self.projections[0].source, ScanSource::BaseRow)
            && self.projections[0].target_slot == 0
    }
}

#[derive(Clone, Debug)]
pub struct ScanProjection {
    pub source: ScanSource,
    pub target_slot: SlotId,
    pub type_hint: TypeHint,
}

#[derive(Clone, Debug)]
pub enum ScanSource {
    ColumnIndex(usize),
    FieldPath(String),
    BaseRow,
}

#[derive(Clone, Copy, Debug)]
pub enum TypeHint {
    Any,
}

pub trait RowReader {
    fn caps(&self) -> ReaderCaps;
    fn set_projection(&mut self, layout: ScanLayout) -> Result<()>;
    fn open(&mut self) -> Result<()>;
    fn next_row(&mut self, out: &mut RowFrame<'_>) -> Result<bool>;
    fn resolve(&self, field_name: &str) -> Option<ScanSource>;
    fn close(&mut self) -> Result<()>;
}

pub trait RowReaderFactory {
    fn create(&self) -> Result<Box<dyn RowReader>>;
}

/// In-memory row reader that generates rows on-the-fly
/// 
/// Similar to InMemoryGeneratedReader in vectorized evaluation, but operates on single rows.
/// Generates fake columnar data with two Int64 columns:
/// - Column "a": starts at 0, increments by 1
/// - Column "b": starts at 100, increments by 1
pub struct InMemGeneratedReader {
    current_row: i64,
    total_rows: usize,
    layout: ScanLayout,
    caps: ReaderCaps,
}

#[derive(Clone)]
pub struct InMemGeneratedReaderFactory {
    total_rows: usize,
}

impl InMemGeneratedReaderFactory {
    pub fn new(total_rows: usize) -> Self {
        InMemGeneratedReaderFactory { total_rows }
    }
}

impl RowReaderFactory for InMemGeneratedReaderFactory {
    fn create(&self) -> Result<Box<dyn RowReader>> {
        Ok(Box::new(InMemGeneratedReader::new(self.total_rows)))
    }
}

/// High-performance streaming Ion text reader with projection pushdown
///
/// This reader uses the ion_rs streaming API to read Ion data directly into
/// row slots, avoiding materialization to Value objects. Similar to the
/// vectorized PIonTextReader but operates on single rows.
///
/// # Performance Characteristics
/// - Zero-copy for primitives (i64, f64, bool)
/// - Minimal string allocations (only for projected string fields)
/// - True projection pushdown (only reads requested fields)
/// - Uses FxHashMap for O(1) field lookups
///
/// # Limitations
/// - Single-level field paths only (no nested navigation like "a.b.c")
/// - Dynamic type dispatch (may optimize to i64-only in future)
pub struct IonRowReader {
    path: String,
    reader: Option<Box<ion_rs_old::Reader<'static>>>,
    layout: ScanLayout,
    caps: ReaderCaps,
    /// Maps field names to target slot IDs for O(1) lookup during reading
    field_to_slot: FxHashMap<String, SlotId>,
    /// Storage for string values to satisfy UntilNext lifetime guarantee
    /// Cleared on each next_row, populated during reading
    string_storage: Vec<String>,
}

impl IonRowReader {
    pub fn new(path: String) -> Self {
        IonRowReader {
            path,
            reader: None,
            layout: ScanLayout::base_row(),
            caps: ReaderCaps {
                stability: BufferStability::UntilNext,
                can_project: true,
                can_return_opaque: false,
            },
            field_to_slot: FxHashMap::default(),
            string_storage: Vec::new(),
        }
    }
}

impl RowReader for IonRowReader {
    fn caps(&self) -> ReaderCaps {
        self.caps
    }

    fn set_projection(&mut self, layout: ScanLayout) -> Result<()> {
        if !self.caps.can_project && !layout.is_base_row_only() {
            return Err(EngineError::ProjectionNotSupported(
                "reader does not support projection",
            ));
        }

        // Build field name to slot mapping for O(1) lookup during reading
        self.field_to_slot.clear();
        for proj in &layout.projections {
            match &proj.source {
                ScanSource::FieldPath(field_name) => {
                    // NOTE: Only single-level field paths supported (no "a.b.c")
                    self.field_to_slot.insert(field_name.clone(), proj.target_slot);
                }
                ScanSource::BaseRow => {
                    // BaseRow projection not supported with projection pushdown
                    // Would require materializing entire row to Value
                }
                ScanSource::ColumnIndex(_) => {
                    // ColumnIndex not applicable to Ion structs
                }
            }
        }

        self.layout = layout;
        Ok(())
    }

    fn open(&mut self) -> Result<()> {
        let file = File::open(&self.path)
            .map_err(|e| EngineError::ReaderError(format!("ion open failed: {e}")))?;
        let buf_reader = BufReader::new(file);
        
        let ion_reader = ReaderBuilder::new()
            .build(buf_reader)
            .map_err(|e| EngineError::ReaderError(format!("ion reader creation failed: {e}")))?;
        
        // Box the reader to work around lifetime constraints
        let boxed_reader: Box<ion_rs_old::Reader<'static>> = unsafe {
            std::mem::transmute(Box::new(ion_reader))
        };
        
        self.reader = Some(boxed_reader);
        Ok(())
    }

    fn next_row(&mut self, out: &mut RowFrame<'_>) -> Result<bool> {
        let reader = match self.reader.as_mut() {
            Some(r) => r,
            None => return Ok(false),
        };

        // Clear string storage from previous row (UntilNext stability)
        self.string_storage.clear();

        // Read next Ion value (should be a struct)
        let stream_item = reader
            .next()
            .map_err(|e| EngineError::ReaderError(format!("ion read failed: {e}")))?;

        match stream_item {
            ion_rs_old::StreamItem::Value(_ion_type) => {
                // Step into the struct
                reader
                    .step_in()
                    .map_err(|e| EngineError::ReaderError(format!("failed to step into struct: {e}")))?;

                // Read struct fields and populate requested slots
                loop {
                    match reader
                        .next()
                        .map_err(|e| EngineError::ReaderError(format!("error reading struct field: {e}")))?
                    {
                        ion_rs_old::StreamItem::Value(ion_type) => {
                            // Get field name
                            let field_name = reader
                                .field_name()
                                .map_err(|e| {
                                    EngineError::ReaderError(format!("failed to get field name: {e}"))
                                })?;
                            
                            let field_text = field_name.text().ok_or_else(|| {
                                EngineError::ReaderError("field name has no text".to_string())
                            })?;

                            // Check if this field is projected
                            if let Some(&target_slot) = self.field_to_slot.get(field_text) {
                                let target_idx = target_slot as usize;
                                if target_idx < out.slots.len() {
                                    // NOTE: Dynamic type dispatch - may optimize to i64-only in future
                                    let value_ref = match ion_type {
                                        IonType::Int => {
                                            let val = reader.read_i64().map_err(|e| {
                                                EngineError::ReaderError(format!("failed to read i64: {e}"))
                                            })?;
                                            ValueRef::I64(val)
                                        }
                                        IonType::Float => {
                                            let val = reader.read_f64().map_err(|e| {
                                                EngineError::ReaderError(format!("failed to read f64: {e}"))
                                            })?;
                                            ValueRef::F64(val)
                                        }
                                        IonType::Bool => {
                                            let val = reader.read_bool().map_err(|e| {
                                                EngineError::ReaderError(format!("failed to read bool: {e}"))
                                            })?;
                                            ValueRef::Bool(val)
                                        }
                                        IonType::String => {
                                            let val = reader.read_str().map_err(|e| {
                                                EngineError::ReaderError(format!("failed to read string: {e}"))
                                            })?;
                                            // Store owned string, return borrowed reference
                                            self.string_storage.push(val.to_string());
                                            let idx = self.string_storage.len() - 1;
                                            ValueRef::Str(self.string_storage[idx].as_str())
                                        }
                                        IonType::Null => ValueRef::Null,
                                        other_type => {
                                            // For other types, skip for now
                                            // Could materialize to Value via arena if needed
                                            return Err(EngineError::ReaderError(format!(
                                                "unsupported ion type for projection: {:?}",
                                                other_type
                                            )));
                                        }
                                    };
                                    
                                    out.slots[target_idx] = SlotValue::Val(extend_value_ref(value_ref));
                                }
                            }
                            // If field not projected, it's automatically skipped (projection pushdown!)
                        }
                        ion_rs_old::StreamItem::Nothing => break,
                        ion_rs_old::StreamItem::Null(_) => continue,
                    }
                }

                // Step out of the struct
                reader
                    .step_out()
                    .map_err(|e| EngineError::ReaderError(format!("failed to step out of struct: {e}")))?;

                Ok(true)
            }
            ion_rs_old::StreamItem::Nothing => Ok(false),
            ion_rs_old::StreamItem::Null(_) => {
                // Skip null at top level, try next value
                self.next_row(out)
            }
        }
    }

    fn resolve(&self, field_name: &str) -> Option<ScanSource> {
        Some(ScanSource::FieldPath(field_name.to_string()))
    }

    fn close(&mut self) -> Result<()> {
        self.reader = None;
        self.string_storage.clear();
        self.field_to_slot.clear();
        Ok(())
    }
}

pub struct IonRowReaderFactory {
    path: String,
}

impl IonRowReaderFactory {
    pub fn new(path: String) -> Self {
        IonRowReaderFactory { path }
    }
}

impl RowReaderFactory for IonRowReaderFactory {
    fn create(&self) -> Result<Box<dyn RowReader>> {
        Ok(Box::new(IonRowReader::new(self.path.clone())))
    }
}

fn extend_value_ref<'a>(value: ValueRef<'_>) -> ValueRef<'a> {
    // Safety: IonRowReader guarantees the borrowed data outlives the row (UntilNext).
    // String references point to string_storage which is cleared only on next next_row call.
    unsafe { std::mem::transmute(value) }
}

impl InMemGeneratedReader {
    pub fn new(total_rows: usize) -> Self {
        InMemGeneratedReader {
            current_row: 0,
            total_rows,
            layout: ScanLayout::base_row(),
            caps: ReaderCaps {
                stability: BufferStability::UntilNext,
                can_project: true,
                can_return_opaque: false,
            },
        }
    }
}

impl RowReader for InMemGeneratedReader {
    fn caps(&self) -> ReaderCaps {
        self.caps
    }

    fn set_projection(&mut self, layout: ScanLayout) -> Result<()> {
        if !self.caps.can_project && !layout.is_base_row_only() {
            return Err(EngineError::ProjectionNotSupported(
                "reader does not support projection",
            ));
        }
        self.layout = layout;
        Ok(())
    }

    fn open(&mut self) -> Result<()> {
        self.current_row = 0;
        Ok(())
    }

    fn next_row(&mut self, out: &mut RowFrame<'_>) -> Result<bool> {
        // Check if we've generated all rows
        if self.current_row >= self.total_rows as i64 {
            return Ok(false);
        }

        // Generate row values on-the-fly
        let a_value = self.current_row;
        let b_value = self.current_row + 100;

        // Populate slots based on projection layout
        for proj in &self.layout.projections {
            let target = proj.target_slot as usize;
            if target >= out.slots.len() {
                continue;
            }
            
            let value_ref = match &proj.source {
                ScanSource::BaseRow => {
                    // For BaseRow, we need to materialize the entire row as a Value
                    use partiql_value::tuple;
                    let row_value = tuple![("a", a_value), ("b", b_value)].into();
                    value_ref_from_value_in_arena(&row_value, out.arena)
                }
                ScanSource::FieldPath(path) => {
                    // Generate value based on field name
                    match path.as_str() {
                        "a" => ValueRef::I64(a_value),
                        "b" => ValueRef::I64(b_value),
                        _ => ValueRef::Missing,
                    }
                }
                ScanSource::ColumnIndex(index) => {
                    // Column index 0 = "a", 1 = "b"
                    match index {
                        0 => ValueRef::I64(a_value),
                        1 => ValueRef::I64(b_value),
                        _ => ValueRef::Missing,
                    }
                }
            };
            
            out.slots[target] = SlotValue::Val(value_ref);
        }

        // Increment current row for next call
        self.current_row += 1;
        Ok(true)
    }

    fn resolve(&self, field_name: &str) -> Option<ScanSource> {
        // Only support fields "a" and "b"
        if field_name == "a" || field_name == "b" {
            Some(ScanSource::FieldPath(field_name.to_string()))
        } else {
            None
        }
    }

    fn close(&mut self) -> Result<()> {
        Ok(())
    }
}

fn get_field_path<'a>(row: &'a Value, path: &str) -> Option<&'a Value> {
    let mut current = row;
    for part in path.split('.') {
        let key = BindingsName::CaseInsensitive(part.into());
        match current {
            Value::Tuple(tuple) => {
                if let Some(value) = tuple.get(&key) {
                    current = value;
                } else {
                    return None;
                }
            }
            _ => return None,
        }
    }
    Some(current)
}

fn get_column_index<'a>(row: &'a Value, index: usize) -> Option<&'a Value> {
    match row {
        Value::Tuple(tuple) => tuple.values().nth(index),
        _ => None,
    }
}
