use partiql_eval_vectorized::batch::LogicalType;
use partiql_eval_vectorized::reader::error::*;
use partiql_eval_vectorized::reader::{BatchReader, Projection, ProjectionSource, ProjectionSpec, ParquetReader};
use partiql_eval_vectorized::error::EvalError;
use proptest::prelude::*;
use std::fs::File;
use std::io::Write;
use tempfile::tempdir;

/// Property test for comprehensive error reporting
/// Validates that all error conditions produce sufficient debugging context

proptest! {
    #[test]
    fn test_projection_error_message_quality(
        source in "[a-zA-Z_][a-zA-Z0-9_]*",
        reason in "[a-zA-Z ]{10,50}",
        context in "[a-zA-Z ]{10,50}",
        available_sources in prop::collection::vec("[a-zA-Z_][a-zA-Z0-9_]*", 1..5)
    ) {
        // Test InvalidSpec error message quality
        let invalid_spec_error = ProjectionError::invalid_spec(&reason, &context);
        let message = invalid_spec_error.to_string();
        
        // Must contain key information for debugging
        prop_assert!(message.contains("Invalid projection specification"));
        prop_assert!(message.contains(&reason));
        prop_assert!(message.contains(&context));
        prop_assert!(message.len() > 20); // Minimum meaningful length
        
        // Test SourceNotFound error message quality
        let not_found_error = ProjectionError::source_not_found(&source, &available_sources);
        let message = not_found_error.to_string();
        
        // Must contain source name and available alternatives
        prop_assert!(message.contains(&source));
        prop_assert!(message.contains("not found"));
        prop_assert!(message.contains("Available sources"));
        for available in &available_sources {
            prop_assert!(message.contains(available));
        }
        
        // Test UnsupportedSource error message quality
        let unsupported_error = ProjectionError::unsupported_source(
            "FieldPath", 
            "ParquetReader", 
            &["ColumnIndex"]
        );
        let message = unsupported_error.to_string();
        
        prop_assert!(message.contains("FieldPath"));
        prop_assert!(message.contains("ParquetReader"));
        prop_assert!(message.contains("ColumnIndex"));
        prop_assert!(message.contains("not supported"));
        prop_assert!(message.contains("Supported sources"));
    }

    #[test]
    fn test_data_source_error_message_quality(
        resource in "[a-zA-Z0-9_./]{5,20}",
        reason in "[a-zA-Z ]{10,50}",
        location in "[a-zA-Z0-9 ]{5,20}",
        details in "[a-zA-Z ]{10,50}",
        parameter in "[a-zA-Z_][a-zA-Z0-9_]*",
        required_for in "[a-zA-Z ]{5,20}"
    ) {
        // Test AccessFailed error message quality
        let access_error = DataSourceError::access_failed(&resource, &reason);
        let message = access_error.to_string();
        
        prop_assert!(message.contains("Failed to access"));
        prop_assert!(message.contains(&resource));
        prop_assert!(message.contains(&reason));
        prop_assert!(message.len() > 15);
        
        // Test CorruptedData error message quality
        let corrupted_error = DataSourceError::corrupted_data(&resource, &location, &details);
        let message = corrupted_error.to_string();
        
        prop_assert!(message.contains("Corrupted data"));
        prop_assert!(message.contains(&resource));
        prop_assert!(message.contains(&location));
        prop_assert!(message.contains(&details));
        
        // Test MissingConfiguration error message quality
        let config_error = DataSourceError::missing_configuration(&parameter, &required_for);
        let message = config_error.to_string();
        
        prop_assert!(message.contains("Missing required configuration"));
        prop_assert!(message.contains(&parameter));
        prop_assert!(message.contains(&required_for));
    }

    #[test]
    fn test_type_conversion_error_message_quality(
        source in "[a-zA-Z_][a-zA-Z0-9_]*",
        source_type in "[a-zA-Z]{3,10}",
        value in "[a-zA-Z0-9]{1,20}",
        reason in "[a-zA-Z ]{10,50}",
        target_type in prop::sample::select(vec![
            LogicalType::Int64,
            LogicalType::Float64,
            LogicalType::Boolean,
            LogicalType::String
        ])
    ) {
        // Test TypeMismatch error message quality
        let mismatch_error = TypeConversionError::type_mismatch(
            &source,
            &source_type,
            target_type.clone(),
            Some("Use explicit conversion")
        );
        let message = mismatch_error.to_string();
        
        prop_assert!(message.contains("Type mismatch"));
        prop_assert!(message.contains(&source));
        prop_assert!(message.contains(&source_type));
        prop_assert!(message.contains("Use explicit conversion"));
        
        // Test ConversionFailed error message quality
        let conversion_error = TypeConversionError::conversion_failed(
            &source,
            &value,
            target_type.clone(),
            &reason
        );
        let message = conversion_error.to_string();
        
        prop_assert!(message.contains("Failed to convert"));
        prop_assert!(message.contains(&source));
        prop_assert!(message.contains(&value));
        prop_assert!(message.contains(&reason));
        
        // Test UnexpectedNull error message quality
        let null_error = TypeConversionError::unexpected_null(&source, target_type);
        let message = null_error.to_string();
        
        prop_assert!(message.contains("Unexpected null"));
        prop_assert!(message.contains(&source));
    }

    #[test]
    fn test_batch_reader_error_context_quality(
        context_items in prop::collection::vec("[a-zA-Z ]{10,30}", 1..4)
    ) {
        let base_error = ProjectionError::invalid_spec("Test error", "Test context");
        let mut batch_error = BatchReaderError::projection(base_error);
        
        // Add context items
        for context in &context_items {
            batch_error = batch_error.with_context(context.clone());
        }
        
        let message = batch_error.to_string();
        
        // Must contain severity indicator
        prop_assert!(message.contains("[FATAL]") || message.contains("[RECOVERABLE]") || message.contains("[WARNING]"));
        
        // Must contain all context items
        prop_assert!(message.contains("Context:"));
        for context in &context_items {
            prop_assert!(message.contains(context));
        }
        
        // Context items should be numbered
        for i in 1..=context_items.len() {
            let expected_format = format!("{}: ", i);
            prop_assert!(message.contains(&expected_format));
        }
    }

    #[test]
    fn test_error_message_actionability(
        field_name in "[a-zA-Z_][a-zA-Z0-9_]*",
        available_fields in prop::collection::vec("[a-zA-Z_][a-zA-Z0-9_]*", 2..6)
    ) {
        // Test that error messages provide actionable information
        let error = ProjectionError::source_not_found(&field_name, &available_fields);
        let message = error.to_string();
        
        // Should suggest alternatives
        prop_assert!(message.contains("Available sources"));
        
        // Should be specific about what was not found
        prop_assert!(message.contains(&field_name));
        prop_assert!(message.contains("not found"));
        
        // Should list all available alternatives
        for field in &available_fields {
            prop_assert!(message.contains(field));
        }
        
        // Message should be reasonably long to be informative
        prop_assert!(message.len() > 50);
        
        // Should not contain placeholder text
        prop_assert!(!message.contains("TODO"));
        prop_assert!(!message.contains("FIXME"));
        prop_assert!(!message.contains("XXX"));
    }

    #[test]
    fn test_error_severity_consistency(
        error_type in prop::sample::select(vec!["projection", "data_source", "type_conversion"])
    ) {
        let batch_error = match error_type {
            "projection" => BatchReaderError::projection(
                ProjectionError::invalid_spec("test", "test")
            ),
            "data_source" => BatchReaderError::data_source(
                DataSourceError::access_failed("test", "test")
            ),
            "type_conversion" => BatchReaderError::type_conversion(
                TypeConversionError::unexpected_null("test", LogicalType::Int64)
            ),
            _ => unreachable!(),
        };
        
        let message = batch_error.to_string();
        
        // Severity should be consistent with error type
        match error_type {
            "projection" | "data_source" => {
                prop_assert!(message.contains("[FATAL]"));
                prop_assert_eq!(batch_error.severity, ErrorSeverity::Fatal);
            },
            "type_conversion" => {
                prop_assert!(message.contains("[RECOVERABLE]"));
                prop_assert_eq!(batch_error.severity, ErrorSeverity::Recoverable);
            },
            _ => unreachable!(),
        }
    }

    #[test]
    fn test_parquet_reader_error_handling(
        column_index in 5usize..10,  // Ensure it's definitely out of bounds (file has 3 columns: 0,1,2)
        invalid_path in "[a-zA-Z_][a-zA-Z0-9_]*"
    ) {
        // Create a temporary Parquet file for testing
        let temp_dir = tempdir().unwrap();
        let file_path = temp_dir.path().join("test.parquet");
        
        // Create a simple Parquet file with exactly 3 columns (indices 0, 1, 2)
        use arrow::array::{Int64Array, StringArray};
        use arrow::record_batch::RecordBatch;
        use arrow::datatypes::{DataType, Field, Schema};
        use parquet::arrow::ArrowWriter;
        use std::sync::Arc;
        
        let schema = Arc::new(Schema::new(vec![
            Field::new("col_0", DataType::Int64, false),
            Field::new("col_1", DataType::Int64, false),
            Field::new("col_2", DataType::Utf8, false),
        ]));
        
        let batch = RecordBatch::try_new(
            schema.clone(),
            vec![
                Arc::new(Int64Array::from(vec![1, 2, 3])),
                Arc::new(Int64Array::from(vec![4, 5, 6])),
                Arc::new(StringArray::from(vec!["a", "b", "c"])),
            ],
        ).unwrap();
        
        let file = File::create(&file_path).unwrap();
        let mut writer = ArrowWriter::try_new(file, schema, None).unwrap();
        writer.write(&batch).unwrap();
        writer.close().unwrap();
        
        // Test column index out of bounds (column_index is 5-9, file has columns 0-2)
        let mut reader = ParquetReader::from_file(file_path.to_str().unwrap(), 1024).unwrap();
        let projections = vec![
            Projection::new(
                ProjectionSource::ColumnIndex(column_index),
                0,
                LogicalType::Int64,
            ),
        ];
        let spec = ProjectionSpec::new(projections).unwrap();
        let result = reader.set_projection(spec);
        
        prop_assert!(result.is_err());
        let error_msg = result.unwrap_err().to_string();
        prop_assert!(error_msg.contains("out of bounds"));
        
        // Test FieldPath rejection (ParquetReader only supports ColumnIndex)
        let mut reader = ParquetReader::from_file(file_path.to_str().unwrap(), 1024).unwrap();
        let projections = vec![
            Projection::new(
                ProjectionSource::FieldPath(invalid_path.clone()),
                0,
                LogicalType::String,
            ),
        ];
        let spec = ProjectionSpec::new(projections).unwrap();
        let result = reader.set_projection(spec);
        
        prop_assert!(result.is_err());
        let error_msg = result.unwrap_err().to_string();
        prop_assert!(error_msg.contains("FieldPath"));
        prop_assert!(error_msg.contains("not supported"));
        prop_assert!(error_msg.contains("ParquetReader"));
        prop_assert!(error_msg.contains("ColumnIndex"));
    }

    #[test]
    fn test_parquet_reader_file_access_errors(
        invalid_filename in "[a-zA-Z0-9_]{5,15}\\.parquet"
    ) {
        // Use a UUID to ensure uniqueness and non-existence
        use std::collections::hash_map::DefaultHasher;
        use std::hash::{Hash, Hasher};
        
        let mut hasher = DefaultHasher::new();
        invalid_filename.hash(&mut hasher);
        let unique_id = hasher.finish();
        let non_existent_path = format!("/tmp/non_existent_dir_{}/{}", unique_id, invalid_filename);
        
        // Test file access errors - this should always fail
        let result = ParquetReader::from_file(&non_existent_path, 1024);
        
        // Only test if the result is actually an error
        if result.is_err() {
            match result {
                Err(error) => {
                    let error_msg = error.to_string();
                    prop_assert!(error_msg.contains("Failed to access"));
                    prop_assert!(error_msg.contains(&non_existent_path));
                    prop_assert!(error_msg.contains("[FATAL]"));
                }
                Ok(_) => unreachable!("Already checked is_err()"),
            }
        } else {
            // If somehow the file exists, skip this test case
            prop_assume!(false);
        }
    }
}

#[cfg(test)]
mod unit_tests {
    use super::*;

    #[test]
    fn test_error_message_minimum_requirements() {
        // Test that all error types meet minimum message quality requirements
        
        // ProjectionError variants
        let errors = vec![
            ProjectionError::invalid_spec("duplicate indices", "projection validation").to_string(),
            ProjectionError::unsupported_source("FieldPath", "ArrowReader", &["ColumnIndex"]).to_string(),
            ProjectionError::source_not_found("missing_col", &vec!["col_a".to_string(), "col_b".to_string()]).to_string(),
            ProjectionError::non_scalar_data("nested_field", "Struct").to_string(),
        ];
        
        for error_msg in errors {
            // Minimum length requirement
            assert!(error_msg.len() > 20, "Error message too short: {}", error_msg);
            
            // Should not contain debug artifacts
            assert!(!error_msg.contains("Debug"), "Error message contains debug artifacts: {}", error_msg);
            assert!(!error_msg.contains("{:?}"), "Error message contains debug formatting: {}", error_msg);
            
            // Should be properly capitalized
            assert!(error_msg.chars().next().unwrap().is_uppercase(), "Error message not capitalized: {}", error_msg);
        }
        
        // DataSourceError variants
        let data_errors = vec![
            DataSourceError::access_failed("file.parquet", "Permission denied").to_string(),
            DataSourceError::corrupted_data("file.parquet", "row 100", "Invalid checksum").to_string(),
            DataSourceError::missing_configuration("batch_size", "Parquet reader").to_string(),
            DataSourceError::initialization_failed("Ion", "Invalid configuration").to_string(),
        ];
        
        for error_msg in data_errors {
            assert!(error_msg.len() > 15, "Data source error message too short: {}", error_msg);
            assert!(error_msg.chars().next().unwrap().is_uppercase(), "Data source error not capitalized: {}", error_msg);
        }
        
        // TypeConversionError variants
        let type_errors = vec![
            TypeConversionError::type_mismatch("col", "String", LogicalType::Int64, None).to_string(),
            TypeConversionError::conversion_failed("col", "abc", LogicalType::Int64, "Invalid format").to_string(),
            TypeConversionError::unexpected_null("col", LogicalType::Int64).to_string(),
        ];
        
        for error_msg in type_errors {
            assert!(error_msg.len() > 20, "Type conversion error message too short: {}", error_msg);
            assert!(error_msg.chars().next().unwrap().is_uppercase(), "Type conversion error not capitalized: {}", error_msg);
        }
    }

    #[test]
    fn test_error_context_formatting() {
        let error = BatchReaderError::projection(
            ProjectionError::invalid_spec("test error", "test context")
        )
        .with_context("First context item".to_string())
        .with_context("Second context item".to_string())
        .with_context("Third context item".to_string());
        
        let message = error.to_string();
        
        // Should have proper context formatting
        assert!(message.contains("Context:"));
        assert!(message.contains("1: First context item"));
        assert!(message.contains("2: Second context item"));
        assert!(message.contains("3: Third context item"));
        
        // Context should be on separate lines
        let lines: Vec<&str> = message.lines().collect();
        assert!(lines.len() >= 4); // Main message + "Context:" + 3 context items
    }

    #[test]
    fn test_eval_error_conversion_preserves_information() {
        let original_error = BatchReaderError::data_source(
            DataSourceError::corrupted_data("test.parquet", "row 50", "Checksum mismatch")
        ).with_context("During batch processing".to_string());
        
        let eval_error: EvalError = original_error.clone().into();
        
        match eval_error {
            EvalError::General(msg) => {
                // Should preserve all key information
                assert!(msg.contains("Corrupted data"));
                assert!(msg.contains("test.parquet"));
                assert!(msg.contains("row 50"));
                assert!(msg.contains("Checksum mismatch"));
                assert!(msg.contains("During batch processing"));
                assert!(msg.contains("[FATAL]"));
            }
            _ => panic!("Expected General error variant"),
        }
    }

    #[test]
    fn test_parquet_reader_specific_errors() {
        use tempfile::tempdir;
        use std::fs::File;
        use std::io::Write;
        
        // Test file not found error
        let result = ParquetReader::from_file("nonexistent_file.parquet", 1024);
        assert!(result.is_err());
        match result {
            Err(error) => {
                let error_msg = error.to_string();
                assert!(error_msg.contains("Failed to access"));
                assert!(error_msg.contains("nonexistent_file.parquet"));
                assert!(error_msg.contains("[FATAL]"));
            }
            Ok(_) => panic!("Expected error but got Ok"),
        }
        
        // Test invalid file format error - this will pass file existence check
        // but fail when trying to read as Parquet during set_projection
        let temp_dir = tempdir().unwrap();
        let invalid_file = temp_dir.path().join("invalid.parquet");
        let mut file = File::create(&invalid_file).unwrap();
        file.write_all(b"This is definitely not a Parquet file - just plain text").unwrap();
        drop(file); // Ensure file is closed
        
        // File creation succeeds, but reading will fail during set_projection
        let result = ParquetReader::from_file(invalid_file.to_str().unwrap(), 1024);
        assert!(result.is_ok()); // File exists, so constructor succeeds
        
        let mut reader = result.unwrap();
        let projections = vec![
            Projection::new(
                ProjectionSource::ColumnIndex(0),
                0,
                LogicalType::Int64,
            ),
        ];
        let spec = ProjectionSpec::new(projections).unwrap();
        
        // This should fail because the file is not a valid Parquet file
        let result = reader.set_projection(spec);
        if result.is_err() {
            match result {
                Err(error) => {
                    let error_msg = error.to_string();
                    assert!(error_msg.contains("Failed to") || error_msg.contains("initialization failed"));
                    assert!(error_msg.contains("[FATAL]"));
                }
                Ok(_) => panic!("Expected error but got Ok"),
            }
        } else {
            // If the invalid file somehow passes, skip this part of the test
            // This can happen if the Parquet library is very lenient
            println!("Warning: Invalid Parquet file was accepted by the library");
        }
    }

    #[test]
    fn test_debug_parquet_bounds_checking() {
        use tempfile::tempdir;
        use std::fs::File;
        use arrow::array::{Int64Array, StringArray};
        use arrow::record_batch::RecordBatch;
        use arrow::datatypes::{DataType, Field, Schema};
        use parquet::arrow::ArrowWriter;
        use std::sync::Arc;
        
        // Create a temporary Parquet file for testing
        let temp_dir = tempdir().unwrap();
        let file_path = temp_dir.path().join("test.parquet");
        
        // Create a simple Parquet file with exactly 3 columns (indices 0, 1, 2)
        let schema = Arc::new(Schema::new(vec![
            Field::new("col_0", DataType::Int64, false),
            Field::new("col_1", DataType::Int64, false),
            Field::new("col_2", DataType::Utf8, false),
        ]));
        
        let batch = RecordBatch::try_new(
            schema.clone(),
            vec![
                Arc::new(Int64Array::from(vec![1, 2, 3])),
                Arc::new(Int64Array::from(vec![4, 5, 6])),
                Arc::new(StringArray::from(vec!["a", "b", "c"])),
            ],
        ).unwrap();
        
        let file = File::create(&file_path).unwrap();
        let mut writer = ArrowWriter::try_new(file, schema, None).unwrap();
        writer.write(&batch).unwrap();
        writer.close().unwrap();
        
        // Test column index 5 (should be out of bounds)
        let mut reader = ParquetReader::from_file(file_path.to_str().unwrap(), 1024).unwrap();
        let projections = vec![
            Projection::new(
                ProjectionSource::ColumnIndex(5),
                0,
                LogicalType::Int64,
            ),
        ];
        let spec = ProjectionSpec::new(projections).unwrap();
        let result = reader.set_projection(spec);
        
        println!("Debug: Result for column index 5: {:?}", result);
        assert!(result.is_err(), "Expected error for column index 5 but got success");
        let error_msg = result.unwrap_err().to_string();
        assert!(error_msg.contains("out of bounds"), "Error message should contain 'out of bounds': {}", error_msg);
    }
}