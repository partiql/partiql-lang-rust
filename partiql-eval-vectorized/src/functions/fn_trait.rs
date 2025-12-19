use crate::batch::{PVector, TypeInfo};
use crate::error::EvalError;
use std::fmt::Debug;

/// Function identifier
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct FnId {
    pub name: &'static str,
    pub id: u32,
    pub signature: Vec<TypeInfo>,
}

/// Vectorized function that operates on column vectors
/// 
/// Contract: Output must be pre-allocated by caller with correct size and type.
/// Functions write directly to output indices without resizing.
pub trait VectorizedFn: Debug {
    /// Execute function, writing results to pre-allocated output
    /// 
    /// Preconditions:
    /// - output.len() == inputs[0].len() (all inputs same length)
    /// - output type matches expected output type
    fn execute(&self, inputs: &[&PVector], output: &mut PVector) -> Result<(), EvalError>;

    /// Get function identifier
    fn fn_id(&self) -> FnId;

    /// Get output type given input types
    fn output_type(&self, input_types: &[TypeInfo]) -> TypeInfo;
}
