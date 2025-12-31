use crate::batch::{SourceTypeDef, VectorizedBatch};
use crate::error::EvalError;

/// Physical operator that produces batches
pub trait VectorizedOperator {
    /// Initialize operator and open children
    fn open(&mut self) -> Result<(), EvalError>;
    
    /// Get next batch of results
    fn next_batch(&mut self) -> Result<Option<VectorizedBatch>, EvalError>;
    
    /// Get the output schema of this operator
    fn output_schema(&self) -> &SourceTypeDef;
    
    /// Clean up operator and close children
    fn close(&mut self) -> Result<(), EvalError>;
}
