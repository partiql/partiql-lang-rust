use crate::engine::error::{EngineError, Result};
use crate::engine::row::SlotId;
use crate::engine::source::api::{
    BufferStability, DataSource, DataSourceFactory, ScanCapabilities, ScanLayout, ScanSource,
};
use ion_rs_old::{IonReader, IonType, ReaderBuilder};
use rustc_hash::FxHashMap;
use std::fs::File;
use std::io::BufReader;

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
pub struct IonDataSource {
    path: String,
    reader: Option<Box<ion_rs_old::Reader<'static>>>,
    /// Maps field names to target slot IDs for O(1) lookup during reading
    field_to_slot: FxHashMap<String, SlotId>,
    /// Storage for string values to satisfy UntilNext lifetime guarantee
    /// Cleared on each next_row, populated during reading
    string_storage: Vec<String>,
}

impl IonDataSource {
    pub fn new(path: String, layout: ScanLayout) -> Self {
        // Build field name to slot mapping for O(1) lookup during reading
        let mut field_to_slot = FxHashMap::default();
        for proj in &layout.projections {
            if let ScanSource::FieldPath(field_name) = &proj.source {
                field_to_slot.insert(field_name.clone(), proj.target_slot);
            }
        }

        IonDataSource {
            path,
            reader: None,
            field_to_slot,
            string_storage: Vec::new(),
        }
    }
}

impl DataSource for IonDataSource {
    fn open(&mut self) -> Result<()> {
        let file = File::open(&self.path)
            .map_err(|e| EngineError::ReaderError(format!("ion open failed: {e}")))?;
        let buf_reader = BufReader::new(file);

        let ion_reader = ReaderBuilder::new()
            .build(buf_reader)
            .map_err(|e| EngineError::ReaderError(format!("ion reader creation failed: {e}")))?;

        // Box the reader to work around lifetime constraints
        let boxed_reader: Box<ion_rs_old::Reader<'static>> =
            unsafe { std::mem::transmute(Box::new(ion_reader)) };

        self.reader = Some(boxed_reader);
        Ok(())
    }

    fn next_row(&mut self, writer: &mut super::RegisterWriter<'_, '_>) -> Result<bool> {
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
                reader.step_in().map_err(|e| {
                    EngineError::ReaderError(format!("failed to step into struct: {e}"))
                })?;

                // Read struct fields and populate requested slot registers
                loop {
                    match reader.next().map_err(|e| {
                        EngineError::ReaderError(format!("error reading struct field: {e}"))
                    })? {
                        ion_rs_old::StreamItem::Value(ion_type) => {
                            // Get field name
                            let field_name = reader.field_name().map_err(|e| {
                                EngineError::ReaderError(format!("failed to get field name: {e}"))
                            })?;

                            let field_text = field_name.text().ok_or_else(|| {
                                EngineError::ReaderError("field name has no text".to_string())
                            })?;

                            // Check if this field is projected
                            if let Some(&target_slot) = self.field_to_slot.get(field_text) {
                                // NOTE: Dynamic type dispatch - may optimize to i64-only in future
                                match ion_type {
                                    IonType::Int => {
                                        let val = reader.read_i64().map_err(|e| {
                                            EngineError::ReaderError(format!(
                                                "failed to read i64: {e}"
                                            ))
                                        })?;
                                        writer.put_i64(target_slot, val)?;
                                    }
                                    IonType::Float => {
                                        let val = reader.read_f64().map_err(|e| {
                                            EngineError::ReaderError(format!(
                                                "failed to read f64: {e}"
                                            ))
                                        })?;
                                        writer.put_f64(target_slot, val)?;
                                    }
                                    IonType::Bool => {
                                        let val = reader.read_bool().map_err(|e| {
                                            EngineError::ReaderError(format!(
                                                "failed to read bool: {e}"
                                            ))
                                        })?;
                                        writer.put_bool(target_slot, val)?;
                                    }
                                    IonType::String => {
                                        let val = reader.read_str().map_err(|e| {
                                            EngineError::ReaderError(format!(
                                                "failed to read string: {e}"
                                            ))
                                        })?;
                                        // Store owned string
                                        self.string_storage.push(val.to_string());
                                        let idx = self.string_storage.len() - 1;
                                        // Get reference to stored string (safety: string_storage won't be modified until next next_row call)
                                        let str_ref = unsafe {
                                            std::mem::transmute::<&str, &str>(
                                                self.string_storage[idx].as_str(),
                                            )
                                        };
                                        writer.put_str(target_slot, str_ref)?;
                                    }
                                    IonType::Null => {
                                        writer.put_null(target_slot)?;
                                    }
                                    other_type => {
                                        // For other types, skip for now
                                        // Could materialize to Value via arena if needed
                                        return Err(EngineError::ReaderError(format!(
                                            "unsupported ion type for projection: {:?}",
                                            other_type
                                        )));
                                    }
                                }
                            }
                            // If field not projected, it's automatically skipped (projection pushdown!)
                        }
                        ion_rs_old::StreamItem::Nothing => break,
                        ion_rs_old::StreamItem::Null(_) => continue,
                    }
                }

                // Step out of the struct
                reader.step_out().map_err(|e| {
                    EngineError::ReaderError(format!("failed to step out of struct: {e}"))
                })?;

                Ok(true)
            }
            ion_rs_old::StreamItem::Nothing => Ok(false),
            ion_rs_old::StreamItem::Null(_) => {
                // Skip null at top level, try next value
                self.next_row(writer)
            }
        }
    }

    fn close(&mut self) -> Result<()> {
        self.reader = None;
        self.string_storage.clear();
        self.field_to_slot.clear();
        Ok(())
    }
}

#[derive(Clone)]
pub struct IonDataSourceFactory {
    pub(crate) path: String,
}

impl IonDataSourceFactory {
    pub fn new(path: String) -> Self {
        IonDataSourceFactory { path }
    }
}

impl DataSourceFactory for IonDataSourceFactory {
    fn create(&self, layout: ScanLayout) -> Result<Box<dyn DataSource>> {
        // Validate that all projections are FieldPath (not ColumnIndex)
        for proj in &layout.projections {
            match &proj.source {
                ScanSource::FieldPath(_) => {
                    // Valid for Ion reader
                }
                ScanSource::ColumnIndex(_) => {
                    return Err(EngineError::ProjectionNotSupported(
                        "Ion reader only supports FieldPath projections, not ColumnIndex",
                    ));
                }
                ScanSource::BaseRow => {
                    return Err(EngineError::ProjectionNotSupported(
                        "Ion reader only supports FieldPath projections, not BaseRow",
                    ));
                }
            }
        }

        Ok(Box::new(IonDataSource::new(self.path.clone(), layout)))
    }

    fn caps(&self) -> ScanCapabilities {
        ScanCapabilities {
            stability: BufferStability::UntilNext,
            can_project: true,
            can_return_opaque: false,
        }
    }

    fn resolve(&self, field_name: &str) -> Option<ScanSource> {
        // Ion reader accepts any field name at compile time
        // Runtime validation happens during actual reading
        Some(ScanSource::FieldPath(field_name.to_string()))
    }
}
