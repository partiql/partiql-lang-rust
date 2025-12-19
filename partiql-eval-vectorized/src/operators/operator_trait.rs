use crate::batch::VectorizedBatch;
use crate::error::EvalError;

/// Physical operator that produces batches
pub trait VectorizedOperator {
    /// Get next batch of results
    fn next_batch(&mut self) -> Result<Option<VectorizedBatch>, EvalError>;
}
