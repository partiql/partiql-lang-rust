use crate::batch::{SourceTypeDef, VectorizedBatch};
use crate::error::EvalError;
use crate::operators::VectorizedOperator;

/// Compiled vectorized query plan ready for execution
pub struct VectorizedPlan {
    root: Box<dyn VectorizedOperator>,
    output_schema: SourceTypeDef,
}

impl VectorizedPlan {
    /// Create a new vectorized plan
    pub fn new(root: Box<dyn VectorizedOperator>, output_schema: SourceTypeDef) -> Self {
        Self {
            root,
            output_schema,
        }
    }

    /// Get the output schema of this plan
    pub fn output_schema(&self) -> &SourceTypeDef {
        &self.output_schema
    }

    /// Execute the plan, returning an iterator over result batches
    pub fn execute(&mut self) -> PlanExecutor<'_> {
        PlanExecutor { plan: self }
    }
}

/// Iterator over batches produced by executing a plan
pub struct PlanExecutor<'a> {
    plan: &'a mut VectorizedPlan,
}

impl<'a> Iterator for PlanExecutor<'a> {
    type Item = Result<VectorizedBatch, EvalError>;

    fn next(&mut self) -> Option<Self::Item> {
        // next_batch returns Result<Option<Batch>>, we need Option<Result<Batch>>
        match self.plan.root.next_batch() {
            Ok(Some(batch)) => Some(Ok(batch)),
            Ok(None) => None,
            Err(e) => Some(Err(e)),
        }
    }
}
