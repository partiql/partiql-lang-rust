use crate::batch::{LogicalType, PhysicalVector, PhysicalVectorEnum, Vector, VectorizedBatch};
use crate::error::EvalError;
use crate::expr::operators::eq_i64;
use smallvec::SmallVec;
use std::marker::PhantomData;

// ============================================================================
// Kernel Function Pointer Types
// ============================================================================

/// Kernel function pointer types for different operation signatures
/// These enable type-safe dispatch to operation kernels
type KernelI64ToI64 = unsafe fn(ExecInput<i64>, ExecInput<i64>, &mut [i64], usize);
type KernelI64ToBool = unsafe fn(ExecInput<i64>, ExecInput<i64>, &mut [bool], usize);
type KernelI32ToI32 = unsafe fn(ExecInput<i32>, ExecInput<i32>, &mut [i32], usize);
type KernelI32ToBool = unsafe fn(ExecInput<i32>, ExecInput<i32>, &mut [bool], usize);
type KernelF64ToF64 = unsafe fn(ExecInput<f64>, ExecInput<f64>, &mut [f64], usize);
type KernelF64ToBool = unsafe fn(ExecInput<f64>, ExecInput<f64>, &mut [bool], usize);
type KernelBoolToBool = unsafe fn(ExecInput<bool>, ExecInput<bool>, &mut [bool], usize);

/// Runtime operand representation for SIMD kernels
/// 
/// Provides efficient access to vector data with support for:
/// - Flat vectors (via data pointer)
/// - Constant/broadcast scalars (via is_constant flag)
/// - Selection vectors (sparse access pattern)
#[derive(Debug)]
pub struct ExecInput<'a, T: Copy> {
    /// Pointer to underlying data (flat base or scalar)
    pub data: *const T,

    /// Optional selection (logical row → physical row)
    pub selection: Option<*const usize>,

    /// Number of logical rows
    pub len: usize,

    /// True if broadcast scalar
    pub is_constant: bool,

    _marker: PhantomData<&'a T>,
}

impl<'a, T: Copy> ExecInput<'a, T> {
    /// Create ExecInput from PhysicalVector with optional selection
    pub fn from_physical(
        phys: &'a PhysicalVector<T>,
        selection: Option<&'a crate::batch::SelectionVector>,
    ) -> Self {
        match phys {
            PhysicalVector::Flat { .. } => {
                let slice = phys.as_slice();
                Self {
                    data: slice.as_ptr(),
                    selection: selection.map(|s| s.indices.as_ptr()),
                    len: slice.len(),
                    is_constant: false,
                    _marker: PhantomData,
                }
            }
            PhysicalVector::Constant { value, len } => {
                Self {
                    data: value as *const T,
                    selection: None, // Selection ignored for constants
                    len: *len,
                    is_constant: true,
                    _marker: PhantomData,
                }
            }
        }
    }

    /// Get value at logical index (handles both flat and constant)
    /// 
    /// # Safety
    /// Caller must ensure idx < len
    #[inline(always)]
    pub unsafe fn get_unchecked(&self, idx: usize) -> T {
        if self.is_constant {
            *self.data
        } else {
            let physical_idx = if let Some(sel_ptr) = self.selection {
                *sel_ptr.add(idx) as usize
            } else {
                idx
            };
            *self.data.add(physical_idx)
        }
    }
}

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
        scratch_types: Vec<LogicalType>,
        outputs: Vec<usize>,
    ) -> Self {
        // Pre-allocate scratch vectors with specified types
        let scratch: Vec<Vector> = scratch_types
            .iter()
            .map(|&ty| Vector::new(ty, 1024)) // TODO: Either pass in, or handle globally.
            .collect();
        
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
        
        // Get selection vector from input (if present)
        let selection = input.selection();
        
        // Phase 1: Execute all expressions (writes to scratch only)
        // Pass selection vector to enable scalar/SIMD path selection
        let exprs = self.exprs.clone();
        for compiled in &exprs {
            self.execute_op(compiled, input, selection)?;
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
    
    // ============================================================================
    // Helper Functions for Operation Execution
    // ============================================================================
    
    /// Validate that we have exactly 2 inputs for a binary operation
    #[inline]
    fn validate_binary_inputs(inputs: &[ExprInput]) -> Result<(), EvalError> {
        if inputs.len() != 2 {
            return Err(EvalError::General(
                format!("Binary operation requires exactly 2 inputs, got {}", inputs.len())
            ));
        }
        Ok(())
    }
    
    /// Decode ExprInput to ExecInput<i64> for runtime execution
    /// 
    /// Handles InputCol, Scratch, and Constant inputs
    fn decode_input_i64<'a>(
        &'a self,
        input: &ExprInput,
        batch: &'a VectorizedBatch,
        selection: Option<&'a crate::batch::SelectionVector>,
    ) -> Result<ExecInput<'a, i64>, EvalError> {
        match input {
            ExprInput::InputCol(idx) => {
                let vec = batch.column(*idx)?;
                if vec.logical_type() != LogicalType::Int64 {
                    return Err(EvalError::General(
                        format!("Expected Int64 column, got {:?}", vec.logical_type())
                    ));
                }
                let phys = vec.physical.as_int64()
                    .ok_or_else(|| EvalError::General("Expected Int64 physical vector".to_string()))?;
                Ok(ExecInput::from_physical(phys, selection))
            }
            ExprInput::Scratch(idx) => {
                let vec = self.scratch.get(*idx)
                    .ok_or_else(|| EvalError::General(format!("Invalid scratch index: {}", idx)))?;
                if vec.logical_type() != LogicalType::Int64 {
                    return Err(EvalError::General(
                        format!("Expected Int64 scratch, got {:?}", vec.logical_type())
                    ));
                }
                let phys = vec.physical.as_int64()
                    .ok_or_else(|| EvalError::General("Expected Int64 physical vector".to_string()))?;
                Ok(ExecInput::from_physical(phys, selection))
            }
            ExprInput::Constant(ConstantValue::Int64(value)) => {
                // Create ExecInput directly for constant (no heap allocation needed)
                let len = batch.row_count();
                Ok(ExecInput {
                    data: value as *const i64,
                    selection: None,
                    len,
                    is_constant: true,
                    _marker: PhantomData,
                })
            }
            ExprInput::Constant(_) => {
                Err(EvalError::General("Type mismatch: expected Int64 constant".to_string()))
            }
        }
    }
    
    /// Prepare output vector and return mutable slice for Int64
    #[inline]
    fn prepare_output_i64(&mut self, output_idx: usize, len: usize) -> Result<&mut [i64], EvalError> {
        self.prepare_output(output_idx, len, LogicalType::Int64)
    }
    
    /// Prepare output vector and return mutable slice for Float64
    #[inline]
    fn prepare_output_f64(&mut self, output_idx: usize, len: usize) -> Result<&mut [f64], EvalError> {
        self.prepare_output(output_idx, len, LogicalType::Float64)
    }
    
    /// Prepare output vector and return mutable slice for Boolean
    #[inline]
    fn prepare_output_bool(&mut self, output_idx: usize, len: usize) -> Result<&mut [bool], EvalError> {
        self.prepare_output(output_idx, len, LogicalType::Boolean)
    }
    
    /// Generic prepare output that dispatches based on LogicalType
    /// 
    /// This is zero-cost at runtime due to:
    /// 1. Monomorphization - compiler generates separate code for each T
    /// 2. Inlining - the match gets optimized away
    /// 3. The accessor methods are also inlined
    #[inline]
    fn prepare_output<T>(&mut self, output_idx: usize, len: usize, logical_type: LogicalType) -> Result<&mut [T], EvalError> {
        let output_vec = self.get_output_mut(output_idx)?;
        
        // Ensure output has correct type and length
        if output_vec.logical_type() != logical_type {
            *output_vec = Vector::new(logical_type, len);
        }
        
        // Get mutable slice via type-specific accessor
        // The match compiles to a jump table or gets optimized away entirely
        let phys_vec: *mut PhysicalVectorEnum = &mut output_vec.physical;
        unsafe {
            match logical_type {
                LogicalType::Int64 => {
                    let vec = (*phys_vec).as_int64_mut()
                        .ok_or_else(|| EvalError::General("Expected Int64 physical vector".to_string()))?;
                    let slice = vec.as_mut_slice();
                    // Cast to generic slice type
                    Ok(std::slice::from_raw_parts_mut(
                        slice.as_mut_ptr() as *mut T,
                        len.min(slice.len())
                    ))
                }
                LogicalType::Float64 => {
                    let vec = (*phys_vec).as_float64_mut()
                        .ok_or_else(|| EvalError::General("Expected Float64 physical vector".to_string()))?;
                    let slice = vec.as_mut_slice();
                    Ok(std::slice::from_raw_parts_mut(
                        slice.as_mut_ptr() as *mut T,
                        len.min(slice.len())
                    ))
                }
                LogicalType::Boolean => {
                    let vec = (*phys_vec).as_boolean_mut()
                        .ok_or_else(|| EvalError::General("Expected Boolean physical vector".to_string()))?;
                    let slice = vec.as_mut_slice();
                    Ok(std::slice::from_raw_parts_mut(
                        slice.as_mut_ptr() as *mut T,
                        len.min(slice.len())
                    ))
                }
                LogicalType::String => {
                    let vec = (*phys_vec).as_string_mut()
                        .ok_or_else(|| EvalError::General("Expected String physical vector".to_string()))?;
                    let slice = vec.as_mut_slice();
                    Ok(std::slice::from_raw_parts_mut(
                        slice.as_mut_ptr() as *mut T,
                        len.min(slice.len())
                    ))
                }
            }
        }
    }
    
    // ============================================================================
    // Generic Binary Operation Executors
    // ============================================================================
    
    /// Execute binary operation: i64 × i64 → i64
    /// 
    /// Generic executor for all arithmetic operations on i64 that produce i64 output.
    /// Handles input decoding, output preparation, and kernel invocation.
    fn execute_binary_i64_to_i64(
        &mut self,
        kernel: KernelI64ToI64,
        compiled: &CompiledExpr,
        input: &VectorizedBatch,
        selection: Option<&crate::batch::SelectionVector>,
    ) -> Result<(), EvalError> {
        Self::validate_binary_inputs(&compiled.inputs)?;
        let len = input.row_count();
        
        // Prepare output buffer
        let out_ptr = {
            let out_slice = self.prepare_output_i64(compiled.output, len)?;
            out_slice.as_mut_ptr()
        };
        
        // Decode inputs
        let lhs = self.decode_input_i64(&compiled.inputs[0], input, selection)?;
        let rhs = self.decode_input_i64(&compiled.inputs[1], input, selection)?;
        
        // Execute kernel
        unsafe {
            let out = std::slice::from_raw_parts_mut(out_ptr, len);
            kernel(lhs, rhs, out, len);
        }
        Ok(())
    }
    
    /// Execute binary operation: i64 × i64 → bool
    /// 
    /// Generic executor for all comparison operations on i64.
    /// Handles input decoding, output preparation, and kernel invocation.
    fn execute_binary_i64_to_bool(
        &mut self,
        kernel: KernelI64ToBool,
        compiled: &CompiledExpr,
        input: &VectorizedBatch,
        selection: Option<&crate::batch::SelectionVector>,
    ) -> Result<(), EvalError> {
        Self::validate_binary_inputs(&compiled.inputs)?;
        let len = input.row_count();
        
        // Prepare output buffer
        let out_ptr = {
            let out_slice = self.prepare_output_bool(compiled.output, len)?;
            out_slice.as_mut_ptr()
        };
        
        // Decode inputs
        let lhs = self.decode_input_i64(&compiled.inputs[0], input, selection)?;
        let rhs = self.decode_input_i64(&compiled.inputs[1], input, selection)?;
        
        // Execute kernel
        unsafe {
            let out = std::slice::from_raw_parts_mut(out_ptr, len);
            kernel(lhs, rhs, out, len);
        }
        Ok(())
    }
    
    // ============================================================================
    // SIMD Kernels
    // ============================================================================
    
    /// Int64 addition kernel - handles all input combinations
    /// 
    /// Efficiently handles 4 cases:
    /// - vector + vector (standard SIMD)
    /// - vector + const (broadcast const)
    /// - const + vector (broadcast const)
    /// - const + const (single computation)
    ///
    /// # Safety
    /// Caller must ensure len <= out.len()
    #[inline]
    unsafe fn kernel_add_i64(
        lhs: ExecInput<i64>,
        rhs: ExecInput<i64>,
        out: &mut [i64],
        len: usize,
    ) {
        match (lhs.is_constant, rhs.is_constant) {
            (false, false) => {
                // Vector + Vector: standard element-wise addition
                for i in 0..len {
                    out[i] = lhs.get_unchecked(i) + rhs.get_unchecked(i);
                }
            }
            (false, true) => {
                // Vector + Constant: broadcast constant
                let c = *rhs.data;
                for i in 0..len {
                    out[i] = lhs.get_unchecked(i) + c;
                }
            }
            (true, false) => {
                // Constant + Vector: broadcast constant
                let c = *lhs.data;
                for i in 0..len {
                    out[i] = c + rhs.get_unchecked(i);
                }
            }
            (true, true) => {
                // Constant + Constant: single computation, fill all
                let result = *lhs.data + *rhs.data;
                for i in 0..len {
                    out[i] = result;
                }
            }
        }
    }
    
    // ============================================================================
    // End SIMD Kernels
    // ============================================================================
    
    /// Execute a single compiled operation
    /// All outputs go to scratch registers only.
    fn execute_op(
        &mut self,
        compiled: &CompiledExpr,
        input: &VectorizedBatch,
        selection: Option<&crate::batch::SelectionVector>,
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
                
                // For identity, we can just clone the vector (Arc clone is cheap)
                // Selection vector is maintained in the batch, not in individual vectors
                let input_vec = self.get_input(&compiled.inputs[0], input)?;
                let cloned_input = input_vec.clone();
                
                // Now get mutable output and assign
                let output_vec = self.get_output_mut(compiled.output)?;
                *output_vec = cloned_input;
            }
            ExprOp::AddI64 => {
                self.execute_binary_i64_to_i64(Self::kernel_add_i64, compiled, input, selection)?;
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
            ExprOp::EqI32 | ExprOp::EqF64 | ExprOp::EqBool => {
                // Stub: would perform vectorized equality comparison
            }
            ExprOp::EqI64 => {
                self.execute_binary_i64_to_bool(eq_i64::kernel_eq_i64, compiled, input, selection)?;
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
                // TODO: This is JUST to test that we can filter some. Actually implement this.
                let output_vec = self.get_output_mut(compiled.output)?;
                let out_phys = output_vec.physical.as_boolean_mut().expect("Needed boolean buffer.");
                let out = out_phys.as_mut_slice();
                out[0] = true;
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
        
        let executor = ExpressionExecutor::new(
            exprs,
            vec![LogicalType::Int64, LogicalType::Int64],
            vec![0]
        );
        
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
        
        let mut executor = ExpressionExecutor::new(
            exprs,
            vec![LogicalType::Int64],
            vec![0]
        );
        
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
