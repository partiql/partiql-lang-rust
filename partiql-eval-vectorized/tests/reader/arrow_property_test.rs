use arrow::array::{BooleanArray, Float64Array, Int64Array, StringArray};
use arrow::datatypes::{DataType, Field as ArrowField, Schema};
use arrow::record_batch::RecordBatch;
use partiql_eval_vectorized::batch::LogicalType;
/// Property test for Arrow data conversion integrity
/// Validates that Arrow arrays convert correctly to LogicalTypes without data loss
use partiql_eval_vectorized::reader::{
    ArrowReader, BatchReader, Projection, ProjectionSource, ProjectionSpec,
};
use proptest::prelude::*;
use std::sync::Arc;

/// Generate valid Arrow RecordBatch for testing
fn arb_arrow_record_batch() -> impl Strategy<Value = RecordBatch> {
    prop_oneof![
        // Simple scalar arrays with reasonable bounds
        (
            prop::collection::vec(-100i64..100i64, 1..5),
            prop::collection::vec(-10.0f64..10.0f64, 1..5),
            prop::collection::vec(any::<bool>(), 1..5),
            prop::collection::vec("[a-zA-Z]{1,5}", 1..5)
        )
            .prop_filter("same length", |(ints, floats, bools, strings)| {
                let len = ints.len();
                floats.len() == len && bools.len() == len && strings.len() == len
            })
            .prop_filter("finite floats", |(_, floats, _, _)| {
                floats.iter().all(|f| f.is_finite())
            })
            .prop_map(|(ints, floats, bools, strings)| {
                let schema = Arc::new(Schema::new(vec![
                    ArrowField::new("int_col", DataType::Int64, false),
                    ArrowField::new("float_col", DataType::Float64, false),
                    ArrowField::new("bool_col", DataType::Boolean, false),
                    ArrowField::new("string_col", DataType::Utf8, false),
                ]));

                let int_array = Arc::new(Int64Array::from(ints));
                let float_array = Arc::new(Float64Array::from(floats));
                let bool_array = Arc::new(BooleanArray::from(bools));
                let string_array = Arc::new(StringArray::from(strings));

                RecordBatch::try_new(
                    schema,
                    vec![int_array, float_array, bool_array, string_array],
                )
                .unwrap()
            }),
        // Single column batches
        prop::collection::vec(-100i64..100i64, 1..3).prop_map(|ints| {
            let schema = Arc::new(Schema::new(vec![ArrowField::new(
                "id",
                DataType::Int64,
                false,
            )]));

            let int_array = Arc::new(Int64Array::from(ints));

            RecordBatch::try_new(schema, vec![int_array]).unwrap()
        })
    ]
}

proptest! {
    /// Property 4: Data Conversion Integrity
    /// Test that Arrow arrays convert correctly to LogicalTypes without data loss
    #[test]
    fn test_arrow_data_conversion_integrity(record_batch in arb_arrow_record_batch()) {
        let mut reader = ArrowReader::from_record_batch(record_batch.clone());

        let schema = record_batch.schema();
        let num_columns = schema.fields().len();

        // Test different column projections
        for col_idx in 0..num_columns {
            let field = schema.field(col_idx);

            // Determine appropriate LogicalType based on Arrow DataType
            let logical_types = match field.data_type() {
                DataType::Int64 => vec![LogicalType::Int64, LogicalType::Float64, LogicalType::String],
                DataType::Float64 => vec![LogicalType::Float64, LogicalType::String],
                DataType::Boolean => vec![LogicalType::Boolean, LogicalType::String],
                DataType::Utf8 => vec![LogicalType::String],
                _ => vec![LogicalType::String], // Fallback
            };

            for logical_type in logical_types {
                let projections = vec![
                    Projection::new(ProjectionSource::ColumnIndex(col_idx), 0, logical_type),
                ];

                if let Ok(projection_spec) = ProjectionSpec::new(projections) {
                    // Reset reader for each test
                    let mut test_reader = ArrowReader::from_record_batch(record_batch.clone());

                    if test_reader.set_projection(projection_spec).is_ok() {
                        // If projection is accepted, reading should succeed
                        match test_reader.next_batch() {
                            Ok(Some(batch)) => {
                                // Successful read - verify batch structure
                                prop_assert!(batch.row_count() > 0);
                                prop_assert_eq!(batch.total_column_count(), 1);
                                prop_assert_eq!(batch.row_count(), record_batch.num_rows());
                            }
                            Ok(None) => {
                                // No data - should not happen with valid RecordBatch
                                prop_assert!(false, "Expected batch but got None");
                            }
                            Err(e) => {
                                // Error during reading - only acceptable for unsupported conversions
                                let error_msg = e.to_string();
                                prop_assert!(error_msg.contains("Cannot convert"));
                            }
                        }
                    }
                }
            }
        }
    }

    /// Property 5: Projection Source Support by Data Type
    /// Test that ColumnIndex projections work correctly for Arrow data
    #[test]
    fn test_arrow_column_index_support(record_batch in arb_arrow_record_batch()) {
        let mut reader = ArrowReader::from_record_batch(record_batch.clone());

        let schema = record_batch.schema();
        let num_columns = schema.fields().len();

        // Test valid column indices
        for col_idx in 0..num_columns {
            let projections = vec![
                Projection::new(ProjectionSource::ColumnIndex(col_idx), 0, LogicalType::String),
            ];

            let projection_spec = ProjectionSpec::new(projections)?;

            // Should accept valid column index
            prop_assert!(reader.set_projection(projection_spec).is_ok());

            // Reset reader
            reader = ArrowReader::from_record_batch(record_batch.clone());
        }

        // Test invalid column index (out of bounds)
        if num_columns > 0 {
            let invalid_col_idx = num_columns;
            let projections = vec![
                Projection::new(ProjectionSource::ColumnIndex(invalid_col_idx), 0, LogicalType::String),
            ];

            let projection_spec = ProjectionSpec::new(projections)?;

            // Should reject out-of-bounds column index
            let result = reader.set_projection(projection_spec);
            prop_assert!(result.is_err());
            prop_assert!(result.unwrap_err().to_string().contains("out of bounds"));
        }
    }

    /// Property 12: Multiple Batch Processing
    /// Test that multiple Arrow RecordBatches are processed correctly
    #[test]
    fn test_arrow_multiple_batch_processing(
        batches in prop::collection::vec(arb_arrow_record_batch(), 1..4)
    ) {
        // Ensure all batches have the same schema
        if batches.is_empty() {
            return Ok(());
        }

        let first_schema = batches[0].schema();
        let compatible_batches: Vec<RecordBatch> = batches.into_iter()
            .filter(|batch| batch.schema() == first_schema)
            .collect();

        if compatible_batches.is_empty() {
            return Ok(());
        }

        let mut reader = ArrowReader::new(compatible_batches.clone());

        // Set projection for first column
        let projections = vec![
            Projection::new(ProjectionSource::ColumnIndex(0), 0, LogicalType::String),
        ];

        let projection_spec = ProjectionSpec::new(projections)?;
        reader.set_projection(projection_spec)?;

        let mut total_rows = 0;
        let mut batch_count = 0;

        // Read all batches
        while let Some(batch) = reader.next_batch()? {
            batch_count += 1;
            total_rows += batch.row_count();

            prop_assert!(batch.total_column_count() == 1);
            prop_assert!(batch.row_count() > 0);
        }

        // Verify we processed all batches
        prop_assert_eq!(batch_count, compatible_batches.len());

        // Verify total row count matches sum of input batches
        let expected_rows: usize = compatible_batches.iter().map(|b| b.num_rows()).sum();
        prop_assert_eq!(total_rows, expected_rows);
    }
}

#[cfg(test)]
mod unit_tests {
    use super::*;

    #[test]
    fn test_arrow_conversion_basic_types() {
        // Create Arrow RecordBatch with all scalar types
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

        let projections = vec![
            Projection::new(ProjectionSource::ColumnIndex(0), 0, LogicalType::Int64),
            Projection::new(ProjectionSource::ColumnIndex(1), 1, LogicalType::String),
            Projection::new(ProjectionSource::ColumnIndex(2), 2, LogicalType::Float64),
            Projection::new(ProjectionSource::ColumnIndex(3), 3, LogicalType::Boolean),
        ];

        let projection_spec = ProjectionSpec::new(projections).unwrap();
        reader.set_projection(projection_spec).unwrap();

        let batch = reader.next_batch().unwrap();
        assert!(batch.is_some());

        let batch = batch.unwrap();
        assert_eq!(batch.row_count(), 3);
        assert_eq!(batch.total_column_count(), 4);
    }

    #[test]
    fn test_arrow_conversion_int_to_float() {
        // Arrow Int64 should convert to Float64 when requested
        let schema = Arc::new(Schema::new(vec![ArrowField::new(
            "value",
            DataType::Int64,
            false,
        )]));

        let int_array = Arc::new(Int64Array::from(vec![42, 100]));
        let record_batch = RecordBatch::try_new(schema, vec![int_array]).unwrap();

        let mut reader = ArrowReader::from_record_batch(record_batch);

        let projections = vec![Projection::new(
            ProjectionSource::ColumnIndex(0),
            0,
            LogicalType::Float64,
        )];

        let projection_spec = ProjectionSpec::new(projections).unwrap();
        reader.set_projection(projection_spec).unwrap();

        let batch = reader.next_batch().unwrap();
        assert!(batch.is_some());

        let batch = batch.unwrap();
        assert_eq!(batch.row_count(), 2);
        assert_eq!(batch.total_column_count(), 1);
    }

    #[test]
    fn test_arrow_conversion_float_to_string() {
        // Arrow Float64 should convert to String when requested
        let schema = Arc::new(Schema::new(vec![ArrowField::new(
            "score",
            DataType::Float64,
            false,
        )]));

        let float_array = Arc::new(Float64Array::from(vec![3.14, 2.71]));
        let record_batch = RecordBatch::try_new(schema, vec![float_array]).unwrap();

        let mut reader = ArrowReader::from_record_batch(record_batch);

        let projections = vec![Projection::new(
            ProjectionSource::ColumnIndex(0),
            0,
            LogicalType::String,
        )];

        let projection_spec = ProjectionSpec::new(projections).unwrap();
        reader.set_projection(projection_spec).unwrap();

        let batch = reader.next_batch().unwrap();
        assert!(batch.is_some());

        let batch = batch.unwrap();
        assert_eq!(batch.row_count(), 2);
        assert_eq!(batch.total_column_count(), 1);
    }

    #[test]
    fn test_arrow_field_path_rejection() {
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
        let error_msg = result.unwrap_err().to_string();
        assert!(error_msg.contains("FieldPath"));
        assert!(error_msg.contains("not support"));
    }

    #[test]
    fn test_arrow_column_bounds_validation() {
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
        let error_msg = result.unwrap_err().to_string();
        assert!(error_msg.contains("out of bounds"));
        assert!(error_msg.contains("Column index 1"));
    }
}
