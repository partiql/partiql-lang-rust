use crate::batch::{LogicalType, VectorizedBatch};
use crate::error::EvalError;
use crate::expr::VectorizedExpr;
use crate::functions::VectorizedFn;

/// Function call expression
///
/// Evaluates input expressions first, then calls the function with those results.
/// Input expressions write to pre-allocated scratch columns, and this expression
/// reads from those columns and writes the function result to its output column.
#[derive(Debug)]
pub struct FnCallExpr {
    function: Box<dyn VectorizedFn>,
    /// Input expressions that will be evaluated before calling the function
    inputs: Vec<Box<dyn VectorizedExpr>>,
    /// Column indices where input expressions will write their results
    input_cols: Vec<usize>,
    output_type: LogicalType,
}

impl FnCallExpr {
    /// Create new function call expression
    pub fn new(
        function: Box<dyn VectorizedFn>,
        inputs: Vec<Box<dyn VectorizedExpr>>,
        input_cols: Vec<usize>,
        output_type: LogicalType,
    ) -> Self {
        assert_eq!(
            inputs.len(),
            input_cols.len(),
            "Must have same number of inputs and input columns"
        );
        Self {
            function,
            inputs,
            input_cols,
            output_type,
        }
    }
}

impl VectorizedExpr for FnCallExpr {
    fn eval(&self, batch: &mut VectorizedBatch, output_col: usize) -> Result<(), EvalError> {
        // Evaluate each input expression into its designated column
        for (input_expr, &input_col) in self.inputs.iter().zip(self.input_cols.iter()) {
            input_expr.eval(batch, input_col)?;
        }

        // Gather cloned input columns (cheap with Arc-based Vector)
        // This avoids borrow checker issues when we need mutable access to output
        let input_vecs: Vec<_> = self
            .input_cols
            .iter()
            .map(|&col_idx| batch.column(col_idx).map(|c| c.clone()))
            .collect::<Result<Vec<_>, _>>()?;

        // Create references for the function call
        let input_refs: Vec<_> = input_vecs.iter().collect();

        // Call function with inputs, writing to output column
        let output = batch.column_mut(output_col)?;
        self.function.execute(&input_refs, output)?;

        Ok(())
    }

    fn output_type(&self) -> LogicalType {
        self.output_type
    }
}
