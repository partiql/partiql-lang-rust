//! PartiQL Vectorized Evaluator
//!
//! A proof-of-concept vectorized evaluation engine for PartiQL that processes
//! data in batches using columnar storage.

pub mod batch;
pub mod compiler;
pub mod error;
pub mod expr;
pub mod functions;
pub mod operators;
pub mod reader;

// Re-export commonly used types
pub use batch::{
    Field, LogicalType, PhysicalVectorEnum, SelectionVector, SourceTypeDef, Vector, VectorizedBatch,
};
pub use compiler::{Compiler, CompilerContext, PlanExecutor, VectorizedPlan};
pub use error::{EvalError, PlanError};
pub use expr::{ColumnRef, FnCallExpr, LiteralExpr, VectorizedExpr};
pub use functions::{BinaryOp, FnId, OpType, UnaryOp, VectorizedFn, VectorizedFnRegistry};
pub use operators::{VectorizedFilter, VectorizedOperator, VectorizedProject, VectorizedScan};
pub use reader::{ArrowReader, BatchReader, IonReader, ParquetReader, TupleIteratorReader};
