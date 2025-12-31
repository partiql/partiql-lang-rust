use crate::batch::{SourceTypeDef, VectorizedBatch};
use crate::error::EvalError;
use crate::expr::ExpressionExecutor;
use crate::operators::VectorizedOperator;

/// Project operator - projects columns
pub struct VectorizedProject {
    input: Box<dyn VectorizedOperator>,
    projections: ExpressionExecutor,
    output_schema: SourceTypeDef,
}

impl VectorizedProject {
    /// Create new project operator with compiled projection expressions
    pub fn new(
        input: Box<dyn VectorizedOperator>,
        projections: ExpressionExecutor,
        output_schema: SourceTypeDef,
    ) -> Self {
        Self {
            input,
            projections,
            output_schema,
        }
    }
}

impl VectorizedOperator for VectorizedProject {
    fn next_batch(&mut self) -> Result<Option<VectorizedBatch>, EvalError> {
        // 1. Get next batch from input operator
        let input_batch = match self.input.next_batch()? {
            Some(batch) => batch,
            None => return Ok(None), // End of input stream
        };

        // 2. Create output batch with projection schema
        let row_count = input_batch.row_count();
        let mut output_batch = VectorizedBatch::new(self.output_schema.clone(), row_count);
        output_batch.set_row_count(input_batch.row_count());
        output_batch.set_selection(input_batch.selection().cloned());

        // 3. Execute projection expressions
        // The executor will:
        //   - Evaluate expressions using input columns
        //   - Write results to scratch registers
        //   - Transfer physical buffers from scratch to output batch
        self.projections.execute(&input_batch, &mut output_batch)?;

        // 4. Return the projected batch
        Ok(Some(output_batch))
    }
}
