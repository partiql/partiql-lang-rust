// Module declarations - all modules are internal to the crate
pub(crate) mod api;
pub(crate) mod internal;
pub(crate) mod ion_reader;
pub(crate) mod mem_reader;

// Re-export ONLY public API types from api.rs - these are the only types visible outside the crate
pub use api::{
    BufferStability, DataSource, DataSourceConfig, DataSourceFactory, PhysicalType, ScanLayout,
    ScanProjection, ScanSource, ScanSourceType,
};

// Re-export ScanId and CatalogScans for catalog implementations
pub use crate::engine::catalog::CatalogScans;
pub use crate::engine::plan::ScanId;

// Internal types - re-exported as pub(crate) for use within the engine
pub(crate) use internal::{DataSourceFactoryInner, DataSourceImpl};

// Internal imports for use within this module only
use ion_reader::IonDataSourceFactory;
use mem_reader::InMemGeneratedDataSourceHandle;

use crate::engine::error::Result;
use partiql_common::catalog::EntryId;
use std::sync::Arc;

// RegisterWriter module
mod value_writer;
pub use value_writer::RegisterWriter;

/// Handle for a data source with compile-time metadata.
///
/// Supports two usage patterns:
/// 1. **Two-Phase Catalog** (ObjectId + Config): For catalog swapping use cases
/// 2. **Direct Factory**: For simple ion/mem use cases
///
/// # Two-Phase Catalog Pattern
/// ```ignore
/// // Compilation catalog provides handle with ObjectId (CatalogId + EntryId)
/// let config = Arc::new(MyTableConfig::new(columns));
/// let object_id = ObjectId::new(catalog_id, entry_id);
/// let handle = DataSourceHandle::new(object_id, config);
///
/// // Compiler uses for optimization
/// if handle.caps().can_project { /* ... */ }
///
/// // VM resolves at execution time via ExecutionContext
/// let catalog = execution_context.get_catalog(handle.object_id()?.catalog_id())?;
/// let data_source = catalog.create(handle.object_id()?.entry_id(), layout)?;
/// ```
///
/// # Direct Factory Pattern  
/// ```ignore
/// // Simple ion/mem readers - no catalog needed
/// let handle = DataSourceHandle::mem(100, vec!["id".to_string()]);
/// let handle = DataSourceHandle::ion("data.ion".to_string());
/// ```
#[derive(Clone)]
pub struct DataSourceHandle {
    inner: DataSourceHandleInner,
}

#[derive(Clone)]
enum DataSourceHandleInner {
    /// Two-phase catalog: EntryId + compile-time config (CatalogId comes from compiler)
    Catalog {
        entry_id: EntryId,
        config: Arc<dyn DataSourceConfig>,
    },
    /// Direct factory for ion/mem convenience
    Direct(DataSourceFactoryInner),
}

impl DataSourceHandle {
    /// Create a new DataSourceHandle with EntryId and configuration (two-phase catalog pattern).
    ///
    /// # Arguments
    /// * `entry_id` - EntryId for execution-time resolution (CatalogId comes from compiler context)
    /// * `config` - Compile-time metadata (caps, field resolution)
    pub fn new(entry_id: EntryId, config: Arc<dyn DataSourceConfig>) -> Self {
        DataSourceHandle {
            inner: DataSourceHandleInner::Catalog { entry_id, config },
        }
    }

    /// Get the EntryId for execution-time resolution (two-phase catalog pattern only).
    ///
    /// Returns `Some(entry_id)` for catalog-based handles, `None` for direct ion/mem handles.
    pub fn entry_id(&self) -> Option<EntryId> {
        match &self.inner {
            DataSourceHandleInner::Catalog { entry_id, .. } => Some(*entry_id),
            DataSourceHandleInner::Direct(_) => None,
        }
    }

    /// Create a reader factory for in-memory generated data (direct factory pattern).
    ///
    /// Generates rows with Int64 columns on-the-fly. All column values start at 0
    /// and increment by 1 for each row.
    ///
    /// # Arguments
    /// * `total_rows` - Number of rows to generate
    /// * `column_names` - Names of the columns in order
    pub fn mem(total_rows: usize, column_names: Vec<String>) -> Self {
        DataSourceHandle {
            inner: DataSourceHandleInner::Direct(DataSourceFactoryInner::InMem(
                InMemGeneratedDataSourceHandle::new(total_rows, column_names),
            )),
        }
    }

    /// Create a reader factory for Ion text files (direct factory pattern).
    ///
    /// Reads Ion data from the specified file path with projection pushdown support.
    pub fn ion(path: String) -> Self {
        DataSourceHandle {
            inner: DataSourceHandleInner::Direct(DataSourceFactoryInner::Ion(
                IonDataSourceFactory::new(path),
            )),
        }
    }

    /// Get buffer stability at compile time.
    ///
    /// Returns information about how long data in buffers remains valid.
    /// This allows the compiler to make informed decisions about query optimization.
    pub fn buffer_stability(&self) -> BufferStability {
        match &self.inner {
            DataSourceHandleInner::Catalog { config, .. } => config.buffer_stability(),
            DataSourceHandleInner::Direct(factory) => factory.buffer_stability(),
        }
    }

    /// Resolve a field name to a ScanSource at compile time.
    ///
    /// Returns Some(ScanSource) if the reader can provide the field, None otherwise.
    /// This enables compile-time validation of field references.
    pub fn resolve(&self, field_name: &str) -> Option<ScanSource> {
        match &self.inner {
            DataSourceHandleInner::Catalog { config, .. } => config.resolve(field_name),
            DataSourceHandleInner::Direct(factory) => factory.resolve(field_name),
        }
    }
}

/// Public factory for creating data sources with uniform API.
///
/// Provides static methods for built-in sources (mem, ion) and accepts
/// custom implementations via the `custom()` method.
///
/// This enables customers to implement custom readers while providing
/// convenient access to built-in readers.
///
/// # Examples
/// ```ignore
/// // Built-in memory source
/// let factory = CompiledSourceFactory::mem(10_000, vec!["id".to_string(), "name".to_string()]);
///
/// // Built-in Ion source
/// let factory = CompiledSourceFactory::ion("data.ion".to_string());
///
/// // Custom source (e.g., random data, CSV, database connection)
/// let custom: Box<dyn DataSourceFactory> = Box::new(MyCustomFactory::new());
/// let factory = CompiledSourceFactory::custom(custom);
/// ```
#[derive(Clone)]
pub struct CompiledSourceFactory {
    inner: CompiledSourceFactoryInner,
}

enum CompiledSourceFactoryInner {
    InMem(InMemGeneratedDataSourceHandle),
    Ion(IonDataSourceFactory),
    Custom(Arc<dyn DataSourceFactory>),
}

impl Clone for CompiledSourceFactoryInner {
    fn clone(&self) -> Self {
        match self {
            CompiledSourceFactoryInner::InMem(f) => CompiledSourceFactoryInner::InMem(f.clone()),
            CompiledSourceFactoryInner::Ion(f) => CompiledSourceFactoryInner::Ion(f.clone()),
            CompiledSourceFactoryInner::Custom(f) => CompiledSourceFactoryInner::Custom(f.clone()),
        }
    }
}

impl CompiledSourceFactory {
    /// Create an in-memory data source factory.
    ///
    /// Generates rows with sequential integer values starting from 0.
    /// All columns have the same value for each row.
    pub fn mem(total_rows: usize, column_names: Vec<String>) -> Self {
        Self {
            inner: CompiledSourceFactoryInner::InMem(InMemGeneratedDataSourceHandle::new(
                total_rows,
                column_names,
            )),
        }
    }

    /// Create an Ion file data source factory.
    ///
    /// Reads Ion-formatted data from the specified file path.
    pub fn ion(file_path: String) -> Self {
        Self {
            inner: CompiledSourceFactoryInner::Ion(IonDataSourceFactory::new(file_path)),
        }
    }

    /// Create a custom data source factory.
    ///
    /// Accepts any implementation of the DataSourceFactory trait,
    /// enabling customers to provide their own data readers.
    pub fn custom(factory: Box<dyn DataSourceFactory>) -> Self {
        Self {
            inner: CompiledSourceFactoryInner::Custom(Arc::from(factory)),
        }
    }

    /// Create a DataSource instance with the given scan layout.
    ///
    /// This is called by ExecutionCatalog implementations to instantiate
    /// the actual data source for query execution.
    pub fn create(&self, layout: ScanLayout) -> Result<Box<dyn DataSource>> {
        match &self.inner {
            CompiledSourceFactoryInner::InMem(f) => f.create(layout),
            CompiledSourceFactoryInner::Ion(f) => f.create(layout),
            CompiledSourceFactoryInner::Custom(f) => f.create(layout),
        }
    }

    /// Get buffer stability of this data source factory.
    pub fn buffer_stability(&self) -> BufferStability {
        match &self.inner {
            CompiledSourceFactoryInner::InMem(f) => f.buffer_stability(),
            CompiledSourceFactoryInner::Ion(f) => f.buffer_stability(),
            CompiledSourceFactoryInner::Custom(f) => f.buffer_stability(),
        }
    }

    /// Resolve a field name to a scan source.
    pub fn resolve(&self, field_name: &str) -> Option<ScanSource> {
        match &self.inner {
            CompiledSourceFactoryInner::InMem(f) => f.resolve(field_name),
            CompiledSourceFactoryInner::Ion(f) => f.resolve(field_name),
            CompiledSourceFactoryInner::Custom(f) => f.resolve(field_name),
        }
    }
}
