// Module declarations
pub(crate) mod api;

// Re-export public API types from api.rs
pub use api::{
    BufferStability, DataSource, DataSourceMetadata, PhysicalType, ScanLayout, ScanProjection,
    ScanSource, ScanSourceType, TableFunction, TableFunctionHandle,
};

// Re-export ScanId and CatalogScans for catalog implementations
pub use crate::engine::catalog::CatalogScans;
pub use crate::engine::plan::ScanId;

// RegisterWriter module
mod value_writer;
pub use value_writer::{RegisterWriter, ValueWriter};

use crate::engine::error::Result;
use crate::engine::value::{ValueOwned, ValueRef};
use partiql_common::catalog::EntryId;
use std::sync::Arc;

/// Internal wrapper for data source implementations.
///
/// Used by the VM to dispatch to catalog-provided data sources.
pub(crate) enum DataSourceImpl {
    Catalog(Box<dyn DataSource>),
    Inline(InlineDataSource),
}

impl DataSourceImpl {
    pub fn open(&mut self) -> Result<()> {
        match self {
            DataSourceImpl::Catalog(ds) => ds.open(),
            DataSourceImpl::Inline(ds) => ds.open(),
        }
    }

    pub fn next_row(&mut self, writer: &mut RegisterWriter<'_, '_>) -> Result<bool> {
        match self {
            DataSourceImpl::Catalog(ds) => ds.next_row(writer),
            DataSourceImpl::Inline(ds) => ds.next_row(writer),
        }
    }

    pub fn close(&mut self) -> Result<()> {
        match self {
            DataSourceImpl::Catalog(ds) => ds.close(),
            DataSourceImpl::Inline(ds) => ds.close(),
        }
    }
}

/// Data source backed by an inline collection of owned values.
pub(crate) struct InlineDataSource {
    pub(crate) values: Vec<ValueOwned>,
    position: usize,
}

impl InlineDataSource {
    pub fn new(values: Vec<ValueOwned>) -> Self {
        InlineDataSource {
            values,
            position: 0,
        }
    }

    fn open(&mut self) -> Result<()> {
        self.position = 0;
        Ok(())
    }

    fn next_row(&mut self, writer: &mut RegisterWriter<'_, '_>) -> Result<bool> {
        if self.position >= self.values.len() {
            return Ok(false);
        }
        let value = &self.values[self.position];
        self.position += 1;
        Self::write_value_to_register(value, writer)
    }

    fn close(&mut self) -> Result<()> {
        Ok(())
    }
}

/// Handle returned by `CompilationCatalog::get_table()` during query compilation.
///
/// Contains the `EntryId` for execution-time data source resolution and
/// compile-time metadata for query optimization. The compiler uses the metadata
/// to resolve field references and determine buffer handling strategies, while
/// the `EntryId` is stored in the compiled plan for later use by the
/// `ExecutionCatalog` to create actual `DataSource` instances.
///
/// # Example
/// ```ignore
/// impl CompilationCatalog for MyCompilationCatalog {
///     fn get_table(&self, path: &[BindingsName<'_>]) -> Option<DataSourceHandle> {
///         let table_name = extract_table_name(path)?;
///         let meta = self.tables.get(&table_name)?;
///         
///         let metadata = Arc::new(MyTableMetadata::new(meta.columns.clone()));
///         Some(DataSourceHandle::new(meta.entry_id, metadata))
///     }
/// }
/// ```
#[derive(Clone)]
pub struct DataSourceHandle {
    /// Catalog-assigned identifier for execution-time resolution.
    ///
    /// Combined with `CatalogId` from the compilation context to form an
    /// `ObjectId` that uniquely identifies the data source.
    pub entry_id: EntryId,

    /// Compile-time metadata for query optimization.
    ///
    /// Provides buffer stability and field resolution without coupling
    /// to execution-time data access.
    pub metadata: Arc<dyn DataSourceMetadata>,
}

impl DataSourceHandle {
    /// Create a new DataSourceHandle with the given entry ID and metadata.
    ///
    /// # Arguments
    /// * `entry_id` - Catalog-assigned identifier for execution-time resolution
    /// * `metadata` - Compile-time metadata for query optimization
    pub fn new(entry_id: EntryId, metadata: Arc<dyn DataSourceMetadata>) -> Self {
        DataSourceHandle { entry_id, metadata }
    }

    /// Get buffer stability at compile time.
    ///
    /// Delegates to the underlying metadata. Returns information about how
    /// long data in buffers remains valid after read operations.
    pub fn buffer_stability(&self) -> BufferStability {
        self.metadata.buffer_stability()
    }

    /// Resolve a field name to a ScanSource at compile time.
    ///
    /// Delegates to the underlying metadata. Returns `Some(ScanSource)` if
    /// the data source can provide the field, `None` otherwise.
    pub fn resolve(&self, field_name: &str) -> Option<ScanSource> {
        self.metadata.resolve(field_name)
    }
}

/// Data source backed by an inline collection of owned values.
impl InlineDataSource {
    /// Write a value to the register using unsafe lifetime extension.
    ///
    /// Safety: InlineDataSource lives in PartiQLVM alongside the arena and registers.
    /// The values persist for the VM's lifetime, which outlives any single row iteration.
    fn write_value_to_register<'w, 'a>(
        value: &ValueOwned,
        writer: &mut RegisterWriter<'w, 'a>,
    ) -> Result<bool> {
        // Safety: value lives in InlineDataSource which lives in PartiQLVM.
        // The arena and registers also live in PartiQLVM. The value outlives
        // the per-row arena reset cycle.
        let value_static: &'a ValueOwned = unsafe { &*(value as *const ValueOwned) };
        let value_ref = ValueRef::from_owned(value_static, writer.arena);
        writer.regs[0] = value_ref;
        Ok(true)
    }
}
