use crate::batch::{LogicalType, VectorizedBatch};
use crate::error::EvalError;
use crate::expr::VectorizedExpr;

/// Column reference expression - references a column by index
#[derive(Debug)]
pub struct ColumnRef {
    /// Index of the column to reference (can be source or scratch column)
    column_idx: usize,
    type_info: LogicalType,
}

impl ColumnRef {
    /// Create new column reference
    pub fn new(column_idx: usize, type_info: LogicalType) -> Self {
        Self {
            column_idx,
            type_info,
        }
    }
}

impl VectorizedExpr for ColumnRef {
    fn eval(&self, batch: &mut VectorizedBatch, output_col: usize) -> Result<(), EvalError> {
        // With Arc-based Vector, we can cheaply clone to avoid borrow checker issues
        let source = batch.column(self.column_idx)?.clone();
        let output = batch.column_mut(output_col)?;
        output.copy_from(&source)?;
        Ok(())
    }

    fn output_type(&self) -> LogicalType {
        self.type_info
    }
}
