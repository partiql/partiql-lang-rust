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
use crate::engine::source::{DataSource, DataSourceHandle, ScanLayout};
use partiql_common::catalog::{CatalogId, EntryId};
use partiql_value::BindingsName;
use rustc_hash::FxHashMap;
use std::collections::HashMap;
use std::sync::Arc;

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

/// Execution-time catalog that creates DataSource instances from EntryIds.
///
/// Used during query execution to:
/// - Resolve EntryIds to actual data sources
/// - Provide access to the underlying data
/// - Enable data swapping without recompilation
///
/// Each ExecutionCatalog instance represents a specific dataset.
/// Different ExecutionCatalogs can provide different data for the same EntryIds.
pub trait ExecutionCatalog: Send + Sync {
    /// Create a DataSource for the given entry and layout.
    ///
    /// # Arguments
    ///
    /// * `entry_id` - The entry ID within this catalog (assigned during compilation)
    /// * `layout` - The scan layout specifying projection and optimization hints
    ///
    /// # Returns
    ///
    /// A boxed DataSource instance ready to read data.
    ///
    /// # Errors
    ///
    /// Returns an error if:
    /// - The entry_id is not found in this catalog
    /// - The data source cannot be created (e.g., file not found, connection failed)
    fn create(&self, entry_id: EntryId, layout: ScanLayout) -> Result<Box<dyn DataSource>>;
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
/// # Thread Safety
///
/// ExecutionContext can be created per-thread to provide different datasets
/// for the same compiled plan. Each thread maintains its own catalog mappings.
///
/// # Example
/// ```ignore
/// // Thread 1 - Dataset A
/// let mut exec_context = ExecutionContext::new();
/// exec_context.add_catalog(catalog_id, Arc::new(MyExecutionCatalog::new(dataset_a)));
/// vm.execute(&exec_context)?;
///
/// // Thread 2 - Dataset B (same catalog_id, different data)
/// let mut exec_context = ExecutionContext::new();
/// exec_context.add_catalog(catalog_id, Arc::new(MyExecutionCatalog::new(dataset_b)));
/// vm.execute(&exec_context)?;
/// ```
pub struct ExecutionContext {
    catalogs: FxHashMap<CatalogId, Arc<dyn ExecutionCatalog>>,
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
    pub fn add_catalog(&mut self, catalog_id: CatalogId, catalog: Arc<dyn ExecutionCatalog>) {
        self.catalogs.insert(catalog_id, catalog);
    }

    /// Get an execution catalog by its CatalogId.
    ///
    /// Returns `None` if no catalog with this ID exists in this context.
    pub fn get_catalog(&self, catalog_id: CatalogId) -> Option<&dyn ExecutionCatalog> {
        self.catalogs.get(&catalog_id).map(|arc| arc.as_ref())
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
    use crate::engine::source::{DataSourceConfig, ScanCapabilities, ScanSource};
    use partiql_common::catalog::EntryId;
    use std::sync::Arc;

    // Mock DataSourceConfig for testing
    struct MockConfig {
        caps: ScanCapabilities,
    }

    impl DataSourceConfig for MockConfig {
        fn caps(&self) -> ScanCapabilities {
            self.caps
        }

        fn resolve(&self, _field_name: &str) -> Option<ScanSource> {
            Some(ScanSource::ColumnIndex(0))
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
                    caps: ScanCapabilities {
                        stability: crate::engine::source::BufferStability::UntilNext,
                        can_project: true,
                        can_return_opaque: false,
                    },
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
            caps: ScanCapabilities {
                stability: crate::engine::source::BufferStability::UntilNext,
                can_project: true,
                can_return_opaque: false,
            },
        });

        let entry_id = EntryId::from(42);
        let handle = DataSourceHandle::new(entry_id, config);

        // Test entry_id accessor
        let retrieved_id = handle.entry_id().unwrap();
        assert_eq!(retrieved_id, EntryId::from(42));

        // Test caps delegation
        let caps = handle.caps();
        assert!(caps.can_project);
        assert!(!caps.can_return_opaque);

        // Test resolve delegation
        assert!(handle.resolve("any_field").is_some());
    }
}
