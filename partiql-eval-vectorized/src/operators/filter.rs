use crate::batch::{LogicalType, SelectionVector, SourceTypeDef, VectorizedBatch};
use crate::error::EvalError;
use crate::expr::ExpressionExecutor;
use crate::operators::VectorizedOperator;

/// Filter operator - applies predicate to filter rows
pub struct VectorizedFilter {
    input: Box<dyn VectorizedOperator>,
    predicate: ExpressionExecutor,
    output_schema: SourceTypeDef,
}

impl VectorizedFilter {
    /// Create new filter operator with compiled predicate expression
    pub fn new(input: Box<dyn VectorizedOperator>, predicate: ExpressionExecutor) -> Self {
        // Cache the input schema since filter doesn't change it
        let output_schema = input.output_schema().clone();
        Self {
            input,
            predicate,
            output_schema,
        }
    }
}

impl VectorizedOperator for VectorizedFilter {
    fn open(&mut self) -> Result<(), EvalError> {
        // Open the child operator
        self.input.open()
    }
    
    fn next_batch(&mut self) -> Result<Option<VectorizedBatch>, EvalError> {
        // 1. Get next batch from input operator
        let input_batch = match self.input.next_batch()? {
            Some(batch) => batch,
            None => return Ok(None), // End of input stream
        };

        // 2. Clone the input batch (zero-copy - just clones Arc references)
        let mut output_batch = input_batch.clone();
        let row_count = output_batch.row_count();

        // 3. Create temporary batch for predicate evaluation
        // The predicate result will be a single Boolean column
        let predicate_schema = crate::batch::SourceTypeDef::new(vec![crate::batch::Field {
            name: "predicate".to_string(),
            type_info: LogicalType::Boolean,
        }]);
        let mut predicate_output = VectorizedBatch::new(predicate_schema, row_count);
        predicate_output.set_row_count(row_count);

        // 4. Evaluate predicate expression on input batch
        self.predicate
            .execute(&input_batch, &mut predicate_output)?;

        // 5. Build selection vector from predicate results
        let predicate_col = predicate_output.column(0)?;
        let predicate_bool = predicate_col
            .physical
            .as_boolean()
            .ok_or_else(|| EvalError::General("Expected Boolean predicate result".to_string()))?;
        let predicate_values = predicate_bool.as_slice();

        let mut selected_indices = Vec::new();
        for i in 0..row_count {
            if predicate_values[i] {
                selected_indices.push(i);
            }
        }

        // 6. Set selection vector on output batch
        // If all rows selected, keep selection as None (optimization)
        // If no rows selected, set to None (empty result)
        if selected_indices.len() == row_count {
            // All rows pass - no selection needed
            output_batch.set_selection(None);
        } else if selected_indices.is_empty() {
            // No rows pass - still return batch but with empty selection
            output_batch.set_selection(None);
            output_batch.set_row_count(0);
        } else {
            // Partial selection - create selection vector
            output_batch.set_selection(Some(SelectionVector {
                indices: selected_indices,
            }));
        }

        // 7. Return filtered batch
        Ok(Some(output_batch))
    }

    fn output_schema(&self) -> &SourceTypeDef {
        &self.output_schema
    }
    
    fn close(&mut self) -> Result<(), EvalError> {
        // Close the child operator
        self.input.close()
    }
}
