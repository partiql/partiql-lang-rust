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
        
        // Ensure predicate values match row count
        let predicate_len = predicate_values.len();
        if predicate_len != row_count {
            return Err(EvalError::General(format!(
                "Predicate output length ({}) does not match input batch row count ({})",
                predicate_len, row_count
            )));
        }
        
        // Optimization: Early exit checks - detect all-true or all-false before building selection vector
        // This avoids unnecessary allocation and iteration for common cases
        if row_count == 0 {
            // Empty batch - return as-is
            output_batch.set_selection(None);
            return Ok(Some(output_batch));
        }
        
        // Quick scan to check for all-true or all-false
        // Check first element to short-circuit common cases
        let first_val = predicate_values[0];
        
        // Fast path: all true (common for filters with high selectivity)
        // If first is true, check if all are true
        if first_val && predicate_values.iter().all(|&x| x) {
            output_batch.set_selection(None);
            return Ok(Some(output_batch));
        }
        
        // Fast path: all false (common for very selective filters)
        // If first is false, check if all are false
        if !first_val && predicate_values.iter().all(|&x| !x) {
            output_batch.set_selection(None);
            output_batch.set_row_count(0);
            return Ok(Some(output_batch));
        }
        
        // Partial selection case - need to build selection vector
        // Optimized: Pre-allocate with estimated capacity and use iterator-based collection
        // 
        // Note: 50% is a heuristic estimate. In a production system, this could be:
        // - Based on column statistics (min/max, histograms)
        // - Learned from previous batches (adaptive estimation)
        // - Query-specific (e.g., range predicates often have lower selectivity)
        // 
        // For now, 50% is a reasonable default that balances memory usage
        // (avoiding over-allocation) with performance (minimizing reallocations).
        // Vec will automatically grow if needed, so this is just an optimization.
        let estimated_capacity = (row_count / 2).max(1);
        let mut selected_indices = Vec::with_capacity(estimated_capacity);
        
        // Use iterator with enumerate for better performance
        selected_indices.extend(
            predicate_values
                .iter()
                .enumerate()
                .filter_map(|(i, &val)| if val { Some(i) } else { None })
        );

        // Set selection vector on output batch
        if selected_indices.len() == row_count {
            // All rows pass - no selection needed (shouldn't happen due to early exit, but handle it)
            output_batch.set_selection(None);
        } else if selected_indices.is_empty() {
            // No rows pass (shouldn't happen due to early exit, but handle it)
            output_batch.set_selection(None);
            output_batch.set_row_count(0);
        } else {
            // Partial selection - create selection vector
            // Shrink to fit to save memory if capacity is much larger than needed
            selected_indices.shrink_to_fit();
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
