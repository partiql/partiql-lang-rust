// Module declarations - all modules are internal to the crate
pub(crate) mod api;
pub(crate) mod internal;
pub(crate) mod ion_reader;
pub(crate) mod mem_reader;

// Re-export ONLY public API types from api.rs - these are the only types visible outside the crate
pub use api::{
    BufferStability, DataSource, DataSourceFactory, ScanCapabilities, ScanLayout, ScanProjection,
    ScanSource, TypeHint,
};

// Internal types - re-exported as pub(crate) for use within the engine
pub(crate) use internal::{DataSourceFactoryInner, DataSourceImpl};

// Internal imports for use within this module only
use ion_reader::{IonDataSource, IonDataSourceFactory};
use mem_reader::{InMemGeneratedDataSourceHandle, InMemGeneratedReader};

use crate::engine::error::Result;
use std::sync::Arc;

// RegisterWriter module
mod value_writer;
pub use value_writer::RegisterWriter;

/// Public facade for creating row readers
///
/// Provides a simple API for creating common reader types without exposing
/// internal implementation details. Users only need visibility of this struct
/// and the DataSourceFactory trait.
///
/// # Examples
/// ```ignore
/// // In-memory generated reader with custom column names
/// let reader = DataSourceHandle::mem(100, vec!["id".to_string(), "value".to_string()]);
///
/// // Ion file reader
/// let reader = DataSourceHandle::ion("data.ion".to_string());
///
/// // Custom reader via trait
/// let reader = DataSourceHandle::custom(Box::new(my_custom_factory));
/// ```
#[derive(Clone)]
pub struct DataSourceHandle {
    inner: DataSourceFactoryInner,
}

impl DataSourceHandle {
    /// Create a reader factory for in-memory generated data
    ///
    /// Generates rows with Int64 columns on-the-fly. All column values start at 0
    /// and increment by 1 for each row.
    ///
    /// # Arguments
    /// * `total_rows` - Number of rows to generate
    /// * `column_names` - Names of the columns in order
    pub fn mem(total_rows: usize, column_names: Vec<String>) -> Self {
        DataSourceHandle {
            inner: DataSourceFactoryInner::InMem(InMemGeneratedDataSourceHandle::new(
                total_rows,
                column_names,
            )),
        }
    }

    /// Create a reader factory for Ion text files
    ///
    /// Reads Ion data from the specified file path with projection pushdown support.
    pub fn ion(path: String) -> Self {
        DataSourceHandle {
            inner: DataSourceFactoryInner::Ion(IonDataSourceFactory::new(path)),
        }
    }

    /// Create a reader factory from a custom implementation
    ///
    /// Allows users to provide their own reader implementation via the DataSourceFactory trait.
    /// Uses Arc for cheap cloning since trait objects aren't directly clonable.
    ///
    /// # Example
    /// ```ignore
    /// let factory = Arc::new(MyCustomFactory::new());
    /// let reader = DataSourceHandle::custom(factory);
    /// ```
    pub fn custom(factory: Arc<dyn DataSourceFactory>) -> Self {
        DataSourceHandle {
            inner: DataSourceFactoryInner::Custom(factory),
        }
    }

    /// Get reader capabilities at compile time
    ///
    /// Returns information about what the reader supports, such as projection pushdown.
    /// This allows the compiler to make informed decisions about query optimization.
    pub fn caps(&self) -> ScanCapabilities {
        self.inner.caps()
    }

    /// Resolve a field name to a ScanSource at compile time
    ///
    /// Returns Some(ScanSource) if the reader can provide the field, None otherwise.
    /// This enables compile-time validation of field references.
    pub fn resolve(&self, field_name: &str) -> Option<ScanSource> {
        self.inner.resolve(field_name)
    }

    /// Internal method for creating a DataSourceImpl with static dispatch
    ///
    /// This method is used internally by PipelineOp to get a DataSourceImpl enum
    /// which enables static dispatch for InMem and Ion readers.
    pub(crate) fn create_impl(&self, layout: ScanLayout) -> Result<DataSourceImpl> {
        match &self.inner {
            DataSourceFactoryInner::InMem(factory) => Ok(DataSourceImpl::InMem(
                InMemGeneratedReader::new(factory.total_rows, factory.column_names.len(), layout),
            )),
            DataSourceFactoryInner::Ion(factory) => Ok(DataSourceImpl::Ion(IonDataSource::new(
                factory.path.clone(),
                layout,
            ))),
            DataSourceFactoryInner::Custom(factory) => {
                // Wrap the Box<dyn DataSource> in DataSourceImpl::Custom
                Ok(DataSourceImpl::Custom(factory.create(layout)?))
            }
        }
    }
}
