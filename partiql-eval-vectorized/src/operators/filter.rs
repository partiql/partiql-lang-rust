use crate::batch::{SelectionVector, VectorizedBatch, LogicalType};
use crate::error::EvalError;
use crate::expr::ExpressionExecutor;
use crate::operators::VectorizedOperator;

/// Filter operator - applies predicate to filter rows
pub struct VectorizedFilter {
    input: Box<dyn VectorizedOperator>,
    predicate: ExpressionExecutor,
}

impl VectorizedFilter {
    /// Create new filter operator with compiled predicate expression
    pub fn new(input: Box<dyn VectorizedOperator>, predicate: ExpressionExecutor) -> Self {
        Self {
            input,
            predicate,
        }
    }
}

impl VectorizedOperator for VectorizedFilter {
    fn next_batch(&mut self) -> Result<Option<VectorizedBatch>, EvalError> {
        // 1. Get next batch from input operator
        let mut input_batch = match self.input.next_batch()? {
            Some(batch) => batch,
            None => return Ok(None), // End of input stream
        };
        
        // 2. Create temporary output batch for predicate evaluation
        // The predicate result will be a single Boolean column
        let predicate_schema = crate::batch::SourceTypeDef::new(vec![
            crate::batch::Field {
                name: "predicate".to_string(),
                type_info: LogicalType::Boolean,
            },
        ]);
        let row_count = input_batch.row_count();
        let mut predicate_output = VectorizedBatch::new(predicate_schema, row_count);
        predicate_output.set_row_count(row_count);  // Set row count for predicate evaluation
        
        // 3. Evaluate predicate expression
        self.predicate.execute(&input_batch, &mut predicate_output)?;
        
        // 4. Build selection vector from predicate results
        let predicate_col = predicate_output.column(0)?;
        let predicate_bool = predicate_col.physical.as_boolean()
            .ok_or_else(|| EvalError::General("Expected Boolean predicate result".to_string()))?;
        let predicate_values = predicate_bool.as_slice();
        
        // Collect indices where predicate is true
        let mut selected_indices = Vec::new();
        for i in 0..row_count {
            if predicate_values[i] {
                selected_indices.push(i);
            }
        }
        
        // 5. Set selection vector on input batch
        // If all rows selected, keep selection as None (optimization)
        // If no rows selected, set to None (empty result)
        if selected_indices.len() == row_count {
            // All rows pass - no selection needed
            input_batch.set_selection(None);
        } else if selected_indices.is_empty() {
            // No rows pass - still return batch but with empty selection
            input_batch.set_selection(None);
            input_batch.set_row_count(0);
        } else {
            // Partial selection - create selection vector
            input_batch.set_selection(Some(SelectionVector {
                indices: selected_indices,
            }));
        }
        
        // 6. Return filtered batch
        Ok(Some(input_batch))
    }
}
