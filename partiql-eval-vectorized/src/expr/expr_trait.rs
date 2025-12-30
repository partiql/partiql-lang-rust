use crate::batch::{LogicalType, VectorizedBatch};
use crate::error::EvalError;
use std::fmt::Debug;

/// Expression that can be evaluated on a batch
pub trait VectorizedExpr: Debug {
    /// Evaluate expression on batch, writing result to the specified output column
    ///
    /// # Arguments
    /// * `batch` - The batch containing input columns and where output will be written
    /// * `output_col` - Column index where the result should be written
    fn eval(&self, batch: &mut VectorizedBatch, output_col: usize) -> Result<(), EvalError>;

    /// Get the output type of this expression
    fn output_type(&self) -> LogicalType;
}
