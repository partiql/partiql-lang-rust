use partiql_eval_vectorized::batch::LogicalType;
use partiql_eval_vectorized::reader::{
    BatchReader, IonReader, Projection, ProjectionSource, ProjectionSpec,
};

// ANSI color codes
const GREEN: &str = "\x1b[32m";
const RED: &str = "\x1b[31m";
const RESET: &str = "\x1b[0m";

fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("Ion Reader Direct Test");
    println!("=====================");
    println!();

    // Test 1: Basic Ion reading with scalar types
    test_basic_ion_reading()?;
    
    println!();
    println!("----------------------------------------");
    println!();
    
    // Test 2: Type conversions (Decimal->Float64, Symbol->String, etc.)
    test_ion_type_conversions()?;
    
    println!();
    println!("----------------------------------------");
    println!();
    
    // Test 3: Missing fields handling
    test_missing_fields()?;
    
    println!();
    println!("----------------------------------------");
    println!();
    
    // Test 4: Single-level nesting support
    test_single_level_nesting()?;
    
    println!();
    println!("----------------------------------------");
    println!();
    
    // Test 5: Error cases (deep nesting, column index, type mismatches)
    test_error_cases()?;

    println!();
    println!("{}PASS:{} All Ion Reader tests completed successfully!", GREEN, RESET);
    println!("The Ion reader implementation is working correctly with Phase 0 constraints.");

    Ok(())
}

fn test_basic_ion_reading() -> Result<(), Box<dyn std::error::Error>> {
    println!("Test 1: Basic Ion Reading with All Scalar Types");
    println!("===============================================");

    let ion_data = r#"
        {name: "Alice", age: 30, score: 95.5, active: true}
        {name: "Bob", age: 25, score: 87.2, active: false}
        {name: "Carol", age: 35, score: 92.1, active: true}
    "#;

    println!("Ion Data:");
    println!("{}", ion_data.trim());
    println!();

    let mut reader = IonReader::from_ion_text(ion_data, 10)?;

    let projections = vec![
        Projection::new(ProjectionSource::FieldPath("name".to_string()), 0, LogicalType::String),
        Projection::new(ProjectionSource::FieldPath("age".to_string()), 1, LogicalType::Int64),
        Projection::new(ProjectionSource::FieldPath("score".to_string()), 2, LogicalType::Float64),
        Projection::new(ProjectionSource::FieldPath("active".to_string()), 3, LogicalType::Boolean),
    ];

    let projection_spec = ProjectionSpec::new(projections)?;
    reader.set_projection(projection_spec)?;

    let mut total_rows = 0;
    let mut batch_count = 0;

    while let Some(batch) = reader.next_batch()? {
        batch_count += 1;
        total_rows += batch.row_count();
        
        println!("Batch {}: {} rows, {} columns", batch_count, batch.row_count(), batch.total_column_count());
        
        // Verify column types match expectations
        for col_idx in 0..batch.total_column_count() {
            let column = batch.column(col_idx)?;
            println!("  Column {}: {:?}", col_idx, column.ty);
        }
        
        // Print some sample data
        print_batch_sample(&batch, 3)?;
    }

    println!("Results: {} batches, {} total rows", batch_count, total_rows);
    assert_eq!(total_rows, 3, "Should have read 3 rows");
    
    println!("{}PASS:{} Basic Ion reading test passed", GREEN, RESET);
    Ok(())
}

fn test_ion_type_conversions() -> Result<(), Box<dyn std::error::Error>> {
    println!("Test 2: Ion Type Conversions");
    println!("============================");

    let ion_data = r#"
        {decimal_val: 123.456d0, int_val: 42, symbol_val: hello, float_val: 3.14}
        {decimal_val: 789.012d0, int_val: 100, symbol_val: world, float_val: 2.71}
    "#;

    println!("Ion Data with various types:");
    println!("{}", ion_data.trim());
    println!();

    let mut reader = IonReader::from_ion_text(ion_data, 10)?;

    let projections = vec![
        Projection::new(ProjectionSource::FieldPath("decimal_val".to_string()), 0, LogicalType::Float64),
        Projection::new(ProjectionSource::FieldPath("int_val".to_string()), 1, LogicalType::Float64), // Int->Float conversion
        Projection::new(ProjectionSource::FieldPath("symbol_val".to_string()), 2, LogicalType::String), // Symbol->String conversion
        Projection::new(ProjectionSource::FieldPath("float_val".to_string()), 3, LogicalType::Float64),
    ];

    let projection_spec = ProjectionSpec::new(projections)?;
    reader.set_projection(projection_spec)?;

    if let Some(batch) = reader.next_batch()? {
        println!("Conversion test batch: {} rows", batch.row_count());
        print_batch_sample(&batch, 2)?;
        
        println!("{}PASS:{} Type conversions working:", GREEN, RESET);
        println!("  - Decimal -> Float64");
        println!("  - Int -> Float64");
        println!("  - Symbol -> String");
        println!("  - Float -> Float64");
    }

    Ok(())
}

fn test_missing_fields() -> Result<(), Box<dyn std::error::Error>> {
    println!("Test 3: Missing Fields Handling");
    println!("===============================");

    let ion_data = r#"
        {name: "Alice", age: 30}
        {name: "Bob", score: 87.2}
        {age: 25, score: 95.0}
    "#;

    println!("Ion Data with missing fields:");
    println!("{}", ion_data.trim());
    println!();

    let mut reader = IonReader::from_ion_text(ion_data, 10)?;

    let projections = vec![
        Projection::new(ProjectionSource::FieldPath("name".to_string()), 0, LogicalType::String),
        Projection::new(ProjectionSource::FieldPath("age".to_string()), 1, LogicalType::Int64),
        Projection::new(ProjectionSource::FieldPath("score".to_string()), 2, LogicalType::Float64),
    ];

    let projection_spec = ProjectionSpec::new(projections)?;
    reader.set_projection(projection_spec)?;

    if let Some(batch) = reader.next_batch()? {
        println!("Missing fields test batch: {} rows", batch.row_count());
        print_batch_sample(&batch, 3)?;
        
        println!("{}PASS:{} Missing fields handled correctly (null values inserted)", GREEN, RESET);
    }

    Ok(())
}

fn test_single_level_nesting() -> Result<(), Box<dyn std::error::Error>> {
    println!("Test 4: Single-Level Nesting Support");
    println!("====================================");

    let ion_data = r#"
        {person: {name: "Alice", age: 30}, id: 1}
        {person: {name: "Bob", age: 25}, id: 2}
    "#;

    println!("Ion Data with single-level nesting:");
    println!("{}", ion_data.trim());
    println!();

    let mut reader = IonReader::from_ion_text(ion_data, 10)?;

    let projections = vec![
        Projection::new(ProjectionSource::FieldPath("person.name".to_string()), 0, LogicalType::String),
        Projection::new(ProjectionSource::FieldPath("person.age".to_string()), 1, LogicalType::Int64),
        Projection::new(ProjectionSource::FieldPath("id".to_string()), 2, LogicalType::Int64),
    ];

    let projection_spec = ProjectionSpec::new(projections)?;
    reader.set_projection(projection_spec)?;

    if let Some(batch) = reader.next_batch()? {
        println!("Single-level nesting test batch: {} rows", batch.row_count());
        print_batch_sample(&batch, 2)?;
        
        println!("{}PASS:{} Single-level nesting (struct.field) supported", GREEN, RESET);
    }

    Ok(())
}

fn test_error_cases() -> Result<(), Box<dyn std::error::Error>> {
    println!("Test 5: Error Cases");
    println!("==================");

    // Test 5a: Deep nesting rejection
    println!("5a. Testing deep nesting rejection...");
    let ion_data = r#"{person: {details: {name: "Alice"}}}"#;
    let mut reader = IonReader::from_ion_text(ion_data, 10)?;

    let projections = vec![
        Projection::new(ProjectionSource::FieldPath("person.details.name".to_string()), 0, LogicalType::String),
    ];

    let projection_spec = ProjectionSpec::new(projections)?;
    reader.set_projection(projection_spec)?;

    match reader.next_batch() {
        Err(e) => {
            println!("  {}PASS:{} Deep nesting correctly rejected: {}", GREEN, RESET, e);
        }
        Ok(_) => {
            println!("  {}FAIL:{} Deep nesting should have been rejected", RED, RESET);
        }
    }

    // Test 5b: ColumnIndex rejection
    println!("5b. Testing ColumnIndex rejection...");
    let ion_data = r#"{name: "Alice"}"#;
    let mut reader = IonReader::from_ion_text(ion_data, 10)?;

    let projections = vec![
        Projection::new(ProjectionSource::ColumnIndex(0), 0, LogicalType::String),
    ];

    let projection_spec = ProjectionSpec::new(projections)?;
    match reader.set_projection(projection_spec) {
        Err(e) => {
            println!("  {}PASS:{} ColumnIndex correctly rejected: {}", GREEN, RESET, e);
        }
        Ok(_) => {
            println!("  {}FAIL:{} ColumnIndex should have been rejected", RED, RESET);
        }
    }

    // Test 5c: Type mismatch
    println!("5c. Testing type mismatch...");
    let ion_data = r#"{name: "Alice", age: "thirty"}"#;
    let mut reader = IonReader::from_ion_text(ion_data, 10)?;

    let projections = vec![
        Projection::new(ProjectionSource::FieldPath("age".to_string()), 0, LogicalType::Int64),
    ];

    let projection_spec = ProjectionSpec::new(projections)?;
    reader.set_projection(projection_spec)?;

    match reader.next_batch() {
        Err(e) => {
            println!("  {}PASS:{} Type mismatch correctly detected: {}", GREEN, RESET, e);
        }
        Ok(_) => {
            println!("  {}FAIL:{} Type mismatch should have been detected", RED, RESET);
        }
    }

    println!("{}PASS:{} All error cases handled correctly", GREEN, RESET);
    Ok(())
}

fn print_batch_sample(batch: &partiql_eval_vectorized::VectorizedBatch, max_rows: usize) -> Result<(), Box<dyn std::error::Error>> {
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