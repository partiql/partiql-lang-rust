use crate::batch::{Vector, LogicalType};
use crate::error::EvalError;
use crate::functions::{FnId, VectorizedFn};

// TODO: Implement comparison functions

/// Greater-than for Int64
#[derive(Debug)]
pub struct VecGtInt64;

impl VectorizedFn for VecGtInt64 {
    fn execute(&self, _inputs: &[&Vector], _output: &mut Vector) -> Result<(), EvalError> {
        // TODO: Implement actual greater-than comparison
        // For each row: output[i] = inputs[0][i] > inputs[1][i]
        // Handle nulls appropriately
        
        // Mock implementation: do nothing (return empty result)
        Ok(())
    }

    fn fn_id(&self) -> FnId {
        FnId {
            name: "gt",
            id: 1,
            signature: vec![LogicalType::Int64, LogicalType::Int64],
        }
    }

    fn output_type(&self, _input_types: &[LogicalType]) -> LogicalType {
        LogicalType::Boolean
    }
}

/// Less-than for Int64
#[derive(Debug)]
pub struct VecLtInt64;

impl VectorizedFn for VecLtInt64 {
    fn execute(&self, _inputs: &[&Vector], _output: &mut Vector) -> Result<(), EvalError> {
        // TODO: Implement actual less-than comparison
        // For each row: output[i] = inputs[0][i] < inputs[1][i]
        // Handle nulls appropriately
        
        // Mock implementation: do nothing (return empty result)
        Ok(())
    }

    fn fn_id(&self) -> FnId {
        FnId {
            name: "lt",
            id: 2,
            signature: vec![LogicalType::Int64, LogicalType::Int64],
        }
    }

    fn output_type(&self, _input_types: &[LogicalType]) -> LogicalType {
        LogicalType::Boolean
    }
}

// TODO: Add VecGteInt64, VecLteInt64, VecEqInt64, VecNeqInt64
