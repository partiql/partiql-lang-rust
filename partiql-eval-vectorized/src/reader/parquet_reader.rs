use crate::batch::{Field, LogicalType, SourceTypeDef, VectorizedBatch};
use crate::error::EvalError;
use crate::reader::error::*;
use crate::reader::{BatchReader, ProjectionSource, ProjectionSpec};
use arrow::array::{Array, BooleanArray, Float64Array, Int64Array, StringArray};
use arrow::record_batch::RecordBatch;
use parquet::arrow::arrow_reader::ParquetRecordBatchReaderBuilder;
use parquet::arrow::ProjectionMask;
use std::fs::File;
use std::path::Path;

/// ParquetReader implements BatchReader for Parquet files
///
/// Key characteristics:
/// - Uses ColumnIndex projections for direct columnar access
/// - Reads Parquet files with row group processing for memory efficiency
/// - Converts Parquet data types to PartiQL LogicalTypes with validation
/// - Handles file I/O and decompression transparently
///
/// Phase 0 Compliance:
/// - Accepts only ColumnIndex projections (rejects FieldPath)
/// - Populates vectors according to ProjectionSpec
/// - Errors on unsupported data types or missing columns
/// - Maintains consistent batch sizes across all vectors
pub struct ParquetReader {
    /// Arrow-based Parquet reader for efficient columnar access
    record_batch_reader:
        Option<Box<dyn Iterator<Item = Result<RecordBatch, arrow::error::ArrowError>> + Send>>,
    /// Current projection specification
    projection_spec: Option<ProjectionSpec>,
    /// File path for error reporting
    file_path: String,
    /// Batch size for reading
    batch_size: usize,
    /// Cached schema built from projection (reused across batches)
    cached_schema: Option<SourceTypeDef>,
    /// Cached column indices (reused in initialize_reader)
    column_indices: Vec<usize>,
    /// Reusable batch structure (pre-allocated in set_projection)
    reusable_batch: Option<VectorizedBatch>,
}

impl ParquetReader {
    /// Create a new ParquetReader from a file path
    pub fn from_file<P: AsRef<Path>>(file_path: P, batch_size: usize) -> Result<Self, EvalError> {
        let path_str = file_path.as_ref().to_string_lossy().to_string();

        // Validate file exists and is readable
        if !file_path.as_ref().exists() {
            return Err(
                BatchReaderError::data_source(DataSourceError::access_failed(
                    &path_str,
                    "File does not exist",
                ))
                .into(),
            );
        }

        Ok(ParquetReader {
            record_batch_reader: None,
            projection_spec: None,
            file_path: path_str,
            batch_size,
            cached_schema: None,
            column_indices: Vec::new(),
            reusable_batch: None,
        })
    }

    /// Initialize the Parquet reader with column projections
    fn initialize_reader(&mut self) -> Result<(), EvalError> {
        if self.record_batch_reader.is_some() {
            return Ok(()); // Already initialized
        }

        let projection_spec = self.projection_spec.as_ref().ok_or_else(|| {
            EvalError::General("set_projection must be called before next_batch".to_string())
        })?;

        // Open the Parquet file
        let file = File::open(&self.file_path).map_err(|e| {
            BatchReaderError::data_source(DataSourceError::access_failed(
                &self.file_path,
                &e.to_string(),
            ))
        })?;

        // Extract column indices from projections
        let mut column_indices = Vec::new();
        for projection in &projection_spec.projections {
            match &projection.source {
                ProjectionSource::ColumnIndex(idx) => {
                    column_indices.push(*idx);
                }
                ProjectionSource::FieldPath(_) => {
                    return Err(BatchReaderError::projection(
                        ProjectionError::unsupported_source(
                            "FieldPath",
                            "ParquetReader",
                            &["ColumnIndex"]
                        )
                    ).with_context(
                        "Parquet files use columnar access. Use ColumnIndex instead of FieldPath".to_string()
                    ).into());
                }
            }
        }

        // Create Arrow-based reader with column projection
        let builder = ParquetRecordBatchReaderBuilder::try_new(file).map_err(|e| {
            BatchReaderError::data_source(DataSourceError::initialization_failed(
                "Parquet",
                &format!("Failed to create reader: {}", e),
            ))
        })?;

        // Column bounds were already validated in set_projection, so we can proceed
        let parquet_schema = builder.parquet_schema();

        // Create projection mask for column selection
        let projection_mask = ProjectionMask::roots(parquet_schema, column_indices.iter().cloned());

        let reader = builder
            .with_batch_size(self.batch_size)
            .with_projection(projection_mask)
            .build()
            .map_err(|e| {
                BatchReaderError::data_source(DataSourceError::initialization_failed(
                    "Parquet",
                    &format!("Failed to build reader: {}", e),
                ))
            })?;

        self.record_batch_reader = Some(Box::new(reader));
        Ok(())
    }

    /// Convert Arrow RecordBatch to VectorizedBatch
    fn convert_record_batch(
        &mut self,
        record_batch: RecordBatch,
    ) -> Result<VectorizedBatch, EvalError> {
        let projection_spec = self.projection_spec.as_ref().unwrap();
        let batch_size = record_batch.num_rows();

        // Use cached schema (built in set_projection) - verify it exists but don't use it directly
        let _schema = self.cached_schema.as_ref().ok_or_else(|| {
            EvalError::General("Schema not cached. set_projection must be called first.".to_string())
        })?;

        // Get or create reusable batch
        let batch = self.reusable_batch.as_mut().ok_or_else(|| {
            EvalError::General("Reusable batch should have been initialized in set_projection".to_string())
        })?;

        // Reset batch metadata (don't clear vectors - they maintain capacity and we'll overwrite data)
        batch.set_row_count(0);
        batch.set_selection(None);

        // Handle variable batch size: if record_batch is smaller than pre-allocated size, that's fine
        // If larger, we need to handle it (for now, we'll use the actual size)
        let actual_batch_size = std::cmp::min(batch_size, self.batch_size);

        // Process each projection
        for (proj_idx, projection) in projection_spec.projections.iter().enumerate() {
            let col_idx = match &projection.source {
                ProjectionSource::ColumnIndex(_) => {
                    // After projection, columns are reordered to match projection order
                    // So we use the projection index, not the original column index
                    proj_idx
                }
                _ => unreachable!("FieldPath should have been rejected earlier"),
            };

            // Get the Arrow array for this column from the projected batch
            let arrow_array = record_batch.column(col_idx);

            // Get the target vector from the batch
            let vector = batch.column_mut(proj_idx)?;

            // Convert Arrow array to PartiQL vector based on logical type
            match projection.logical_type {
                LogicalType::Int64 => {
                    if let crate::batch::PhysicalVectorEnum::Int64(v) = &mut vector.physical {
                        Self::convert_arrow_to_int64(arrow_array, v.as_mut_slice())?;
                    }
                }
                LogicalType::Float64 => {
                    if let crate::batch::PhysicalVectorEnum::Float64(v) = &mut vector.physical {
                        Self::convert_arrow_to_float64(arrow_array, v.as_mut_slice())?;
                    }
                }
                LogicalType::Boolean => {
                    if let crate::batch::PhysicalVectorEnum::Boolean(v) = &mut vector.physical {
                        Self::convert_arrow_to_boolean(arrow_array, v.as_mut_slice())?;
                    }
                }
                LogicalType::String => {
                    if let crate::batch::PhysicalVectorEnum::String(v) = &mut vector.physical {
                        Self::convert_arrow_to_string(arrow_array, v.as_mut_slice())?;
                    }
                }
            }
        }

        batch.set_row_count(actual_batch_size);
        
        // Clone the batch to return (the reusable batch stays in the reader for next iteration)
        Ok(batch.clone())
    }

    /// Convert Arrow array to Int64 slice
    fn convert_arrow_to_int64(
        array: &dyn Array,
        target: &mut [i64],
    ) -> Result<(), EvalError> {
        let array_len = array.len();
        // Verify capacity in debug builds
        #[cfg(debug_assertions)]
        debug_assert!(
            target.len() >= array_len,
            "Int64 vector buffer too small: expected {}, got {}",
            array_len,
            target.len()
        );
        
        match array.data_type() {
            arrow::datatypes::DataType::Int64 => {
                let int_array = array.as_any().downcast_ref::<Int64Array>().ok_or_else(|| {
                    BatchReaderError::type_conversion(TypeConversionError::conversion_failed(
                        "Arrow array",
                        "downcast",
                        LogicalType::Int64,
                        "Failed to downcast to Int64Array",
                    ))
                })?;

                for (i, value) in int_array.iter().enumerate() {
                    if i >= target.len() {
                        break;
                    }
                    target[i] = value.unwrap_or(0);
                }
            }
            arrow::datatypes::DataType::Float64 => {
                // Convert Float64 to Int64 (truncation)
                let float_array =
                    array
                        .as_any()
                        .downcast_ref::<Float64Array>()
                        .ok_or_else(|| {
                            BatchReaderError::type_conversion(
                                TypeConversionError::conversion_failed(
                                    "Arrow Float64Array",
                                    "downcast",
                                    LogicalType::Int64,
                                    "Failed to downcast to Float64Array",
                                ),
                            )
                        })?;

                for (i, value) in float_array.iter().enumerate() {
                    if i >= target.len() {
                        break;
                    }
                    target[i] = value
                        .map(|v| if v.is_finite() { v as i64 } else { 0 })
                        .unwrap_or(0);
                }
            }
            _ => {
                return Err(
                    BatchReaderError::type_conversion(TypeConversionError::type_mismatch(
                        "Arrow array",
                        &format!("{:?}", array.data_type()),
                        LogicalType::Int64,
                        Some("Use explicit conversion or cast to Int64"),
                    ))
                    .into(),
                )
            }
        }
        Ok(())
    }

    /// Convert Arrow array to Float64 slice
    fn convert_arrow_to_float64(
        array: &dyn Array,
        target: &mut [f64],
    ) -> Result<(), EvalError> {
        let array_len = array.len();
        // Verify capacity in debug builds
        #[cfg(debug_assertions)]
        debug_assert!(
            target.len() >= array_len,
            "Float64 vector buffer too small: expected {}, got {}",
            array_len,
            target.len()
        );
        
        match array.data_type() {
            arrow::datatypes::DataType::Float64 => {
                let float_array =
                    array
                        .as_any()
                        .downcast_ref::<Float64Array>()
                        .ok_or_else(|| {
                            BatchReaderError::type_conversion(
                                TypeConversionError::conversion_failed(
                                    "Arrow array",
                                    "downcast",
                                    LogicalType::Float64,
                                    "Failed to downcast to Float64Array",
                                ),
                            )
                        })?;

                for (i, value) in float_array.iter().enumerate() {
                    if i >= target.len() {
                        break;
                    }
                    target[i] = value.unwrap_or(0.0);
                }
            }
            arrow::datatypes::DataType::Int64 => {
                // Convert Int64 to Float64
                let int_array = array.as_any().downcast_ref::<Int64Array>().ok_or_else(|| {
                    BatchReaderError::type_conversion(TypeConversionError::conversion_failed(
                        "Arrow Int64Array",
                        "downcast",
                        LogicalType::Float64,
                        "Failed to downcast to Int64Array",
                    ))
                })?;

                for (i, value) in int_array.iter().enumerate() {
                    if i >= target.len() {
                        break;
                    }
                    target[i] = value.map(|v| v as f64).unwrap_or(0.0);
                }
            }
            _ => {
                return Err(
                    BatchReaderError::type_conversion(TypeConversionError::type_mismatch(
                        "Arrow array",
                        &format!("{:?}", array.data_type()),
                        LogicalType::Float64,
                        Some("Use explicit conversion or cast to Float64"),
                    ))
                    .into(),
                )
            }
        }
        Ok(())
    }

    /// Convert Arrow array to Boolean slice
    fn convert_arrow_to_boolean(
        array: &dyn Array,
        target: &mut [bool],
    ) -> Result<(), EvalError> {
        let array_len = array.len();
        // Verify capacity in debug builds
        #[cfg(debug_assertions)]
        debug_assert!(
            target.len() >= array_len,
            "Boolean vector buffer too small: expected {}, got {}",
            array_len,
            target.len()
        );
        
        match array.data_type() {
            arrow::datatypes::DataType::Boolean => {
                let bool_array =
                    array
                        .as_any()
                        .downcast_ref::<BooleanArray>()
                        .ok_or_else(|| {
                            BatchReaderError::type_conversion(
                                TypeConversionError::conversion_failed(
                                    "Arrow array",
                                    "downcast",
                                    LogicalType::Boolean,
                                    "Failed to downcast to BooleanArray",
                                ),
                            )
                        })?;

                for (i, value) in bool_array.iter().enumerate() {
                    if i >= target.len() {
                        break;
                    }
                    target[i] = value.unwrap_or(false);
                }
            }
            _ => {
                return Err(
                    BatchReaderError::type_conversion(TypeConversionError::type_mismatch(
                        "Arrow array",
                        &format!("{:?}", array.data_type()),
                        LogicalType::Boolean,
                        Some("Use explicit conversion or cast to Boolean"),
                    ))
                    .into(),
                )
            }
        }
        Ok(())
    }

    /// Convert Arrow array to String slice
    fn convert_arrow_to_string(
        array: &dyn Array,
        target: &mut [String],
    ) -> Result<(), EvalError> {
        let array_len = array.len();
        // Verify capacity in debug builds
        #[cfg(debug_assertions)]
        debug_assert!(
            target.len() >= array_len,
            "String vector buffer too small: expected {}, got {}",
            array_len,
            target.len()
        );
        
        match array.data_type() {
            arrow::datatypes::DataType::Utf8 => {
                let string_array =
                    array
                        .as_any()
                        .downcast_ref::<StringArray>()
                        .ok_or_else(|| {
                            BatchReaderError::type_conversion(
                                TypeConversionError::conversion_failed(
                                    "Arrow array",
                                    "downcast",
                                    LogicalType::String,
                                    "Failed to downcast to StringArray",
                                ),
                            )
                        })?;

                for (i, value) in string_array.iter().enumerate() {
                    if i >= target.len() {
                        break;
                    }
                    target[i] = value.map(|s| s.to_string()).unwrap_or_default();
                }
            }
            arrow::datatypes::DataType::Int64 => {
                // Convert Int64 to String
                let int_array = array.as_any().downcast_ref::<Int64Array>().ok_or_else(|| {
                    BatchReaderError::type_conversion(TypeConversionError::conversion_failed(
                        "Arrow Int64Array",
                        "downcast",
                        LogicalType::String,
                        "Failed to downcast to Int64Array",
                    ))
                })?;

                for (i, value) in int_array.iter().enumerate() {
                    if i >= target.len() {
                        break;
                    }
                    target[i] = value.map(|v| v.to_string()).unwrap_or_default();
                }
            }
            arrow::datatypes::DataType::Float64 => {
                // Convert Float64 to String
                let float_array =
                    array
                        .as_any()
                        .downcast_ref::<Float64Array>()
                        .ok_or_else(|| {
                            BatchReaderError::type_conversion(
                                TypeConversionError::conversion_failed(
                                    "Arrow Float64Array",
                                    "downcast",
                                    LogicalType::String,
                                    "Failed to downcast to Float64Array",
                                ),
                            )
                        })?;

                for (i, value) in float_array.iter().enumerate() {
                    if i >= target.len() {
                        break;
                    }
                    target[i] = value.map(|v| v.to_string()).unwrap_or_default();
                }
            }
            arrow::datatypes::DataType::Boolean => {
                // Convert Boolean to String
                let bool_array =
                    array
                        .as_any()
                        .downcast_ref::<BooleanArray>()
                        .ok_or_else(|| {
                            BatchReaderError::type_conversion(
                                TypeConversionError::conversion_failed(
                                    "Arrow BooleanArray",
                                    "downcast",
                                    LogicalType::String,
                                    "Failed to downcast to BooleanArray",
                                ),
                            )
                        })?;

                for (i, value) in bool_array.iter().enumerate() {
                    if i >= target.len() {
                        break;
                    }
                    target[i] = value.map(|v| v.to_string()).unwrap_or_default();
                }
            }
            _ => {
                return Err(
                    BatchReaderError::type_conversion(TypeConversionError::type_mismatch(
                        "Arrow array",
                        &format!("{:?}", array.data_type()),
                        LogicalType::String,
                        Some("Most types can be converted to String"),
                    ))
                    .into(),
                )
            }
        }
        Ok(())
    }
}

impl BatchReader for ParquetReader {
    fn open(&mut self) -> Result<(), EvalError> {
        // No-op for ParquetReader - initialization happens in initialize_reader
        Ok(())
    }

    // TODO: Keep the schema of the file outside so I don't need to open the file every time.
    fn resolve(&self, field_name: &str) -> Option<ProjectionSource> {
        // For Parquet reader, resolve field names to column indices
        // We need to open the file temporarily to read the schema
        let file = match File::open(&self.file_path) {
            Ok(f) => f,
            Err(_) => return None,
        };

        let builder = match ParquetRecordBatchReaderBuilder::try_new(file) {
            Ok(b) => b,
            Err(_) => return None,
        };

        let arrow_schema = builder.schema();
        
        for (idx, field) in arrow_schema.fields().iter().enumerate() {
            if field.name() == field_name {
                return Some(ProjectionSource::ColumnIndex(idx));
            }
        }

        None
    }

    fn set_projection(&mut self, spec: ProjectionSpec) -> Result<(), EvalError> {
        // Validate projection sources - only ColumnIndex allowed
        for projection in &spec.projections {
            match &projection.source {
                ProjectionSource::ColumnIndex(_) => {
                    // Valid for Parquet (columnar data)
                }
                ProjectionSource::FieldPath(path) => {
                    return Err(BatchReaderError::projection(
                        ProjectionError::unsupported_source(
                            "FieldPath",
                            "ParquetReader",
                            &["ColumnIndex"]
                        )
                    ).with_context(format!(
                        "Parquet files use columnar access. Use ColumnIndex instead of FieldPath '{}'",
                        path
                    )).into());
                }
            }
        }

        // Validate column indices against Parquet schema (if we can read it)
        // We need to open the file temporarily to get the schema for validation
        if let Ok(file) = File::open(&self.file_path) {
            if let Ok(builder) = ParquetRecordBatchReaderBuilder::try_new(file) {
                let parquet_schema = builder.parquet_schema();
                let num_columns = parquet_schema.columns().len();

                for projection in &spec.projections {
                    if let ProjectionSource::ColumnIndex(col_idx) = &projection.source {
                        if *col_idx >= num_columns {
                            return Err(EvalError::General(format!(
                                "Column index {} is out of bounds. Parquet schema has {} columns.",
                                col_idx, num_columns
                            )));
                        }
                    }
                }
            }
        }

        // Build and cache schema from projection
        let fields: Vec<Field> = spec
            .projections
            .iter()
            .map(|p| Field {
                name: match &p.source {
                    ProjectionSource::ColumnIndex(idx) => format!("col_{}", idx),
                    ProjectionSource::FieldPath(_) => unreachable!("FieldPath should have been rejected earlier"),
                },
                type_info: p.logical_type,
            })
            .collect();
        let schema = SourceTypeDef::new(fields);
        self.cached_schema = Some(schema);

        // Cache column indices for reuse in initialize_reader
        let mut column_indices = Vec::new();
        for projection in &spec.projections {
            if let ProjectionSource::ColumnIndex(idx) = &projection.source {
                column_indices.push(*idx);
            }
        }
        self.column_indices = column_indices;

        // Pre-allocate reusable batch structure
        self.reusable_batch = Some(VectorizedBatch::new(
            self.cached_schema.as_ref().unwrap().clone(),
            self.batch_size,
        ));

        self.projection_spec = Some(spec);
        Ok(())
    }

    fn next_batch(&mut self) -> Result<Option<VectorizedBatch>, EvalError> {
        // Initialize reader if not already done
        self.initialize_reader()?;

        let reader = self.record_batch_reader.as_mut().unwrap();

        match reader.next() {
            Some(Ok(record_batch)) => {
                let batch = self.convert_record_batch(record_batch)?;
                Ok(Some(batch))
            }
            Some(Err(e)) => Err(
                BatchReaderError::data_source(DataSourceError::corrupted_data(
                    &self.file_path,
                    "record batch",
                    &format!("Failed to read Parquet data: {}", e),
                ))
                .into(),
            ),
            None => Ok(None), // End of data
        }
    }

    fn close(&mut self) -> Result<(), EvalError> {
        // No-op for ParquetReader
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::reader::Projection;
    use arrow::array::{Float64Array, Int64Array, StringArray};
    use arrow::datatypes::{DataType, Field, Schema};
    use arrow::record_batch::RecordBatch;
    use parquet::arrow::ArrowWriter;
    use parquet::file::properties::WriterProperties;
    use std::sync::Arc;
    use tempfile::NamedTempFile;

    fn create_test_parquet_file() -> Result<NamedTempFile, Box<dyn std::error::Error>> {
        // Create test data
        let schema = Arc::new(Schema::new(vec![
            Field::new("id", DataType::Int64, false),
            Field::new("name", DataType::Utf8, false),
            Field::new("score", DataType::Float64, false),
        ]));

        let id_array = Arc::new(Int64Array::from(vec![1, 2, 3, 4, 5]));
        let name_array = Arc::new(StringArray::from(vec![
            "Alice", "Bob", "Charlie", "Diana", "Eve",
        ]));
        let score_array = Arc::new(Float64Array::from(vec![95.5, 87.2, 92.8, 88.1, 94.3]));

        let record_batch =
            RecordBatch::try_new(schema.clone(), vec![id_array, name_array, score_array])?;

        // Create temporary file
        let temp_file = NamedTempFile::new()?;
        let file = File::create(temp_file.path())?;

        // Write Parquet data
        let props = WriterProperties::builder().build();
        let mut writer = ArrowWriter::try_new(file, schema, Some(props))?;
        writer.write(&record_batch)?;
        writer.close()?;

        Ok(temp_file)
    }

    #[test]
    fn test_parquet_reader_basic_functionality() {
        let temp_file = create_test_parquet_file().unwrap();
        let mut reader = ParquetReader::from_file(temp_file.path(), 10).unwrap();

        // Set projection
        let projections = vec![
            Projection::new(ProjectionSource::ColumnIndex(0), 0, LogicalType::Int64),
            Projection::new(ProjectionSource::ColumnIndex(1), 1, LogicalType::String),
            Projection::new(ProjectionSource::ColumnIndex(2), 2, LogicalType::Float64),
        ];
        let projection_spec = ProjectionSpec::new(projections).unwrap();
        reader.set_projection(projection_spec).unwrap();

        // Read batch
        let batch = reader.next_batch().unwrap();
        assert!(batch.is_some());

        let batch = batch.unwrap();
        assert_eq!(batch.row_count(), 5);
        assert_eq!(batch.total_column_count(), 3);

        // Should be no more batches
        let next_batch = reader.next_batch().unwrap();
        assert!(next_batch.is_none());
    }

    #[test]
    fn test_parquet_reader_column_projection() {
        let temp_file = create_test_parquet_file().unwrap();
        let mut reader = ParquetReader::from_file(temp_file.path(), 10).unwrap();

        // Project only columns 0 and 2 (skip column 1)
        let projections = vec![
            Projection::new(ProjectionSource::ColumnIndex(0), 0, LogicalType::Int64),
            Projection::new(ProjectionSource::ColumnIndex(2), 1, LogicalType::Float64),
        ];
        let projection_spec = ProjectionSpec::new(projections).unwrap();
        reader.set_projection(projection_spec).unwrap();

        // Read batch
        let batch = reader.next_batch().unwrap();
        assert!(batch.is_some());

        let batch = batch.unwrap();
        assert_eq!(batch.row_count(), 5);
        assert_eq!(batch.total_column_count(), 2); // Only 2 columns projected
    }

    #[test]
    fn test_parquet_reader_field_path_rejection() {
        let temp_file = create_test_parquet_file().unwrap();
        let mut reader = ParquetReader::from_file(temp_file.path(), 10).unwrap();

        // Try to set projection with FieldPath - should fail
        let projections = vec![Projection::new(
            ProjectionSource::FieldPath("name".to_string()),
            0,
            LogicalType::String,
        )];
        let projection_spec = ProjectionSpec::new(projections).unwrap();

        let result = reader.set_projection(projection_spec);
        assert!(result.is_err());
        let error_msg = result.unwrap_err().to_string();
        assert!(error_msg.contains("FieldPath"));
        assert!(error_msg.contains("not supported"));
        assert!(error_msg.contains("ColumnIndex"));
    }

    #[test]
    fn test_parquet_reader_column_bounds_validation() {
        let temp_file = create_test_parquet_file().unwrap();
        let mut reader = ParquetReader::from_file(temp_file.path(), 10).unwrap();

        // Try to access column index 5 when only columns 0, 1, 2 exist
        let projections = vec![Projection::new(
            ProjectionSource::ColumnIndex(5),
            0,
            LogicalType::Int64,
        )];
        let projection_spec = ProjectionSpec::new(projections).unwrap();

        // Error should occur during set_projection now (not next_batch)
        let result = reader.set_projection(projection_spec);
        assert!(result.is_err());

        let error_msg = result.unwrap_err().to_string();
        assert!(error_msg.contains("out of bounds"));
        assert!(error_msg.contains("Column index 5"));
        assert!(error_msg.contains("3 columns"));
    }

    #[test]
    fn test_parquet_reader_file_not_found() {
        let result = ParquetReader::from_file("/nonexistent/file.parquet", 10);
        assert!(result.is_err());
        if let Err(error) = result {
            let error_msg = format!("{}", error);
            assert!(error_msg.contains("File does not exist"));
        }
    }

    #[test]
    fn test_parquet_reader_type_conversions() {
        let temp_file = create_test_parquet_file().unwrap();
        let mut reader = ParquetReader::from_file(temp_file.path(), 10).unwrap();

        // Test type conversions: Int64 to Float64, Float64 to String
        let projections = vec![
            Projection::new(ProjectionSource::ColumnIndex(0), 0, LogicalType::Float64), // Int64 -> Float64
            Projection::new(ProjectionSource::ColumnIndex(2), 1, LogicalType::String), // Float64 -> String
        ];
        let projection_spec = ProjectionSpec::new(projections).unwrap();
        reader.set_projection(projection_spec).unwrap();

        // Read batch
        let batch = reader.next_batch().unwrap();
        assert!(batch.is_some());

        let batch = batch.unwrap();
        assert_eq!(batch.row_count(), 5);
        assert_eq!(batch.total_column_count(), 2);
    }
}
