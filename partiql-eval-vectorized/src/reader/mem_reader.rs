use crate::batch::VectorizedBatch;
use crate::error::EvalError;
use crate::reader::{BatchReader, ProjectionSource, ProjectionSpec};
use wide::i64x4;

/// Generates fake columnar data for testing and benchmarking
/// 
/// Produces batches with two Int64 columns:
/// - Column "a": starts at 0, increments by 1
/// - Column "b": starts at 100, increments by 1
pub struct InMemoryGeneratedReader {
    current_row: i64,
    current_batch: usize,
    num_batches: usize,
    batch_size: usize,
    projection: Option<ProjectionSpec>,
    finished: bool,
    // Cache schema to avoid recreating it for every batch
    cached_schema: Option<crate::batch::SourceTypeDef>,
    // Reusable batch structure to avoid recreating for every batch
    reusable_batch: Option<VectorizedBatch>,
}

impl InMemoryGeneratedReader {
    /// Create a new fake data generator
    /// 
    /// Default configuration:
    /// - batch_size: 1024 rows per batch
    /// - num_batches: 10,000 batches (10,240,000 total rows)
    pub fn new() -> Self {
        Self {
            current_row: 0,
            current_batch: 0,
            num_batches: 10_000,
            batch_size: 1024,
            projection: None,
            finished: false,
            cached_schema: None,
            reusable_batch: None,
        }
    }

    /// Create a new fake data generator with custom configuration
    pub fn with_config(batch_size: usize, num_batches: usize) -> Self {
        Self {
            current_row: 0,
            current_batch: 0,
            num_batches,
            batch_size,
            projection: None,
            finished: false,
            cached_schema: None,
            reusable_batch: None,
        }
    }
}

impl Default for InMemoryGeneratedReader {
    fn default() -> Self {
        Self::new()
    }
}

impl BatchReader for InMemoryGeneratedReader {
    fn open(&mut self) -> Result<(), EvalError> {
        // No-op for InMemoryGeneratedReader
        Ok(())
    }

    fn set_projection(&mut self, spec: ProjectionSpec) -> Result<(), EvalError> {
        // Validate that all projections use FieldPath (not ColumnIndex)
        for proj in &spec.projections {
            match &proj.source {
                ProjectionSource::FieldPath(path) => {
                    // Validate that the field name is either "a" or "b"
                    if path != "a" && path != "b" {
                        return Err(EvalError::General(format!(
                            "InMemoryGeneratedReader only supports fields 'a' and 'b'. \
                            Found field '{}' at target vector index {}.",
                            path, proj.target_vector_idx
                        )));
                    }
                }
                ProjectionSource::ColumnIndex(idx) => {
                    return Err(EvalError::General(format!(
                        "InMemoryGeneratedReader does not support ColumnIndex projections. \
                        Found ColumnIndex({}) at target vector index {}. \
                        Use FieldPath for field access.",
                        idx, proj.target_vector_idx
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
                    ProjectionSource::FieldPath(path) => path.clone(),
                    ProjectionSource::ColumnIndex(idx) => format!("col_{}", idx),
                },
                type_info: p.logical_type,
            })
            .collect();
        let schema = SourceTypeDef::new(fields);
        
        // Cache schema (clone for storage, but we'll reuse the original for batch creation)
        self.cached_schema = Some(schema.clone());

        // Pre-allocate reusable batch structure (use the original schema, not the clone)
        self.reusable_batch = Some(VectorizedBatch::new(schema, self.batch_size));

        self.projection = Some(spec);
        Ok(())
    }

    fn next_batch(&mut self) -> Result<Option<VectorizedBatch>, EvalError> {
        // Check if projection has been set
        let projection = self.projection.as_ref().ok_or_else(|| {
            EvalError::General("set_projection must be called before next_batch".to_string())
        })?;

        // Check if we've generated all batches
        if self.finished || self.current_batch >= self.num_batches {
            self.finished = true;
            return Ok(None);
        }

        // Get or create reusable batch
        let batch = self.reusable_batch.as_mut().ok_or_else(|| {
            EvalError::General("Reusable batch should have been initialized in set_projection".to_string())
        })?;

        // Reset batch metadata (don't clear vectors - they maintain capacity and we'll overwrite data)
        batch.set_row_count(0);
        batch.set_selection(None);

        // Generate data directly into the reusable batch
        generate_fake_batch_into(
            batch,
            self.current_row,
            self.batch_size,
            projection,
        )?;

        // Update state
        self.current_row += self.batch_size as i64;
        self.current_batch += 1;

        // Clone the batch to return (the reusable batch stays in the reader for next iteration)
        Ok(Some(batch.clone()))
    }

    fn resolve(&self, field_name: &str) -> Option<ProjectionSource> {
        // Only support fields "a" and "b"
        if field_name == "a" || field_name == "b" {
            Some(ProjectionSource::FieldPath(field_name.to_string()))
        } else {
            None
        }
    }

    fn close(&mut self) -> Result<(), EvalError> {
        // No-op for InMemoryGeneratedReader
        Ok(())
    }
}

/// Generate a sequence of integers using SIMD optimization
/// 
/// Generates [start, start+1, start+2, ..., start+len-1] into the slice
/// Uses SIMD to process 4 elements at a time, then handles remainder with scalar code
/// 
/// Alignment: For optimal SIMD performance, the slice should be 32-byte aligned
/// (i64x4 = 4 * 8 bytes = 32 bytes). Rust's Vec allocator typically provides
/// sufficient alignment, but this is verified at runtime in debug builds.
#[inline]
fn generate_sequence_simd(slice: &mut [i64], start: i64, len: usize) {
    // Ensure we don't exceed slice bounds
    let len = len.min(slice.len());
    
    // Verify alignment in debug builds (32 bytes for i64x4)
    #[cfg(debug_assertions)]
    {
        const SIMD_ALIGN: usize = 32; // 4 * i64 = 32 bytes
        let ptr = slice.as_ptr() as usize;
        if ptr % SIMD_ALIGN != 0 {
            // Not a hard error - wide crate handles unaligned loads, but aligned is faster
            eprintln!("Warning: slice not 32-byte aligned (ptr={:p}, align={})", 
                     slice.as_ptr(), ptr % SIMD_ALIGN);
        }
    }
    
    unsafe {
        let mut ptr = slice.as_mut_ptr();
        let mut current = start;
        let simd_chunks = len / 4;
        let remainder = len % 4;
        
        // SIMD path: process 4 elements at a time
        // Create base sequence [0, 1, 2, 3] and add current value
        let base_sequence = i64x4::new([0, 1, 2, 3]);
        
        for _ in 0..simd_chunks {
            // Create sequence [current, current+1, current+2, current+3]
            let sequence = base_sequence + i64x4::splat(current);
            
            // Write to output
            // Note: copy_from_nonoverlapping is efficient and handles alignment
            let values = sequence.to_array();
            ptr.copy_from_nonoverlapping(values.as_ptr(), 4);
            
            ptr = ptr.add(4);
            current += 4;
        }
        
        // Scalar path: handle remainder
        for i in 0..remainder {
            *ptr.add(i) = current + i as i64;
        }
    }
}

/// Generate fake data into an existing batch (optimized for reuse)
/// 
/// Fills the batch with generated data based on the projection.
/// The batch should already be cleared and have the correct schema.
fn generate_fake_batch_into(
    batch: &mut VectorizedBatch,
    start_row: i64,
    batch_size: usize,
    projection: &ProjectionSpec,
) -> Result<(), EvalError> {
    use crate::batch::{LogicalType, PhysicalVectorEnum};

    // Generate data for each projection
    for proj in &projection.projections {
        let field_path = match &proj.source {
            ProjectionSource::FieldPath(path) => path,
            ProjectionSource::ColumnIndex(_) => {
                return Err(EvalError::General(
                    "ColumnIndex not supported for tuple reader".to_string(),
                ));
            }
        };

        // Get the column vector
        let vector = batch.column_mut(proj.target_vector_idx)?;

        // Only support Int64 type
        if proj.logical_type != LogicalType::Int64 {
            return Err(EvalError::General(format!(
                "InMemoryGeneratedReader only supports Int64 type. \
                Field '{}' has type {:?}",
                field_path, proj.logical_type
            )));
        }

        // Generate fake data based on field name
        // Optimization: Write directly to vector buffer, eliminating intermediate Vec allocation
        if let PhysicalVectorEnum::Int64(v) = &mut vector.physical {
            // Get mutable slice to write directly to the buffer
            let slice = v.as_mut_slice();
            
            // Verify capacity: Vector should be pre-allocated with batch_size capacity
            // This ensures no reallocations occur during data generation
            if slice.len() < batch_size {
                return Err(EvalError::General(format!(
                    "Vector buffer too small: expected {}, got {}. \
                    This indicates a capacity pre-allocation issue.",
                    batch_size, slice.len()
                )));
            }
            
            // In debug builds, verify we're not writing beyond capacity
            #[cfg(debug_assertions)]
            debug_assert_eq!(
                slice.len(), batch_size,
                "Vector should be pre-allocated with exact batch_size capacity"
            );
            
            // Generate sequence directly into the slice using SIMD optimization
            // Process 4 elements at a time using SIMD, then handle remainder with scalar code
            match field_path.as_str() {
                "a" => {
                    // Column "a": starts at start_row, increments by 1
                    generate_sequence_simd(slice, start_row, batch_size);
                }
                "b" => {
                    // Column "b": starts at start_row + 100, increments by 1
                    generate_sequence_simd(slice, start_row + 100, batch_size);
                }
                _ => {
                    return Err(EvalError::General(format!(
                        "Unknown field '{}'. Only 'a' and 'b' are supported.",
                        field_path
                    )));
                }
            }
        }
    }

    batch.set_row_count(batch_size);
    Ok(())
}

/// Generate a fake data batch with columns "a" and "b"
/// 
/// Legacy function - creates a new batch. For better performance, use generate_fake_batch_into
/// with a reusable batch.
fn generate_fake_batch(
    start_row: i64,
    batch_size: usize,
    projection: &ProjectionSpec,
    cached_schema: Option<&crate::batch::SourceTypeDef>,
) -> Result<VectorizedBatch, EvalError> {
    use crate::batch::SourceTypeDef;

    // Use cached schema if available, otherwise create it (shouldn't happen in normal flow)
    let schema = cached_schema
        .cloned()
        .unwrap_or_else(|| {
            use crate::batch::Field;
            let fields: Vec<Field> = projection
                .projections
                .iter()
                .map(|p| Field {
                    name: match &p.source {
                        ProjectionSource::FieldPath(path) => path.clone(),
                        ProjectionSource::ColumnIndex(idx) => format!("col_{}", idx),
                    },
                    type_info: p.logical_type,
                })
                .collect();
            SourceTypeDef::new(fields)
        });

    let mut batch = VectorizedBatch::new(schema, batch_size);
    generate_fake_batch_into(&mut batch, start_row, batch_size, projection)?;
    Ok(batch)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::batch::LogicalType;
    use crate::reader::{Projection, ProjectionSource, ProjectionSpec};

    #[test]
    fn test_fake_data_generation() {
        let mut reader = InMemoryGeneratedReader::with_config(10, 3);

        // Set projection for both columns
        let projections = vec![
            Projection::new(
                ProjectionSource::FieldPath("a".to_string()),
                0,
                LogicalType::Int64,
            ),
            Projection::new(
                ProjectionSource::FieldPath("b".to_string()),
                1,
                LogicalType::Int64,
            ),
        ];
        let projection_spec = ProjectionSpec::new(projections).unwrap();
        reader.set_projection(projection_spec).unwrap();

        // Read first batch
        let batch1 = reader.next_batch().unwrap().unwrap();
        assert_eq!(batch1.row_count(), 10);
        assert_eq!(batch1.total_column_count(), 2);

        // Verify data in first batch
        use crate::batch::PhysicalVectorEnum;
        let col_a = batch1.column(0).unwrap();
        let col_b = batch1.column(1).unwrap();
        
        if let PhysicalVectorEnum::Int64(v) = &col_a.physical {
            let slice = v.as_slice();
            assert_eq!(slice[0], 0);
            assert_eq!(slice[9], 9);
        }
        
        if let PhysicalVectorEnum::Int64(v) = &col_b.physical {
            let slice = v.as_slice();
            assert_eq!(slice[0], 100);
            assert_eq!(slice[9], 109);
        }

        // Read second batch
        let batch2 = reader.next_batch().unwrap().unwrap();
        assert_eq!(batch2.row_count(), 10);
        
        // Verify data continues from previous batch
        let col_a = batch2.column(0).unwrap();
        if let PhysicalVectorEnum::Int64(v) = &col_a.physical {
            let slice = v.as_slice();
            assert_eq!(slice[0], 10);
            assert_eq!(slice[9], 19);
        }

        // Read third batch
        let batch3 = reader.next_batch().unwrap().unwrap();
        assert_eq!(batch3.row_count(), 10);

        // No more batches (num_batches = 3)
        assert!(reader.next_batch().unwrap().is_none());
    }

    #[test]
    fn test_projection_single_column() {
        let mut reader = InMemoryGeneratedReader::with_config(5, 1);

        // Project only column "b"
        let projections = vec![
            Projection::new(
                ProjectionSource::FieldPath("b".to_string()),
                0,
                LogicalType::Int64,
            ),
        ];
        let projection_spec = ProjectionSpec::new(projections).unwrap();
        reader.set_projection(projection_spec).unwrap();

        let batch = reader.next_batch().unwrap().unwrap();
        assert_eq!(batch.row_count(), 5);
        assert_eq!(batch.total_column_count(), 1);

        // Verify only column "b" is present
        use crate::batch::PhysicalVectorEnum;
        let col = batch.column(0).unwrap();
        if let PhysicalVectorEnum::Int64(v) = &col.physical {
            let slice = v.as_slice();
            assert_eq!(slice[0], 100);
            assert_eq!(slice[4], 104);
        }
    }

    #[test]
    fn test_invalid_field_rejection() {
        let mut reader = InMemoryGeneratedReader::with_config(5, 1);

        // Try to project invalid field "c"
        let projections = vec![
            Projection::new(
                ProjectionSource::FieldPath("c".to_string()),
                0,
                LogicalType::Int64,
            ),
        ];
        let projection_spec = ProjectionSpec::new(projections).unwrap();

        let result = reader.set_projection(projection_spec);
        assert!(result.is_err());
        assert!(result.unwrap_err().to_string().contains("only supports fields 'a' and 'b'"));
    }

    #[test]
    fn test_column_index_rejection() {
        let mut reader = InMemoryGeneratedReader::with_config(5, 1);

        // Try to use ColumnIndex - should fail
        let projections = vec![Projection::new(
            ProjectionSource::ColumnIndex(0),
            0,
            LogicalType::Int64,
        )];
        let projection_spec = ProjectionSpec::new(projections).unwrap();

        let result = reader.set_projection(projection_spec);
        assert!(result.is_err());
        assert!(result.unwrap_err().to_string().contains("ColumnIndex"));
    }

    #[test]
    fn test_missing_projection_error() {
        let mut reader = InMemoryGeneratedReader::with_config(5, 1);

        // Try to read batch without setting projection
        let result = reader.next_batch();
        assert!(result.is_err());
        assert!(result.unwrap_err().to_string().contains("set_projection must be called"));
    }

    #[test]
    fn test_default_configuration() {
        let reader = InMemoryGeneratedReader::new();
        assert_eq!(reader.batch_size, 1024);
        assert_eq!(reader.num_batches, 10_000);
        assert_eq!(reader.current_row, 0);
        assert_eq!(reader.current_batch, 0);
    }

    #[test]
    fn test_resolve() {
        let reader = InMemoryGeneratedReader::new();
        
        // Should resolve "a" and "b"
        assert!(reader.resolve("a").is_some());
        assert!(reader.resolve("b").is_some());
        
        // Should not resolve other fields
        assert!(reader.resolve("c").is_none());
        assert!(reader.resolve("name").is_none());
    }
}
