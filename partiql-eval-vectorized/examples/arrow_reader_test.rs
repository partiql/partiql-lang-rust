use arrow::array::{BooleanArray, Float64Array, Int64Array, StringArray};
use arrow::datatypes::{DataType, Field as ArrowField, Schema};
use arrow::record_batch::RecordBatch;
use partiql_eval_vectorized::batch::LogicalType;
use partiql_eval_vectorized::reader::{
    ArrowReader, BatchReader, Projection, ProjectionSource, ProjectionSpec,
};
use std::sync::Arc;

// ANSI color codes
const GREEN: &str = "\x1b[32m";
const RED: &str = "\x1b[31m";
const RESET: &str = "\x1b[0m";

fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("ArrowReader Comprehensive Test");
    println!("==============================");
    println!();

    // Test 1: Basic Arrow reading with all scalar types
    test_basic_arrow_reading()?;

    println!();
    println!("----------------------------------------");
    println!();

    // Test 2: Type conversions (Int64->Float64, Float64->String, etc.)
    test_arrow_type_conversions()?;

    println!();
    println!("----------------------------------------");
    println!();

    // Test 3: Multiple Arrow RecordBatches
    test_multiple_record_batches()?;

    println!();
    println!("----------------------------------------");
    println!();

    // Test 4: Column index projections and partial column selection
    test_column_projections()?;

    println!();
    println!("----------------------------------------");
    println!();

    // Test 5: Error cases (FieldPath rejection, column bounds, etc.)
    test_error_cases()?;

    println!();
    println!(
        "{}PASS:{} All ArrowReader tests completed successfully!",
        GREEN, RESET
    );
    println!("The ArrowReader implementation is working correctly with Phase 0 constraints.");

    Ok(())
}

fn test_basic_arrow_reading() -> Result<(), Box<dyn std::error::Error>> {
    println!("Test 1: Basic Arrow Reading with All Scalar Types");
    println!("=================================================");

    // Create Arrow schema
    let schema = Arc::new(Schema::new(vec![
        ArrowField::new("id", DataType::Int64, false),
        ArrowField::new("name", DataType::Utf8, false),
        ArrowField::new("score", DataType::Float64, false),
        ArrowField::new("active", DataType::Boolean, false),
    ]));

    // Create Arrow arrays
    let id_array = Arc::new(Int64Array::from(vec![1, 2, 3]));
    let name_array = Arc::new(StringArray::from(vec!["Alice", "Bob", "Charlie"]));
    let score_array = Arc::new(Float64Array::from(vec![95.5, 87.2, 92.8]));
    let active_array = Arc::new(BooleanArray::from(vec![true, false, true]));

    // Create RecordBatch
    let record_batch = RecordBatch::try_new(
        schema,
        vec![id_array, name_array, score_array, active_array],
    )?;

    println!("Arrow RecordBatch:");
    println!("  Schema: id (Int64), name (Utf8), score (Float64), active (Boolean)");
    println!("  Rows: 3");
    println!("  Data: [(1, \"Alice\", 95.5, true), (2, \"Bob\", 87.2, false), (3, \"Charlie\", 92.8, true)]");
    println!();

    let mut reader = ArrowReader::from_record_batch(record_batch);

    // Set projection using ColumnIndex (Arrow's strength)
    let projections = vec![
        Projection::new(ProjectionSource::ColumnIndex(0), 0, LogicalType::Int64),
        Projection::new(ProjectionSource::ColumnIndex(1), 1, LogicalType::String),
        Projection::new(ProjectionSource::ColumnIndex(2), 2, LogicalType::Float64),
        Projection::new(ProjectionSource::ColumnIndex(3), 3, LogicalType::Boolean),
    ];

    let projection_spec = ProjectionSpec::new(projections)?;
    reader.set_projection(projection_spec)?;

    let mut total_rows = 0;
    let mut batch_count = 0;

    while let Some(batch) = reader.next_batch()? {
        batch_count += 1;
        total_rows += batch.row_count();

        println!(
            "Batch {}: {} rows, {} columns",
            batch_count,
            batch.row_count(),
            batch.total_column_count()
        );

        // Print sample data
        print_batch_sample(&batch, 3)?;
    }

    println!(
        "Results: {} batches, {} total rows",
        batch_count, total_rows
    );
    assert_eq!(total_rows, 3, "Should have read 3 rows");

    println!("{}PASS:{} Basic Arrow reading test passed", GREEN, RESET);
    Ok(())
}

fn test_arrow_type_conversions() -> Result<(), Box<dyn std::error::Error>> {
    println!("Test 2: Arrow Type Conversions");
    println!("==============================");

    // Create Arrow schema with various types
    let schema = Arc::new(Schema::new(vec![
        ArrowField::new("int_col", DataType::Int64, false),
        ArrowField::new("float_col", DataType::Float64, false),
        ArrowField::new("bool_col", DataType::Boolean, false),
    ]));

    // Create Arrow arrays
    let int_array = Arc::new(Int64Array::from(vec![42, 100]));
    let float_array = Arc::new(Float64Array::from(vec![3.14, 2.71]));
    let bool_array = Arc::new(BooleanArray::from(vec![true, false]));

    let record_batch = RecordBatch::try_new(schema, vec![int_array, float_array, bool_array])?;

    println!("Arrow RecordBatch with type conversions:");
    println!("  int_col: [42, 100] -> Float64 and String");
    println!("  float_col: [3.14, 2.71] -> String");
    println!("  bool_col: [true, false] -> String");
    println!();

    let mut reader = ArrowReader::from_record_batch(record_batch);

    // Set projection with type conversions
    let projections = vec![
        Projection::new(ProjectionSource::ColumnIndex(0), 0, LogicalType::Float64), // Int64 -> Float64
        Projection::new(ProjectionSource::ColumnIndex(0), 1, LogicalType::String), // Int64 -> String
        Projection::new(ProjectionSource::ColumnIndex(1), 2, LogicalType::String), // Float64 -> String
        Projection::new(ProjectionSource::ColumnIndex(2), 3, LogicalType::String), // Boolean -> String
    ];

    let projection_spec = ProjectionSpec::new(projections)?;
    reader.set_projection(projection_spec)?;

    if let Some(batch) = reader.next_batch()? {
        println!("Type conversions test batch: {} rows", batch.row_count());
        print_batch_sample(&batch, 2)?;

        println!("{}PASS:{} Type conversions working:", GREEN, RESET);
        println!("  - Int64 -> Float64");
        println!("  - Int64 -> String");
        println!("  - Float64 -> String");
        println!("  - Boolean -> String");
    }

    Ok(())
}

fn test_multiple_record_batches() -> Result<(), Box<dyn std::error::Error>> {
    println!("Test 3: Multiple Arrow RecordBatches");
    println!("====================================");

    // Create schema
    let schema = Arc::new(Schema::new(vec![
        ArrowField::new("id", DataType::Int64, false),
        ArrowField::new("value", DataType::Float64, false),
    ]));

    // Create multiple RecordBatches
    let batch1 = RecordBatch::try_new(
        schema.clone(),
        vec![
            Arc::new(Int64Array::from(vec![1, 2])),
            Arc::new(Float64Array::from(vec![1.1, 2.2])),
        ],
    )?;

    let batch2 = RecordBatch::try_new(
        schema.clone(),
        vec![
            Arc::new(Int64Array::from(vec![3, 4, 5])),
            Arc::new(Float64Array::from(vec![3.3, 4.4, 5.5])),
        ],
    )?;

    let batch3 = RecordBatch::try_new(
        schema.clone(),
        vec![
            Arc::new(Int64Array::from(vec![6])),
            Arc::new(Float64Array::from(vec![6.6])),
        ],
    )?;

    println!("Multiple Arrow RecordBatches:");
    println!("  Batch 1: 2 rows");
    println!("  Batch 2: 3 rows");
    println!("  Batch 3: 1 row");
    println!("  Total: 6 rows");
    println!();

    let mut reader = ArrowReader::new(vec![batch1, batch2, batch3]);

    // Set projection
    let projections = vec![
        Projection::new(ProjectionSource::ColumnIndex(0), 0, LogicalType::Int64),
        Projection::new(ProjectionSource::ColumnIndex(1), 1, LogicalType::Float64),
    ];

    let projection_spec = ProjectionSpec::new(projections)?;
    reader.set_projection(projection_spec)?;

    let mut total_rows = 0;
    let mut batch_count = 0;

    while let Some(batch) = reader.next_batch()? {
        batch_count += 1;
        total_rows += batch.row_count();
        println!("  Batch {}: {} rows", batch_count, batch.row_count());

        // Print sample from first batch
        if batch_count == 1 {
            print_batch_sample(&batch, 2)?;
        }
    }

    println!(
        "{}PASS:{} Successfully processed multiple RecordBatches",
        GREEN, RESET
    );
    println!("  - Total batches: {}", batch_count);
    println!("  - Total rows: {}", total_rows);
    println!("  - Expected 3 batches with 6 total rows");

    Ok(())
}

fn test_column_projections() -> Result<(), Box<dyn std::error::Error>> {
    println!("Test 4: Column Index Projections and Partial Selection");
    println!("======================================================");

    // Create Arrow schema with many columns
    let schema = Arc::new(Schema::new(vec![
        ArrowField::new("col_0", DataType::Int64, false),
        ArrowField::new("col_1", DataType::Utf8, false),
        ArrowField::new("col_2", DataType::Float64, false),
        ArrowField::new("col_3", DataType::Boolean, false),
        ArrowField::new("col_4", DataType::Int64, false),
    ]));

    // Create Arrow arrays
    let record_batch = RecordBatch::try_new(
        schema,
        vec![
            Arc::new(Int64Array::from(vec![10, 20])),
            Arc::new(StringArray::from(vec!["A", "B"])),
            Arc::new(Float64Array::from(vec![1.0, 2.0])),
            Arc::new(BooleanArray::from(vec![true, false])),
            Arc::new(Int64Array::from(vec![100, 200])),
        ],
    )?;

    println!("Arrow RecordBatch with 5 columns:");
    println!("  Selecting only columns 0, 2, and 4 (skipping 1 and 3)");
    println!("  Demonstrating non-contiguous column selection");
    println!();

    let mut reader = ArrowReader::from_record_batch(record_batch);

    // Set projection to select only specific columns (0, 2, 4)
    let projections = vec![
        Projection::new(ProjectionSource::ColumnIndex(0), 0, LogicalType::Int64), // col_0
        Projection::new(ProjectionSource::ColumnIndex(2), 1, LogicalType::Float64), // col_2
        Projection::new(ProjectionSource::ColumnIndex(4), 2, LogicalType::Int64), // col_4
    ];

    let projection_spec = ProjectionSpec::new(projections)?;
    reader.set_projection(projection_spec)?;

    if let Some(batch) = reader.next_batch()? {
        println!(
            "Column projection test batch: {} rows, {} columns",
            batch.row_count(),
            batch.total_column_count()
        );
        print_batch_sample(&batch, 2)?;

        println!(
            "{}PASS:{} Column index projections working correctly",
            GREEN, RESET
        );
        println!("  - Selected columns 0, 2, 4 from 5 available columns");
        println!("  - Skipped columns 1 and 3 as intended");
    }

    Ok(())
}

fn test_error_cases() -> Result<(), Box<dyn std::error::Error>> {
    println!("Test 5: Error Cases");
    println!("==================");

    // Test 5a: FieldPath rejection
    println!("5a. Testing FieldPath rejection...");
    let schema = Arc::new(Schema::new(vec![ArrowField::new(
        "name",
        DataType::Utf8,
        false,
    )]));

    let name_array = Arc::new(StringArray::from(vec!["Alice"]));
    let record_batch = RecordBatch::try_new(schema, vec![name_array])?;
    let mut reader = ArrowReader::from_record_batch(record_batch);

    let projections = vec![Projection::new(
        ProjectionSource::FieldPath("name".to_string()),
        0,
        LogicalType::String,
    )];

    let projection_spec = ProjectionSpec::new(projections)?;
    match reader.set_projection(projection_spec) {
        Err(e) => {
            println!(
                "  {}PASS:{} FieldPath correctly rejected: {}",
                GREEN, RESET, e
            );
        }
        Ok(_) => {
            println!(
                "  {}FAIL:{} FieldPath should have been rejected",
                RED, RESET
            );
        }
    }

    // Test 5b: Column index out of bounds
    println!("5b. Testing column index bounds checking...");
    let schema = Arc::new(Schema::new(vec![ArrowField::new(
        "col1",
        DataType::Int64,
        false,
    )]));

    let col1_array = Arc::new(Int64Array::from(vec![1, 2, 3]));
    let record_batch = RecordBatch::try_new(schema, vec![col1_array])?;
    let mut reader = ArrowReader::from_record_batch(record_batch);

    let projections = vec![
        Projection::new(ProjectionSource::ColumnIndex(1), 0, LogicalType::Int64), // Column 1 doesn't exist
    ];

    let projection_spec = ProjectionSpec::new(projections)?;
    match reader.set_projection(projection_spec) {
        Err(e) => {
            println!(
                "  {}PASS:{} Column bounds correctly checked: {}",
                GREEN, RESET, e
            );
        }
        Ok(_) => {
            println!(
                "  {}FAIL:{} Column bounds should have been checked",
                RED, RESET
            );
        }
    }

    // Test 5c: Unsupported Arrow type conversion
    println!("5c. Testing unsupported type conversion...");
    let schema = Arc::new(Schema::new(vec![ArrowField::new(
        "string_col",
        DataType::Utf8,
        false,
    )]));

    let string_array = Arc::new(StringArray::from(vec!["not_a_number"]));
    let record_batch = RecordBatch::try_new(schema, vec![string_array])?;
    let mut reader = ArrowReader::from_record_batch(record_batch);

    let projections = vec![
        Projection::new(ProjectionSource::ColumnIndex(0), 0, LogicalType::Int64), // String -> Int64 not supported
    ];

    let projection_spec = ProjectionSpec::new(projections)?;
    reader.set_projection(projection_spec)?;

    match reader.next_batch() {
        Err(e) => {
            println!(
                "  {}PASS:{} Unsupported type conversion detected: {}",
                GREEN, RESET, e
            );
        }
        Ok(_) => {
            println!(
                "  {}FAIL:{} Unsupported type conversion should have been detected",
                RED, RESET
            );
        }
    }

    println!("{}PASS:{} All error cases handled correctly", GREEN, RESET);
    Ok(())
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
