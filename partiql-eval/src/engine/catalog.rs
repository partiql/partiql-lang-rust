//! Database catalog support for PartiQL
//!
//! This module provides a two-phase catalog architecture that separates compilation-time
//! metadata from execution-time data access, enabling query plan reuse across different datasets.
//!
//! # Two-Phase Catalog Design
//!
//! ## CompilationCatalog
//! Used during query compilation to provide table metadata (schema, capabilities) and assign
//! EntryIds. The same CompilationCatalog can be used for multiple datasets with the same schema.
//!
//! ## ExecutionCatalog  
//! Used during query execution to create actual DataSource instances from EntryIds.
//! Different ExecutionCatalogs can provide different data for the same EntryIds, enabling
//! compiled plan reuse across datasets.
//!
//! # Example
//!
//! ```ignore
//! // Compile once with metadata
//! let compilation_catalog = MyCompilationCatalog::new();
//! let compiled = compiler.compile(&logical, &compilation_catalog)?;
//!
//! // Execute with dataset A
//! let exec_catalog_a = MyExecutionCatalog::new(dataset_a);
//! let mut vm = PartiQLVM::new(compiled.clone(), Arc::new(exec_catalog_a))?;
//! vm.execute()?;
//!
//! // Execute with dataset B (reuse compiled plan!)
//! let exec_catalog_b = MyExecutionCatalog::new(dataset_b);
//! let mut vm = PartiQLVM::new(compiled.clone(), Arc::new(exec_catalog_b))?;
//! vm.execute()?;
//! ```

use crate::engine::error::Result;
use crate::engine::plan::ScanId;
use crate::engine::source::{DataSource, DataSourceHandle, ScanLayout};
use partiql_common::catalog::{CatalogId, EntryId};
use partiql_value::BindingsName;
use rustc_hash::FxHashMap;
use std::collections::HashMap;
use std::sync::Arc;

/// Information about scans belonging to a specific catalog.
///
/// This is passed to `ExecutionCatalog::prepare()` and contains only the scans
/// that belong to that catalog, pre-filtered by CatalogId.
#[derive(Debug, Clone, Default)]
pub struct CatalogScans {
    /// Mapping from ScanId to (EntryId, ScanLayout) for scans in this catalog
    scans: Vec<(ScanId, EntryId, ScanLayout)>,
}

impl CatalogScans {
    /// Create a new empty CatalogScans
    pub fn new() -> Self {
        CatalogScans { scans: Vec::new() }
    }

    /// Add a scan to this catalog's scans
    pub fn add(&mut self, scan_id: ScanId, entry_id: EntryId, layout: ScanLayout) {
        self.scans.push((scan_id, entry_id, layout));
    }

    /// Iterate over all scans
    pub fn iter(&self) -> impl Iterator<Item = (ScanId, EntryId, &ScanLayout)> {
        self.scans
            .iter()
            .map(|(scan_id, entry_id, layout)| (*scan_id, *entry_id, layout))
    }

    /// Check if this catalog has any scans
    pub fn is_empty(&self) -> bool {
        self.scans.is_empty()
    }

    /// Get the number of scans
    pub fn len(&self) -> usize {
        self.scans.len()
    }
}

/// Compilation-time catalog that provides table metadata and EntryIds.
///
/// Used during query compilation to:
/// - Validate table existence
/// - Provide schema information (via DataSourceHandle)
/// - Assign EntryIds for execution-time resolution
/// - Enable optimization decisions (projection pushdown, etc.)
///
/// The CompilationCatalog is data-independent - it only provides metadata.
/// Multiple datasets with the same schema can share the same CompilationCatalog.
pub trait CompilationCatalog: Send + Sync {
    /// Get table metadata by path.
    ///
    /// Returns a `DataSourceHandle` containing:
    /// - EntryId for execution-time resolution
    /// - DataSourceConfig for compile-time metadata (caps, field resolution)
    ///
    /// # Path Examples
    ///
    /// - Single table: `[BindingsName("users")]`
    /// - Schema.table: `[BindingsName("public"), BindingsName("users")]`
    /// - Multi-level: `[BindingsName("db"), BindingsName("public"), BindingsName("users")]`
    ///
    /// # Returns
    ///
    /// - `Some(DataSourceHandle)` if the table exists
    /// - `None` if the table is not found
    fn get_table(&self, path: &[BindingsName<'_>]) -> Option<DataSourceHandle>;
}

/// Execution-time catalog that creates DataSource instances from ScanIds.
///
/// Used during query execution to:
/// - Resolve ScanIds to actual data sources
/// - Provide access to the underlying data
/// - Enable data swapping without recompilation
///
/// # Pre-Processing Pattern
///
/// Customers are expected to inspect the `CompiledPlan` **before** execution and build
/// their internal mappings from ScanId to data source implementations. Use
/// `CompiledPlan::scans()` to iterate over all scans and `CompiledPlan::get_scan(scan_id)`
/// to retrieve the `ScanMetadata` (layout, object_id) for each scan.
///
/// # Example
/// ```ignore
/// // Setup phase: inspect plan and prepare catalog
/// let compiled = compiler.compile(&logical)?;
/// let mut my_catalog = MyExecutionCatalog::new();
///
/// for (scan_id, scan_meta) in compiled.scans() {
///     // Customer builds their own mapping based on scan metadata
///     let layout = &scan_meta.layout;
///     let object_id = &scan_meta.object_id;
///     my_catalog.prepare_scan(scan_id, object_id.entry_id(), layout.clone());
/// }
///
/// // Execution phase: VM calls create() with just the ScanId
/// let exec_context = ExecutionContext::new();
/// exec_context.add_catalog(catalog_id, Arc::new(my_catalog));
/// let mut vm = PartiQLVM::new(compiled, &exec_context)?;
/// ```
pub trait ExecutionCatalog: Send + Sync {
    /// Prepare the catalog with its assigned scans.
    ///
    /// Called to build internal mappings from ScanId to data source implementations.
    /// Must be called after compilation but before execution.
    ///
    /// Use `CompiledPlan::scans_for_catalog(catalog_id)` to extract the scans
    /// that belong to this catalog.
    ///
    /// # Arguments
    ///
    /// * `scans` - The scans assigned to this catalog (pre-filtered by CatalogId)
    fn prepare(&mut self, scans: &CatalogScans);

    /// Create a DataSource for the given scan ID.
    ///
    /// The catalog is expected to have already been prepared via `prepare()`.
    /// This method is called during VM instantiation for each scan in the plan.
    ///
    /// # Arguments
    ///
    /// * `scan_id` - Unique identifier for this scan operation
    ///
    /// # Returns
    ///
    /// A boxed DataSource instance ready to read data.
    ///
    /// # Errors
    ///
    /// Returns an error if:
    /// - The scan_id is not found in this catalog's mappings
    /// - The data source cannot be created (e.g., file not found, connection failed)
    fn create(&self, scan_id: ScanId) -> Result<Box<dyn DataSource>>;
}

/// Registry that maps catalog names to CompilationCatalog instances.
///
/// Used during compilation to resolve catalog names in queries (e.g., `FROM catalog.table`)
/// to actual CompilationCatalog instances. Returns CatalogIds that can be used to set up
/// ExecutionContext for execution-time catalog resolution.
///
/// # Example
/// ```ignore
/// let mut context = CompilationContext::new();
/// let catalog_id = context.add_catalog("main", Arc::new(main_catalog));
///
/// // Save catalog_id for execution time
/// // Query: SELECT * FROM main.users
/// // Compiler uses context to find "main" catalog
/// ```
pub struct CompilationContext {
    next_catalog_id: u64,
    catalogs: HashMap<CatalogId, Arc<dyn CompilationCatalog>>,
    name_to_id: HashMap<String, CatalogId>,
}

impl CompilationContext {
    /// Create a new empty CompilationContext.
    pub fn new() -> Self {
        CompilationContext {
            next_catalog_id: 0,
            catalogs: HashMap::new(),
            name_to_id: HashMap::new(),
        }
    }

    /// Add a catalog with the given name and return its CatalogId.
    ///
    /// The returned CatalogId should be used when setting up ExecutionContext
    /// to map the same catalog ID to an ExecutionCatalog instance.
    ///
    /// If a catalog with this name already exists, it will be replaced and
    /// a new CatalogId will be generated.
    pub fn add_catalog(
        &mut self,
        name: impl Into<String>,
        catalog: Arc<dyn CompilationCatalog>,
    ) -> CatalogId {
        let id = CatalogId::from(self.next_catalog_id);
        self.next_catalog_id += 1;
        let name = name.into();
        self.catalogs.insert(id, catalog);
        self.name_to_id.insert(name, id);
        id
    }

    /// Get a catalog by name, returning both its ID and reference.
    ///
    /// Returns `None` if no catalog with this name exists.
    pub fn get_catalog(&self, name: &str) -> Option<(CatalogId, &dyn CompilationCatalog)> {
        let id = self.name_to_id.get(name)?;
        let catalog = self.catalogs.get(id)?;
        Some((*id, catalog.as_ref()))
    }
}

impl Default for CompilationContext {
    fn default() -> Self {
        Self::new()
    }
}

/// Registry that maps CatalogIds to ExecutionCatalog instances.
///
/// Used during query execution to resolve CatalogIds (from ObjectIds in the compiled plan)
/// to ExecutionCatalog instances that provide access to actual data.
///
/// # Single-Threaded Design
///
/// ExecutionContext is designed for single-threaded use. Each execution should have
/// its own ExecutionContext instance with its own catalog instances.
///
/// # Example
/// ```ignore
/// let mut exec_context = ExecutionContext::new();
/// exec_context.add_catalog(catalog_id, Box::new(MyExecutionCatalog::new(dataset)));
/// vm.execute(&exec_context)?;
/// ```
pub struct ExecutionContext {
    catalogs: FxHashMap<CatalogId, Box<dyn ExecutionCatalog>>,
}

impl ExecutionContext {
    /// Create a new empty ExecutionContext.
    pub fn new() -> Self {
        ExecutionContext {
            catalogs: FxHashMap::default(),
        }
    }

    /// Add an execution catalog with the given CatalogId.
    ///
    /// The CatalogId should match the one returned by CompilationContext::add_catalog()
    /// during compilation. This allows the execution context to resolve the same
    /// logical catalog to a different physical dataset.
    ///
    /// If a catalog with this ID already exists, it will be replaced.
    pub fn add_catalog(&mut self, catalog_id: CatalogId, catalog: Box<dyn ExecutionCatalog>) {
        self.catalogs.insert(catalog_id, catalog);
    }

    /// Get an execution catalog by its CatalogId.
    ///
    /// Returns `None` if no catalog with this ID exists in this context.
    pub fn get_catalog(&self, catalog_id: CatalogId) -> Option<&dyn ExecutionCatalog> {
        self.catalogs.get(&catalog_id).map(|b| b.as_ref())
    }

    /// Get mutable access to an execution catalog by its CatalogId.
    ///
    /// Returns `None` if no catalog with this ID exists in this context.
    /// Use this to call `prepare()` on catalogs before execution.
    pub fn get_catalog_mut(
        &mut self,
        catalog_id: CatalogId,
    ) -> Option<&mut (dyn ExecutionCatalog + '_)> {
        if let Some(b) = self.catalogs.get_mut(&catalog_id) {
            Some(b.as_mut())
        } else {
            None
        }
    }
}

impl Default for ExecutionContext {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::engine::source::{BufferStability, DataSourceConfig, PhysicalType, ScanSource};
    use partiql_common::catalog::EntryId;
    use std::sync::Arc;

    // Mock DataSourceConfig for testing
    struct MockConfig {
        stability: BufferStability,
    }

    impl DataSourceConfig for MockConfig {
        fn buffer_stability(&self) -> BufferStability {
            self.stability
        }

        fn resolve(&self, _field_name: &str) -> Option<ScanSource> {
            Some(ScanSource::column(0, PhysicalType::I64))
        }
    }

    // Mock CompilationCatalog for testing
    struct MockCompilationCatalog {
        entry_id: EntryId,
    }

    impl CompilationCatalog for MockCompilationCatalog {
        fn get_table(&self, path: &[BindingsName<'_>]) -> Option<DataSourceHandle> {
            let table_name = match path.get(0)? {
                BindingsName::CaseSensitive(s) => s.as_ref(),
                BindingsName::CaseInsensitive(s) => s.as_ref(),
            };

            if table_name == "test_table" {
                let config = Arc::new(MockConfig {
                    stability: BufferStability::UntilNext,
                });
                Some(DataSourceHandle::new(self.entry_id, config))
            } else {
                None
            }
        }
    }

    #[test]
    fn test_compilation_catalog_basic() {
        let catalog = MockCompilationCatalog {
            entry_id: EntryId::from(1),
        };

        // Should find existing table
        let handle = catalog.get_table(&[BindingsName::CaseInsensitive("test_table".into())]);
        assert!(handle.is_some());

        // Should not find non-existent table
        let handle = catalog.get_table(&[BindingsName::CaseInsensitive("missing_table".into())]);
        assert!(handle.is_none());
    }

    #[test]
    fn test_data_source_handle() {
        let config = Arc::new(MockConfig {
            stability: BufferStability::UntilNext,
        });

        let entry_id = EntryId::from(42);
        let handle = DataSourceHandle::new(entry_id, config);

        // Test entry_id accessor
        let retrieved_id = handle.entry_id().unwrap();
        assert_eq!(retrieved_id, EntryId::from(42));

        // Test buffer_stability delegation
        assert!(matches!(
            handle.buffer_stability(),
            BufferStability::UntilNext
        ));

        // Test resolve delegation
        let resolved = handle.resolve("any_field");
        assert!(resolved.is_some());
        let scan_source = resolved.unwrap();
        assert_eq!(scan_source.physical_type, PhysicalType::I64);
    }
}
