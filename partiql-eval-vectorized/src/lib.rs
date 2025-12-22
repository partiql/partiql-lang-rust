//! PartiQL Vectorized Evaluator
//!
//! A proof-of-concept vectorized evaluation engine for PartiQL that processes
//! data in batches using columnar storage.

pub mod error;
pub mod batch;
pub mod reader;
pub mod functions;
pub mod expr;
pub mod operators;
pub mod compiler;

// Re-export commonly used types
pub use error::{EvalError, PlanError};
pub use batch::{Field, Vector, SourceTypeDef, LogicalType, VectorizedBatch};
pub use reader::{BatchReader, Tuple, TupleIteratorReader};
pub use functions::{BinaryOp, FnId, OpType, UnaryOp, VectorizedFn, VectorizedFnRegistry};
pub use expr::{ColumnRef, FnCallExpr, LiteralExpr, VectorizedExpr};
pub use operators::{VectorizedFilter, VectorizedOperator, VectorizedProject, VectorizedScan};
pub use compiler::{Compiler, CompilerContext, PlanExecutor, VectorizedPlan};
