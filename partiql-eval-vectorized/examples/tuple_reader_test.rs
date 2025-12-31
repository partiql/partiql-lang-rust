use partiql_eval_vectorized::batch::LogicalType;
use partiql_eval_vectorized::reader::{
    BatchReader, InMemoryGeneratedReader, Projection, ProjectionSource, ProjectionSpec,
};

// ANSI color codes
const GREEN: &str = "\x1b[32m";
const RED: &str = "\x1b[31m";
const BLUE: &str = "\x1b[34m";
const RESET: &str = "\x1b[0m";

fn main() {
    println!("{}InMemoryGeneratedReader Fake Data Generation Test{}", BLUE, RESET);
    println!("===================================================");
    println!();
    println!("InMemoryGeneratedReader now generates fake data with two Int64 columns:");
    println!("  - Column 'a': starts at 0, increments by 1");
    println!("  - Column 'b': starts at 100, increments by 1");
    println!();
    println!("Default configuration:");
    println!("  - batch_size: 1024 rows per batch");
    println!("  - num_batches: 10,000 batches");
    println!("  - Total rows: 10,240,000");
    println!();

    // Test 1: Basic fake data generation
    println!("{}Test 1: Basic Fake Data Generation{}", BLUE, RESET);
    println!("----------------------------------");
    test_basic_fake_data();

    println!();
    println!("----------------------------------------");
    println!();

    // Test 2: Custom configuration
    println!("{}Test 2: Custom Configuration{}", BLUE, RESET);
    println!("----------------------------");
    test_custom_config();

    println!();
    println!("----------------------------------------");
    println!();

    // Test 3: Single column projection
    println!("{}Test 3: Single Column Projection{}", BLUE, RESET);
    println!("--------------------------------");
    test_single_column();

    println!();
    println!("----------------------------------------");
    println!();

    // Test 4: Multiple batches
    println!("{}Test 4: Multiple Batches{}", BLUE, RESET);
    println!("------------------------");
    test_multiple_batches();

    println!();
    println!("----------------------------------------");
    println!();

    // Test 5: Large batch processing
    println!("{}Test 5: Large Batch Processing (Performance Test){}", BLUE, RESET);
    println!("------------------------------------------------");
    test_large_batch_processing();

    println!();
    println!(
        "{}PASS:{} All InMemoryGeneratedReader tests completed successfully!",
        GREEN, RESET
    );
}

fn test_basic_fake_data() {
    println!("Creating reader with default configuration (batch_size=1024, num_batches=10,000)");
    let mut reader = InMemoryGeneratedReader::new();

    // Set projection for both columns
    let projections = vec![
        Projection::new(
            ProjectionSource::FieldPath("a".to_string()),
            0,
            LogicalType::Int64,
        ),
        Projection::new(
            ProjectionSource::FieldPath("b".to_string()),
            1,
            LogicalType::Int64,
        ),
    ];
    let projection_spec = ProjectionSpec::new(projections).unwrap();
    reader.set_projection(projection_spec).unwrap();

    // Read first batch
    println!("Reading first batch...");
    match reader.next_batch().unwrap() {
        Some(batch) => {
            println!(
                "  Batch 1: {} rows, {} columns",
                batch.row_count(),
                batch.total_column_count()
            );
            print_batch_sample(&batch, 5).unwrap();

            println!(
                "{}PASS:{} Successfully generated first batch with fake data",
                GREEN, RESET
            );
        }
        None => {
            println!("{}FAIL:{} Expected batch but got None", RED, RESET);
            return;
        }
    }
}

fn test_custom_config() {
    println!("Creating reader with custom config (batch_size=10, num_batches=3)");
    let mut reader = InMemoryGeneratedReader::with_config(10, 3);

    // Set projection for both columns
    let projections = vec![
        Projection::new(
            ProjectionSource::FieldPath("a".to_string()),
            0,
            LogicalType::Int64,
        ),
        Projection::new(
            ProjectionSource::FieldPath("b".to_string()),
            1,
            LogicalType::Int64,
        ),
    ];
    let projection_spec = ProjectionSpec::new(projections).unwrap();
    reader.set_projection(projection_spec).unwrap();

    // Read first batch
    println!("Reading first batch (rows 0-9)...");
    match reader.next_batch().unwrap() {
        Some(batch) => {
            println!("  Batch 1: {} rows", batch.row_count());
            print_batch_sample(&batch, 10).unwrap();

            // Verify data values
            use partiql_eval_vectorized::PhysicalVectorEnum;
            let col_a = batch.column(0).unwrap();
            let col_b = batch.column(1).unwrap();

            if let PhysicalVectorEnum::Int64(v) = &col_a.physical {
                let slice = v.as_slice();
                assert_eq!(slice[0], 0, "First value of 'a' should be 0");
                assert_eq!(slice[9], 9, "Last value of 'a' should be 9");
            }

            if let PhysicalVectorEnum::Int64(v) = &col_b.physical {
                let slice = v.as_slice();
                assert_eq!(slice[0], 100, "First value of 'b' should be 100");
                assert_eq!(slice[9], 109, "Last value of 'b' should be 109");
            }

            println!(
                "{}PASS:{} Data values verified: a=[0..9], b=[100..109]",
                GREEN, RESET
            );
        }
        None => {
            println!("{}FAIL:{} Expected batch but got None", RED, RESET);
            return;
        }
    }

    // Read second batch
    println!("Reading second batch (rows 10-19)...");
    match reader.next_batch().unwrap() {
        Some(batch) => {
            use partiql_eval_vectorized::PhysicalVectorEnum;
            let col_a = batch.column(0).unwrap();

            if let PhysicalVectorEnum::Int64(v) = &col_a.physical {
                let slice = v.as_slice();
                assert_eq!(slice[0], 10, "Batch 2: First value of 'a' should be 10");
                println!(
                    "{}PASS:{} Data continues correctly from previous batch",
                    GREEN, RESET
                );
            }
        }
        None => {
            println!("{}FAIL:{} Expected batch but got None", RED, RESET);
        }
    }
}

fn test_single_column() {
    println!("Creating reader and projecting only column 'b'");
    let mut reader = InMemoryGeneratedReader::with_config(5, 1);

    // Project only column "b"
    let projections = vec![Projection::new(
        ProjectionSource::FieldPath("b".to_string()),
        0,
        LogicalType::Int64,
    )];
    let projection_spec = ProjectionSpec::new(projections).unwrap();
    reader.set_projection(projection_spec).unwrap();

    // Read batch
    match reader.next_batch().unwrap() {
        Some(batch) => {
            println!("  Batch: {} rows, {} column", batch.row_count(), batch.total_column_count());
            print_batch_sample(&batch, 5).unwrap();

            // Verify only column "b" is present
            use partiql_eval_vectorized::PhysicalVectorEnum;
            let col = batch.column(0).unwrap();
            if let PhysicalVectorEnum::Int64(v) = &col.physical {
                let slice = v.as_slice();
                assert_eq!(slice[0], 100);
                assert_eq!(slice[4], 104);
            }

            println!(
                "{}PASS:{} Single column projection works correctly",
                GREEN, RESET
            );
        }
        None => {
            println!("{}FAIL:{} Expected batch but got None", RED, RESET);
        }
    }
}

fn test_multiple_batches() {
    println!("Creating reader with 5 batches of 10 rows each");
    let mut reader = InMemoryGeneratedReader::with_config(10, 5);

    // Set projection
    let projections = vec![
        Projection::new(
            ProjectionSource::FieldPath("a".to_string()),
            0,
            LogicalType::Int64,
        ),
        Projection::new(
            ProjectionSource::FieldPath("b".to_string()),
            1,
            LogicalType::Int64,
        ),
    ];
    let projection_spec = ProjectionSpec::new(projections).unwrap();
    reader.set_projection(projection_spec).unwrap();

    let mut batch_count = 0;
    let mut total_rows = 0;

    println!("Reading all batches...");
    while let Some(batch) = reader.next_batch().unwrap() {
        batch_count += 1;
        total_rows += batch.row_count();
        println!("  Batch {}: {} rows", batch_count, batch.row_count());

        // Print first batch
        if batch_count == 1 {
            print_batch_sample(&batch, 3).unwrap();
        }
    }

    println!(
        "{}PASS:{} Processed {} batches with {} total rows",
        GREEN, RESET, batch_count, total_rows
    );
    assert_eq!(batch_count, 5, "Should have 5 batches");
    assert_eq!(total_rows, 50, "Should have 50 total rows");
}

fn test_large_batch_processing() {
    println!("Creating reader with default config (1024 rows × 10,000 batches = 10,240,000 rows)");
    println!("Processing first 10 batches...");
    
    let mut reader = InMemoryGeneratedReader::new();

    // Set projection
    let projections = vec![
        Projection::new(
            ProjectionSource::FieldPath("a".to_string()),
            0,
            LogicalType::Int64,
        ),
        Projection::new(
            ProjectionSource::FieldPath("b".to_string()),
            1,
            LogicalType::Int64,
        ),
    ];
    let projection_spec = ProjectionSpec::new(projections).unwrap();
    reader.set_projection(projection_spec).unwrap();

    let start = std::time::Instant::now();
    let mut batch_count = 0;
    let mut total_rows = 0;

    // Process first 10 batches for demonstration
    for _ in 0..10 {
        if let Some(batch) = reader.next_batch().unwrap() {
            batch_count += 1;
            total_rows += batch.row_count();
        } else {
            break;
        }
    }

    let duration = start.elapsed();

    println!("  Processed {} batches", batch_count);
    println!("  Total rows: {}", total_rows);
    println!("  Time: {:.3}ms", duration.as_secs_f64() * 1000.0);
    println!(
        "  Throughput: {:.2} million rows/sec",
        (total_rows as f64) / duration.as_secs_f64() / 1_000_000.0
    );

    println!(
        "{}PASS:{} Large batch processing completed successfully",
        GREEN, RESET
    );
    println!("  Note: This is a mock data generator. Actual readers will have different performance characteristics.");
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
                        if slice[row_idx] {
                            "true"
                        } else {
                            "false"
                        }
                        .to_string()
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
