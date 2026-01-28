use arrow::array::Int64Array;
use arrow::datatypes::{DataType, Field, Schema};
use arrow::record_batch::RecordBatch;
use arrow_ipc::writer::FileWriter;
use parquet::arrow::ArrowWriter;
use parquet::file::properties::WriterProperties;
use std::fs::File;
use std::io::{BufWriter, Write};
use std::path::Path;
use std::sync::Arc;

// Ion writer imports
use ion_rs::IonWriter;
use ion_rs::element::writer::TextKind;

/// Generate mock data in Arrow, Parquet, and Ion formats
/// 
/// Usage:
///   generate_data --format <arrow|parquet|ion|all> --output-dir <dir> [--batch-size <size>] [--num-batches <count>]
fn main() -> Result<(), Box<dyn std::error::Error>> {
    let args: Vec<String> = std::env::args().collect();
    
    if args.len() < 5 {
        eprintln!("Usage: {} --format <arrow|parquet|ion|all> --output-dir <dir> [--batch-size <size>] [--num-batches <count>]", args[0]);
        eprintln!("Example: {} --format all --output-dir ./data --batch-size 1024 --num-batches 10000", args[0]);
        std::process::exit(1);
    }

    let mut format = "all".to_string();
    let mut output_dir = ".".to_string();
    let mut batch_size = 1024;
    let mut num_batches = 10_000;

    // Parse arguments
    let mut i = 1;
    while i < args.len() {
        match args[i].as_str() {
            "--format" => {
                if i + 1 < args.len() {
                    format = args[i + 1].clone();
                    i += 2;
                } else {
                    eprintln!("Error: --format requires a value");
                    std::process::exit(1);
                }
            }
            "--output-dir" => {
                if i + 1 < args.len() {
                    output_dir = args[i + 1].clone();
                    i += 2;
                } else {
                    eprintln!("Error: --output-dir requires a value");
                    std::process::exit(1);
                }
            }
            "--batch-size" => {
                if i + 1 < args.len() {
                    batch_size = args[i + 1].parse().unwrap_or(1024);
                    i += 2;
                } else {
                    eprintln!("Error: --batch-size requires a value");
                    std::process::exit(1);
                }
            }
            "--num-batches" => {
                if i + 1 < args.len() {
                    num_batches = args[i + 1].parse().unwrap_or(10_000);
                    i += 2;
                } else {
                    eprintln!("Error: --num-batches requires a value");
                    std::process::exit(1);
                }
            }
            _ => {
                eprintln!("Unknown argument: {}", args[i]);
                std::process::exit(1);
            }
        }
    }

    let total_rows = batch_size * num_batches;
    println!("Generating data:");
    println!("  Format: {}", format);
    println!("  Output directory: {}", output_dir);
    println!("  Batch size: {}", batch_size);
    println!("  Number of batches: {}", num_batches);
    println!("  Total rows: {}", total_rows);
    println!();

    // Create output directory if it doesn't exist
    std::fs::create_dir_all(&output_dir)?;

    match format.as_str() {
        "arrow" => generate_arrow(&output_dir, batch_size, num_batches)?,
        "parquet" => generate_parquet(&output_dir, batch_size, num_batches)?,
        "ion" | "iont" => generate_ion_text(&output_dir, batch_size, num_batches)?,
        "ionb" => generate_ion(&output_dir, batch_size, num_batches)?,
        "all" => {
            generate_arrow(&output_dir, batch_size, num_batches)?;
            generate_parquet(&output_dir, batch_size, num_batches)?;
            generate_ion(&output_dir, batch_size, num_batches)?;
            generate_ion_text(&output_dir, batch_size, num_batches)?;
        }
        _ => {
            eprintln!("Error: Unknown format '{}'. Use: arrow, parquet, ion, ionb, or all", format);
            std::process::exit(1);
        }
    }

    println!("Data generation complete!");
    Ok(())
}

fn generate_arrow(output_dir: &str, batch_size: usize, num_batches: usize) -> Result<(), Box<dyn std::error::Error>> {
    println!("Generating Arrow format...");
    
    // Create Arrow schema
    let schema = Arc::new(Schema::new(vec![
        Field::new("a", DataType::Int64, false),
        Field::new("b", DataType::Int64, false),
    ]));

    // Generate batches and write to file
    let file_path = Path::new(output_dir).join(format!("data_b{}_n{}.arrow", batch_size, num_batches));
    let file = File::create(&file_path)?;
    let mut writer = FileWriter::try_new(file, &schema)?;

    let mut current_row = 0i64;
    for batch_idx in 0..num_batches {
        // Generate data for this batch
        let mut a_values = Vec::with_capacity(batch_size);
        let mut b_values = Vec::with_capacity(batch_size);

        for i in 0..batch_size {
            let row_num = current_row + i as i64;
            a_values.push(row_num);
            b_values.push(row_num + 100);
        }

        let a_array = Arc::new(Int64Array::from(a_values));
        let b_array = Arc::new(Int64Array::from(b_values));

        let record_batch = RecordBatch::try_new(
            schema.clone(),
            vec![a_array, b_array],
        )?;

        writer.write(&record_batch)?;
        current_row += batch_size as i64;
    }

    writer.finish()?;
    println!("  Created: {}", file_path.display());
    Ok(())
}

fn generate_parquet(output_dir: &str, batch_size: usize, num_batches: usize) -> Result<(), Box<dyn std::error::Error>> {
    println!("Generating Parquet format...");
    
    // Create Arrow schema
    let schema = Arc::new(Schema::new(vec![
        Field::new("a", DataType::Int64, false),
        Field::new("b", DataType::Int64, false),
    ]));

    // Generate batches and write to file
    let file_path = Path::new(output_dir).join(format!("data_b{}_n{}.parquet", batch_size, num_batches));
    let file = File::create(&file_path)?;
    let props = WriterProperties::builder().build();
    let mut writer = ArrowWriter::try_new(file, schema.clone(), Some(props))?;

    let mut current_row = 0i64;
    for batch_idx in 0..num_batches {
        // Generate data for this batch
        let mut a_values = Vec::with_capacity(batch_size);
        let mut b_values = Vec::with_capacity(batch_size);

        for i in 0..batch_size {
            let row_num = current_row + i as i64;
            a_values.push(row_num);
            b_values.push(row_num + 100);
        }

        let a_array = Arc::new(Int64Array::from(a_values));
        let b_array = Arc::new(Int64Array::from(b_values));

        let record_batch = RecordBatch::try_new(
            schema.clone(),
            vec![a_array, b_array],
        )?;

        writer.write(&record_batch)?;
        current_row += batch_size as i64;
    }

    writer.close()?;
    println!("  Created: {}", file_path.display());
    Ok(())
}

fn generate_ion(output_dir: &str, batch_size: usize, num_batches: usize) -> Result<(), Box<dyn std::error::Error>> {
    println!("Generating Ion format (binary)...");
    
    // Use .10n extension for binary Ion format
    let file_path = Path::new(output_dir).join(format!("data_b{}_n{}.10n", batch_size, num_batches));
    let file = File::create(&file_path)?;
    
    // CRITICAL: Use binary writer for symbol IDs
    // BinaryWriterBuilder creates binary Ion format (not text)
    let mut writer = ion_rs::BinaryWriterBuilder::new()
        .build(file)?;

    let mut current_row = 0i64;
    for _batch_idx in 0..num_batches {
        for i in 0..batch_size {
            let row_num = current_row + i as i64;
            
            // Write struct with field names (encoded as symbol IDs in binary format)
            writer.step_in(ion_rs::IonType::Struct)?;
            
            writer.set_field_name("a");
            writer.write_i64(row_num)?;
            
            writer.set_field_name("b");
            writer.write_i64(row_num + 100)?;
            
            writer.step_out()?;
        }
        current_row += batch_size as i64;
    }
    
    writer.flush()?;
    println!("  Created: {} (binary format with symbol IDs)", file_path.display());
    Ok(())
}

fn generate_ion_text(output_dir: &str, batch_size: usize, num_batches: usize) -> Result<(), Box<dyn std::error::Error>> {
    println!("Generating Ion format (text)...");
    
    // Use .ion extension for text Ion format
    let file_path = Path::new(output_dir).join(format!("data_b{}_n{}.ion", batch_size, num_batches));
    let file = File::create(&file_path)?;
    let mut writer = BufWriter::new(file);
    
    let mut current_row = 0i64;
    for _batch_idx in 0..num_batches {
        for i in 0..batch_size {
            let row_num = current_row + i as i64;
            
            // Write struct in compact text Ion format with newline after each struct
            writeln!(writer, "{{a: {}, b: {}}}", row_num, row_num + 100)?;
        }
        current_row += batch_size as i64;
    }
    
    writer.flush()?;
    println!("  Created: {} (compact text format)", file_path.display());
    Ok(())
}
