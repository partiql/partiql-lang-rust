use crate::batch::{VectorizedBatch, Vector, LogicalType};
use crate::error::EvalError;
use smallvec::SmallVec;

/// Constant values used in expressions
#[derive(Clone, Debug, PartialEq)]
pub enum ConstantValue {
    Int32(i32),
    Int64(i64),
    Float64(f64),
    Boolean(bool),
}

/// Input to an expression operation
#[derive(Clone, Debug, PartialEq)]
pub enum ExprInput {
    /// Read from input batch column
    InputCol(usize),
    /// Read from scratch register
    Scratch(usize),
    /// Read from constant value
    Constant(ConstantValue),
}

/// Expression operation types
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum ExprOp {
    // Identity operation (pass-through)
    Identity,
    
    // Arithmetic operations
    AddI32,
    AddI64,
    AddF64,
    SubI32,
    SubI64,
    SubF64,
    MulI32,
    MulI64,
    MulF64,
    DivI32,
    DivI64,
    DivF64,
    
    // Comparison operations
    GtI32,
    GtI64,
    GtF64,
    LtI32,
    LtI64,
    LtF64,
    EqI32,
    EqI64,
    EqF64,
    EqBool,
    NeI32,
    NeI64,
    NeF64,
    NeBool,
    GeI32,
    GeI64,
    GeF64,
    LeI32,
    LeI64,
    LeF64,
    
    // Logical operations
    AndBool,
    OrBool,
    NotBool,
}

/// A single compiled expression operation
#[derive(Clone, Debug)]
pub struct CompiledExpr {
    /// The operation to perform
    pub op: ExprOp,
    /// Input operands (uses SmallVec to avoid heap allocation for common binary ops)
    pub inputs: SmallVec<[ExprInput; 2]>,
    /// Output scratch register index
    pub output: usize,
}

/// Register-based expression executor
/// 
/// Executes a sequence of compiled expressions using a register machine model.
/// This avoids recursion overhead and enables better vectorization.
pub struct ExpressionExecutor {
    /// Compiled expression sequence
    exprs: Vec<CompiledExpr>,
    /// Scratch registers for intermediate results
    scratch: Vec<Vector>,
    /// Mapping from scratch register indices to output data chunk column indices
    /// Used to copy final expression results to the output batch
    outputs: Vec<usize>,
}

impl ExpressionExecutor {
    /// Create a new expression executor
    pub fn new(
        exprs: Vec<CompiledExpr>,
        num_scratch: usize,
        outputs: Vec<usize>,
    ) -> Self {
        // TODO: Determine proper types and sizes for scratch vectors from expression analysis
        // Pre-allocate scratch vectors
        let scratch = vec![Vector::new(LogicalType::Int64, 0); num_scratch];
        
        Self {
            exprs,
            scratch,
            outputs,
        }
    }
    
    /// Execute the compiled expressions
    /// 
    /// Reads from input batch, writes intermediate results to scratch,
    /// then transfers final scratch results to output batch (zero-copy).
    pub fn execute(
        &mut self,
        input: &VectorizedBatch,
        output: &mut VectorizedBatch,
    ) -> Result<(), EvalError> {
        let batch_size = input.row_count();
        
        // Ensure scratch vectors have correct size
        for scratch_vec in &mut self.scratch {
            // Resize if needed (stub - would need proper resizing logic)
            if scratch_vec.len() != batch_size {
                // In real implementation: resize or recreate vector
            }
        }
        
        // Phase 1: Execute all expressions (writes to scratch only)
        let exprs = self.exprs.clone();
        for compiled in &exprs {
            self.execute_op(compiled, input)?;
        }
        
        // Phase 2: Transfer final results from scratch to output (zero-copy!)
        // Clone physical buffers (just Arc ref count increment)
        for (output_col_idx, &scratch_idx) in self.outputs.iter().enumerate() {
            let output_col = output.column_mut(output_col_idx)?;
            // Transfer the physical buffer by cloning the Arc (cheap - just ref count)
            output_col.physical = self.scratch[scratch_idx].physical.clone();
        }
        
        Ok(())
    }
    
    /// Execute a single compiled operation
    /// All outputs go to scratch registers only.
    fn execute_op(
        &mut self,
        compiled: &CompiledExpr,
        input: &VectorizedBatch,
    ) -> Result<(), EvalError> {
        // Execute operation
        match compiled.op {
            ExprOp::Identity => {
                // Identity: copy input to output (pass-through)
                if compiled.inputs.len() != 1 {
                    return Err(EvalError::General(
                        format!("Identity operation requires exactly 1 input, got {}", compiled.inputs.len())
                    ));
                }
                
                // Clone the input vector first to avoid borrow checker issues
                let input_vec = self.get_input(&compiled.inputs[0], input)?;
                let cloned_input = input_vec.clone();
                
                // Now get mutable output and assign
                let output_vec = self.get_output_mut(compiled.output)?;
                *output_vec = cloned_input;
            }
            ExprOp::AddI64 => {
                // AddI64: element-wise addition of two Int64 vectors
                if compiled.inputs.len() != 2 {
                    return Err(EvalError::General(
                        format!("AddI64 operation requires exactly 2 inputs, got {}", compiled.inputs.len())
                    ));
                }
                
                // Clone input vectors to avoid borrow conflicts
                // Note: Vector uses Arc internally, so this just increments ref counts (cheap)
                let left_vec = self.get_input(&compiled.inputs[0], input)?.clone();
                let right_vec = self.get_input(&compiled.inputs[1], input)?.clone();
                
                // Verify types
                if left_vec.logical_type() != LogicalType::Int64 {
                    return Err(EvalError::General(
                        format!("AddI64 left operand must be Int64, got {:?}", left_vec.logical_type())
                    ));
                }
                if right_vec.logical_type() != LogicalType::Int64 {
                    return Err(EvalError::General(
                        format!("AddI64 right operand must be Int64, got {:?}", right_vec.logical_type())
                    ));
                }
                
                // Extract physical vectors and get slices
                let left_phys = left_vec.physical.as_int64()
                    .ok_or_else(|| EvalError::General("Expected Int64 physical vector".to_string()))?;
                let right_phys = right_vec.physical.as_int64()
                    .ok_or_else(|| EvalError::General("Expected Int64 physical vector".to_string()))?;
                
                let lhs = left_phys.as_slice();
                let rhs = right_phys.as_slice();
                
                // Verify lengths match
                let len = lhs.len();
                if len != rhs.len() {
                    return Err(EvalError::General(
                        format!("Vector length mismatch: {} != {}", len, rhs.len())
                    ));
                }
                
                // Now we can safely get mutable output (no conflicting borrows)
                let output_vec = self.get_output_mut(compiled.output)?;
                
                // Ensure output is Int64 type with correct length
                if output_vec.logical_type() != LogicalType::Int64 {
                    *output_vec = Vector::new(LogicalType::Int64, len);
                }
                
                // Get mutable slice and perform addition
                let output_phys = output_vec.physical.as_int64_mut()
                    .ok_or_else(|| EvalError::General("Expected Int64 physical vector".to_string()))?;
                let out = output_phys.as_mut_slice();
                
                // Vectorized addition: for i = 0; i < len; i++ { output[i] = lhs[i] + rhs[i] }
                for i in 0..len {
                    out[i] = lhs[i] + rhs[i];
                }
            }
            ExprOp::AddI32 | ExprOp::AddF64 => {
                // Stub: would perform vectorized addition
            }
            ExprOp::SubI32 | ExprOp::SubI64 | ExprOp::SubF64 => {
                // Stub: would perform vectorized subtraction
            }
            ExprOp::MulI32 | ExprOp::MulI64 | ExprOp::MulF64 => {
                // Stub: would perform vectorized multiplication
            }
            ExprOp::DivI32 | ExprOp::DivI64 | ExprOp::DivF64 => {
                // Stub: would perform vectorized division
            }
            ExprOp::GtI32 | ExprOp::GtI64 | ExprOp::GtF64 => {
                // Stub: would perform vectorized greater-than comparison
            }
            ExprOp::LtI32 | ExprOp::LtI64 | ExprOp::LtF64 => {
                // Stub: would perform vectorized less-than comparison
            }
            ExprOp::EqI32 | ExprOp::EqI64 | ExprOp::EqF64 | ExprOp::EqBool => {
                // Stub: would perform vectorized equality comparison
            }
            ExprOp::NeI32 | ExprOp::NeI64 | ExprOp::NeF64 | ExprOp::NeBool => {
                // Stub: would perform vectorized inequality comparison
            }
            ExprOp::GeI32 | ExprOp::GeI64 | ExprOp::GeF64 => {
                // Stub: would perform vectorized greater-or-equal comparison
            }
            ExprOp::LeI32 | ExprOp::LeI64 | ExprOp::LeF64 => {
                // Stub: would perform vectorized less-or-equal comparison
            }
            ExprOp::AndBool => {
                // Stub: would perform vectorized logical AND
            }
            ExprOp::OrBool => {
                // Stub: would perform vectorized logical OR
            }
            ExprOp::NotBool => {
                // Stub: would perform vectorized logical NOT
            }
        }
        
        Ok(())
    }
    
    /// Get input vector reference
    /// Inputs can only come from input batch columns or scratch registers
    fn get_input<'a>(
        &'a self,
        input: &ExprInput,
        batch: &'a VectorizedBatch,
    ) -> Result<&'a Vector, EvalError> {
        match input {
            ExprInput::InputCol(idx) => batch.column(*idx),
            ExprInput::Scratch(idx) => {
                self.scratch.get(*idx)
                    .ok_or_else(|| EvalError::General(
                        format!("Invalid scratch register index: {}", idx)
                    ))
            }
            ExprInput::Constant(_val) => {
                // Stub: In real implementation, would materialize constant to a vector
                Err(EvalError::General(
                    "Constant materialization not implemented".to_string(),
                ))
            }
        }
    }
    
    /// Get mutable output vector reference
    /// Outputs can ONLY go to scratch registers
    fn get_output_mut(&mut self, output_idx: usize) -> Result<&mut Vector, EvalError> {
        self.scratch.get_mut(output_idx)
            .ok_or_else(|| EvalError::General(
                format!("Invalid scratch register index: {}", output_idx)
            ))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::batch::{Field, SourceTypeDef};

    #[test]
    fn test_expression_executor_creation() {
        let exprs = vec![CompiledExpr {
            op: ExprOp::AddI64,
            inputs: smallvec![
                ExprInput::InputCol(0),
                ExprInput::Constant(ConstantValue::Int64(42))
            ],
            output: 0,
        }];
        
        let executor = ExpressionExecutor::new(exprs, 2, vec![0]);
        
        assert_eq!(executor.exprs.len(), 1);
        assert_eq!(executor.scratch.len(), 2);
        assert_eq!(executor.outputs.len(), 1);
    }
    
    #[test]
    fn test_expression_executor_execute() {
        let exprs = vec![CompiledExpr {
            op: ExprOp::AddI64,
            inputs: smallvec![
                ExprInput::InputCol(0),
                ExprInput::InputCol(1)
            ],
            output: 0,
        }];
        
        let mut executor = ExpressionExecutor::new(exprs, 1, vec![0]);
        
        // Create input batch
        let schema = SourceTypeDef::new(vec![
            Field {
                name: "a".to_string(),
                type_info: LogicalType::Int64,
            },
            Field {
                name: "b".to_string(),
                type_info: LogicalType::Int64,
            },
        ]);
        
        let input = VectorizedBatch::new(schema.clone(), 10);
        let mut output = VectorizedBatch::new(schema, 10);
        
        // Execute (stub implementation won't actually compute anything)
        let result = executor.execute(&input, &mut output);
        assert!(result.is_ok());
    }
}
