use crate::batch::VectorizedBatch;
use crate::error::EvalError;
use crate::reader::projection::{ProjectionSource, ProjectionSpec};

/// Phase 0 BatchReader trait - minimal, strict, vectorized interface for reading scalar data
pub trait BatchReader {
    /// Configure the reader with a fixed projection.
    /// Must be called exactly once before reading.
    ///
    /// The reader must either accept the projection (returning Ok(())) or reject it
    /// with a descriptive error. If accepted, all subsequent batches must conform
    /// exactly to the projection specification.
    fn set_projection(&mut self, spec: ProjectionSpec) -> Result<(), EvalError>;

    /// Initialize the reader
    fn open(&mut self) -> Result<(), EvalError>;

    /// Produce the next batch, or None at end of input.
    ///
    /// set_projection() must be called successfully before calling this method.
    /// All returned batches must have:
    /// - Vectors at indices specified by the projection
    /// - Types matching the declared LogicalTypes
    /// - Consistent row counts across all vectors
    fn next_batch(&mut self) -> Result<Option<VectorizedBatch>, EvalError>;

    /// Resolve a field name to a ProjectionSource
    /// Returns None if the field doesn't exist
    fn resolve(&self, field_name: &str) -> Option<ProjectionSource>;

    /// Clean up reader resources
    fn close(&mut self) -> Result<(), EvalError>;
}
