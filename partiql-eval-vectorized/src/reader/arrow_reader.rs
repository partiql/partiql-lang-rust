use crate::batch::VectorizedBatch;
use crate::error::EvalError;
use crate::reader::{BatchReader, ProjectionSource, ProjectionSpec};
use arrow::record_batch::RecordBatch;
use arrow::datatypes::Schema;
use arrow_array::{Array, ArrayRef, BooleanArray, Float64Array, Int64Array, StringArray};
use arrow_buffer::{Buffer, MutableBuffer};
use bytes::Bytes;
use arrow_ipc::convert::fb_to_schema;
use arrow_ipc::reader::{FileDecoder, read_footer_length};
use arrow_ipc::{Block, root_as_footer};
use memmap2::Mmap;
use std::fs::File;
use std::path::Path;
use std::sync::Arc;

/// Arrow IPC file reader with true zero-copy memory-mapped access
/// 
/// This reader uses Arrow's `FileDecoder` API to create RecordBatches that directly
/// reference memory-mapped file data, avoiding data copying during deserialization.
/// 
/// # Zero-Copy Implementation
/// 
/// The zero-copy chain works as follows:
/// 1. Memory-map the file using `memmap2::Mmap`
/// 2. Convert to `bytes::Bytes` (zero-copy, owns the mmap)
/// 3. Convert to `arrow_buffer::Buffer` (zero-copy, reference counted)
/// 4. Use `FileDecoder` to create arrays that reference the buffer directly
/// 
/// Batches are decoded lazily on-demand, keeping memory usage low for large files.
pub struct ArrowReader {
    /// Zero-copy decoder that references memory-mapped file data
    decoder: Option<ZeroCopyDecoder>,
    
    /// Current batch index for iteration
    current_batch_idx: usize,
    
    /// Schema from the Arrow file
    schema: Option<Arc<Schema>>,
    
    /// File path for debugging/error messages
    file_path: Option<String>,
    
    /// Projection specification
    projection: Option<ProjectionSpec>,
    
    /// Whether we've finished reading all batches
    finished: bool,
    
    /// Cached schema built from projection (reused across batches)
    cached_schema: Option<crate::batch::SourceTypeDef>,
    
    /// Batch size for output batches (controls re-batching)
    batch_size: usize,
    
    /// Reusable batch structure (pre-allocated in set_projection)
    reusable_batch: Option<VectorizedBatch>,
    
    /// Current Arrow batch being sliced for re-batching
    current_arrow_batch: Option<RecordBatch>,
    
    /// Current offset within the Arrow batch for re-batching
    current_offset: usize,
}

/// Zero-copy decoder for Arrow IPC files
/// 
/// This decoder keeps the memory-mapped file data alive through the Buffer,
/// and uses FileDecoder to create RecordBatches that reference the buffer
/// directly without copying data.
struct ZeroCopyDecoder {
    /// Buffer containing the memory-mapped file data
    /// This keeps the mmap alive through reference counting
    buffer: Buffer,
    
    /// Low-level Arrow decoder that creates zero-copy arrays
    decoder: FileDecoder,
    
    /// Locations of record batches within the buffer
    batch_blocks: Vec<Block>,
    
    /// Schema from the Arrow file
    schema: Arc<Schema>,
}

impl ZeroCopyDecoder {
    /// Create a new zero-copy decoder from a memory-mapped file
    /// 
    /// This performs the following steps:
    /// 1. Converts the mmap to a Buffer (zero-copy chain: Mmap → Bytes → Buffer)
    /// 2. Reads the Arrow IPC footer to extract schema and batch locations
    /// 3. Creates a FileDecoder for lazy, zero-copy batch reading
    /// 4. Reads any dictionary batches (required for certain data types)
    fn new(mmap: Mmap) -> Result<Self, EvalError> {
        // Convert mmap to Buffer through Bytes (both conversions are zero-copy)
        // The Bytes owns the Mmap and keeps it alive
        let bytes = Bytes::from_owner(mmap);

        // Create Buffer from the bytes slice - Buffer will hold Arc to the bytes
        // NOTE: We can only do this since we assume it's aligned.
        let buffer = unsafe {
            Buffer::from_custom_allocation(
                std::ptr::NonNull::new(bytes.as_ptr() as _).expect("should be a valid pointer"),
                bytes.len(),
                Arc::new(bytes),
            )
        };

        // Read the Arrow IPC footer
        let trailer_start = buffer.len() - 10;
        let footer_len = read_footer_length(
            buffer[trailer_start..].try_into()
                .map_err(|_| EvalError::General("Failed to read footer length".to_string()))?
        ).map_err(|e| EvalError::General(format!("Failed to read footer length: {}", e)))?;
        
        let footer_start = trailer_start - footer_len;
        let footer = root_as_footer(&buffer[footer_start..trailer_start])
            .map_err(|e| EvalError::General(format!("Failed to parse footer: {}", e)))?;
        
        // Extract schema from footer
        let schema = fb_to_schema(footer.schema()
            .ok_or_else(|| EvalError::General("Footer missing schema".to_string()))?);
        
        // Create the FileDecoder
        let mut decoder = FileDecoder::new(Arc::new(schema.clone()), footer.version());
        
        // Read dictionary batches if present (required for dictionary-encoded columns)
        if let Some(dictionaries) = footer.dictionaries() {
            for block in dictionaries.iter() {
                let block_len = block.bodyLength() as usize + block.metaDataLength() as usize;
                let data = buffer.slice_with_length(block.offset() as usize, block_len);
                decoder.read_dictionary(&block, &data)
                    .map_err(|e| EvalError::General(format!("Failed to read dictionary: {}", e)))?;
            }
        }
        
        // Extract batch locations from footer
        let batch_blocks: Vec<Block> = footer.recordBatches()
            .map(|batches| batches.iter().copied().collect())
            .unwrap_or_default();
        
        Ok(Self {
            buffer,
            decoder,
            batch_blocks,
            schema: Arc::new(schema),
        })
    }
    
    /// Get the number of record batches in the file
    fn num_batches(&self) -> usize {
        self.batch_blocks.len()
    }
    
    /// Read a record batch at the specified index (zero-copy)
    /// 
    /// This creates a RecordBatch whose arrays reference the underlying buffer
    /// directly, without copying data. The Buffer keeps the mmap alive through
    /// reference counting.
    fn get_batch(&self, index: usize) -> Result<RecordBatch, EvalError> {
        if index >= self.batch_blocks.len() {
            return Err(EvalError::General(format!(
                "Batch index {} out of range (0..{})",
                index,
                self.batch_blocks.len()
            )));
        }
        
        let block = &self.batch_blocks[index];
        let block_len = block.bodyLength() as usize + block.metaDataLength() as usize;
        
        // This slice operation is zero-copy - it creates a view into the buffer
        let data = self.buffer.slice_with_length(block.offset() as usize, block_len);
        
        // FileDecoder creates arrays that reference the buffer directly
        self.decoder.read_record_batch(block, &data)
            .map_err(|e| EvalError::General(format!("Failed to read batch: {}", e)))?
            .ok_or_else(|| EvalError::General("Batch was None".to_string()))
    }
    
    /// Get the schema
    fn schema(&self) -> &Arc<Schema> {
        &self.schema
    }
}

impl ArrowReader {
    /// Create ArrowReader from an Arrow IPC file using zero-copy memory mapping
    /// 
    /// This method:
    /// 1. Memory-maps the file for efficient OS-level access
    /// 2. Creates a zero-copy decoder that references the mapped memory
    /// 3. Enables lazy, on-demand batch reading without upfront data loading
    /// 
    /// # Performance Benefits
    /// - **True zero-copy**: Arrays reference memory-mapped data directly
    /// - **Lazy loading**: Batches decoded only when requested
    /// - **Memory efficient**: Only active batch data in memory
    /// - **OS-optimized**: Kernel handles page loading and caching
    /// 
    /// # Example
    /// ```no_run
    /// use partiql_eval_vectorized::reader::ArrowReader;
    /// 
    /// let reader = ArrowReader::from_file("data.arrow", 1024)?;
    /// // File is memory-mapped, but no batches loaded yet
    /// # Ok::<(), partiql_eval_vectorized::error::EvalError>(())
    /// ```
    pub fn from_file<P: AsRef<Path>>(file_path: P, batch_size: usize) -> Result<Self, EvalError> {
        let path_str = file_path.as_ref().to_string_lossy().to_string();
        
        // Open and memory-map the file
        let file = File::open(file_path.as_ref()).map_err(|e| {
            EvalError::General(format!("Failed to open Arrow file '{}': {}", path_str, e))
        })?;
        
        // Safety: We're mapping a read-only file that won't be modified during reading
        let mmap = unsafe {
            Mmap::map(&file).map_err(|e| {
                EvalError::General(format!("Failed to memory-map file '{}': {}", path_str, e))
            })?
        };
        
        // Create zero-copy decoder
        let decoder = ZeroCopyDecoder::new(mmap)?;
        let schema = decoder.schema().clone();
        
        Ok(Self {
            decoder: Some(decoder),
            current_batch_idx: 0,
            schema: Some(schema),
            file_path: Some(path_str),
            projection: None,
            finished: false,
            cached_schema: None,
            batch_size,
            reusable_batch: None,
            current_arrow_batch: None,
            current_offset: 0,
        })
    }
}

impl BatchReader for ArrowReader {
    fn open(&mut self) -> Result<(), EvalError> {
        // No-op - file is already opened and mapped in from_file()
        Ok(())
    }

    fn resolve(&self, field_name: &str) -> Option<ProjectionSource> {
        let schema = self.schema.as_ref()?;
        
        for (idx, field) in schema.fields().iter().enumerate() {
            if field.name() == field_name {
                return Some(ProjectionSource::ColumnIndex(idx));
            }
        }
        
        None
    }

    fn set_projection(&mut self, spec: ProjectionSpec) -> Result<(), EvalError> {
        // Validate that all projections use ColumnIndex (not FieldPath)
        for proj in &spec.projections {
            match &proj.source {
                ProjectionSource::ColumnIndex(_) => {
                    // Valid for Arrow reader
                }
                ProjectionSource::FieldPath(path) => {
                    return Err(EvalError::General(format!(
                        "ArrowReader does not support FieldPath projections. \
                        Found FieldPath('{}') at target vector index {}. \
                        Use ColumnIndex for columnar data access.",
                        path, proj.target_vector_idx
                    )));
                }
            }
        }

        // Validate column indices against schema
        let schema = self.schema.as_ref()
            .ok_or_else(|| EvalError::General("No schema available".to_string()))?;

        let num_columns = schema.fields().len();
        for proj in &spec.projections {
            if let ProjectionSource::ColumnIndex(col_idx) = &proj.source {
                if *col_idx >= num_columns {
                    return Err(EvalError::General(format!(
                        "Column index {} is out of bounds. Arrow schema has {} columns.",
                        col_idx, num_columns
                    )));
                }
            }
        }

        // Build and cache schema from projection
        use crate::batch::{Field, SourceTypeDef};
        let fields: Vec<Field> = spec
            .projections
            .iter()
            .map(|p| Field {
                name: match &p.source {
                    ProjectionSource::ColumnIndex(idx) => {
                        schema.field(*idx).name().to_string()
                    }
                    ProjectionSource::FieldPath(path) => path.clone(),
                },
                type_info: p.logical_type,
            })
            .collect();
        let cached_schema = SourceTypeDef::new(fields);
        self.cached_schema = Some(cached_schema.clone());

        // Pre-allocate reusable batch structure
        self.reusable_batch = Some(VectorizedBatch::new(cached_schema, self.batch_size));

        self.projection = Some(spec);
        Ok(())
    }

    fn next_batch(&mut self) -> Result<Option<VectorizedBatch>, EvalError> {
        // Check if projection has been set
        if self.projection.is_none() {
            return Err(EvalError::General(
                "set_projection must be called before next_batch".to_string()
            ));
        }

        // Check if we're already finished
        if self.finished {
            return Ok(None);
        }

        // Get the decoder
        let decoder = self.decoder.as_ref()
            .ok_or_else(|| EvalError::General("Decoder not initialized".to_string()))?;

        loop {
            // If we have a current Arrow batch with remaining rows
            if let Some(arrow_batch) = &self.current_arrow_batch {
                let remaining = arrow_batch.num_rows() - self.current_offset;
                
                if remaining > 0 {
                    // Calculate slice size (min of batch_size and remaining)
                    let slice_size = remaining.min(self.batch_size);
                    
                    // Zero-copy slice the Arrow batch
                    let sliced = arrow_batch.slice(self.current_offset, slice_size);
                    self.current_offset += slice_size;
                    
                    // Convert slice to VectorizedBatch
                    let batch = self.convert_arrow_to_vectorized_batch(&sliced)?;
                    return Ok(Some(batch));
                }
            }
            
            // Need to read next Arrow batch from file
            if self.current_batch_idx >= decoder.num_batches() {
                self.finished = true;
                return Ok(None);  // All done
            }
            
            // Read next Arrow RecordBatch (zero-copy from memory-mapped file)
            let arrow_batch = decoder.get_batch(self.current_batch_idx)?;
            self.current_batch_idx += 1;
            self.current_arrow_batch = Some(arrow_batch);
            self.current_offset = 0;
            
            // Loop back to slice it
        }
    }

    fn close(&mut self) -> Result<(), EvalError> {
        // No-op - decoder and mmap will be dropped automatically
        Ok(())
    }
}

impl ArrowReader {
    /// Convert an Arrow RecordBatch to a PartiQL VectorizedBatch
    /// 
    /// Note: This step does copy data from Arrow arrays to PartiQL vectors,
    /// as VectorizedBatch uses owned Vec<T> for storage. The zero-copy
    /// optimization applies to reading from the file into Arrow arrays.
    fn convert_arrow_to_vectorized_batch(
        &mut self,
        arrow_batch: &RecordBatch,
    ) -> Result<VectorizedBatch, EvalError> {
        use crate::batch::{LogicalType, PhysicalVectorEnum};

        let batch_size = arrow_batch.num_rows();
        let projection = self.projection.as_ref().unwrap(); // Safe: checked in next_batch

        // Use cached schema
        let schema = self.cached_schema.as_ref().ok_or_else(|| {
            EvalError::General("Schema not cached. set_projection must be called first.".to_string())
        })?;

        // Handle variable batch size
        if batch_size > self.batch_size {
            self.batch_size = batch_size;
            self.reusable_batch = Some(VectorizedBatch::new(schema.clone(), batch_size));
        }

        // Get reusable batch
        let batch = self.reusable_batch.as_mut().ok_or_else(|| {
            EvalError::General("Reusable batch not initialized".to_string())
        })?;

        // Reset batch metadata
        batch.set_row_count(0);
        batch.set_selection(None);

        // Convert each projected column from Arrow to PartiQL
        for proj in &projection.projections {
            let col_idx = match &proj.source {
                ProjectionSource::ColumnIndex(idx) => *idx,
                ProjectionSource::FieldPath(_) => {
                    return Err(EvalError::General(
                        "FieldPath not supported for Arrow reader".to_string(),
                    ));
                }
            };

            let arrow_column = arrow_batch.column(col_idx);
            let vector = batch.column_mut(proj.target_vector_idx)?;

            // Convert based on logical type
            match proj.logical_type {
                LogicalType::Int64 => {
                    if let PhysicalVectorEnum::Int64(v) = &mut vector.physical {
                        convert_arrow_to_int64(arrow_column, v.as_mut_slice())?;
                    }
                }
                LogicalType::Float64 => {
                    if let PhysicalVectorEnum::Float64(v) = &mut vector.physical {
                        convert_arrow_to_float64(arrow_column, v.as_mut_slice())?;
                    }
                }
                LogicalType::Boolean => {
                    if let PhysicalVectorEnum::Boolean(v) = &mut vector.physical {
                        convert_arrow_to_boolean(arrow_column, v.as_mut_slice())?;
                    }
                }
                LogicalType::String => {
                    if let PhysicalVectorEnum::String(v) = &mut vector.physical {
                        convert_arrow_to_string(arrow_column, v.as_mut_slice())?;
                    }
                }
            }
        }

        batch.set_row_count(batch_size);
        
        // Clone the batch to return
        Ok(batch.clone())
    }
}

/// Convert Arrow array to Int64 vector
fn convert_arrow_to_int64(arrow_array: &ArrayRef, target: &mut [i64]) -> Result<(), EvalError> {
    let array_len = arrow_array.len();
    #[cfg(debug_assertions)]
    debug_assert!(
        target.len() >= array_len,
        "Int64 vector buffer too small: expected {}, got {}",
        array_len,
        target.len()
    );
    
    if let Some(int64_array) = arrow_array.as_any().downcast_ref::<Int64Array>() {
        for (i, value) in int64_array.iter().enumerate() {
            if i >= target.len() {
                break;
            }
            target[i] = value.unwrap_or(0);
        }
    } else if let Some(float64_array) = arrow_array.as_any().downcast_ref::<Float64Array>() {
        for (i, value) in float64_array.iter().enumerate() {
            if i >= target.len() {
                break;
            }
            target[i] = value.unwrap_or(0.0) as i64;
        }
    } else {
        return Err(EvalError::General(format!(
            "Cannot convert Arrow array type {:?} to Int64",
            arrow_array.data_type()
        )));
    }
    Ok(())
}

/// Convert Arrow array to Float64 vector
fn convert_arrow_to_float64(arrow_array: &ArrayRef, target: &mut [f64]) -> Result<(), EvalError> {
    let array_len = arrow_array.len();
    #[cfg(debug_assertions)]
    debug_assert!(
        target.len() >= array_len,
        "Float64 vector buffer too small: expected {}, got {}",
        array_len,
        target.len()
    );
    
    if let Some(float64_array) = arrow_array.as_any().downcast_ref::<Float64Array>() {
        for (i, value) in float64_array.iter().enumerate() {
            if i >= target.len() {
                break;
            }
            target[i] = value.unwrap_or(0.0);
        }
    } else if let Some(int64_array) = arrow_array.as_any().downcast_ref::<Int64Array>() {
        for (i, value) in int64_array.iter().enumerate() {
            if i >= target.len() {
                break;
            }
            target[i] = value.unwrap_or(0) as f64;
        }
    } else {
        return Err(EvalError::General(format!(
            "Cannot convert Arrow array type {:?} to Float64",
            arrow_array.data_type()
        )));
    }
    Ok(())
}

/// Convert Arrow array to Boolean vector
fn convert_arrow_to_boolean(arrow_array: &ArrayRef, target: &mut [bool]) -> Result<(), EvalError> {
    let array_len = arrow_array.len();
    #[cfg(debug_assertions)]
    debug_assert!(
        target.len() >= array_len,
        "Boolean vector buffer too small: expected {}, got {}",
        array_len,
        target.len()
    );
    
    if let Some(bool_array) = arrow_array.as_any().downcast_ref::<BooleanArray>() {
        for (i, value) in bool_array.iter().enumerate() {
            if i >= target.len() {
                break;
            }
            target[i] = value.unwrap_or(false);
        }
    } else {
        return Err(EvalError::General(format!(
            "Cannot convert Arrow array type {:?} to Boolean",
            arrow_array.data_type()
        )));
    }
    Ok(())
}

/// Convert Arrow array to String vector
fn convert_arrow_to_string(arrow_array: &ArrayRef, target: &mut [String]) -> Result<(), EvalError> {
    let array_len = arrow_array.len();
    #[cfg(debug_assertions)]
    debug_assert!(
        target.len() >= array_len,
        "String vector buffer too small: expected {}, got {}",
        array_len,
        target.len()
    );
    
    if let Some(string_array) = arrow_array.as_any().downcast_ref::<StringArray>() {
        for (i, value) in string_array.iter().enumerate() {
            if i >= target.len() {
                break;
            }
            target[i] = value.unwrap_or("").to_string();
        }
    } else if let Some(int64_array) = arrow_array.as_any().downcast_ref::<Int64Array>() {
        for (i, value) in int64_array.iter().enumerate() {
            if i >= target.len() {
                break;
            }
            target[i] = value.unwrap_or(0).to_string();
        }
    } else if let Some(float64_array) = arrow_array.as_any().downcast_ref::<Float64Array>() {
        for (i, value) in float64_array.iter().enumerate() {
            if i >= target.len() {
                break;
            }
            target[i] = value.unwrap_or(0.0).to_string();
        }
    } else if let Some(bool_array) = arrow_array.as_any().downcast_ref::<BooleanArray>() {
        for (i, value) in bool_array.iter().enumerate() {
            if i >= target.len() {
                break;
            }
            target[i] = value.unwrap_or(false).to_string();
        }
    } else {
        return Err(EvalError::General(format!(
            "Cannot convert Arrow array type {:?} to String",
            arrow_array.data_type()
        )));
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::batch::LogicalType;
    use crate::reader::{Projection, ProjectionSource, ProjectionSpec};
    use arrow::array::{Int64Array, StringArray, Float64Array, BooleanArray};
    use arrow::datatypes::{DataType, Field as ArrowField, Schema};
    use arrow::record_batch::RecordBatch;
    use arrow_ipc::writer::FileWriter;
    use std::sync::Arc;
    use tempfile::NamedTempFile;

    fn create_test_ipc_file() -> NamedTempFile {
        // Create a temporary file
        let temp_file = NamedTempFile::new().unwrap();
        
        // Create test schema
        let schema = Arc::new(Schema::new(vec![
            ArrowField::new("id", DataType::Int64, false),
            ArrowField::new("name", DataType::Utf8, false),
            ArrowField::new("score", DataType::Float64, false),
            ArrowField::new("active", DataType::Boolean, false),
        ]));
        
        // Create test data
        let batch = RecordBatch::try_new(
            schema.clone(),
            vec![
                Arc::new(Int64Array::from(vec![1, 2, 3])),
                Arc::new(StringArray::from(vec!["Alice", "Bob", "Charlie"])),
                Arc::new(Float64Array::from(vec![95.5, 87.2, 92.8])),
                Arc::new(BooleanArray::from(vec![true, false, true])),
            ],
        )
        .unwrap();
        
        // Write to IPC file
        let file = temp_file.reopen().unwrap();
        let mut writer = FileWriter::try_new(file, &schema).unwrap();
        writer.write(&batch).unwrap();
        writer.finish().unwrap();
        
        temp_file
    }

    #[test]
    fn test_arrow_reader_zero_copy() {
        let temp_file = create_test_ipc_file();
        let mut reader = ArrowReader::from_file(temp_file.path(), 1024).unwrap();

        // Set projection
        let projections = vec![
            Projection::new(ProjectionSource::ColumnIndex(0), 0, LogicalType::Int64),
            Projection::new(ProjectionSource::ColumnIndex(1), 1, LogicalType::String),
            Projection::new(ProjectionSource::ColumnIndex(2), 2, LogicalType::Float64),
            Projection::new(ProjectionSource::ColumnIndex(3), 3, LogicalType::Boolean),
        ];
        let projection_spec = ProjectionSpec::new(projections).unwrap();
        reader.set_projection(projection_spec).unwrap();

        // Read batch
        let batch = reader.next_batch().unwrap().unwrap();
        assert_eq!(batch.row_count(), 3);
        assert_eq!(batch.total_column_count(), 4);

        // Verify no more batches
        assert!(reader.next_batch().unwrap().is_none());
    }

    #[test]
    fn test_arrow_reader_field_path_rejection() {
        let temp_file = create_test_ipc_file();
        let mut reader = ArrowReader::from_file(temp_file.path(), 1024).unwrap();

        // Try to set projection with FieldPath - should fail
        let projections = vec![Projection::new(
            ProjectionSource::FieldPath("name".to_string()),
            0,
            LogicalType::String,
        )];
        let projection_spec = ProjectionSpec::new(projections).unwrap();

        let result = reader.set_projection(projection_spec);
        assert!(result.is_err());
        assert!(result.unwrap_err().to_string().contains("FieldPath"));
    }

    #[test]
    fn test_arrow_reader_column_bounds_check() {
        let temp_file = create_test_ipc_file();
        let mut reader = ArrowReader::from_file(temp_file.path(), 1024).unwrap();

        // Try to access column index 10 when only 4 columns exist
        let projections = vec![Projection::new(
            ProjectionSource::ColumnIndex(10),
            0,
            LogicalType::Int64,
        )];
        let projection_spec = ProjectionSpec::new(projections).unwrap();

        let result = reader.set_projection(projection_spec);
        assert!(result.is_err());
        assert!(result.unwrap_err().to_string().contains("out of bounds"));
    }
}
