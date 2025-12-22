use crate::batch::{Vector, LogicalType};
use crate::error::EvalError;
use crate::functions::{FnId, VectorizedFn};

// TODO: Implement arithmetic functions

/// Add for Int64
#[derive(Debug)]
pub struct VecAddInt64;

impl VectorizedFn for VecAddInt64 {
    fn execute(&self, _inputs: &[&Vector], _output: &mut Vector) -> Result<(), EvalError> {
        // TODO: Implement actual addition
        // For each row: output[i] = inputs[0][i] + inputs[1][i]
        // Handle nulls and overflow appropriately
        
        // Mock implementation: do nothing (return empty result)
        Ok(())
    }

    fn fn_id(&self) -> FnId {
        FnId {
            name: "add",
            id: 20,
            signature: vec![LogicalType::Int64, LogicalType::Int64],
        }
    }

    fn output_type(&self, _input_types: &[LogicalType]) -> LogicalType {
        LogicalType::Int64
    }
}

// TODO: Add VecSubInt64, VecMulInt64, VecDivInt64
