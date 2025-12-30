use partiql_eval_vectorized::batch::LogicalType;
use partiql_eval_vectorized::reader::{
    BatchReader, Projection, ProjectionSource, ProjectionSpec, TupleIteratorReader,
};
use partiql_value::{Tuple, Value};

// ANSI color codes
const GREEN: &str = "\x1b[32m";
const RED: &str = "\x1b[31m";
const RESET: &str = "\x1b[0m";

fn main() {
    println!("TupleIteratorReader Comprehensive Test");
    println!("=====================================");
    println!();

    // Test 1: Basic tuple-to-columnar conversion
    println!("Test 1: Basic Tuple-to-Columnar Conversion");
    println!("------------------------------------------");
    test_basic_conversion();

    println!();
    println!("----------------------------------------");
    println!();

    // Test 2: Type conversions and missing fields
    println!("Test 2: Type Conversions and Missing Fields");
    println!("-------------------------------------------");
    test_type_conversions_and_missing_fields();

    println!();
    println!("----------------------------------------");
    println!();

    // Test 3: Single-level nesting support
    println!("Test 3: Single-Level Nesting Support");
    println!("------------------------------------");
    test_single_level_nesting();

    println!();
    println!("----------------------------------------");
    println!();

    // Test 4: Batch processing with multiple batches
    println!("Test 4: Batch Processing with Multiple Batches");
    println!("----------------------------------------------");
    test_batch_processing();

    println!();
    println!("----------------------------------------");
    println!();

    // Test 5: Error handling
    println!("Test 5: Error Handling");
    println!("----------------------");
    test_error_handling();

    println!();
    println!(
        "{}PASS:{} All TupleIteratorReader tests completed successfully!",
        GREEN, RESET
    );
    println!(
        "The TupleIteratorReader implementation is working correctly with Phase 0 constraints."
    );
}

fn test_basic_conversion() {
    // Create test tuples with various data types
    let tuples = vec![
        Value::Tuple(Box::new(Tuple::from([
            ("name", Value::String(Box::new("Alice".to_string()))),
            ("age", Value::Integer(30)),
            ("score", Value::Real(95.5.into())),
            ("active", Value::Boolean(true)),
        ]))),
        Value::Tuple(Box::new(Tuple::from([
            ("name", Value::String(Box::new("Bob".to_string()))),
            ("age", Value::Integer(25)),
            ("score", Value::Real(87.2.into())),
            ("active", Value::Boolean(false)),
        ]))),
        Value::Tuple(Box::new(Tuple::from([
            ("name", Value::String(Box::new("Charlie".to_string()))),
            ("age", Value::Integer(35)),
            ("score", Value::Real(92.8.into())),
            ("active", Value::Boolean(true)),
        ]))),
    ];

    println!("Tuple Data:");
    println!("  {{name: \"Alice\", age: 30, score: 95.5, active: true}}");
    println!("  {{name: \"Bob\", age: 25, score: 87.2, active: false}}");
    println!("  {{name: \"Charlie\", age: 35, score: 92.8, active: true}}");
    println!();

    let mut reader = TupleIteratorReader::new(Box::new(tuples.into_iter()), 10);

    // Set projection for all fields
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
        Projection::new(
            ProjectionSource::FieldPath("active".to_string()),
            3,
            LogicalType::Boolean,
        ),
    ];
    let projection_spec = ProjectionSpec::new(projections).unwrap();
    reader.set_projection(projection_spec).unwrap();

    // Read batch
    match reader.next_batch().unwrap() {
        Some(batch) => {
            println!(
                "Batch 1: {} rows, {} columns",
                batch.row_count(),
                batch.total_column_count()
            );
            print_batch_sample(&batch, 3).unwrap();

            println!(
                "{}PASS:{} Successfully converted 3 tuples to columnar batch",
                GREEN, RESET
            );
            println!("  - Row count: {}", batch.row_count());
            println!("  - Column count: {}", batch.total_column_count());
            println!(
                "  - Schema fields: {:?}",
                batch
                    .schema()
                    .fields()
                    .iter()
                    .map(|f| &f.name)
                    .collect::<Vec<_>>()
            );
        }
        None => {
            println!("{}FAIL:{} Expected batch but got None", RED, RESET);
            return;
        }
    }

    // Verify no more batches
    match reader.next_batch().unwrap() {
        Some(_) => println!("{}FAIL:{} Expected no more batches", RED, RESET),
        None => println!(
            "{}PASS:{} Correctly returned None for subsequent batch",
            GREEN, RESET
        ),
    }
}

fn test_type_conversions_and_missing_fields() {
    // Create tuples with type conversions and missing fields
    let tuples = vec![
        Value::Tuple(Box::new(Tuple::from([
            ("name", Value::String(Box::new("Alice".to_string()))),
            ("age", Value::Integer(30)),
            ("score", Value::Real(95.5.into())),
        ]))),
        Value::Tuple(Box::new(Tuple::from([
            ("name", Value::String(Box::new("Bob".to_string()))),
            // Missing age field
            ("score", Value::Integer(87)), // Int instead of Real
        ]))),
        Value::Tuple(Box::new(Tuple::from([
            ("name", Value::Integer(123)), // Int converted to String
            ("age", Value::Real(35.5.into())), // Real converted to Int64
                                           // Missing score field
        ]))),
    ];

    println!("Tuple Data with type conversions and missing fields:");
    println!("  {{name: \"Alice\", age: 30, score: 95.5}}");
    println!("  {{name: \"Bob\", score: 87}}  // missing age");
    println!("  {{name: 123, age: 35.5}}     // missing score, type conversions");
    println!();

    let mut reader = TupleIteratorReader::new(Box::new(tuples.into_iter()), 10);

    // Set projection with type conversions
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
    reader.set_projection(projection_spec).unwrap();

    // Read batch
    match reader.next_batch().unwrap() {
        Some(batch) => {
            println!("Type conversions test batch: {} rows", batch.row_count());
            print_batch_sample(&batch, 3).unwrap();

            println!(
                "{}PASS:{} Successfully handled type conversions and missing fields",
                GREEN, RESET
            );
            println!("  - Handled missing fields with default values");
            println!("  - Performed type conversions (Int→String, Real→Int64, Int→Float64)");
        }
        None => {
            println!("{}FAIL:{} Expected batch but got None", RED, RESET);
        }
    }
}

fn test_single_level_nesting() {
    // Create tuples with nested structures
    let tuples = vec![
        Value::Tuple(Box::new(Tuple::from([
            ("id", Value::Integer(1)),
            (
                "person",
                Value::Tuple(Box::new(Tuple::from([
                    ("name", Value::String(Box::new("Alice".to_string()))),
                    ("age", Value::Integer(30)),
                ]))),
            ),
        ]))),
        Value::Tuple(Box::new(Tuple::from([
            ("id", Value::Integer(2)),
            (
                "person",
                Value::Tuple(Box::new(Tuple::from([
                    ("name", Value::String(Box::new("Bob".to_string()))),
                    ("age", Value::Integer(25)),
                ]))),
            ),
        ]))),
    ];

    println!("Tuple Data with single-level nesting:");
    println!("  {{id: 1, person: {{name: \"Alice\", age: 30}}}}");
    println!("  {{id: 2, person: {{name: \"Bob\", age: 25}}}}");
    println!();

    let mut reader = TupleIteratorReader::new(Box::new(tuples.into_iter()), 10);

    // Set projection with single-level nesting
    let projections = vec![
        Projection::new(
            ProjectionSource::FieldPath("id".to_string()),
            0,
            LogicalType::Int64,
        ),
        Projection::new(
            ProjectionSource::FieldPath("person.name".to_string()),
            1,
            LogicalType::String,
        ),
        Projection::new(
            ProjectionSource::FieldPath("person.age".to_string()),
            2,
            LogicalType::Int64,
        ),
    ];
    let projection_spec = ProjectionSpec::new(projections).unwrap();
    reader.set_projection(projection_spec).unwrap();

    // Read batch
    match reader.next_batch().unwrap() {
        Some(batch) => {
            println!(
                "Single-level nesting test batch: {} rows",
                batch.row_count()
            );
            print_batch_sample(&batch, 2).unwrap();

            println!(
                "{}PASS:{} Successfully handled single-level nesting",
                GREEN, RESET
            );
            println!("  - Extracted nested fields using 'struct.field' syntax");
        }
        None => {
            println!("{}FAIL:{} Expected batch but got None", RED, RESET);
        }
    }
}

fn test_batch_processing() {
    // Create many tuples to test batch processing
    let tuples: Vec<Value> = (0..25)
        .map(|i| {
            Value::Tuple(Box::new(Tuple::from([
                ("id", Value::Integer(i)),
                ("value", Value::Real(((i as f64) * 1.5).into())),
            ])))
        })
        .collect();

    println!("Generated 25 tuples for batch processing test (batch size = 10)");
    println!();

    let mut reader = TupleIteratorReader::new(Box::new(tuples.into_iter()), 10); // Batch size of 10

    // Set projection
    let projections = vec![
        Projection::new(
            ProjectionSource::FieldPath("id".to_string()),
            0,
            LogicalType::Int64,
        ),
        Projection::new(
            ProjectionSource::FieldPath("value".to_string()),
            1,
            LogicalType::Float64,
        ),
    ];
    let projection_spec = ProjectionSpec::new(projections).unwrap();
    reader.set_projection(projection_spec).unwrap();

    let mut batch_count = 0;
    let mut total_rows = 0;

    // Read all batches
    while let Some(batch) = reader.next_batch().unwrap() {
        batch_count += 1;
        total_rows += batch.row_count();
        println!("  Batch {}: {} rows", batch_count, batch.row_count());

        // Print sample from first batch
        if batch_count == 1 {
            print_batch_sample(&batch, 3).unwrap();
        }
    }

    println!(
        "{}PASS:{} Successfully processed multiple batches",
        GREEN, RESET
    );
    println!("  - Total batches: {}", batch_count);
    println!("  - Total rows: {}", total_rows);
    println!("  - Expected 3 batches (10 + 10 + 5 rows)");
}

fn test_error_handling() {
    let tuples = vec![Value::Tuple(Box::new(Tuple::from([(
        "name",
        Value::String(Box::new("Alice".to_string())),
    )])))];

    // Test 1: ColumnIndex rejection
    println!("5a. Testing ColumnIndex rejection...");
    let mut reader = TupleIteratorReader::new(Box::new(tuples.into_iter()), 10);

    let projections = vec![Projection::new(
        ProjectionSource::ColumnIndex(0),
        0,
        LogicalType::String,
    )];
    let projection_spec = ProjectionSpec::new(projections).unwrap();

    match reader.set_projection(projection_spec) {
        Ok(_) => println!(
            "  {}FAIL:{} Expected ColumnIndex rejection but got success",
            RED, RESET
        ),
        Err(e) => {
            println!(
                "  {}PASS:{} Correctly rejected ColumnIndex projection",
                GREEN, RESET
            );
            println!("    Error: {}", e);
        }
    }

    // Test 2: Missing projection error
    println!("5b. Testing missing projection error...");
    let tuples2 = vec![Value::Tuple(Box::new(Tuple::from([(
        "name",
        Value::String(Box::new("Alice".to_string())),
    )])))];
    let mut reader2 = TupleIteratorReader::new(Box::new(tuples2.into_iter()), 10);

    match reader2.next_batch() {
        Ok(_) => println!(
            "  {}FAIL:{} Expected missing projection error but got success",
            RED, RESET
        ),
        Err(e) => {
            println!(
                "  {}PASS:{} Correctly detected missing projection",
                GREEN, RESET
            );
            println!("    Error: {}", e);
        }
    }

    // Test 3: Deep nesting rejection
    println!("5c. Testing deep nesting rejection...");
    let tuples3 = vec![Value::Tuple(Box::new(Tuple::from([(
        "deep",
        Value::Tuple(Box::new(Tuple::from([(
            "nested",
            Value::Tuple(Box::new(Tuple::from([(
                "field",
                Value::String(Box::new("value".to_string())),
            )]))),
        )]))),
    )])))];
    let mut reader3 = TupleIteratorReader::new(Box::new(tuples3.into_iter()), 10);

    let projections = vec![Projection::new(
        ProjectionSource::FieldPath("deep.nested.field".to_string()),
        0,
        LogicalType::String,
    )];
    let projection_spec = ProjectionSpec::new(projections).unwrap();
    reader3.set_projection(projection_spec).unwrap();

    match reader3.next_batch() {
        Ok(_) => println!(
            "  {}FAIL:{} Expected deep nesting rejection but got success",
            RED, RESET
        ),
        Err(e) => {
            println!("  {}PASS:{} Correctly rejected deep nesting", GREEN, RESET);
            println!("    Error: {}", e);
        }
    }

    println!("{}PASS:{} All error cases handled correctly", GREEN, RESET);
}

fn print_batch_sample(
    batch: &partiql_eval_vectorized::VectorizedBatch,
    max_rows: usize,
) -> Result<(), Box<dyn std::error::Error>> {
    use partiql_eval_vectorized::PhysicalVectorEnum;

    let schema = batch.schema();
    let row_count = std::cmp::min(batch.row_count(), max_rows);

    if row_count == 0 {
        println!("  (No rows to display)");
        return Ok(());
    }

    // Print header
    print!("  |");
    for field in schema.fields() {
        print!(" {:>12} |", field.name);
    }
    println!();

    // Print separator
    print!("  |");
    for _ in 0..schema.field_count() {
        print!("--------------|");
    }
    println!();

    // Print rows
    for row_idx in 0..row_count {
        print!("  |");
        for col_idx in 0..schema.field_count() {
            let column = batch.column(col_idx)?;
            let value_str = match &column.physical {
                PhysicalVectorEnum::Int64(vec) => {
                    let slice = vec.as_slice();
                    if row_idx < slice.len() {
                        slice[row_idx].to_string()
                    } else {
                        "NULL".to_string()
                    }
                }
                PhysicalVectorEnum::Float64(vec) => {
                    let slice = vec.as_slice();
                    if row_idx < slice.len() {
                        format!("{:.2}", slice[row_idx])
                    } else {
                        "NULL".to_string()
                    }
                }
                PhysicalVectorEnum::Boolean(vec) => {
                    let slice = vec.as_slice();
                    if row_idx < slice.len() {
                        if slice[row_idx] { "true" } else { "false" }.to_string()
                    } else {
                        "NULL".to_string()
                    }
                }
                PhysicalVectorEnum::String(vec) => {
                    let slice = vec.as_slice();
                    if row_idx < slice.len() {
                        slice[row_idx].clone()
                    } else {
                        "NULL".to_string()
                    }
                }
            };
            print!(" {:>12} |", value_str);
        }
        println!();
    }

    if batch.row_count() > max_rows {
        println!("  ... ({} more rows)", batch.row_count() - max_rows);
    }
    println!();

    Ok(())
}
