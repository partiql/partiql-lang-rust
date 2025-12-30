use partiql_eval_vectorized::batch::LogicalType;
use partiql_eval_vectorized::reader::{BatchReader, Projection, ProjectionSource, ProjectionSpec, ParquetReader};
use std::fs::File;
use std::sync::Arc;
use arrow::array::{Int64Array, StringArray, Float64Array, BooleanArray};
use arrow::datatypes::{DataType, Field, Schema};
use arrow::record_batch::RecordBatch;
use parquet::arrow::ArrowWriter;
use parquet::file::properties::WriterProperties;
use tempfile::NamedTempFile;

/// Create a test Parquet file with sample data for property testing
fn create_test_parquet_file_with_data(
    int_data: Vec<i64>,
    string_data: Vec<String>,
    float_data: Vec<f64>,
    bool_data: Vec<bool>,
) -> Result<NamedTempFile, Box<dyn std::error::Error>> {
    let len = int_data.len();
    assert_eq!(string_data.len(), len);
    assert_eq!(float_data.len(), len);
    assert_eq!(bool_data.len(), len);

    // Create test schema
    let schema = Arc::new(Schema::new(vec![
        Field::new("id", DataType::Int64, false),
        Field::new("name", DataType::Utf8, false),
        Field::new("score", DataType::Float64, false),
        Field::new("active", DataType::Boolean, false),
    ]));

    // Create test data
    let id_array = Arc::new(Int64Array::from(int_data));
    let name_array = Arc::new(StringArray::from(string_data));
    let score_array = Arc::new(Float64Array::from(float_data));
    let active_array = Arc::new(BooleanArray::from(bool_data));

    let record_batch = RecordBatch::try_new(
        schema.clone(),
        vec![id_array, name_array, score_array, active_array],
    )?;

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

#[cfg(test)]
mod unit_tests {
    use super::*;

    #[test]
    fn test_parquet_conversion_basic_types() -> Result<(), Box<dyn std::error::Error>> {
        let temp_file = create_test_parquet_file_with_data(
            vec![1, 2, 3],
            vec!["Alice".to_string(), "Bob".to_string(), "Charlie".to_string()],
            vec![95.5, 87.2, 92.8],
            vec![true, false, true],
        )?;

        let mut reader = ParquetReader::from_file(temp_file.path(), 10)?;

        let projections = vec![
            Projection::new(ProjectionSource::ColumnIndex(0), 0, LogicalType::Int64),
            Projection::new(ProjectionSource::ColumnIndex(1), 1, LogicalType::String),
            Projection::new(ProjectionSource::ColumnIndex(2), 2, LogicalType::Float64),
            Projection::new(ProjectionSource::ColumnIndex(3), 3, LogicalType::Boolean),
        ];

        let projection_spec = ProjectionSpec::new(projections)?;
        reader.set_projection(projection_spec)?;

        let batch = reader.next_batch()?.expect("Should have batch");
        assert_eq!(batch.row_count(), 3);
        assert_eq!(batch.total_column_count(), 4);

        Ok(())
    }

    #[test]
    fn test_parquet_conversion_int_to_float() -> Result<(), Box<dyn std::error::Error>> {
        let temp_file = create_test_parquet_file_with_data(
            vec![42, 100, -5],
            vec!["A".to_string(), "B".to_string(), "C".to_string()],
            vec![1.0, 2.0, 3.0],
            vec![true, false, true],
        )?;

        let mut reader = ParquetReader::from_file(temp_file.path(), 10)?;

        let projections = vec![
            Projection::new(ProjectionSource::ColumnIndex(0), 0, LogicalType::Float64), // Int64 -> Float64
        ];

        let projection_spec = ProjectionSpec::new(projections)?;
        reader.set_projection(projection_spec)?;

        let batch = reader.next_batch()?.expect("Should have batch");
        assert_eq!(batch.row_count(), 3);
        assert_eq!(batch.total_column_count(), 1);

        Ok(())
    }

    #[test]
    fn test_parquet_conversion_float_to_string() -> Result<(), Box<dyn std::error::Error>> {
        let temp_file = create_test_parquet_file_with_data(
            vec![1, 2, 3],
            vec!["A".to_string(), "B".to_string(), "C".to_string()],
            vec![3.14, 2.71, 1.41],
            vec![true, false, true],
        )?;

        let mut reader = ParquetReader::from_file(temp_file.path(), 10)?;

        let projections = vec![
            Projection::new(ProjectionSource::ColumnIndex(2), 0, LogicalType::String), // Float64 -> String
        ];

        let projection_spec = ProjectionSpec::new(projections)?;
        reader.set_projection(projection_spec)?;

        let batch = reader.next_batch()?.expect("Should have batch");
        assert_eq!(batch.row_count(), 3);
        assert_eq!(batch.total_column_count(), 1);

        Ok(())
    }

    #[test]
    fn test_parquet_field_path_rejection() -> Result<(), Box<dyn std::error::Error>> {
        let temp_file = create_test_parquet_file_with_data(
            vec![1],
            vec!["test".to_string()],
            vec![1.0],
            vec![true],
        )?;

        let mut reader = ParquetReader::from_file(temp_file.path(), 10)?;

        let projections = vec![
            Projection::new(ProjectionSource::FieldPath("name".to_string()), 0, LogicalType::String),
        ];

        let projection_spec = ProjectionSpec::new(projections)?;
        let result = reader.set_projection(projection_spec);

        assert!(result.is_err());
        let error_msg = format!("{}", result.unwrap_err());
        assert!(error_msg.contains("FieldPath"));
        assert!(error_msg.contains("not supported"));

        Ok(())
    }

    #[test]
    fn test_parquet_column_bounds_validation() -> Result<(), Box<dyn std::error::Error>> {
        let temp_file = create_test_parquet_file_with_data(
            vec![1],
            vec!["test".to_string()],
            vec![1.0],
            vec![true],
        )?;

        let mut reader = ParquetReader::from_file(temp_file.path(), 10)?;

        // Try to access column 5 when only 4 columns exist (0-3)
        let projections = vec![
            Projection::new(ProjectionSource::ColumnIndex(5), 0, LogicalType::Int64),
        ];

        let projection_spec = ProjectionSpec::new(projections)?;
        
        // Error should occur during set_projection now (bounds checking moved there)
        let result = reader.set_projection(projection_spec);
        assert!(result.is_err(), "Expected column bounds error during set_projection");
        
        let error_msg = result.unwrap_err().to_string();
        assert!(error_msg.contains("out of bounds"), "Error should mention 'out of bounds'");
        assert!(error_msg.contains("Column index 5"), "Error should mention the invalid column index");
        assert!(error_msg.contains("4 columns"), "Error should mention the actual number of columns");

        Ok(())
    }
}

/// Property test for Parquet data conversion integrity
/// Validates that Parquet column data converts correctly to PartiQL vectors
#[test]
fn test_parquet_data_conversion_integrity() -> Result<(), Box<dyn std::error::Error>> {
    // Test with various data patterns
    let test_cases = vec![
        // Small dataset
        (vec![1, 2, 3], vec!["a".to_string(), "b".to_string(), "c".to_string()]),
        // Larger dataset
        (
            (0..50).collect::<Vec<i64>>(),
            (0..50).map(|i| format!("item_{}", i)).collect::<Vec<String>>()
        ),
        // Edge cases
        (vec![i64::MIN, 0, i64::MAX], vec!["min".to_string(), "zero".to_string(), "max".to_string()]),
    ];

    for (int_data, string_data) in test_cases {
        let len = int_data.len();
        let float_data: Vec<f64> = int_data.iter().map(|&i| i as f64 + 0.5).collect();
        let bool_data: Vec<bool> = int_data.iter().map(|&i| i % 2 == 0).collect();

        let temp_file = create_test_parquet_file_with_data(
            int_data.clone(),
            string_data.clone(),
            float_data.clone(),
            bool_data.clone(),
        )?;

        let mut reader = ParquetReader::from_file(temp_file.path(), 10)?;

        let projections = vec![
            Projection::new(ProjectionSource::ColumnIndex(0), 0, LogicalType::Int64),
            Projection::new(ProjectionSource::ColumnIndex(1), 1, LogicalType::String),
            Projection::new(ProjectionSource::ColumnIndex(2), 2, LogicalType::Float64),
            Projection::new(ProjectionSource::ColumnIndex(3), 3, LogicalType::Boolean),
        ];

        let projection_spec = ProjectionSpec::new(projections)?;
        reader.set_projection(projection_spec)?;

        let mut total_rows = 0;
        while let Some(batch) = reader.next_batch()? {
            total_rows += batch.row_count();
            assert_eq!(batch.total_column_count(), 4);
            assert!(batch.row_count() > 0);
        }

        assert_eq!(total_rows, len);
    }

    Ok(())
}

/// Property test for Parquet column index support
/// Validates that ColumnIndex projections work correctly for Parquet data
#[test]
fn test_parquet_column_index_support() -> Result<(), Box<dyn std::error::Error>> {
    let temp_file = create_test_parquet_file_with_data(
        vec![1, 2, 3, 4, 5],
        vec!["a".to_string(), "b".to_string(), "c".to_string(), "d".to_string(), "e".to_string()],
        vec![1.1, 2.2, 3.3, 4.4, 5.5],
        vec![true, false, true, false, true],
    )?;

    // Test various column index combinations
    let test_cases = vec![
        // Single column
        vec![0],
        // Multiple contiguous columns
        vec![0, 1, 2],
        // Non-contiguous columns
        vec![0, 2],
        vec![1, 3],
        // All columns
        vec![0, 1, 2, 3],
    ];

    for column_indices in test_cases {
        let mut reader = ParquetReader::from_file(temp_file.path(), 10)?;

        let projections: Vec<Projection> = column_indices.iter().enumerate()
            .map(|(target_idx, &col_idx)| {
                let logical_type = match col_idx {
                    0 => LogicalType::Int64,
                    1 => LogicalType::String,
                    2 => LogicalType::Float64,
                    3 => LogicalType::Boolean,
                    _ => unreachable!(),
                };
                Projection::new(ProjectionSource::ColumnIndex(col_idx), target_idx, logical_type)
            })
            .collect();

        let projection_spec = ProjectionSpec::new(projections)?;
        reader.set_projection(projection_spec)?;

        let batch = reader.next_batch()?.expect("Should have batch");
        assert_eq!(batch.row_count(), 5);
        assert_eq!(batch.total_column_count(), column_indices.len());
    }

    Ok(())
}

/// Property test for Parquet file I/O efficiency
/// Validates that Parquet reading with column projection is efficient
#[test]
fn test_parquet_io_efficiency() -> Result<(), Box<dyn std::error::Error>> {
    // Create a larger dataset to test I/O efficiency
    let size = 1000;
    let int_data: Vec<i64> = (0..size).collect();
    let string_data: Vec<String> = (0..size).map(|i| format!("row_{:04}", i)).collect();
    let float_data: Vec<f64> = (0..size).map(|i| i as f64 * 1.5).collect();
    let bool_data: Vec<bool> = (0..size).map(|i| i % 3 == 0).collect();

    let temp_file = create_test_parquet_file_with_data(
        int_data,
        string_data,
        float_data,
        bool_data,
    )?;

    // Test reading with different batch sizes
    let batch_sizes = vec![10, 50, 100, 500];

    for batch_size in batch_sizes {
        let mut reader = ParquetReader::from_file(temp_file.path(), batch_size)?;

        // Project only 2 out of 4 columns for efficiency
        let projections = vec![
            Projection::new(ProjectionSource::ColumnIndex(0), 0, LogicalType::Int64),
            Projection::new(ProjectionSource::ColumnIndex(2), 1, LogicalType::Float64),
        ];

        let projection_spec = ProjectionSpec::new(projections)?;
        reader.set_projection(projection_spec)?;

        let mut total_rows = 0;
        let mut batch_count = 0;

        while let Some(batch) = reader.next_batch()? {
            batch_count += 1;
            total_rows += batch.row_count();
            assert_eq!(batch.total_column_count(), 2);
            assert!(batch.row_count() <= batch_size);
            assert!(batch.row_count() > 0);
        }

        assert_eq!(total_rows, size as usize);
        // Verify batching worked correctly
        let expected_batches = (size as usize + batch_size - 1) / batch_size; // Ceiling division
        assert_eq!(batch_count, expected_batches);
    }

    Ok(())
}