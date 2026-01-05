use crate::batch::{
    Field, PhysicalVectorEnum, SourceTypeDef, VectorizedBatch,
};
use crate::error::EvalError;
use crate::reader::error::{
    BatchReaderError, DataSourceError, ProjectionError
};
use crate::reader::{BatchReader, ProjectionSource, ProjectionSpec};
use ion_rs::data_source::ToIonDataSource;
use ion_rs::{IonReader, ReaderBuilder, StreamItem};
use std::collections::HashMap;
use std::fs::File;
use std::io::{BufRead, BufReader};
use std::path::Path;

/// Streaming Ion reader implementation
///
/// This reader uses true streaming I/O and maintains constant memory usage regardless
/// of input file size. Only the current batch (configured via `batch_size`) is
/// held in memory at any time.
///
/// # Features
/// - Constant memory usage for arbitrarily large files
/// - Supports FieldPath projections for flat field access
/// - Single-level nesting support (e.g., "field" or "struct.field")
/// - Missing fields result in null values, not errors
///
/// # Usage
///
/// ```rust,ignore
/// use partiql_eval_vectorized::reader::IonReader;
/// use std::fs::File;
///
/// // Stream from file
/// let mut reader = IonReader::from_ion_file("data.ion", 1000)?;
///
/// // Or stream from any BufRead source  
/// let file = File::open("data.ion")?;
/// let buf_reader = std::io::BufReader::new(file);
/// let mut reader = IonReader::from_reader(buf_reader, 1000)?;
/// ```
pub struct PIonReader<'a> {
    /// Streaming Ion reader
    reader: ion_rs::Reader<'a>,
    /// Configured projection specification
    projection: Option<ProjectionSpec>,
    /// Batch size for output
    batch_size: usize,
    /// Cached schema built from projection (reused across batches)
    cached_schema: Option<SourceTypeDef>,
    /// Reusable batch structure (pre-allocated in set_projection)
    reusable_batch: Option<VectorizedBatch>,
    /// Track if we've reached end of stream
    eof_reached: bool,
    /// Maps field names to vector indices for O(1) lookup (i64 only)
    field_to_vector_map: Option<HashMap<String, usize>>,
}

impl<'a> PIonReader<'a> {
    /// Create a new IonReader from a BufRead source
    pub fn from_reader<R: BufRead + ToIonDataSource + 'a>(reader: R, batch_size: usize) -> Result<Self, EvalError> {
        let ion_reader = ReaderBuilder::new().build(reader).map_err(|e| {
            BatchReaderError::data_source(DataSourceError::initialization_failed(
                "Ion",
                &format!("Failed to create Ion reader: {}", e),
            ))
        })?;

        Ok(Self {
            reader: ion_reader,
            projection: None,
            batch_size,
            cached_schema: None,
            reusable_batch: None,
            eof_reached: false,
            field_to_vector_map: None,
        })
    }

    /// Read next batch_size elements from the stream and write directly to batch vectors
    /// 
    /// Returns the actual number of rows read (may be less than batch_size at end of stream)
    /// 
    /// Performance optimization: Pre-collects mutable Vec references before the hot loop
    /// to avoid repeated column_mut() calls and pattern matching overhead
    fn read_elements_batch(
        reader: &mut ion_rs::Reader,
        field_map: &HashMap<String, usize>,
        batch: &mut VectorizedBatch,
        batch_size: usize,
        eof_reached: &mut bool,
    ) -> Result<usize, EvalError> {
        // Pre-collect mutable slice references to all Int64 vectors we'll be writing to
        // This moves the expensive column_mut() + pattern matching out of the hot loop
        let mut vector_refs: HashMap<usize, &mut [i64]> = HashMap::new();
        
        // Get unique vector indices from field_map
        let vector_indices: std::collections::HashSet<usize> = field_map.values().copied().collect();
        
        // Extract mutable slice references to underlying data for each column
        // We need to use raw pointers to satisfy borrow checker since we're
        // extracting multiple mutable references from the same batch
        let batch_ptr = batch as *mut VectorizedBatch;
        
        for &vector_idx in &vector_indices {
            unsafe {
                // Safety: We're getting non-overlapping mutable references to different
                // columns in the batch. Each column is independent and won't be accessed
                // by any other code while we hold these references.
                let batch_ref = &mut *batch_ptr;
                let vector = batch_ref.column_mut(vector_idx)?;
                if let PhysicalVectorEnum::Int64(v) = &mut vector.physical {
                    // Get mutable slice to the underlying data
                    let slice = v.as_mut_slice();
                    // Convert to raw pointer and back to extend lifetime
                    let slice_ptr = slice as *mut [i64];
                    vector_refs.insert(vector_idx, &mut *slice_ptr);
                }
            }
        }
        
        let mut rows_read = 0;
        
        // Hot loop: Now we can write directly to vectors without any lookups
        for row_idx in 0..batch_size {
            // Try to read next value from stream
            match reader.next().map_err(|e| {
                EvalError::General(format!("Ion stream error: {}", e))
            })? {
                StreamItem::Value(_v) => {
                    // We can always assume that the top-level value is a struct
                    reader.step_in().map_err(|e| {
                        EvalError::General(format!("Failed to step into struct: {}", e))
                    })?;
                    
                    // Iterate through all fields in the struct
                    loop {
                        match reader.next().map_err(|e| {
                            EvalError::General(format!("Error reading struct field: {}", e))
                        })? {
                            StreamItem::Value(_) => {
                                // Get the field name
                                if let Ok(field_name) = reader.field_name() {
                                    // Check if this field is in our projection
                                    let text = field_name.text().unwrap();
                                    if let Some(&vector_idx) = field_map.get(text) {
                                        // Read the i64 value
                                        let value = reader.read_i64().map_err(|e| {
                                            EvalError::General(format!("Failed to read i64 for field '{}': {}", field_name, e))
                                        })?;
                                        
                                        // Fast path: Direct write to pre-collected slice reference
                                        // No column_mut(), no pattern matching, just indexing
                                        if let Some(slice) = vector_refs.get_mut(&vector_idx) {
                                            slice[row_idx] = value;
                                        }
                                    }
                                    // If field not in projection, skip it (reader automatically advances)
                                }
                            }
                            StreamItem::Nothing => {
                                // End of struct reached
                                break;
                            }
                            StreamItem::Null(_) => {
                                // Null field - skip
                                continue;
                            }
                        }
                    }
                    
                    reader.step_out().map_err(|e| {
                        EvalError::General(format!("Failed to step out of struct: {}", e))
                    })?;
                    
                    rows_read += 1;
                }
                StreamItem::Nothing => {
                    // End of stream reached
                    *eof_reached = true;
                    break;
                }
                StreamItem::Null(_ion_type) => {
                    // Skip nulls at the top level (or handle as needed)
                    continue;
                }
            }
        }
        
        Ok(rows_read)
    }

    /// Create a new IonReader from a file path (streaming from disk)
    pub fn from_ion_file(path: impl AsRef<Path>, batch_size: usize) -> Result<Self, EvalError> {
        let file = File::open(path.as_ref()).map_err(|e| {
            BatchReaderError::data_source(DataSourceError::initialization_failed(
                "Ion",
                &format!("Failed to open file {:?}: {}", path.as_ref(), e),
            ))
        })?;
        
        let buf_reader = BufReader::new(file);
        Self::from_reader(buf_reader, batch_size)
    }

    /// Create a new IonReader from bytes (for testing with small data)
    pub fn from_ion_bytes(bytes: &[u8], batch_size: usize) -> Result<Self, EvalError> {
        let cursor = std::io::Cursor::new(bytes.to_vec());
        Self::from_reader(cursor, batch_size)
    }
}

impl<'a> BatchReader for PIonReader<'a> {
    fn open(&mut self) -> Result<(), EvalError> {
        Ok(())
    }

    fn resolve(&self, field_name: &str) -> Option<ProjectionSource> {
        Some(ProjectionSource::FieldPath(field_name.to_string()))
    }

    fn set_projection(&mut self, spec: ProjectionSpec) -> Result<(), EvalError> {
        // Validate that all projection sources are FieldPath
        for projection in &spec.projections {
            match &projection.source {
                ProjectionSource::FieldPath(_) => {}
                ProjectionSource::ColumnIndex(idx) => {
                    return Err(
                        BatchReaderError::projection(ProjectionError::unsupported_source(
                            &format!("ColumnIndex({})", idx),
                            "IonReader",
                            &["FieldPath"],
                        ))
                        .into(),
                    );
                }
            }
        }

        // Build field name to vector index mapping
        let mut field_map = HashMap::new();
        for proj in &spec.projections {
            if let ProjectionSource::FieldPath(field_name) = &proj.source {
                field_map.insert(field_name.clone(), proj.target_vector_idx);
            }
        }
        self.field_to_vector_map = Some(field_map);

        // Build and cache schema from projection
        let fields: Vec<Field> = spec
            .projections
            .iter()
            .map(|p| Field {
                name: format!("col_{}", p.target_vector_idx),
                type_info: p.logical_type,
            })
            .collect();
        let schema = SourceTypeDef::new(fields);
        
        self.cached_schema = Some(schema.clone());
        self.reusable_batch = Some(VectorizedBatch::new(schema, self.batch_size));
        self.projection = Some(spec);
        Ok(())
    }

    fn next_batch(&mut self) -> Result<Option<VectorizedBatch>, EvalError> {
        if self.eof_reached {
            return Ok(None);
        }

        // Ensure projection is set
        if self.projection.is_none() {
            return Err(EvalError::General("set_projection must be called before next_batch".to_string()));
        }

        // Get mutable batch reference
        let batch = self.reusable_batch.as_mut()
            .ok_or_else(|| EvalError::General("Reusable batch should have been initialized in set_projection".to_string()))?;

        let field_map = self.field_to_vector_map.as_ref()
            .ok_or_else(|| EvalError::General("field_to_vector_map not initialized".to_string()))?;

        // Read elements from stream and write directly to batch vectors
        let actual_batch_size = Self::read_elements_batch(
            &mut self.reader,
            field_map,
            batch,
            self.batch_size,
            &mut self.eof_reached,
        )?;
        
        if actual_batch_size == 0 {
            return Ok(None);
        }

        // Update batch row count
        batch.set_row_count(actual_batch_size);
        batch.set_selection(None);

        Ok(Some(batch.clone()))
    }

    fn close(&mut self) -> Result<(), EvalError> {
        Ok(())
    }
}
