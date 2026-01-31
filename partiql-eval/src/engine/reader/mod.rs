// Module declarations - all modules are internal to the crate
pub(crate) mod api;
pub(crate) mod internal;
pub(crate) mod ion_reader;
pub(crate) mod mem_reader;

// Re-export ONLY public API types from api.rs - these are the only types visible outside the crate
pub use api::{
    BufferStability, ReaderCaps, RowReader, RowReaderFactory, ScanLayout, ScanProjection,
    ScanSource, TypeHint,
};

// Internal types - re-exported as pub(crate) for use within the engine
pub(crate) use internal::{ReaderFactoryInner, ReaderImpl};

// Internal imports for use within this module only
use ion_reader::{IonRowReader, IonRowReaderFactory};
use mem_reader::{InMemGeneratedReader, InMemGeneratedReaderFactory};

use crate::engine::error::Result;
use std::sync::Arc;

/// Public facade for creating row readers
///
/// Provides a simple API for creating common reader types without exposing
/// internal implementation details. Users only need visibility of this struct
/// and the RowReaderFactory trait.
///
/// # Examples
/// ```ignore
/// // In-memory generated reader with custom column names
/// let reader = ReaderFactory::mem(100, vec!["id".to_string(), "value".to_string()]);
///
/// // Ion file reader
/// let reader = ReaderFactory::ion("data.ion".to_string());
///
/// // Custom reader via trait
/// let reader = ReaderFactory::custom(Box::new(my_custom_factory));
/// ```
#[derive(Clone)]
pub struct ReaderFactory {
    inner: ReaderFactoryInner,
}

impl ReaderFactory {
    /// Create a reader factory for in-memory generated data
    ///
    /// Generates rows with Int64 columns on-the-fly. All column values start at 0
    /// and increment by 1 for each row.
    ///
    /// # Arguments
    /// * `total_rows` - Number of rows to generate
    /// * `column_names` - Names of the columns in order
    pub fn mem(total_rows: usize, column_names: Vec<String>) -> Self {
        ReaderFactory {
            inner: ReaderFactoryInner::InMem(InMemGeneratedReaderFactory::new(
                total_rows,
                column_names,
            )),
        }
    }

    /// Create a reader factory for Ion text files
    ///
    /// Reads Ion data from the specified file path with projection pushdown support.
    pub fn ion(path: String) -> Self {
        ReaderFactory {
            inner: ReaderFactoryInner::Ion(IonRowReaderFactory::new(path)),
        }
    }

    /// Create a reader factory from a custom implementation
    ///
    /// Allows users to provide their own reader implementation via the RowReaderFactory trait.
    /// Uses Arc for cheap cloning since trait objects aren't directly clonable.
    ///
    /// # Example
    /// ```ignore
    /// let factory = Arc::new(MyCustomFactory::new());
    /// let reader = ReaderFactory::custom(factory);
    /// ```
    pub fn custom(factory: Arc<dyn RowReaderFactory>) -> Self {
        ReaderFactory {
            inner: ReaderFactoryInner::Custom(factory),
        }
    }

    /// Get reader capabilities at compile time
    ///
    /// Returns information about what the reader supports, such as projection pushdown.
    /// This allows the compiler to make informed decisions about query optimization.
    pub fn caps(&self) -> ReaderCaps {
        self.inner.caps()
    }

    /// Resolve a field name to a ScanSource at compile time
    ///
    /// Returns Some(ScanSource) if the reader can provide the field, None otherwise.
    /// This enables compile-time validation of field references.
    pub fn resolve(&self, field_name: &str) -> Option<ScanSource> {
        self.inner.resolve(field_name)
    }

    /// Internal method for creating a ReaderImpl with static dispatch
    ///
    /// This method is used internally by PipelineOp to get a ReaderImpl enum
    /// which enables static dispatch for InMem and Ion readers.
    pub(crate) fn create_impl(&self, layout: ScanLayout) -> Result<ReaderImpl> {
        match &self.inner {
            ReaderFactoryInner::InMem(factory) => Ok(ReaderImpl::InMem(InMemGeneratedReader::new(
                factory.total_rows,
                factory.column_names.len(),
                layout,
            ))),
            ReaderFactoryInner::Ion(factory) => Ok(ReaderImpl::Ion(IonRowReader::new(
                factory.path.clone(),
                layout,
            ))),
            ReaderFactoryInner::Custom(factory) => {
                // Wrap the Box<dyn RowReader> in ReaderImpl::Custom
                Ok(ReaderImpl::Custom(factory.create(layout)?))
            }
        }
    }
}
