#![deny(rust_2018_idioms)]
#![deny(clippy::all)]

mod arena;
mod builtins;
mod catalog;
mod compiler;
mod error;
mod expr;
mod field_resolver;
mod plan;
#[allow(dead_code)]
mod sorter;

// Reader Contract (for custom data sources)
pub mod source;

// Value types for query results
pub mod value;

// Catalog Support
pub use catalog::{CompilationCatalog, CompilationContext, ExecutionCatalog, ExecutionContext};

// Compilation & Execution
pub use compiler::PlanCompiler;
pub use plan::{CompiledPlan, EvaluationMode, ExecutionResult, PartiQLVM, QueryIterator};

// Error Handling
pub use error::{EngineError, Result};
