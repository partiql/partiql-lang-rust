//! The planning session: the current catalog and namespace, the catalogs, and the routine path.

use super::catalog::{Catalogs, SystemCatalog, TypedCatalog};
use super::identifier::{Namespace, Path};
use std::collections::BTreeMap;
use std::fmt;
use std::sync::Arc;

/// The errors of [`SessionBuilder::build`].
#[derive(thiserror::Error, Debug, Clone, PartialEq, Eq)]
#[non_exhaustive]
pub enum SessionError {
    /// The current catalog was not set.
    #[error("Session catalog must be set")]
    MissingCatalog,
    /// Two catalogs, including the system catalog, have this name.
    #[error("Catalog names must be unique: {0}")]
    DuplicateCatalogName(String),
    /// The current catalog is not one of the catalogs (nor the system catalog).
    #[error("Session catalog does not exist: {0}")]
    UnknownCatalog(String),
}

/// A planning session. Build one with [`Session::builder`].
///
/// EXPERIMENTAL: may change until the typed-plan RFC settles.
///
/// Its `Debug` output omits the identity and the property values, which may be sensitive;
/// property keys are shown, so they should not hold secrets.
#[derive(Clone)]
pub struct Session {
    identity: String,
    catalog: String,
    catalogs: Catalogs,
    namespace: Namespace,
    system_namespace: Namespace,
    path: Option<Path>,
    properties: BTreeMap<String, String>,
}

impl Session {
    #[must_use]
    pub fn builder() -> SessionBuilder {
        SessionBuilder::default()
    }

    /// A session whose current catalog is the system catalog, and which has no other.
    #[must_use]
    pub fn empty() -> Session {
        Session::builder()
            .catalog(super::SYSTEM_CATALOG_NAME)
            .build()
            .expect("the system catalog is the current catalog")
    }

    #[must_use]
    pub fn identity(&self) -> &str {
        &self.identity
    }

    /// The name of the current catalog.
    #[must_use]
    pub fn catalog(&self) -> &str {
        &self.catalog
    }

    /// The catalogs, which include the system catalog.
    #[must_use]
    pub fn catalogs(&self) -> &Catalogs {
        &self.catalogs
    }

    /// The current namespace in the current catalog.
    #[must_use]
    pub fn namespace(&self) -> &Namespace {
        &self.namespace
    }

    /// The routine path. If none was set, it is the current catalog and namespace followed by
    /// the system catalog; a path that was set has the system catalog appended unless it already
    /// contains it.
    #[must_use]
    pub fn path(&self) -> Path {
        match &self.path {
            Some(path) => path.clone(),
            None => {
                let current = Namespace::of([self.catalog.as_str()])
                    .append(self.namespace.levels().iter().cloned());
                Path::of([current, self.system_namespace.clone()])
            }
        }
    }

    #[must_use]
    pub fn properties(&self) -> &BTreeMap<String, String> {
        &self.properties
    }
}

/// The builder of a [`Session`]. The current catalog must be set; the system catalog defaults to
/// [`SystemCatalog`]. Its `Debug` output omits the identity and the property values.
///
/// [`SessionBuilder::build`] checks that the current catalog is set and is one of the catalogs
/// (the system catalog counts), and that all the catalogs' names, the system catalog's included,
/// are unique (compared exactly, so `archive` and `ARCHIVE` may coexist).
#[derive(Clone)]
pub struct SessionBuilder {
    identity: String,
    catalog: Option<String>,
    system: Arc<dyn TypedCatalog>,
    catalogs: Vec<Arc<dyn TypedCatalog>>,
    namespace: Namespace,
    path: Option<Path>,
    properties: BTreeMap<String, String>,
}

impl Default for SessionBuilder {
    fn default() -> Self {
        SessionBuilder {
            identity: "unknown".to_string(),
            catalog: None,
            system: Arc::new(SystemCatalog::default()),
            catalogs: vec![],
            namespace: Namespace::empty(),
            path: None,
            properties: BTreeMap::new(),
        }
    }
}

impl SessionBuilder {
    #[must_use]
    pub fn identity(mut self, identity: impl Into<String>) -> Self {
        self.identity = identity.into();
        self
    }

    /// The name of the current catalog: an added catalog's name, or the system catalog's.
    #[must_use]
    pub fn catalog(mut self, catalog: impl Into<String>) -> Self {
        self.catalog = Some(catalog.into());
        self
    }

    /// The current namespace.
    #[must_use]
    pub fn namespace(mut self, namespace: Namespace) -> Self {
        self.namespace = namespace;
        self
    }

    /// The routine path; see [`Session::path`].
    #[must_use]
    pub fn path(mut self, path: Path) -> Self {
        self.path = Some(path);
        self
    }

    #[must_use]
    pub fn property(mut self, name: impl Into<String>, value: impl Into<String>) -> Self {
        self.properties.insert(name.into(), value.into());
        self
    }

    /// The system catalog, in place of [`SystemCatalog`].
    #[must_use]
    pub fn system(mut self, catalog: Arc<dyn TypedCatalog>) -> Self {
        self.system = catalog;
        self
    }

    /// Adds catalogs, whose names must be unique and differ from the system catalog's.
    #[must_use]
    pub fn catalogs(mut self, catalogs: impl IntoIterator<Item = Arc<dyn TypedCatalog>>) -> Self {
        self.catalogs.extend(catalogs);
        self
    }

    pub fn build(self) -> Result<Session, SessionError> {
        let mut catalogs = Catalogs::default();
        let mut names: Vec<&str> = vec![];
        for catalog in &self.catalogs {
            if names.contains(&catalog.name()) {
                return Err(SessionError::DuplicateCatalogName(
                    catalog.name().to_string(),
                ));
            }
            names.push(catalog.name());
            catalogs.add(catalog.clone());
        }
        let catalog = self.catalog.ok_or(SessionError::MissingCatalog)?;
        if names.contains(&self.system.name()) {
            return Err(SessionError::DuplicateCatalogName(
                self.system.name().to_string(),
            ));
        }
        if catalog != self.system.name() && !names.contains(&catalog.as_str()) {
            return Err(SessionError::UnknownCatalog(catalog));
        }
        let system_namespace = Namespace::of([self.system.name()]);
        catalogs.add(self.system);
        let path = self.path.map(|path| {
            if path.contains(&system_namespace) {
                path
            } else {
                Path::of(
                    path.namespaces()
                        .iter()
                        .cloned()
                        .chain([system_namespace.clone()]),
                )
            }
        });
        Ok(Session {
            identity: self.identity,
            catalog,
            catalogs,
            namespace: self.namespace,
            system_namespace,
            path,
            properties: self.properties,
        })
    }
}

// The catalogs by name, the property keys only, and no identity.
impl fmt::Debug for Session {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("Session")
            .field("catalog", &self.catalog)
            .field("catalogs", &self.catalogs)
            .field("namespace", &self.namespace)
            .field("path", &self.path)
            .field("property_keys", &self.properties.keys().collect::<Vec<_>>())
            .finish_non_exhaustive()
    }
}

impl fmt::Debug for SessionBuilder {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("SessionBuilder")
            .field("catalog", &self.catalog)
            .field(
                "catalogs",
                &self.catalogs.iter().map(|c| c.name()).collect::<Vec<_>>(),
            )
            .field("system", &self.system.name())
            .field("namespace", &self.namespace)
            .field("path", &self.path)
            .field("property_keys", &self.properties.keys().collect::<Vec<_>>())
            .finish_non_exhaustive()
    }
}
