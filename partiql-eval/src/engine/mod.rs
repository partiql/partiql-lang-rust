pub mod arena;
pub(crate) mod builtins;
pub mod catalog;
pub mod compiler;
pub mod error;
pub mod expr;
pub mod field_resolver;
pub mod plan;
pub mod source;
pub mod value;

pub use catalog::{CompilationCatalog, CompilationContext, ExecutionCatalog, ExecutionContext};
pub use compiler::PlanCompiler;
pub use error::{EngineError, Result};
pub use expr::SlotResolver;
pub use plan::{CompiledPlan, ExecutionResult, PartiQLVM, QueryIterator};
