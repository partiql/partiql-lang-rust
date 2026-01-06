use crate::batch::{LogicalType, PhysicalVector, PhysicalVectorEnum, Vector, VectorizedBatch};
use crate::error::EvalError;
use crate::expr::operators::{
    add_i64, and_bool, div_i64, eq_i64, ge_i64, gt_i64, le_i64, lt_i64, mod_i64, mul_i64, ne_i64,
    not_bool, or_bool, sub_i64,
};
use smallvec::{smallvec, SmallVec};
use std::marker::PhantomData;

// ============================================================================
// Kernel Function Pointer Types
// ============================================================================

/// Kernel function pointer types for different operation signatures
/// These enable type-safe dispatch to operation kernels
/// Note: Approach 2 adds out_selection parameter for selection-aware output
type KernelI64ToI64 = unsafe fn(ExecInput<i64>, ExecInput<i64>, &mut [i64], Option<*const usize>, usize);
type KernelI64ToBool = unsafe fn(ExecInput<i64>, ExecInput<i64>, &mut [bool], Option<*const usize>, usize);
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
    ModI64,

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
        // Optimization: Only recreate if batch is larger than current capacity
        // For smaller batches, we can reuse existing vectors (they're already large enough)
        // This avoids unnecessary allocations when processing smaller batches after larger ones
        for scratch_vec in &mut self.scratch {
            if scratch_vec.len() < batch_size {
                // Recreate vector with correct size
                // Note: We could potentially extend in-place if PhysicalVector supported it,
                // but for now, recreating is simpler and the cost is amortized over many batches
                let ty = scratch_vec.logical_type();
                *scratch_vec = Vector::new(ty, batch_size);
            }
            // If batch_size <= scratch_vec.len(), we can reuse it as-is
            // The actual data will be overwritten by the expression execution
        }

        // Get selection vector from input (if present)
        let selection = input.selection();

        // Phase 1: Execute all expressions (writes to scratch only)
        // Pass selection vector to enable scalar/SIMD path selection
        // Note: We clone exprs here because execute_op needs &mut self
        // The CompiledExpr structs are small (just metadata), so this is cheap
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
            return Err(EvalError::General(format!(
                "Binary operation requires exactly 2 inputs, got {}",
                inputs.len()
            )));
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
                    return Err(EvalError::General(format!(
                        "Expected Int64 column, got {:?}",
                        vec.logical_type()
                    )));
                }
                let phys = vec.physical.as_int64().ok_or_else(|| {
                    EvalError::General("Expected Int64 physical vector".to_string())
                })?;
                Ok(ExecInput::from_physical(phys, selection))
            }
            ExprInput::Scratch(idx) => {
                let vec = self
                    .scratch
                    .get(*idx)
                    .ok_or_else(|| EvalError::General(format!("Invalid scratch index: {}", idx)))?;
                if vec.logical_type() != LogicalType::Int64 {
                    return Err(EvalError::General(format!(
                        "Expected Int64 scratch, got {:?}",
                        vec.logical_type()
                    )));
                }
                let phys = vec.physical.as_int64().ok_or_else(|| {
                    EvalError::General("Expected Int64 physical vector".to_string())
                })?;
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
            ExprInput::Constant(_) => Err(EvalError::General(
                "Type mismatch: expected Int64 constant".to_string(),
            )),
        }
    }

    /// Decode ExprInput to ExecInput<bool> for runtime execution
    ///
    /// Handles InputCol, Scratch, and Constant inputs
    fn decode_input_bool<'a>(
        &'a self,
        input: &ExprInput,
        batch: &'a VectorizedBatch,
        selection: Option<&'a crate::batch::SelectionVector>,
    ) -> Result<ExecInput<'a, bool>, EvalError> {
        match input {
            ExprInput::InputCol(idx) => {
                let vec = batch.column(*idx)?;
                if vec.logical_type() != LogicalType::Boolean {
                    return Err(EvalError::General(format!(
                        "Expected Boolean column, got {:?}",
                        vec.logical_type()
                    )));
                }
                let phys = vec.physical.as_boolean().ok_or_else(|| {
                    EvalError::General("Expected Boolean physical vector".to_string())
                })?;
                Ok(ExecInput::from_physical(phys, selection))
            }
            ExprInput::Scratch(idx) => {
                let vec = self
                    .scratch
                    .get(*idx)
                    .ok_or_else(|| EvalError::General(format!("Invalid scratch index: {}", idx)))?;
                if vec.logical_type() != LogicalType::Boolean {
                    return Err(EvalError::General(format!(
                        "Expected Boolean scratch, got {:?}",
                        vec.logical_type()
                    )));
                }
                let phys = vec.physical.as_boolean().ok_or_else(|| {
                    EvalError::General("Expected Boolean physical vector".to_string())
                })?;
                Ok(ExecInput::from_physical(phys, selection))
            }
            ExprInput::Constant(ConstantValue::Boolean(value)) => {
                // Create ExecInput directly for constant (no heap allocation needed)
                let len = batch.row_count();
                Ok(ExecInput {
                    data: value as *const bool,
                    selection: None,
                    len,
                    is_constant: true,
                    _marker: PhantomData,
                })
            }
            ExprInput::Constant(_) => Err(EvalError::General(
                "Type mismatch: expected Boolean constant".to_string(),
            )),
        }
    }

    /// Prepare output vector and return mutable slice for Int64
    #[inline]
    fn prepare_output_i64(
        &mut self,
        output_idx: usize,
        len: usize,
    ) -> Result<&mut [i64], EvalError> {
        self.prepare_output(output_idx, len, LogicalType::Int64)
    }

    /// Prepare output vector and return mutable slice for Float64
    #[inline]
    fn prepare_output_f64(
        &mut self,
        output_idx: usize,
        len: usize,
    ) -> Result<&mut [f64], EvalError> {
        self.prepare_output(output_idx, len, LogicalType::Float64)
    }

    /// Prepare output vector and return mutable slice for Boolean
    #[inline]
    fn prepare_output_bool(
        &mut self,
        output_idx: usize,
        len: usize,
    ) -> Result<&mut [bool], EvalError> {
        self.prepare_output(output_idx, len, LogicalType::Boolean)
    }

    /// Generic prepare output that dispatches based on LogicalType
    ///
    /// This is zero-cost at runtime due to:
    /// 1. Monomorphization - compiler generates separate code for each T
    /// 2. Inlining - the match gets optimized away
    /// 3. The accessor methods are also inlined
    #[inline]
    fn prepare_output<T>(
        &mut self,
        output_idx: usize,
        len: usize,
        logical_type: LogicalType,
    ) -> Result<&mut [T], EvalError> {
        let output_vec = self.get_output_mut(output_idx)?;

        // Optimization: Only recreate if type mismatch OR capacity insufficient
        // This avoids unnecessary reallocations when the vector already has the right type and capacity
        if output_vec.logical_type() != logical_type || output_vec.len() < len {
            *output_vec = Vector::new(logical_type, len);
        }
        // If type matches and capacity is sufficient, we can reuse the existing vector
        // The data will be overwritten by the expression execution

        // Get mutable slice via type-specific accessor
        // The match compiles to a jump table or gets optimized away entirely
        let phys_vec: *mut PhysicalVectorEnum = &mut output_vec.physical;
        unsafe {
            match logical_type {
                LogicalType::Int64 => {
                    let vec = (*phys_vec).as_int64_mut().ok_or_else(|| {
                        EvalError::General("Expected Int64 physical vector".to_string())
                    })?;
                    let slice = vec.as_mut_slice();
                    // Cast to generic slice type
                    Ok(std::slice::from_raw_parts_mut(
                        slice.as_mut_ptr() as *mut T,
                        len.min(slice.len()),
                    ))
                }
                LogicalType::Float64 => {
                    let vec = (*phys_vec).as_float64_mut().ok_or_else(|| {
                        EvalError::General("Expected Float64 physical vector".to_string())
                    })?;
                    let slice = vec.as_mut_slice();
                    Ok(std::slice::from_raw_parts_mut(
                        slice.as_mut_ptr() as *mut T,
                        len.min(slice.len()),
                    ))
                }
                LogicalType::Boolean => {
                    let vec = (*phys_vec).as_boolean_mut().ok_or_else(|| {
                        EvalError::General("Expected Boolean physical vector".to_string())
                    })?;
                    let slice = vec.as_mut_slice();
                    Ok(std::slice::from_raw_parts_mut(
                        slice.as_mut_ptr() as *mut T,
                        len.min(slice.len()),
                    ))
                }
                LogicalType::String => {
                    let vec = (*phys_vec).as_string_mut().ok_or_else(|| {
                        EvalError::General("Expected String physical vector".to_string())
                    })?;
                    let slice = vec.as_mut_slice();
                    Ok(std::slice::from_raw_parts_mut(
                        slice.as_mut_ptr() as *mut T,
                        len.min(slice.len()),
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
    ///
    /// Selection Vector Behavior:
    /// - Inputs respect selection vectors (read from sparse physical indices)
    /// - Output is always dense (written to consecutive indices 0..len)
    /// - len is the selection count if present, otherwise the full batch row count
    fn execute_binary_i64_to_i64(
        &mut self,
        kernel: KernelI64ToI64,
        compiled: &CompiledExpr,
        input: &VectorizedBatch,
        selection: Option<&crate::batch::SelectionVector>,
    ) -> Result<(), EvalError> {
        Self::validate_binary_inputs(&compiled.inputs)?;
        
        // Use selection-aware length: if selection present, use selection count
        // Otherwise use full batch row count
        let len = match selection {
            Some(sel) => sel.indices.len(),
            None => input.row_count(),
        };

        // Prepare output buffer
        let out_ptr = {
            let out_slice = self.prepare_output_i64(compiled.output, len)?;
            out_slice.as_mut_ptr()
        };

        // Decode inputs
        let lhs = self.decode_input_i64(&compiled.inputs[0], input, selection)?;
        let rhs = self.decode_input_i64(&compiled.inputs[1], input, selection)?;

        // Get output selection for Approach 2
        let out_selection = selection.map(|s| s.indices.as_ptr());

        // Execute kernel
        unsafe {
            let out = std::slice::from_raw_parts_mut(out_ptr, len);
            kernel(lhs, rhs, out, out_selection, len);
        }
        Ok(())
    }

    /// Execute binary operation: i64 × i64 → bool
    ///
    /// Generic executor for all comparison operations on i64.
    /// Handles input decoding, output preparation, and kernel invocation.
    ///
    /// Selection Vector Behavior (Approach 2):
    /// - Inputs respect selection vectors (read from sparse physical indices)
    /// - Output writes to sparse physical indices if selection present
    /// - len is the selection count if present, otherwise the full batch row count
    fn execute_binary_i64_to_bool(
        &mut self,
        kernel: KernelI64ToBool,
        compiled: &CompiledExpr,
        input: &VectorizedBatch,
        selection: Option<&crate::batch::SelectionVector>,
    ) -> Result<(), EvalError> {
        Self::validate_binary_inputs(&compiled.inputs)?;
        
        // For Approach 2: len is always original batch size (selection writes to sparse indices)
        let len = input.row_count();

        // Prepare output buffer - allocate full batch size for sparse writes
        let out_ptr = {
            let out_slice = self.prepare_output_bool(compiled.output, len)?;
            out_slice.as_mut_ptr()
        };

        // Decode inputs
        let lhs = self.decode_input_i64(&compiled.inputs[0], input, selection)?;
        let rhs = self.decode_input_i64(&compiled.inputs[1], input, selection)?;

        // Get output selection for Approach 2
        let out_selection = selection.map(|s| s.indices.as_ptr());
        let selection_len = selection.map(|s| s.indices.len()).unwrap_or(len);

        // Execute kernel
        unsafe {
            let out = std::slice::from_raw_parts_mut(out_ptr, len);
            kernel(lhs, rhs, out, out_selection, selection_len);
        }
        Ok(())
    }

    /// Execute binary operation: bool × bool → bool
    ///
    /// Generic executor for logical operations (AND, OR).
    /// Handles input decoding, output preparation, and kernel invocation.
    ///
    /// Selection Vector Behavior (Approach 2):
    /// - Inputs respect selection vectors (read from sparse physical indices)
    /// - Output writes to sparse physical indices if selection present
    fn execute_binary_bool_to_bool(
        &mut self,
        kernel: unsafe fn(ExecInput<bool>, ExecInput<bool>, &mut [bool], Option<*const usize>, usize),
        compiled: &CompiledExpr,
        input: &VectorizedBatch,
        selection: Option<&crate::batch::SelectionVector>,
    ) -> Result<(), EvalError> {
        Self::validate_binary_inputs(&compiled.inputs)?;
        
        // For Approach 2: len is always original batch size (selection writes to sparse indices)
        let len = input.row_count();

        // Prepare output buffer - allocate full batch size for sparse writes
        let out_ptr = {
            let out_slice = self.prepare_output_bool(compiled.output, len)?;
            out_slice.as_mut_ptr()
        };

        // Decode inputs
        let lhs = self.decode_input_bool(&compiled.inputs[0], input, selection)?;
        let rhs = self.decode_input_bool(&compiled.inputs[1], input, selection)?;

        // Get output selection for Approach 2
        let out_selection = selection.map(|s| s.indices.as_ptr());
        let selection_len = selection.map(|s| s.indices.len()).unwrap_or(len);

        // Execute kernel
        unsafe {
            let out = std::slice::from_raw_parts_mut(out_ptr, len);
            kernel(lhs, rhs, out, out_selection, selection_len);
        }
        Ok(())
    }

    /// Execute unary operation: bool → bool
    ///
    /// Executor for unary logical operations (NOT).
    /// Handles input decoding, output preparation, and kernel invocation.
    ///
    /// Selection Vector Behavior (Approach 2):
    /// - Input respects selection vector (read from sparse physical indices)
    /// - Output writes to sparse physical indices if selection present
    fn execute_unary_bool_to_bool(
        &mut self,
        kernel: unsafe fn(ExecInput<bool>, &mut [bool], Option<*const usize>, usize),
        compiled: &CompiledExpr,
        input: &VectorizedBatch,
        selection: Option<&crate::batch::SelectionVector>,
    ) -> Result<(), EvalError> {
        if compiled.inputs.len() != 1 {
            return Err(EvalError::General(format!(
                "Unary operation requires exactly 1 input, got {}",
                compiled.inputs.len()
            )));
        }
        
        // For Approach 2: len is always original batch size (selection writes to sparse indices)
        let len = input.row_count();

        // Prepare output buffer - allocate full batch size for sparse writes
        let out_ptr = {
            let out_slice = self.prepare_output_bool(compiled.output, len)?;
            out_slice.as_mut_ptr()
        };

        // Decode input
        let input_exec = self.decode_input_bool(&compiled.inputs[0], input, selection)?;

        // Get output selection for Approach 2
        let out_selection = selection.map(|s| s.indices.as_ptr());
        let selection_len = selection.map(|s| s.indices.len()).unwrap_or(len);

        // Execute kernel
        unsafe {
            let out = std::slice::from_raw_parts_mut(out_ptr, len);
            kernel(input_exec, out, out_selection, selection_len);
        }
        Ok(())
    }


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
                    return Err(EvalError::General(format!(
                        "Identity operation requires exactly 1 input, got {}",
                        compiled.inputs.len()
                    )));
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
                self.execute_binary_i64_to_i64(
                    add_i64::kernel_add_i64,
                    compiled,
                    input,
                    selection,
                )?;
            }
            ExprOp::SubI64 => {
                self.execute_binary_i64_to_i64(
                    sub_i64::kernel_sub_i64,
                    compiled,
                    input,
                    selection,
                )?;
            }
            ExprOp::MulI64 => {
                self.execute_binary_i64_to_i64(
                    mul_i64::kernel_mul_i64,
                    compiled,
                    input,
                    selection,
                )?;
            }
            ExprOp::DivI64 => {
                self.execute_binary_i64_to_i64(
                    div_i64::kernel_div_i64,
                    compiled,
                    input,
                    selection,
                )?;
            }
            ExprOp::ModI64 => {
                self.execute_binary_i64_to_i64(
                    mod_i64::kernel_mod_i64,
                    compiled,
                    input,
                    selection,
                )?;
            }
            ExprOp::GtI64 => {
                self.execute_binary_i64_to_bool(gt_i64::kernel_gt_i64, compiled, input, selection)?;
            }
            ExprOp::LtI64 => {
                self.execute_binary_i64_to_bool(lt_i64::kernel_lt_i64, compiled, input, selection)?;
            }
            ExprOp::EqI64 => {
                self.execute_binary_i64_to_bool(eq_i64::kernel_eq_i64, compiled, input, selection)?;
            }
            ExprOp::NeI64 => {
                self.execute_binary_i64_to_bool(ne_i64::kernel_ne_i64, compiled, input, selection)?;
            }
            ExprOp::GeI64 => {
                self.execute_binary_i64_to_bool(ge_i64::kernel_ge_i64, compiled, input, selection)?;
            }
            ExprOp::LeI64 => {
                self.execute_binary_i64_to_bool(le_i64::kernel_le_i64, compiled, input, selection)?;
            }
            ExprOp::AddI32 | ExprOp::AddF64 => {
                // Stub: would perform vectorized addition
            }
            ExprOp::SubI32 | ExprOp::SubF64 => {
                // Stub: would perform vectorized subtraction
            }
            ExprOp::MulI32 | ExprOp::MulF64 => {
                // Stub: would perform vectorized multiplication
            }
            ExprOp::DivI32 | ExprOp::DivF64 => {
                // Stub: would perform vectorized division
            }
            ExprOp::GtI32 | ExprOp::GtF64 => {
                // Stub: would perform vectorized greater-than comparison
            }
            ExprOp::LtI32 | ExprOp::LtF64 => {
                // Stub: would perform vectorized less-than comparison
            }
            ExprOp::EqI32 | ExprOp::EqF64 | ExprOp::EqBool => {
                // Stub: would perform vectorized equality comparison
            }
            ExprOp::NeI32 | ExprOp::NeF64 | ExprOp::NeBool => {
                // Stub: would perform vectorized inequality comparison
            }
            ExprOp::GeI32 | ExprOp::GeF64 => {
                // Stub: would perform vectorized greater-or-equal comparison
            }
            ExprOp::LeI32 | ExprOp::LeF64 => {
                // Stub: would perform vectorized less-or-equal comparison
            }
            ExprOp::AndBool => {
                self.execute_binary_bool_to_bool(and_bool::kernel_and_bool, compiled, input, selection)?;
            }
            ExprOp::OrBool => {
                self.execute_binary_bool_to_bool(or_bool::kernel_or_bool, compiled, input, selection)?;
            }
            ExprOp::NotBool => {
                self.execute_unary_bool_to_bool(not_bool::kernel_not_bool, compiled, input, selection)?;
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
            ExprInput::Scratch(idx) => self.scratch.get(*idx).ok_or_else(|| {
                EvalError::General(format!("Invalid scratch register index: {}", idx))
            }),
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
        self.scratch.get_mut(output_idx).ok_or_else(|| {
            EvalError::General(format!("Invalid scratch register index: {}", output_idx))
        })
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

        let executor =
            ExpressionExecutor::new(exprs, vec![LogicalType::Int64, LogicalType::Int64], vec![0]);

        assert_eq!(executor.exprs.len(), 1);
        assert_eq!(executor.scratch.len(), 2);
        assert_eq!(executor.outputs.len(), 1);
    }

    #[test]
    fn test_expression_executor_execute() {
        let exprs = vec![CompiledExpr {
            op: ExprOp::AddI64,
            inputs: smallvec![ExprInput::InputCol(0), ExprInput::InputCol(1)],
            output: 0,
        }];

        let mut executor = ExpressionExecutor::new(exprs, vec![LogicalType::Int64], vec![0]);

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

    #[test]
    fn test_sub_i64_kernel() {
        // Test the SubI64 kernel directly
        let lhs_data = vec![10i64, 20, 30, 40, 50];
        let rhs_data = vec![1i64, 2, 3, 4, 5];
        let mut out = vec![0i64; 5];

        unsafe {
            sub_i64::kernel_sub_i64(
                ExecInput {
                    data: lhs_data.as_ptr(),
                    selection: None,
                    len: 5,
                    is_constant: false,
                    _marker: PhantomData,
                },
                ExecInput {
                    data: rhs_data.as_ptr(),
                    selection: None,
                    len: 5,
                    is_constant: false,
                    _marker: PhantomData,
                },
                &mut out,
                None,
                5,
            );
        }

        assert_eq!(out, vec![9, 18, 27, 36, 45]);
    }

    #[test]
    fn test_sub_i64_with_constant() {
        // Test SubI64 with constant (vector - constant)
        let vec_data = vec![10i64, 20, 30, 40, 50];
        let constant = 5i64;
        let mut out = vec![0i64; 5];

        unsafe {
            sub_i64::kernel_sub_i64(
                ExecInput {
                    data: vec_data.as_ptr(),
                    selection: None,
                    len: 5,
                    is_constant: false,
                    _marker: PhantomData,
                },
                ExecInput {
                    data: &constant as *const i64,
                    selection: None,
                    len: 5,
                    is_constant: true,
                    _marker: PhantomData,
                },
                &mut out,
                None,
                5,
            );
        }

        assert_eq!(out, vec![5, 15, 25, 35, 45]);
    }

    #[test]
    fn test_sub_i64_constant_minus_vector() {
        // Test SubI64 with constant - vector (order matters!)
        let constant = 100i64;
        let vec_data = vec![10i64, 20, 30, 40, 50];
        let mut out = vec![0i64; 5];

        unsafe {
            sub_i64::kernel_sub_i64(
                ExecInput {
                    data: &constant as *const i64,
                    selection: None,
                    len: 5,
                    is_constant: true,
                    _marker: PhantomData,
                },
                ExecInput {
                    data: vec_data.as_ptr(),
                    selection: None,
                    len: 5,
                    is_constant: false,
                    _marker: PhantomData,
                },
                &mut out,
                None,
                5,
            );
        }

        assert_eq!(out, vec![90, 80, 70, 60, 50]);
    }

    #[test]
    fn test_add_i64_with_selection_vector() {
        // Test that addition with selection vector produces dense output
        // Input: [10, 20, 30, 40, 50] with selection [0, 2, 4]
        // Expected output: [11, 33, 55] (dense, not sparse)
        let lhs_data = vec![10i64, 20, 30, 40, 50];
        let rhs_data = vec![1i64, 2, 3, 4, 5];
        let selection = vec![0usize, 2, 4]; // Select indices 0, 2, 4
        let mut out = vec![0i64; 3]; // Output should be 3 elements (selection count)

        unsafe {
            add_i64::kernel_add_i64(
                ExecInput {
                    data: lhs_data.as_ptr(),
                    selection: Some(selection.as_ptr()),
                    len: lhs_data.len(),
                    is_constant: false,
                    _marker: PhantomData,
                },
                ExecInput {
                    data: rhs_data.as_ptr(),
                    selection: Some(selection.as_ptr()),
                    len: rhs_data.len(),
                    is_constant: false,
                    _marker: PhantomData,
                },
                &mut out,
                None, // No out_selection - output is dense
                3, // len should be selection count, not original data length
            );
        }

        // Output should be dense: [10+1, 30+3, 50+5] = [11, 33, 55]
        assert_eq!(out, vec![11, 33, 55]);
    }

    #[test]
    fn test_comparison_with_selection_vector() {
        // Test that comparison with selection vector produces sparse output (Approach 2)
        // Input: [10, 20, 30, 40, 50] with selection [1, 3]
        // Compare with: [15, 25, 35, 45, 55]
        // Expected: sparse writes at indices 1 and 3
        let lhs_data = vec![10i64, 20, 30, 40, 50];
        let rhs_data = vec![15i64, 25, 35, 45, 55];
        let selection = vec![1usize, 3]; // Select indices 1, 3
        let mut out = vec![false; 5]; // Allocate full batch size for sparse writes

        unsafe {
            gt_i64::kernel_gt_i64(
                ExecInput {
                    data: lhs_data.as_ptr(),
                    selection: Some(selection.as_ptr()),
                    len: lhs_data.len(),
                    is_constant: false,
                    _marker: PhantomData,
                },
                ExecInput {
                    data: rhs_data.as_ptr(),
                    selection: Some(selection.as_ptr()),
                    len: rhs_data.len(),
                    is_constant: false,
                    _marker: PhantomData,
                },
                &mut out,
                Some(selection.as_ptr()), // Pass selection for sparse output
                2, // len should be selection count
            );
        }

        // Output: sparse writes at indices 1 and 3: [20 > 25, 40 > 45] = [false, false]
        assert_eq!(out[1], false); // 20 > 25
        assert_eq!(out[3], false); // 40 > 45
    }

    #[test]
    fn test_selection_with_constant() {
        // Test selection vector with one constant operand
        // Input: [10, 20, 30, 40, 50] with selection [0, 4]
        // Add constant: 100
        // Expected: [110, 150]
        let vec_data = vec![10i64, 20, 30, 40, 50];
        let constant = 100i64;
        let selection = vec![0usize, 4];
        let mut out = vec![0i64; 2];

        unsafe {
            add_i64::kernel_add_i64(
                ExecInput {
                    data: vec_data.as_ptr(),
                    selection: Some(selection.as_ptr()),
                    len: vec_data.len(),
                    is_constant: false,
                    _marker: PhantomData,
                },
                ExecInput {
                    data: &constant as *const i64,
                    selection: None, // Constants don't use selection
                    len: vec_data.len(),
                    is_constant: true,
                    _marker: PhantomData,
                },
                &mut out,
                None, // Dense output
                2,
            );
        }

        // Output: [10+100, 50+100] = [110, 150]
        assert_eq!(out, vec![110, 150]);
    }

    #[test]
    fn test_empty_selection() {
        // Test with empty selection vector
        let lhs_data = vec![10i64, 20, 30, 40, 50];
        let rhs_data = vec![1i64, 2, 3, 4, 5];
        let selection: Vec<usize> = vec![]; // Empty selection
        let mut out = vec![0i64; 0]; // No output expected

        unsafe {
            add_i64::kernel_add_i64(
                ExecInput {
                    data: lhs_data.as_ptr(),
                    selection: Some(selection.as_ptr()),
                    len: lhs_data.len(),
                    is_constant: false,
                    _marker: PhantomData,
                },
                ExecInput {
                    data: rhs_data.as_ptr(),
                    selection: Some(selection.as_ptr()),
                    len: rhs_data.len(),
                    is_constant: false,
                    _marker: PhantomData,
                },
                &mut out,
                None, // Dense output
                0, // Zero length for empty selection
            );
        }

        assert_eq!(out.len(), 0);
    }

    #[test]
    fn test_mod_i64_kernel() {
        // Test the ModI64 kernel directly
        let lhs_data = vec![10i64, 21, 30, 47, 55];
        let rhs_data = vec![3i64, 4, 7, 5, 10];
        let mut out = vec![0i64; 5];

        unsafe {
            mod_i64::kernel_mod_i64(
                ExecInput {
                    data: lhs_data.as_ptr(),
                    selection: None,
                    len: 5,
                    is_constant: false,
                    _marker: PhantomData,
                },
                ExecInput {
                    data: rhs_data.as_ptr(),
                    selection: None,
                    len: 5,
                    is_constant: false,
                    _marker: PhantomData,
                },
                &mut out,
                None,
                5,
            );
        }

        // 10 % 3 = 1, 21 % 4 = 1, 30 % 7 = 2, 47 % 5 = 2, 55 % 10 = 5
        assert_eq!(out, vec![1, 1, 2, 2, 5]);
    }

    #[test]
    fn test_mod_i64_with_constant() {
        // Test ModI64 with constant (vector % constant)
        let vec_data = vec![10i64, 21, 30, 47, 55];
        let constant = 7i64;
        let mut out = vec![0i64; 5];

        unsafe {
            mod_i64::kernel_mod_i64(
                ExecInput {
                    data: vec_data.as_ptr(),
                    selection: None,
                    len: 5,
                    is_constant: false,
                    _marker: PhantomData,
                },
                ExecInput {
                    data: &constant as *const i64,
                    selection: None,
                    len: 5,
                    is_constant: true,
                    _marker: PhantomData,
                },
                &mut out,
                None,
                5,
            );
        }

        // 10 % 7 = 3, 21 % 7 = 0, 30 % 7 = 2, 47 % 7 = 5, 55 % 7 = 6
        assert_eq!(out, vec![3, 0, 2, 5, 6]);
    }

    #[test]
    fn test_mod_i64_constant_mod_vector() {
        // Test ModI64 with constant % vector
        let constant = 100i64;
        let vec_data = vec![7i64, 9, 11, 13, 17];
        let mut out = vec![0i64; 5];

        unsafe {
            mod_i64::kernel_mod_i64(
                ExecInput {
                    data: &constant as *const i64,
                    selection: None,
                    len: 5,
                    is_constant: true,
                    _marker: PhantomData,
                },
                ExecInput {
                    data: vec_data.as_ptr(),
                    selection: None,
                    len: 5,
                    is_constant: false,
                    _marker: PhantomData,
                },
                &mut out,
                None,
                5,
            );
        }

        // 100 % 7 = 2, 100 % 9 = 1, 100 % 11 = 1, 100 % 13 = 9, 100 % 17 = 15
        assert_eq!(out, vec![2, 1, 1, 9, 15]);
    }
}
