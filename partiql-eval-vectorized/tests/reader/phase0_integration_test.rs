use partiql_eval_vectorized::batch::LogicalType;
use partiql_eval_vectorized::reader::{
    BatchReader, ParquetReader, Projection, ProjectionSource, ProjectionSpec, TupleIteratorReader,
};

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_phase0_batch_reader_workflow() {
        // Create test tuples with data
        let tuples: Vec<partiql_value::Value> = vec![
            partiql_value::Value::Tuple(Box::new(partiql_value::Tuple::from([
                (
                    "name",
                    partiql_value::Value::String(Box::new("Alice".to_string())),
                ),
                ("age", partiql_value::Value::Integer(30)),
                ("score", partiql_value::Value::Real(95.5.into())),
            ]))),
            partiql_value::Value::Tuple(Box::new(partiql_value::Tuple::from([
                (
                    "name",
                    partiql_value::Value::String(Box::new("Bob".to_string())),
                ),
                ("age", partiql_value::Value::Integer(25)),
                ("score", partiql_value::Value::Real(87.2.into())),
            ]))),
        ];

        let mut reader = TupleIteratorReader::new(Box::new(tuples.into_iter()), 1024);

        // Create a Phase 0 projection specification
        let projections = vec![
            Projection::new(
                ProjectionSource::FieldPath("name".to_string()),
                0,
                LogicalType::String,
            ),
            Projection::new(
                ProjectionSource::FieldPath("age".to_string()),
                1,
                LogicalType::Int64,
            ),
            Projection::new(
                ProjectionSource::FieldPath("score".to_string()),
                2,
                LogicalType::Float64,
            ),
        ];

        let projection_spec = ProjectionSpec::new(projections).unwrap();

        // Phase 0 workflow: set_projection must be called before next_batch
        assert!(reader.set_projection(projection_spec).is_ok());

        // Now we can read batches
        let batch_result = reader.next_batch();
        assert!(batch_result.is_ok());

        let batch = batch_result.unwrap();
        assert!(batch.is_some());

        let batch = batch.unwrap();
        assert_eq!(batch.total_column_count(), 3); // 3 projections = 3 columns
        assert_eq!(batch.row_count(), 2); // 2 tuples we provided
    }

    #[test]
    fn test_phase0_projection_validation() {
        // Test that projection validation works correctly

        // Valid projection - contiguous indices starting from 0
        let valid_projections = vec![
            Projection::new(ProjectionSource::ColumnIndex(0), 0, LogicalType::Int64),
            Projection::new(ProjectionSource::ColumnIndex(1), 1, LogicalType::String),
        ];
        assert!(ProjectionSpec::new(valid_projections).is_ok());

        // Invalid projection - non-contiguous indices
        let invalid_projections = vec![
            Projection::new(ProjectionSource::ColumnIndex(0), 0, LogicalType::Int64),
            Projection::new(ProjectionSource::ColumnIndex(1), 2, LogicalType::String), // Skip 1
        ];
        assert!(ProjectionSpec::new(invalid_projections).is_err());

        // Invalid projection - duplicate indices
        let duplicate_projections = vec![
            Projection::new(ProjectionSource::ColumnIndex(0), 0, LogicalType::Int64),
            Projection::new(ProjectionSource::ColumnIndex(1), 0, LogicalType::String), // Duplicate 0
        ];
        assert!(ProjectionSpec::new(duplicate_projections).is_err());
    }

    #[test]
    fn test_phase0_reader_requires_projection() {
        // Test that calling next_batch without set_projection fails
        let tuples: Vec<partiql_value::Value> = vec![];
        let mut reader = TupleIteratorReader::new(Box::new(tuples.into_iter()), 1024);

        // Should fail because set_projection hasn't been called
        let result = reader.next_batch();
        assert!(result.is_err());
        assert!(result
            .unwrap_err()
            .to_string()
            .contains("set_projection must be called"));
    }

    #[test]
    fn test_phase0_projection_sources() {
        // Test both types of projection sources
        let projections = vec![
            // Column-based access (for columnar data like Arrow/Parquet)
            Projection::new(ProjectionSource::ColumnIndex(0), 0, LogicalType::Int64),
            Projection::new(ProjectionSource::ColumnIndex(2), 1, LogicalType::Float64),
            // Field-based access (for row/struct data like Ion)
            Projection::new(
                ProjectionSource::FieldPath("name".to_string()),
                2,
                LogicalType::String,
            ),
            Projection::new(
                ProjectionSource::FieldPath("active".to_string()),
                3,
                LogicalType::Boolean,
            ),
        ];

        let spec = ProjectionSpec::new(projections).unwrap();
        assert_eq!(spec.output_vector_count(), 4);

        // Verify the projection sources are preserved correctly
        assert!(matches!(
            spec.projections[0].source,
            ProjectionSource::ColumnIndex(0)
        ));
        assert!(matches!(
            spec.projections[1].source,
            ProjectionSource::ColumnIndex(2)
        ));
        assert!(
            matches!(spec.projections[2].source, ProjectionSource::FieldPath(ref path) if path == "name")
        );
        assert!(
            matches!(spec.projections[3].source, ProjectionSource::FieldPath(ref path) if path == "active")
        );
    }

    #[test]
    fn test_phase0_scalar_types_only() {
        // Test that all Phase 0 scalar types are supported
        let projections = vec![
            Projection::new(
                ProjectionSource::FieldPath("int_field".to_string()),
                0,
                LogicalType::Int64,
            ),
            Projection::new(
                ProjectionSource::FieldPath("float_field".to_string()),
                1,
                LogicalType::Float64,
            ),
            Projection::new(
                ProjectionSource::FieldPath("bool_field".to_string()),
                2,
                LogicalType::Boolean,
            ),
            Projection::new(
                ProjectionSource::FieldPath("string_field".to_string()),
                3,
                LogicalType::String,
            ),
        ];

        let spec = ProjectionSpec::new(projections).unwrap();
        assert_eq!(spec.output_vector_count(), 4);

        // Verify all scalar types are represented
        assert_eq!(spec.projections[0].logical_type, LogicalType::Int64);
        assert_eq!(spec.projections[1].logical_type, LogicalType::Float64);
        assert_eq!(spec.projections[2].logical_type, LogicalType::Boolean);
        assert_eq!(spec.projections[3].logical_type, LogicalType::String);
    }

    #[test]
    fn test_phase0_parquet_reader_workflow() {
        use arrow::array::{Float64Array, Int64Array, StringArray};
        use arrow::datatypes::{DataType, Field, Schema};
        use arrow::record_batch::RecordBatch;
        use parquet::arrow::ArrowWriter;
        use std::fs::File;
        use std::sync::Arc;
        use tempfile::tempdir;

        // Create a temporary Parquet file for testing
        let temp_dir = tempdir().unwrap();
        let file_path = temp_dir.path().join("test.parquet");

        // Create test data
        let schema = Arc::new(Schema::new(vec![
            Field::new("id", DataType::Int64, false),
            Field::new("score", DataType::Float64, false),
            Field::new("name", DataType::Utf8, false),
        ]));

        let batch = RecordBatch::try_new(
            schema.clone(),
            vec![
                Arc::new(Int64Array::from(vec![1, 2, 3])),
                Arc::new(Float64Array::from(vec![95.5, 87.2, 92.1])),
                Arc::new(StringArray::from(vec!["Alice", "Bob", "Charlie"])),
            ],
        )
        .unwrap();

        let file = File::create(&file_path).unwrap();
        let mut writer = ArrowWriter::try_new(file, schema, None).unwrap();
        writer.write(&batch).unwrap();
        writer.close().unwrap();

        // Test Phase 0 workflow with ParquetReader
        let mut reader = ParquetReader::from_file(file_path.to_str().unwrap(), 1024).unwrap();

        // Create a Phase 0 projection specification using ColumnIndex
        let projections = vec![
            Projection::new(
                ProjectionSource::ColumnIndex(0), // id column
                0,
                LogicalType::Int64,
            ),
            Projection::new(
                ProjectionSource::ColumnIndex(2), // name column (skip score)
                1,
                LogicalType::String,
            ),
        ];

        let projection_spec = ProjectionSpec::new(projections).unwrap();

        // Phase 0 workflow: set_projection must be called before next_batch
        assert!(reader.set_projection(projection_spec).is_ok());

        // Now we can read batches
        let batch_result = reader.next_batch();
        assert!(batch_result.is_ok());

        let batch = batch_result.unwrap();
        assert!(batch.is_some());

        let batch = batch.unwrap();
        assert_eq!(batch.total_column_count(), 2); // 2 projections = 2 columns
        assert_eq!(batch.row_count(), 3); // 3 rows in our test data
    }

    #[test]
    fn test_phase0_parquet_reader_requires_projection() {
        use arrow::array::Int64Array;
        use arrow::datatypes::{DataType, Field, Schema};
        use arrow::record_batch::RecordBatch;
        use parquet::arrow::ArrowWriter;
        use std::fs::File;
        use std::sync::Arc;
        use tempfile::tempdir;

        // Create a minimal Parquet file
        let temp_dir = tempdir().unwrap();
        let file_path = temp_dir.path().join("test.parquet");

        let schema = Arc::new(Schema::new(vec![Field::new("col", DataType::Int64, false)]));

        let batch = RecordBatch::try_new(schema.clone(), vec![Arc::new(Int64Array::from(vec![1]))])
            .unwrap();

        let file = File::create(&file_path).unwrap();
        let mut writer = ArrowWriter::try_new(file, schema, None).unwrap();
        writer.write(&batch).unwrap();
        writer.close().unwrap();

        // Test that calling next_batch without set_projection fails
        let mut reader = ParquetReader::from_file(file_path.to_str().unwrap(), 1024).unwrap();

        // Should fail because set_projection hasn't been called
        let result = reader.next_batch();
        assert!(result.is_err());
        assert!(result
            .unwrap_err()
            .to_string()
            .contains("set_projection must be called"));
    }

    #[test]
    fn test_phase0_parquet_reader_column_index_only() {
        use arrow::array::Int64Array;
        use arrow::datatypes::{DataType, Field, Schema};
        use arrow::record_batch::RecordBatch;
        use parquet::arrow::ArrowWriter;
        use std::fs::File;
        use std::sync::Arc;
        use tempfile::tempdir;

        // Create a minimal Parquet file
        let temp_dir = tempdir().unwrap();
        let file_path = temp_dir.path().join("test.parquet");

        let schema = Arc::new(Schema::new(vec![Field::new("col", DataType::Int64, false)]));

        let batch = RecordBatch::try_new(schema.clone(), vec![Arc::new(Int64Array::from(vec![1]))])
            .unwrap();

        let file = File::create(&file_path).unwrap();
        let mut writer = ArrowWriter::try_new(file, schema, None).unwrap();
        writer.write(&batch).unwrap();
        writer.close().unwrap();

        // Test that ParquetReader rejects FieldPath projections
        let mut reader = ParquetReader::from_file(file_path.to_str().unwrap(), 1024).unwrap();

        let projections = vec![Projection::new(
            ProjectionSource::FieldPath("col".to_string()), // Should be rejected
            0,
            LogicalType::Int64,
        )];

        let projection_spec = ProjectionSpec::new(projections).unwrap();
        let result = reader.set_projection(projection_spec);

        assert!(result.is_err());
        let error_msg = result.unwrap_err().to_string();
        assert!(error_msg.contains("FieldPath"));
        assert!(error_msg.contains("not supported"));
        assert!(error_msg.contains("ParquetReader"));
        assert!(error_msg.contains("ColumnIndex"));
    }
}
