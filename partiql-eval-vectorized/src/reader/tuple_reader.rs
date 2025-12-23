use crate::batch::{SourceTypeDef, VectorizedBatch};
use crate::error::EvalError;
use crate::reader::BatchReader;

/// Placeholder for Tuple type (will come from partiql-value)
pub type Tuple = ();

/// Convert PartiQL Value/Tuple stream to columnar batches
pub struct TupleIteratorReader {
    _iter: Box<dyn Iterator<Item = Tuple>>,
    schema: SourceTypeDef,
    batch_size: usize,
    batches_generated: usize,
    max_batches: usize,
}

impl TupleIteratorReader {
    /// Create new reader
    pub fn new(
        iter: Box<dyn Iterator<Item = Tuple>>,
        schema: SourceTypeDef,
        batch_size: usize,
    ) -> Self {
        Self {
            _iter: iter,
            schema,
            batch_size,
            batches_generated: 0,
            max_batches: 1000, // Generate 1000 batches for testing
        }
    }
}

impl BatchReader for TupleIteratorReader {
    fn next_batch(&mut self) -> Result<Option<VectorizedBatch>, EvalError> {
        // Check if we've generated enough batches
        if self.batches_generated >= self.max_batches {
            return Ok(None);
        }
        
        // TODO: Replace this mock data generation with actual tuple-to-columnar conversion
        // This generates synthetic batches for testing the vectorized execution pipeline
        // Pass batches_generated as start offset to generate incrementing values across batches
        let batch = generate_mock_batch(&self.schema, self.batch_size, (self.batches_generated * self.batch_size) as i64)?;
        
        self.batches_generated += 1;
        Ok(Some(batch))
    }

    fn schema(&self) -> &SourceTypeDef {
        &self.schema
    }
}

/// TODO: Replace this with actual data reading logic
/// Generates a mock batch with synthetic data for testing
fn generate_mock_batch(
    schema: &SourceTypeDef,
    batch_size: usize,
    start: i64
) -> Result<VectorizedBatch, EvalError> {
    use crate::batch::{PhysicalVectorEnum, LogicalType};
    
    let mut batch = VectorizedBatch::new(schema.clone(), batch_size);
    
    // Generate synthetic data for each column based on its type
    for (col_idx, field) in schema.fields().iter().enumerate() {
        let vector = batch.column_mut(col_idx)?;
        
        match field.type_info {
            LogicalType::Int64 => {
                if let PhysicalVectorEnum::Int64(v) = &mut vector.physical {
                    // Generate sequential integers for testing
                    // Column 0 (a): 0, 1, 2, ..., batch_size-1
                    // Column 1 (b): 100, 101, 102, ..., 100+batch_size-1
                    let offset = col_idx as i64 * 100;
                    let slice = v.as_mut_slice();
                    for i in 0..batch_size {
                        slice[i] = start + offset + i as i64;
                    }
                }
            }
            LogicalType::Float64 => {
                if let PhysicalVectorEnum::Float64(v) = &mut vector.physical {
                    // Generate sequential floats for testing
                    let offset = col_idx as f64 * 100.0;
                    let slice = v.as_mut_slice();
                    for i in 0..batch_size {
                        slice[i] = start as f64 + offset + i as f64;
                    }
                }
            }
            LogicalType::Boolean => {
                if let PhysicalVectorEnum::Boolean(v) = &mut vector.physical {
                    // Alternate true/false
                    let slice = v.as_mut_slice();
                    for i in 0..batch_size {
                        slice[i] = i % 2 == 0;
                    }
                }
            }
            LogicalType::String => {
                if let PhysicalVectorEnum::String(v) = &mut vector.physical {
                    // Generate simple string values
                    let slice = v.as_mut_slice();
                    let offset = col_idx as i64 * 100;
                    for i in 0..batch_size {
                        let value = start + offset + i as i64;
                        slice[i] = format!("value_{}", value);
                    }
                }
            }
        }
    }
    
    batch.set_row_count(batch_size);
    Ok(batch)
}
