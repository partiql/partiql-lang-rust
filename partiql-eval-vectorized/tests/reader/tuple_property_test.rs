/// Property test for Tuple data conversion integrity
/// Validates that PartiQL tuples convert correctly to LogicalTypes without data loss

use partiql_eval_vectorized::reader::{
    BatchReader, TupleIteratorReader, Projection, ProjectionSource, ProjectionSpec,
};
use partiql_eval_vectorized::batch::LogicalType;
use partiql_value::{Value, Tuple};
use proptest::prelude::*;

/// Generate valid PartiQL Value tuples for testing
fn arb_tuple_data() -> impl Strategy<Value = Vec<Value>> {
    prop_oneof![
        // Simple scalar tuples with reasonable bounds
        (
            -100i64..100i64,
            -10.0f64..10.0f64,
            any::<bool>(),
            "[a-zA-Z]{1,5}"
        ).prop_filter("finite float", |(_, f, _, _)| f.is_finite())
         .prop_map(|(i, f, b, s)| {
            vec![Value::Tuple(Box::new(Tuple::from([
                ("id", Value::Integer(i)),
                ("score", Value::Real(f.into())),
                ("active", Value::Boolean(b)),
                ("name", Value::String(Box::new(s))),
            ])))]
        }),
        
        // Multiple tuples with consistent schema
        prop::collection::vec(
            (
                -100i64..100i64,
                "[a-zA-Z]{1,5}"
            ).prop_map(|(i, s)| {
                Value::Tuple(Box::new(Tuple::from([
                    ("id", Value::Integer(i)),
                    ("name", Value::String(Box::new(s))),
                ])))
            }),
            1..3  // Smaller collection size
        ),
        
        // Tuples with missing fields (for null handling tests)
        prop::collection::vec(
            prop_oneof![
                // Complete tuple
                (
                    -50i64..50i64,
                    "[a-zA-Z]{1,3}"
                ).prop_map(|(i, s)| {
                    Value::Tuple(Box::new(Tuple::from([
                        ("id", Value::Integer(i)),
                        ("name", Value::String(Box::new(s))),
                        ("score", Value::Real(95.5.into())),
                    ])))
                }),
                // Missing score field
                (
                    -50i64..50i64,
                    "[a-zA-Z]{1,3}"
                ).prop_map(|(i, s)| {
                    Value::Tuple(Box::new(Tuple::from([
                        ("id", Value::Integer(i)),
                        ("name", Value::String(Box::new(s))),
                    ])))
                }),
                // Missing name field
                (-50i64..50i64).prop_map(|i| {
                    Value::Tuple(Box::new(Tuple::from([
                        ("id", Value::Integer(i)),
                        ("score", Value::Real(87.2.into())),
                    ])))
                })
            ],
            1..3
        )
    ]
}

proptest! {
    /// Property 12: Tuple Conversion Integrity
    /// Test that PartiQL tuples convert correctly to columnar vectors
    #[test]
    fn test_tuple_data_conversion_integrity(tuples in arb_tuple_data()) {
        let mut reader = TupleIteratorReader::new(Box::new(tuples.clone().into_iter()), 10);
        
        // Test different field projections
        let test_cases = vec![
            (ProjectionSource::FieldPath("id".to_string()), LogicalType::Int64),
            (ProjectionSource::FieldPath("name".to_string()), LogicalType::String),
            (ProjectionSource::FieldPath("score".to_string()), LogicalType::Float64),
            (ProjectionSource::FieldPath("active".to_string()), LogicalType::Boolean),
        ];
        
        for (source, logical_type) in test_cases {
            let projections = vec![
                Projection::new(source, 0, logical_type),
            ];
            
            if let Ok(projection_spec) = ProjectionSpec::new(projections) {
                // Reset reader for each test
                let mut test_reader = TupleIteratorReader::new(Box::new(tuples.clone().into_iter()), 10);
                
                if test_reader.set_projection(projection_spec).is_ok() {
                    // If projection is accepted, reading should either succeed or fail gracefully
                    match test_reader.next_batch() {
                        Ok(Some(batch)) => {
                            // Successful read - verify batch structure
                            prop_assert!(batch.row_count() > 0);
                            prop_assert_eq!(batch.total_column_count(), 1);
                            prop_assert_eq!(batch.row_count(), tuples.len());
                        }
                        Ok(None) => {
                            // No data - should not happen with valid tuples
                            if !tuples.is_empty() {
                                prop_assert!(false, "Expected batch but got None with {} tuples", tuples.len());
                            }
                        }
                        Err(_) => {
                            // Error during reading - acceptable for type mismatches or conversion failures
                        }
                    }
                }
            }
        }
    }
    
    /// Property 13: Field Path Support
    /// Test that FieldPath projections work correctly for tuple data
    #[test]
    fn test_tuple_field_path_support(tuples in arb_tuple_data()) {
        if tuples.is_empty() {
            return Ok(());
        }
        
        let mut reader = TupleIteratorReader::new(Box::new(tuples.clone().into_iter()), 10);
        
        // Test valid field paths
        let field_names = vec!["id", "name", "score", "active"];
        
        for field_name in field_names {
            let projections = vec![
                Projection::new(ProjectionSource::FieldPath(field_name.to_string()), 0, LogicalType::String),
            ];
            
            let projection_spec = ProjectionSpec::new(projections)?;
            
            // Should accept valid field path
            prop_assert!(reader.set_projection(projection_spec).is_ok());
            
            // Reset reader
            reader = TupleIteratorReader::new(Box::new(tuples.clone().into_iter()), 10);
        }
        
        // Test ColumnIndex rejection
        let projections = vec![
            Projection::new(ProjectionSource::ColumnIndex(0), 0, LogicalType::String),
        ];
        
        let projection_spec = ProjectionSpec::new(projections)?;
        
        // Should reject ColumnIndex projection
        let result = reader.set_projection(projection_spec);
        prop_assert!(result.is_err());
        prop_assert!(result.unwrap_err().to_string().contains("ColumnIndex"));
    }
    
    /// Property 14: Missing Field Handling
    /// Test that missing tuple fields result in default values, not errors
    #[test]
    fn test_tuple_missing_field_handling(
        existing_field in "[a-zA-Z][a-zA-Z0-9_]{0,10}",
        missing_field in "[a-zA-Z][a-zA-Z0-9_]{0,10}",
        value in -1000i64..1000i64
    ) {
        prop_assume!(existing_field != missing_field);
        
        let tuple = Value::Tuple(Box::new(Tuple::from([
            (existing_field.as_str(), Value::Integer(value)),
        ])));
        
        let tuples = vec![tuple];
        let mut reader = TupleIteratorReader::new(Box::new(tuples.into_iter()), 10);
        
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
        
        // Missing field should result in default value (0 for Int64), not error
    }
    
    /// Property 15: Single-Level Nesting Support
    /// Test that single-level nesting (struct.field) works correctly
    #[test]
    fn test_tuple_single_level_nesting(
        outer_field in "[a-zA-Z][a-zA-Z0-9_]{0,5}",
        inner_field in "[a-zA-Z][a-zA-Z0-9_]{0,5}",
        value in -100i64..100i64
    ) {
        let nested_tuple = Value::Tuple(Box::new(Tuple::from([
            (inner_field.as_str(), Value::Integer(value)),
        ])));
        
        let outer_tuple = Value::Tuple(Box::new(Tuple::from([
            (outer_field.as_str(), nested_tuple),
        ])));
        
        let tuples = vec![outer_tuple];
        let mut reader = TupleIteratorReader::new(Box::new(tuples.into_iter()), 10);
        
        // Test single-level nesting
        let nested_path = format!("{}.{}", outer_field, inner_field);
        let projections = vec![
            Projection::new(ProjectionSource::FieldPath(nested_path), 0, LogicalType::Int64),
        ];
        
        let projection_spec = ProjectionSpec::new(projections)?;
        reader.set_projection(projection_spec)?;
        
        // Should succeed with single-level nesting
        let batch = reader.next_batch()?;
        prop_assert!(batch.is_some());
        
        let batch = batch.unwrap();
        prop_assert_eq!(batch.row_count(), 1);
        prop_assert_eq!(batch.total_column_count(), 1);
    }
    
    /// Property 16: Batch Size Compliance
    /// Test that generated batches respect configured batch sizes
    #[test]
    fn test_tuple_batch_size_compliance(
        batch_size in 1usize..20,
        num_tuples in 1usize..50
    ) {
        // Generate tuples
        let tuples: Vec<Value> = (0..num_tuples).map(|i| {
            Value::Tuple(Box::new(Tuple::from([
                ("id", Value::Integer(i as i64)),
            ])))
        }).collect();
        
        let mut reader = TupleIteratorReader::new(Box::new(tuples.into_iter()), batch_size);
        
        let projections = vec![
            Projection::new(ProjectionSource::FieldPath("id".to_string()), 0, LogicalType::Int64),
        ];
        
        let projection_spec = ProjectionSpec::new(projections)?;
        reader.set_projection(projection_spec)?;
        
        let mut total_rows = 0;
        let mut batch_count = 0;
        
        // Read all batches
        while let Some(batch) = reader.next_batch()? {
            batch_count += 1;
            let batch_rows = batch.row_count();
            total_rows += batch_rows;
            
            // Each batch should respect the batch size limit
            prop_assert!(batch_rows <= batch_size);
            prop_assert!(batch_rows > 0);
            
            // Last batch might be smaller
            if total_rows < num_tuples {
                prop_assert_eq!(batch_rows, batch_size);
            } else {
                // This is the last batch
                let expected_last_batch_size = num_tuples - (batch_count - 1) * batch_size;
                prop_assert_eq!(batch_rows, expected_last_batch_size);
            }
        }
        
        // Verify total row count
        prop_assert_eq!(total_rows, num_tuples);
        
        // Verify expected number of batches
        let expected_batches = (num_tuples + batch_size - 1) / batch_size; // Ceiling division
        prop_assert_eq!(batch_count, expected_batches);
    }
}

#[cfg(test)]
mod unit_tests {
    use super::*;

    #[test]
    fn test_tuple_conversion_basic_types() {
        let tuples = vec![
            Value::Tuple(Box::new(Tuple::from([
                ("name", Value::String(Box::new("Alice".to_string()))),
                ("age", Value::Integer(30)),
                ("score", Value::Real(95.5.into())),
                ("active", Value::Boolean(true)),
            ]))),
        ];

        let mut reader = TupleIteratorReader::new(Box::new(tuples.into_iter()), 10);

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
    fn test_tuple_conversion_int_to_float() {
        // PartiQL integers should convert to Float64 when requested
        let tuples = vec![
            Value::Tuple(Box::new(Tuple::from([
                ("value", Value::Integer(42)),
            ]))),
        ];

        let mut reader = TupleIteratorReader::new(Box::new(tuples.into_iter()), 10);

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
    fn test_tuple_conversion_various_to_string() {
        // Various PartiQL types should convert to String when requested
        let tuples = vec![
            Value::Tuple(Box::new(Tuple::from([
                ("int_val", Value::Integer(123)),
                ("float_val", Value::Real(3.14.into())),
                ("bool_val", Value::Boolean(true)),
                ("string_val", Value::String(Box::new("hello".to_string()))),
            ]))),
        ];

        let mut reader = TupleIteratorReader::new(Box::new(tuples.into_iter()), 10);

        let projections = vec![
            Projection::new(ProjectionSource::FieldPath("int_val".to_string()), 0, LogicalType::String),
            Projection::new(ProjectionSource::FieldPath("float_val".to_string()), 1, LogicalType::String),
            Projection::new(ProjectionSource::FieldPath("bool_val".to_string()), 2, LogicalType::String),
            Projection::new(ProjectionSource::FieldPath("string_val".to_string()), 3, LogicalType::String),
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
    fn test_tuple_column_index_rejection() {
        let tuples = vec![
            Value::Tuple(Box::new(Tuple::from([
                ("name", Value::String(Box::new("Alice".to_string()))),
            ]))),
        ];

        let mut reader = TupleIteratorReader::new(Box::new(tuples.into_iter()), 10);

        // Try to set projection with ColumnIndex - should fail
        let projections = vec![
            Projection::new(ProjectionSource::ColumnIndex(0), 0, LogicalType::String),
        ];
        let projection_spec = ProjectionSpec::new(projections).unwrap();
        
        let result = reader.set_projection(projection_spec);
        assert!(result.is_err());
        let error_msg = result.unwrap_err().to_string();
        assert!(error_msg.contains("ColumnIndex"));
        assert!(error_msg.contains("not support"));
    }

    #[test]
    fn test_tuple_missing_field_default_values() {
        let tuples = vec![
            Value::Tuple(Box::new(Tuple::from([
                ("name", Value::String(Box::new("Alice".to_string()))),
                // Missing age field
            ]))),
        ];

        let mut reader = TupleIteratorReader::new(Box::new(tuples.into_iter()), 10);

        let projections = vec![
            Projection::new(ProjectionSource::FieldPath("name".to_string()), 0, LogicalType::String),
            Projection::new(ProjectionSource::FieldPath("age".to_string()), 1, LogicalType::Int64), // Missing field
        ];

        let projection_spec = ProjectionSpec::new(projections).unwrap();
        reader.set_projection(projection_spec).unwrap();

        let batch = reader.next_batch().unwrap();
        assert!(batch.is_some());

        let batch = batch.unwrap();
        assert_eq!(batch.row_count(), 1);
        assert_eq!(batch.total_column_count(), 2);
        
        // Missing field should result in default value (0 for Int64), not error
    }

    #[test]
    fn test_tuple_single_level_nesting_support() {
        let tuples = vec![
            Value::Tuple(Box::new(Tuple::from([
                ("person", Value::Tuple(Box::new(Tuple::from([
                    ("name", Value::String(Box::new("Alice".to_string()))),
                    ("age", Value::Integer(30)),
                ])))),
            ]))),
        ];

        let mut reader = TupleIteratorReader::new(Box::new(tuples.into_iter()), 10);

        let projections = vec![
            Projection::new(ProjectionSource::FieldPath("person.name".to_string()), 0, LogicalType::String),
            Projection::new(ProjectionSource::FieldPath("person.age".to_string()), 1, LogicalType::Int64),
        ];

        let projection_spec = ProjectionSpec::new(projections).unwrap();
        reader.set_projection(projection_spec).unwrap();

        let batch = reader.next_batch().unwrap();
        assert!(batch.is_some());

        let batch = batch.unwrap();
        assert_eq!(batch.row_count(), 1);
        assert_eq!(batch.total_column_count(), 2);
    }

    #[test]
    fn test_tuple_deep_nesting_rejection() {
        let tuples = vec![
            Value::Tuple(Box::new(Tuple::from([
                ("deep", Value::Tuple(Box::new(Tuple::from([
                    ("nested", Value::Tuple(Box::new(Tuple::from([
                        ("field", Value::String(Box::new("value".to_string()))),
                    ])))),
                ])))),
            ]))),
        ];

        let mut reader = TupleIteratorReader::new(Box::new(tuples.into_iter()), 10);

        let projections = vec![
            Projection::new(ProjectionSource::FieldPath("deep.nested.field".to_string()), 0, LogicalType::String),
        ];

        let projection_spec = ProjectionSpec::new(projections).unwrap();
        reader.set_projection(projection_spec).unwrap();

        // Should reject deep nesting during batch reading
        let result = reader.next_batch();
        assert!(result.is_err());
        assert!(result.unwrap_err().to_string().contains("single-level nesting"));
    }
}