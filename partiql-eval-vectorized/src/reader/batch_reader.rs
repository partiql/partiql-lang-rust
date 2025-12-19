use crate::batch::{SourceTypeDef, VectorizedBatch};
use crate::error::EvalError;

/// Reads data in batches
pub trait BatchReader {
    /// Get next batch, returns None when exhausted
    fn next_batch(&mut self) -> Result<Option<VectorizedBatch>, EvalError>;

    /// Get schema
    fn schema(&self) -> &SourceTypeDef;
}
