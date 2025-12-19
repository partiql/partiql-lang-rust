mod operator_trait;
mod scan;
mod filter;
mod project;

pub use operator_trait::VectorizedOperator;
pub use scan::VectorizedScan;
pub use filter::VectorizedFilter;
pub use project::VectorizedProject;
