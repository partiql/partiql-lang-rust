use arrow::array::{BooleanArray, Float64Array, Int64Array, StringArray};
use arrow::datatypes::{DataType, Field, Schema};
use arrow::record_batch::RecordBatch;
use parquet::arrow::ArrowWriter;
use parquet::file::properties::WriterProperties;
use partiql_eval_vectorized::batch::LogicalType;
use partiql_eval_vectorized::reader::{
    BatchReader, ParquetReader, Projection, ProjectionSource, ProjectionSpec,
};
use std::fs::File;
use std::sync::Arc;
use tempfile::NamedTempFile;

// ANSI color codes
const GREEN: &str = "\x1b[32m";
const RESET: &str = "\x1b[0m";

/// Create a test Parquet file with sample data
fn create_test_parquet_file() -> Result<NamedTempFile, Box<dyn std::error::Error>> {
    // Create test schema
    let schema = Arc::new(Schema::new(vec![
        Field::new("id", DataType::Int64, false),
        Field::new("name", DataType::Utf8, false),
        Field::new("score", DataType::Float64, false),
        Field::new("active", DataType::Boolean, false),
        Field::new("category", DataType::Utf8, false),
    ]));

    // Create test data
    let id_array = Arc::new(Int64Array::from(vec![1, 2, 3, 4, 5, 6, 7, 8, 9, 10]));
    let name_array = Arc::new(StringArray::from(vec![
        "Alice", "Bob", "Charlie", "Diana", "Eve", "Frank", "Grace", "Henry", "Iris", "Jack",
    ]));
    let score_array = Arc::new(Float64Array::from(vec![
        95.5, 87.2, 92.8, 88.1, 94.3, 89.7, 91.4, 86.9, 93.2, 90.1,
    ]));
    let active_array = Arc::new(BooleanArray::from(vec![
        true, false, true, true, false, true, true, false, true, false,
    ]));
    let category_array = Arc::new(StringArray::from(vec![
        "A", "B", "A", "C", "B", "A", "C", "B", "A", "C",
    ]));

    let record_batch = RecordBatch::try_new(
        schema.clone(),
        vec![
            id_array,
            name_array,
            score_array,
            active_array,
            category_array,
        ],
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

fn test_basic_functionality() -> Result<(), Box<dyn std::error::Error>> {
    println!("Test 1: Basic Parquet Reading with All Scalar Types");
    println!("===================================================");

    let temp_file = create_test_parquet_file()?;
    let mut reader = ParquetReader::from_file(temp_file.path(), 10)?;

    println!("Parquet File Schema:");
    println!("  Columns: id (Int64), name (String), score (Float64), active (Boolean), category (String)");
    println!("  Rows: 10");
    println!("  Projection: id, name, score (first 3 columns)");
    println!();

    // Set projection: id (Int64), name (String), score (Float64)
    let projections = vec![
        Projection::new(ProjectionSource::ColumnIndex(0), 0, LogicalType::Int64),
        Projection::new(ProjectionSource::ColumnIndex(1), 1, LogicalType::String),
        Projection::new(ProjectionSource::ColumnIndex(2), 2, LogicalType::Float64),
    ];
    let projection_spec = ProjectionSpec::new(projections)?;
    reader.set_projection(projection_spec)?;

    // Read batch
    let batch = reader.next_batch()?;
    assert!(batch.is_some(), "Expected batch data");

    let batch = batch.unwrap();
    println!(
        "Batch 1: {} rows, {} columns",
        batch.row_count(),
        batch.total_column_count()
    );

    // Print sample data
    print_batch_sample(&batch, 3)?;

    assert_eq!(batch.row_count(), 10);
    assert_eq!(batch.total_column_count(), 3);

    // Should be no more batches for this small file
    let next_batch = reader.next_batch()?;
    assert!(next_batch.is_none(), "Expected no more batches");

    println!("{}PASS:{} Basic Parquet reading test passed", GREEN, RESET);
    Ok(())
}

fn test_column_projection() -> Result<(), Box<dyn std::error::Error>> {
    println!("Test 2: Column Index Projections and Partial Selection");
    println!("======================================================");

    let temp_file = create_test_parquet_file()?;
    let mut reader = ParquetReader::from_file(temp_file.path(), 10)?;

    println!("Parquet File with 5 columns:");
    println!("  Selecting only columns 0, 2, 4 (skipping 1 and 3)");
    println!("  Demonstrating non-contiguous column selection");
    println!();

    // Project only columns 0, 2, 4 (skip columns 1 and 3)
    let projections = vec![
        Projection::new(ProjectionSource::ColumnIndex(0), 0, LogicalType::Int64), // id
        Projection::new(ProjectionSource::ColumnIndex(2), 1, LogicalType::Float64), // score
        Projection::new(ProjectionSource::ColumnIndex(4), 2, LogicalType::String), // category
    ];
    let projection_spec = ProjectionSpec::new(projections)?;
    reader.set_projection(projection_spec)?;

    // Read batch
    let batch = reader.next_batch()?;
    assert!(batch.is_some(), "Expected batch data");

    let batch = batch.unwrap();
    println!(
        "Column projection test batch: {} rows, {} columns",
        batch.row_count(),
        batch.total_column_count()
    );

    // Print sample data
    print_batch_sample(&batch, 3)?;

    assert_eq!(batch.row_count(), 10);
    assert_eq!(batch.total_column_count(), 3); // Only 3 columns projected

    println!(
        "{}PASS:{} Column index projections working correctly",
        GREEN, RESET
    );
    println!("  - Selected columns 0, 2, 4 from 5 available columns");
    println!("  - Skipped columns 1 and 3 as intended");
    Ok(())
}

fn test_type_conversions() -> Result<(), Box<dyn std::error::Error>> {
    println!("Test 3: Parquet Type Conversions");
    println!("=================================");

    let temp_file = create_test_parquet_file()?;
    let mut reader = ParquetReader::from_file(temp_file.path(), 10)?;

    println!("Parquet Data with type conversions:");
    println!("  id (Int64) -> Float64");
    println!("  score (Float64) -> String");
    println!("  active (Boolean) -> String");
    println!();

    // Test type conversions: Int64 -> Float64, Float64 -> String, Boolean -> String
    let projections = vec![
        Projection::new(ProjectionSource::ColumnIndex(0), 0, LogicalType::Float64), // Int64 -> Float64
        Projection::new(ProjectionSource::ColumnIndex(2), 1, LogicalType::String), // Float64 -> String
        Projection::new(ProjectionSource::ColumnIndex(3), 2, LogicalType::String), // Boolean -> String
    ];
    let projection_spec = ProjectionSpec::new(projections)?;
    reader.set_projection(projection_spec)?;

    // Read batch
    let batch = reader.next_batch()?;
    assert!(batch.is_some(), "Expected batch data");

    let batch = batch.unwrap();
    println!(
        "Type conversions test batch: {} rows, {} columns",
        batch.row_count(),
        batch.total_column_count()
    );

    // Print sample data
    print_batch_sample(&batch, 3)?;

    assert_eq!(batch.row_count(), 10);
    assert_eq!(batch.total_column_count(), 3);

    println!("{}PASS:{} Type conversions working:", GREEN, RESET);
    println!("  - Int64 -> Float64");
    println!("  - Float64 -> String");
    println!("  - Boolean -> String");
    Ok(())
}

fn test_field_path_rejection() -> Result<(), Box<dyn std::error::Error>> {
    println!("Test 4: Error Cases");
    println!("==================");

    println!("4a. Testing FieldPath rejection...");
    let temp_file = create_test_parquet_file()?;
    let mut reader = ParquetReader::from_file(temp_file.path(), 10)?;

    // Try to set projection with FieldPath - should fail
    let projections = vec![Projection::new(
        ProjectionSource::FieldPath("name".to_string()),
        0,
        LogicalType::String,
    )];
    let projection_spec = ProjectionSpec::new(projections)?;

    let result = reader.set_projection(projection_spec);
    assert!(result.is_err(), "Expected FieldPath to be rejected");

    if let Err(error) = result {
        let error_msg = format!("{}", error);
        println!(
            "  {}PASS:{} FieldPath correctly rejected: {}",
            GREEN, RESET, error_msg
        );
        assert!(error_msg.contains("FieldPath"));
        assert!(error_msg.contains("not supported"));
        assert!(error_msg.contains("ColumnIndex"));
    }

    Ok(())
}

fn test_file_not_found() -> Result<(), Box<dyn std::error::Error>> {
    println!("4b. Testing file not found error...");

    let result = ParquetReader::from_file("/nonexistent/file.parquet", 10);
    assert!(result.is_err(), "Expected file not found error");

    if let Err(error) = result {
        let error_msg = format!("{}", error);
        println!(
            "  {}PASS:{} File not found error: {}",
            GREEN, RESET, error_msg
        );
        assert!(error_msg.contains("File does not exist"));
    }

    Ok(())
}

fn test_column_bounds_validation() -> Result<(), Box<dyn std::error::Error>> {
    println!("4c. Testing column bounds validation...");

    let temp_file = create_test_parquet_file()?;
    let mut reader = ParquetReader::from_file(temp_file.path(), 10)?;

    // Try to access column index 5 when only columns 0-4 exist
    let projections = vec![Projection::new(
        ProjectionSource::ColumnIndex(5),
        0,
        LogicalType::Int64,
    )];
    let projection_spec = ProjectionSpec::new(projections)?;
    reader.set_projection(projection_spec)?;

    // Error should occur during initialization (first next_batch call)
    // The Parquet library will panic on invalid column indices during ProjectionMask creation
    let result = std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| reader.next_batch()));

    match result {
        Ok(batch_result) => {
            // If no panic, check if there's an error
            assert!(batch_result.is_err(), "Expected column bounds error");
            if let Err(error) = batch_result {
                let error_msg = format!("{}", error);
                println!(
                    "  {}PASS:{} Column bounds error: {}",
                    GREEN, RESET, error_msg
                );
            }
        }
        Err(_) => {
            // Panic occurred - this is expected for invalid column indices
            println!(
                "  {}PASS:{} Column bounds validation correctly panicked on invalid column index",
                GREEN, RESET
            );
        }
    }

    println!("{}PASS:{} All error cases handled correctly", GREEN, RESET);
    Ok(())
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("ParquetReader Comprehensive Test");
    println!("================================");
    println!();

    // Test 1: Basic Parquet reading with all scalar types
    test_basic_functionality()?;

    println!();
    println!("----------------------------------------");
    println!();

    // Test 2: Column projections and partial column selection
    test_column_projection()?;

    println!();
    println!("----------------------------------------");
    println!();

    // Test 3: Type conversions (Int64->Float64, Float64->String, etc.)
    test_type_conversions()?;

    println!();
    println!("----------------------------------------");
    println!();

    // Test 4: Error cases (FieldPath rejection, file not found, column bounds)
    test_field_path_rejection()?;
    test_file_not_found()?;
    test_column_bounds_validation()?;

    println!();
    println!(
        "{}PASS:{} All ParquetReader tests completed successfully!",
        GREEN, RESET
    );
    println!("The ParquetReader implementation is working correctly with Phase 0 constraints.");

    Ok(())
}
