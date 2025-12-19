mod fn_trait;
mod registry;
mod comparison;
mod logical;
mod arithmetic;

pub use fn_trait::{FnId, VectorizedFn};
pub use registry::{BinaryOp, OpType, UnaryOp, VectorizedFnRegistry};
pub use comparison::{VecGtInt64, VecLtInt64};
pub use logical::{VecAnd, VecNot, VecOr};
pub use arithmetic::VecAddInt64;
