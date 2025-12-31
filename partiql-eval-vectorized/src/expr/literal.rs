use crate::batch::{LogicalType, Vector, VectorizedBatch};
use crate::error::EvalError;
use crate::expr::VectorizedExpr;

/// Literal value expression - a constant that gets broadcast to all rows
#[derive(Debug)]
pub struct LiteralExpr {
    value: Vector,
    type_info: LogicalType,
}

impl LiteralExpr {
    /// Create new literal expression
    pub fn new(value: Vector, type_info: LogicalType) -> Self {
        Self { value, type_info }
    }
}

impl VectorizedExpr for LiteralExpr {
    fn eval(&self, batch: &mut VectorizedBatch, output_col: usize) -> Result<(), EvalError> {
        // TODO: Implement broadcast of literal to batch size
        let _row_count = batch.row_count();
        let output = batch.column_mut(output_col)?;
        output.copy_from(&self.value)?;
        Ok(())
    }

    fn output_type(&self) -> LogicalType {
        self.type_info
    }
}
