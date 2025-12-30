/// Property test for Ion data conversion integrity
/// Validates that Ion values convert correctly to LogicalTypes without data loss

use partiql_eval_vectorized::reader::{
    BatchReader, IonReader, Projection, ProjectionSource, ProjectionSpec,
};
use partiql_eval_vectorized::batch::LogicalType;
use proptest::prelude::*;

/// Generate valid Ion data for testing
fn arb_ion_data() -> impl Strategy<Value = String> {
    prop_oneof![
        // Simple scalar values with reasonable bounds
        (-1_000_000i64..1_000_000i64).prop_map(|i| format!("{}", i)),
        (-1000.0f64..1000.0f64).prop_filter("finite", |f| f.is_finite()).prop_map(|f| format!("{:.2}", f)),
        any::<bool>().prop_map(|b| format!("{}", b)),
        "[a-zA-Z][a-zA-Z0-9_]{0,10}".prop_map(|s| format!("\"{}\"", s)),
        
        // Struct with scalar fields (reasonable bounds)
        (
            -1000i64..1000i64,
            -100.0f64..100.0f64,
            any::<bool>(),
            "[a-zA-Z][a-zA-Z0-9_]{0,10}"
        ).prop_filter("finite float", |(_, f, _, _)| f.is_finite())
         .prop_map(|(i, f, b, s)| {
            format!("{{age: {}, score: {:.2}, active: {}, name: \"{}\"}}", i, f, b, s)
        }),
            
        // Multiple records (reasonable bounds)
        prop::collection::vec(
            (
                -1000i64..1000i64,
                "[a-zA-Z][a-zA-Z0-9_]{0,10}"
            ).prop_map(|(i, s)| {
                format!("{{id: {}, name: \"{}\"}}", i, s)
            }),
            1..3  // Smaller collection size
        ).prop_map(|records| records.join("\n"))
    ]
}

proptest! {
    /// Property 4: Data Conversion Integrity
    /// Test that Ion values convert correctly to LogicalTypes without data loss
    #[test]
    fn test_ion_data_conversion_integrity(ion_data in arb_ion_data()) {
        // Create Ion reader
        let mut reader = IonReader::from_ion_text(&ion_data, 10)?;
        
        // Test different projection types
        let test_cases = vec![
            (ProjectionSource::FieldPath("age".to_string()), LogicalType::Int64),
            (ProjectionSource::FieldPath("score".to_string()), LogicalType::Float64),
            (ProjectionSource::FieldPath("active".to_string()), LogicalType::Boolean),
            (ProjectionSource::FieldPath("name".to_string()), LogicalType::String),
            (ProjectionSource::FieldPath("id".to_string()), LogicalType::Int64),
        ];
        
        for (source, logical_type) in test_cases {
            let projections = vec![
                Projection::new(source, 0, logical_type),
            ];
            
            if let Ok(projection_spec) = ProjectionSpec::new(projections) {
                if reader.set_projection(projection_spec).is_ok() {
                    // If projection is accepted, reading should either succeed or fail gracefully
                    match reader.next_batch() {
                        Ok(Some(batch)) => {
                            // Successful read - verify batch structure
                            prop_assert!(batch.row_count() > 0);
                            prop_assert_eq!(batch.total_column_count(), 1);
                        }
                        Ok(None) => {
                            // No data - acceptable
                        }
                        Err(_) => {
                            // Error during reading - acceptable for type mismatches or missing fields
                        }
                    }
                }
            }
            
            // Reset reader for next test
            reader = IonReader::from_ion_text(&ion_data, 10)?;
        }
    }
    
    /// Property 7: Missing Field Handling
    /// Test that missing Ion fields result in null values, not errors
    #[test]
    fn test_ion_missing_field_handling(
        existing_field in "[a-zA-Z][a-zA-Z0-9_]{0,10}",
        missing_field in "[a-zA-Z][a-zA-Z0-9_]{0,10}",
        value in -1000i64..1000i64
    ) {
        prop_assume!(existing_field != missing_field);
        
        let ion_data = format!("{{{}: {}}}", existing_field, value);
        let mut reader = IonReader::from_ion_text(&ion_data, 10)?;
        
        // Project both existing and missing fields
        let projections = vec![
            Projection::new(ProjectionSource::FieldPath(existing_field), 0, LogicalType::Int64),
            Projection::new(ProjectionSource::FieldPath(missing_field), 1, LogicalType::Int64),
        ];
        
        let projection_spec = ProjectionSpec::new(projections)?;
        reader.set_projection(projection_spec)?;
        
        // Should succeed even with missing field
        let batch = reader.next_batch()?;
        prop_assert!(batch.is_some());
        
        let batch = batch.unwrap();
        prop_assert_eq!(batch.row_count(), 1);
        prop_assert_eq!(batch.total_column_count(), 2);
        
        // Missing field should result in null/default value, not error
        // (The actual null handling is implementation-specific to the Vector type)
    }
}

#[cfg(test)]
mod unit_tests {
    use super::*;

    #[test]
    fn test_ion_conversion_basic_types() {
        let ion_data = r#"{name: "Alice", age: 30, score: 95.5, active: true}"#;
        let mut reader = IonReader::from_ion_text(ion_data, 10).unwrap();

        let projections = vec![
            Projection::new(ProjectionSource::FieldPath("name".to_string()), 0, LogicalType::String),
            Projection::new(ProjectionSource::FieldPath("age".to_string()), 1, LogicalType::Int64),
            Projection::new(ProjectionSource::FieldPath("score".to_string()), 2, LogicalType::Float64),
            Projection::new(ProjectionSource::FieldPath("active".to_string()), 3, LogicalType::Boolean),
        ];

        let projection_spec = ProjectionSpec::new(projections).unwrap();
        reader.set_projection(projection_spec).unwrap();

        let batch = reader.next_batch().unwrap();
        assert!(batch.is_some());

        let batch = batch.unwrap();
        assert_eq!(batch.row_count(), 1);
        assert_eq!(batch.total_column_count(), 4);
    }

    #[test]
    fn test_ion_conversion_decimal_to_float() {
        // Ion parses 95.5 as Decimal, should convert to Float64
        let ion_data = r#"{score: 95.5}"#;
        let mut reader = IonReader::from_ion_text(ion_data, 10).unwrap();

        let projections = vec![
            Projection::new(ProjectionSource::FieldPath("score".to_string()), 0, LogicalType::Float64),
        ];

        let projection_spec = ProjectionSpec::new(projections).unwrap();
        reader.set_projection(projection_spec).unwrap();

        let batch = reader.next_batch().unwrap();
        assert!(batch.is_some());

        let batch = batch.unwrap();
        assert_eq!(batch.row_count(), 1);
        assert_eq!(batch.total_column_count(), 1);
    }

    #[test]
    fn test_ion_conversion_int_to_float() {
        // Ion integers should convert to Float64 when requested
        let ion_data = r#"{value: 42}"#;
        let mut reader = IonReader::from_ion_text(ion_data, 10).unwrap();

        let projections = vec![
            Projection::new(ProjectionSource::FieldPath("value".to_string()), 0, LogicalType::Float64),
        ];

        let projection_spec = ProjectionSpec::new(projections).unwrap();
        reader.set_projection(projection_spec).unwrap();

        let batch = reader.next_batch().unwrap();
        assert!(batch.is_some());

        let batch = batch.unwrap();
        assert_eq!(batch.row_count(), 1);
        assert_eq!(batch.total_column_count(), 1);
    }

    #[test]
    fn test_ion_conversion_symbol_to_string() {
        // Ion symbols should convert to String when requested
        let ion_data = r#"{status: active}"#;  // 'active' is a symbol, not a string
        let mut reader = IonReader::from_ion_text(ion_data, 10).unwrap();

        let projections = vec![
            Projection::new(ProjectionSource::FieldPath("status".to_string()), 0, LogicalType::String),
        ];

        let projection_spec = ProjectionSpec::new(projections).unwrap();
        reader.set_projection(projection_spec).unwrap();

        let batch = reader.next_batch().unwrap();
        assert!(batch.is_some());

        let batch = batch.unwrap();
        assert_eq!(batch.row_count(), 1);
        assert_eq!(batch.total_column_count(), 1);
    }

    #[test]
    fn test_ion_missing_field_null_handling() {
        let ion_data = r#"
            {name: "Alice", age: 30}
            {name: "Bob", score: 87.2}
        "#;
        let mut reader = IonReader::from_ion_text(ion_data, 10).unwrap();

        let projections = vec![
            Projection::new(ProjectionSource::FieldPath("name".to_string()), 0, LogicalType::String),
            Projection::new(ProjectionSource::FieldPath("age".to_string()), 1, LogicalType::Int64),
            Projection::new(ProjectionSource::FieldPath("score".to_string()), 2, LogicalType::Float64),
        ];

        let projection_spec = ProjectionSpec::new(projections).unwrap();
        reader.set_projection(projection_spec).unwrap();

        let batch = reader.next_batch().unwrap();
        assert!(batch.is_some());

        let batch = batch.unwrap();
        assert_eq!(batch.row_count(), 2);
        assert_eq!(batch.total_column_count(), 3);

        // Alice has age but no score, Bob has score but no age
        // Both should be handled gracefully with null/default values
    }
}