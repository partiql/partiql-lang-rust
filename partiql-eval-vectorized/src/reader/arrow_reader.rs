use crate::batch::VectorizedBatch;
use crate::error::EvalError;
use crate::reader::{BatchReader, ProjectionSource, ProjectionSpec};
use arrow::record_batch::RecordBatch;
use arrow_array::{Array, ArrayRef, BooleanArray, Float64Array, Int64Array, StringArray};
use std::sync::Arc;

/// Arrow RecordBatch reader for Phase 0 compliance
/// Converts Arrow RecordBatch data to PartiQL VectorizedBatch format
pub struct ArrowReader {
    batches: Vec<RecordBatch>,
    current_batch_idx: usize,
    projection: Option<ProjectionSpec>,
    finished: bool,
}

impl ArrowReader {
    /// Create new ArrowReader from a vector of Arrow RecordBatches
    pub fn new(batches: Vec<RecordBatch>) -> Self {
        Self {
            batches,
            current_batch_idx: 0,
            projection: None,
            finished: false,
        }
    }

    /// Create ArrowReader from a single RecordBatch
    pub fn from_record_batch(batch: RecordBatch) -> Self {
        Self::new(vec![batch])
    }
}

impl BatchReader for ArrowReader {
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

        // Validate column indices against available batches
        if !self.batches.is_empty() {
            let schema = self.batches[0].schema();
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
        }

        self.projection = Some(spec);
        Ok(())
    }

    fn next_batch(&mut self) -> Result<Option<VectorizedBatch>, EvalError> {
        // Check if projection has been set
        let projection = self.projection.as_ref().ok_or_else(|| {
            EvalError::General("set_projection must be called before next_batch".to_string())
        })?;

        // Check if we're already finished
        if self.finished || self.current_batch_idx >= self.batches.len() {
            return Ok(None);
        }

        // Get current Arrow RecordBatch
        let arrow_batch = &self.batches[self.current_batch_idx];
        self.current_batch_idx += 1;

        // Check if we've processed all batches
        if self.current_batch_idx >= self.batches.len() {
            self.finished = true;
        }

        // Convert Arrow RecordBatch to PartiQL VectorizedBatch
        let batch = convert_arrow_to_vectorized_batch(arrow_batch, projection)?;
        Ok(Some(batch))
    }
}

/// Convert an Arrow RecordBatch to a PartiQL VectorizedBatch
fn convert_arrow_to_vectorized_batch(
    arrow_batch: &RecordBatch,
    projection: &ProjectionSpec,
) -> Result<VectorizedBatch, EvalError> {
    use crate::batch::{Field, LogicalType, PhysicalVectorEnum, SourceTypeDef};

    let batch_size = arrow_batch.num_rows();

    // Create schema from projection
    let fields: Vec<Field> = projection
        .projections
        .iter()
        .map(|p| Field {
            name: match &p.source {
                ProjectionSource::ColumnIndex(idx) => {
                    // Get field name from Arrow schema if available
                    if let Some(field) = arrow_batch.schema().field(*idx).name().get(0..) {
                        field.to_string()
                    } else {
                        format!("col_{}", idx)
                    }
                }
                ProjectionSource::FieldPath(path) => path.clone(),
            },
            type_info: p.logical_type,
        })
        .collect();

    let schema = SourceTypeDef::new(fields);
    let mut batch = VectorizedBatch::new(schema, batch_size);

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

        // Get Arrow column
        let arrow_column = arrow_batch.column(col_idx);

        // Get target PartiQL vector
        let vector = batch.column_mut(proj.target_vector_idx)?;

        // Convert Arrow array to PartiQL vector based on logical type
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
    Ok(batch)
}

/// Convert Arrow array to Int64 vector
fn convert_arrow_to_int64(arrow_array: &ArrayRef, target: &mut [i64]) -> Result<(), EvalError> {
    // Try different Arrow array types that can convert to Int64
    if let Some(int64_array) = arrow_array.as_any().downcast_ref::<Int64Array>() {
        // Direct Int64 conversion
        for (i, value) in int64_array.iter().enumerate() {
            if i >= target.len() {
                break;
            }
            target[i] = value.unwrap_or(0); // Use 0 for null values
        }
    } else if let Some(float64_array) = arrow_array.as_any().downcast_ref::<Float64Array>() {
        // Float64 to Int64 conversion
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
    // Try different Arrow array types that can convert to Float64
    if let Some(float64_array) = arrow_array.as_any().downcast_ref::<Float64Array>() {
        // Direct Float64 conversion
        for (i, value) in float64_array.iter().enumerate() {
            if i >= target.len() {
                break;
            }
            target[i] = value.unwrap_or(0.0); // Use 0.0 for null values
        }
    } else if let Some(int64_array) = arrow_array.as_any().downcast_ref::<Int64Array>() {
        // Int64 to Float64 conversion
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
    if let Some(bool_array) = arrow_array.as_any().downcast_ref::<BooleanArray>() {
        for (i, value) in bool_array.iter().enumerate() {
            if i >= target.len() {
                break;
            }
            target[i] = value.unwrap_or(false); // Use false for null values
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
    if let Some(string_array) = arrow_array.as_any().downcast_ref::<StringArray>() {
        for (i, value) in string_array.iter().enumerate() {
            if i >= target.len() {
                break;
            }
            target[i] = value.unwrap_or("").to_string(); // Use empty string for null values
        }
    } else if let Some(int64_array) = arrow_array.as_any().downcast_ref::<Int64Array>() {
        // Int64 to String conversion
        for (i, value) in int64_array.iter().enumerate() {
            if i >= target.len() {
                break;
            }
            target[i] = value.unwrap_or(0).to_string();
        }
    } else if let Some(float64_array) = arrow_array.as_any().downcast_ref::<Float64Array>() {
        // Float64 to String conversion
        for (i, value) in float64_array.iter().enumerate() {
            if i >= target.len() {
                break;
            }
            target[i] = value.unwrap_or(0.0).to_string();
        }
    } else if let Some(bool_array) = arrow_array.as_any().downcast_ref::<BooleanArray>() {
        // Boolean to String conversion
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
    use arrow::array::{BooleanArray, Float64Array, Int64Array, StringArray};
    use arrow::datatypes::{DataType, Field as ArrowField, Schema};
    use std::sync::Arc;

    #[test]
    fn test_arrow_reader_basic() {
        // Create Arrow RecordBatch
        let schema = Arc::new(Schema::new(vec![
            ArrowField::new("id", DataType::Int64, false),
            ArrowField::new("name", DataType::Utf8, false),
            ArrowField::new("score", DataType::Float64, false),
            ArrowField::new("active", DataType::Boolean, false),
        ]));

        let id_array = Arc::new(Int64Array::from(vec![1, 2, 3]));
        let name_array = Arc::new(StringArray::from(vec!["Alice", "Bob", "Charlie"]));
        let score_array = Arc::new(Float64Array::from(vec![95.5, 87.2, 92.8]));
        let active_array = Arc::new(BooleanArray::from(vec![true, false, true]));

        let record_batch = RecordBatch::try_new(
            schema,
            vec![id_array, name_array, score_array, active_array],
        )
        .unwrap();

        let mut reader = ArrowReader::from_record_batch(record_batch);

        // Set projection using ColumnIndex
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
    fn test_arrow_reader_type_conversions() {
        // Create Arrow RecordBatch with type conversions
        let schema = Arc::new(Schema::new(vec![
            ArrowField::new("int_col", DataType::Int64, false),
            ArrowField::new("float_col", DataType::Float64, false),
        ]));

        let int_array = Arc::new(Int64Array::from(vec![42, 100]));
        let float_array = Arc::new(Float64Array::from(vec![3.14, 2.71]));

        let record_batch = RecordBatch::try_new(schema, vec![int_array, float_array]).unwrap();

        let mut reader = ArrowReader::from_record_batch(record_batch);

        // Set projection with type conversions
        let projections = vec![
            Projection::new(ProjectionSource::ColumnIndex(0), 0, LogicalType::Float64), // Int64 -> Float64
            Projection::new(ProjectionSource::ColumnIndex(1), 1, LogicalType::String), // Float64 -> String
        ];
        let projection_spec = ProjectionSpec::new(projections).unwrap();
        reader.set_projection(projection_spec).unwrap();

        // Read batch
        let batch = reader.next_batch().unwrap().unwrap();
        assert_eq!(batch.row_count(), 2);
        assert_eq!(batch.total_column_count(), 2);
    }

    #[test]
    fn test_arrow_reader_field_path_rejection() {
        let schema = Arc::new(Schema::new(vec![ArrowField::new(
            "name",
            DataType::Utf8,
            false,
        )]));

        let name_array = Arc::new(StringArray::from(vec!["Alice"]));
        let record_batch = RecordBatch::try_new(schema, vec![name_array]).unwrap();

        let mut reader = ArrowReader::from_record_batch(record_batch);

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
    fn test_arrow_reader_column_index_bounds_check() {
        let schema = Arc::new(Schema::new(vec![ArrowField::new(
            "col1",
            DataType::Int64,
            false,
        )]));

        let col1_array = Arc::new(Int64Array::from(vec![1, 2, 3]));
        let record_batch = RecordBatch::try_new(schema, vec![col1_array]).unwrap();

        let mut reader = ArrowReader::from_record_batch(record_batch);

        // Try to access column index 1 when only column 0 exists
        let projections = vec![Projection::new(
            ProjectionSource::ColumnIndex(1),
            0,
            LogicalType::Int64,
        )];
        let projection_spec = ProjectionSpec::new(projections).unwrap();

        let result = reader.set_projection(projection_spec);
        assert!(result.is_err());
        assert!(result.unwrap_err().to_string().contains("out of bounds"));
    }

    #[test]
    fn test_arrow_reader_multiple_batches() {
        // Create multiple Arrow RecordBatches
        let schema = Arc::new(Schema::new(vec![ArrowField::new(
            "id",
            DataType::Int64,
            false,
        )]));

        let batch1 =
            RecordBatch::try_new(schema.clone(), vec![Arc::new(Int64Array::from(vec![1, 2]))])
                .unwrap();

        let batch2 = RecordBatch::try_new(
            schema.clone(),
            vec![Arc::new(Int64Array::from(vec![3, 4, 5]))],
        )
        .unwrap();

        let mut reader = ArrowReader::new(vec![batch1, batch2]);

        // Set projection
        let projections = vec![Projection::new(
            ProjectionSource::ColumnIndex(0),
            0,
            LogicalType::Int64,
        )];
        let projection_spec = ProjectionSpec::new(projections).unwrap();
        reader.set_projection(projection_spec).unwrap();

        // Read first batch
        let batch1 = reader.next_batch().unwrap().unwrap();
        assert_eq!(batch1.row_count(), 2);

        // Read second batch
        let batch2 = reader.next_batch().unwrap().unwrap();
        assert_eq!(batch2.row_count(), 3);

        // No more batches
        assert!(reader.next_batch().unwrap().is_none());
    }
}
