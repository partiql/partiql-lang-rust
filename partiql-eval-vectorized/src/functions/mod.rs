mod arithmetic;
mod comparison;
mod fn_trait;
mod logical;
mod registry;

pub use arithmetic::VecAddInt64;
pub use comparison::{VecGtInt64, VecLtInt64};
pub use fn_trait::{FnId, VectorizedFn};
pub use logical::{VecAnd, VecNot, VecOr};
pub use registry::{BinaryOp, OpType, UnaryOp, VectorizedFnRegistry};
