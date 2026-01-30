//! Database catalog support for PartiQL
//!
//! This module provides the infrastructure for registering and querying data catalogs.
//! Catalogs provide table access via `ReaderFactory` instances, enabling qualified table
//! references like `my_catalog.schema.table`.

use crate::engine::reader::ReaderFactory;
use partiql_value::BindingsName;
use rustc_hash::FxHashMap;
use std::sync::Arc;

/// A data catalog provides table access by name/path.
///
/// Catalogs are registered with `CatalogRegistry` and queried during plan compilation.
/// Each catalog has a simple string name and can provide `ReaderFactory` instances for tables
/// identified by multi-part paths (where path components use `BindingsName` for case sensitivity).
///
/// # Examples
///
/// ```ignore
/// use partiql_eval::engine::catalog::DataCatalog;
/// use partiql_value::BindingsName;
///
/// struct MyCatalog {
///     name: String,
///     // ... table storage
/// }
///
/// impl DataCatalog for MyCatalog {
///     fn name(&self) -> &str {
///         &self.name
///     }
///     
///     fn get_table(&self, path: &[BindingsName<'_>]) -> Option<ReaderFactory> {
///         // Look up table by path
///         // ...
///         None
///     }
/// }
/// ```
pub trait DataCatalog: Send + Sync {
    /// Returns the catalog's name as a string.
    ///
    /// The name is used for catalog lookup in the registry.
    fn name(&self) -> &str;

    /// Gets a `ReaderFactory` for a table by path.
    ///
    /// Each path component has case sensitivity information via `BindingsName`.
    ///
    /// # Path Examples
    ///
    /// - Single table: `[BindingsName("users")]`
    /// - Schema.table: `[BindingsName("public"), BindingsName("users")]`
    /// - Multi-level: `[BindingsName("db"), BindingsName("public"), BindingsName("users")]`
    ///
    /// # Returns
    ///
    /// - `Some(ReaderFactory)` if the table exists
    /// - `None` if the table is not found
    fn get_table(&self, path: &[BindingsName<'_>]) -> Option<ReaderFactory>;
}

/// Registry for managing multiple data catalogs.
///
/// The registry stores catalogs by their string names and provides lookup functionality.
///
/// # Thread Safety
///
/// `CatalogRegistry` is `Send + Sync` since `DataCatalog` instances are
/// wrapped in `Arc` and the internal `FxHashMap` uses string keys.
///
/// # Examples
///
/// ```ignore
/// use partiql_eval::engine::catalog::CatalogRegistry;
/// use std::sync::Arc;
///
/// let mut registry = CatalogRegistry::new();
/// registry.register_catalog(Arc::new(my_catalog));
///
/// // Look up catalog by name
/// if let Some(catalog) = registry.get_catalog("my_catalog") {
///     if let Some(reader) = catalog.get_table(&table_path) {
///         // Use reader...
///     }
/// }
/// ```
#[derive(Clone, Default)]
pub struct CatalogRegistry {
    /// Maps catalog names to catalog instances.
    catalogs: FxHashMap<String, Arc<dyn DataCatalog>>,
}

impl CatalogRegistry {
    /// Creates a new empty catalog registry.
    #[must_use]
    pub fn new() -> Self {
        Self::default()
    }

    /// Registers a catalog in the registry.
    ///
    /// The catalog's name is used as the lookup key.
    /// If a catalog with the same name already exists, it will be replaced.
    ///
    /// # Examples
    ///
    /// ```ignore
    /// let mut registry = CatalogRegistry::new();
    /// registry.register_catalog(Arc::new(my_catalog));
    /// ```
    pub fn register_catalog(&mut self, catalog: Arc<dyn DataCatalog>) {
        let name = catalog.name().to_string();
        self.catalogs.insert(name, catalog);
    }

    /// Gets a catalog by name.
    ///
    /// # Returns
    ///
    /// - `Some(&Arc<dyn DataCatalog>)` if the catalog exists
    /// - `None` if no catalog with the given name is registered
    ///
    /// # Examples
    ///
    /// ```ignore
    /// if let Some(catalog) = registry.get_catalog("my_catalog") {
    ///     // Use catalog...
    /// }
    /// ```
    #[must_use]
    pub fn get_catalog(&self, name: &str) -> Option<&Arc<dyn DataCatalog>> {
        self.catalogs.get(name)
    }

    /// Returns the number of registered catalogs.
    #[must_use]
    pub fn len(&self) -> usize {
        self.catalogs.len()
    }

    /// Returns `true` if no catalogs are registered.
    #[must_use]
    pub fn is_empty(&self) -> bool {
        self.catalogs.is_empty()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    // Mock catalog for testing
    struct MockCatalog {
        name: String,
    }

    impl MockCatalog {
        fn new(name: &str) -> Self {
            MockCatalog {
                name: name.to_string(),
            }
        }
    }

    impl DataCatalog for MockCatalog {
        fn name(&self) -> &str {
            &self.name
        }

        fn get_table(&self, _path: &[BindingsName<'_>]) -> Option<ReaderFactory> {
            None
        }
    }

    #[test]
    fn test_registry_basic() {
        let mut registry = CatalogRegistry::new();
        let catalog = Arc::new(MockCatalog::new("my_catalog"));
        registry.register_catalog(catalog);

        // Should find with exact name
        assert!(registry.get_catalog("my_catalog").is_some());

        // Should not find with different name
        assert!(registry.get_catalog("other_catalog").is_none());
    }

    #[test]
    fn test_registry_empty() {
        let registry = CatalogRegistry::new();
        assert!(registry.is_empty());
        assert_eq!(registry.len(), 0);
    }

    #[test]
    fn test_registry_len() {
        let mut registry = CatalogRegistry::new();
        assert_eq!(registry.len(), 0);

        registry.register_catalog(Arc::new(MockCatalog::new("catalog1")));
        assert_eq!(registry.len(), 1);

        registry.register_catalog(Arc::new(MockCatalog::new("catalog2")));
        assert_eq!(registry.len(), 2);
    }

    #[test]
    fn test_registry_replace() {
        let mut registry = CatalogRegistry::new();
        let catalog1 = Arc::new(MockCatalog::new("my_catalog"));
        registry.register_catalog(catalog1);
        assert_eq!(registry.len(), 1);

        // Register another catalog with same name
        let catalog2 = Arc::new(MockCatalog::new("my_catalog"));
        registry.register_catalog(catalog2);
        assert_eq!(registry.len(), 1); // Should replace, not add
    }
}
