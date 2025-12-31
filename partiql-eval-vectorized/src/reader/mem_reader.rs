use crate::batch::VectorizedBatch;
use crate::error::EvalError;
use crate::reader::{BatchReader, ProjectionSource, ProjectionSpec};

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

        // Generate the batch
        let batch = generate_fake_batch(
            self.current_row,
            self.batch_size,
            projection,
        )?;

        // Update state
        self.current_row += self.batch_size as i64;
        self.current_batch += 1;

        Ok(Some(batch))
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

/// Generate a fake data batch with columns "a" and "b"
fn generate_fake_batch(
    start_row: i64,
    batch_size: usize,
    projection: &ProjectionSpec,
) -> Result<VectorizedBatch, EvalError> {
    use crate::batch::{Field, LogicalType, PhysicalVectorEnum, SourceTypeDef};

    // Create schema from projection
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

    let schema = SourceTypeDef::new(fields);
    let mut batch = VectorizedBatch::new(schema, batch_size);

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
        if let PhysicalVectorEnum::Int64(v) = &mut vector.physical {
            let slice = v.as_mut_slice();
            for i in 0..batch_size {
                let row_num = start_row + i as i64;
                slice[i] = match field_path.as_str() {
                    "a" => row_num,           // Column "a": starts at 0, increments by 1
                    "b" => row_num + 100,     // Column "b": starts at 100, increments by 1
                    _ => {
                        return Err(EvalError::General(format!(
                            "Unknown field '{}'. Only 'a' and 'b' are supported.",
                            field_path
                        )));
                    }
                };
            }
        }
    }

    batch.set_row_count(batch_size);
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
