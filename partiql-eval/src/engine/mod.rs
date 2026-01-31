pub mod catalog;
pub mod compiler;
pub mod error;
pub mod expr;
pub mod plan;
pub mod row;
pub mod source;
pub mod value;

pub use catalog::{CatalogRegistry, DataCatalog};
pub use compiler::{PlanCompiler, ScanProvider};
pub use error::{EngineError, Result};
pub use expr::SlotResolver;
pub(crate) use expr::UdfRegistry;
pub use plan::{CompiledPlan, ExecutionResult, PartiQLVM, QueryIterator, Schema};
