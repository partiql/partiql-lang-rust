//! Typed catalogs: tables with [`PartiqlShape`] schemas, and routine signatures.
//!
//! Ported from partiql-lang-kotlin v1.6.1 `org.partiql.spi.catalog.{Catalog, RoutineCatalog,
//! Table, Catalogs, System}`.

use super::identifier::{Identifier, Name};
use super::routine::{AggOverload, FnOverload, RoutineBinding};
use super::session::Session;
use partiql_types::PartiqlShape;
use std::fmt::Debug;
use std::sync::Arc;

/// The name of the system catalog, which every [`Session`] has.
pub const SYSTEM_CATALOG_NAME: &str = "$system";

/// A table: its canonical name and its schema, normally a bag of rows.
#[derive(Debug, Clone)]
pub struct Table {
    name: Name,
    schema: PartiqlShape,
}

impl Table {
    #[must_use]
    pub fn new(name: Name, schema: PartiqlShape) -> Self {
        Table { name, schema }
    }

    /// A table whose schema is `DYNAMIC`, partiql-lang-kotlin's default.
    #[must_use]
    pub fn dynamic(name: Name) -> Self {
        Table::new(name, PartiqlShape::Dynamic)
    }

    #[must_use]
    pub fn name(&self) -> &Name {
        &self.name
    }

    #[must_use]
    pub fn schema(&self) -> &PartiqlShape {
        &self.schema
    }
}

/// A catalog for typed planning (partiql-lang-kotlin `Catalog`).
///
/// The catalog decides how an [`Identifier`] matches its names, in [`TypedCatalog::resolve_table`];
/// the convention is that a regular part matches ignoring case and a delimited part exactly (see
/// [`super::IdentPart::matches`]).
pub trait TypedCatalog: Debug + Send + Sync {
    /// The catalog's name.
    fn name(&self) -> &str;

    /// The table with the canonical name `name`.
    fn get_table(&self, _session: &Session, _name: &Name) -> Option<Table> {
        None
    }

    /// The canonical name of the table that `identifier`, or a prefix of it, names.
    /// The parts of `identifier` after the matched name become a path into the table.
    ///
    /// The case rules are the implementation's. [`super::LegacyCatalogAdapter`] applies its
    /// inner catalog's rules: over [`crate::catalog::PartiqlCatalog`], delimited parts also
    /// ignore case, by full Unicode case folding.
    fn resolve_table(&self, _session: &Session, _identifier: &Identifier) -> Option<Name> {
        None
    }

    /// Whether the catalog's names can have namespaces. A catalog without them (`false`) is
    /// never searched with the session namespace prepended by [`super::resolve_table`]; the
    /// default is `true`, as every partiql-lang-kotlin catalog is treated.
    fn has_namespaces(&self) -> bool {
        true
    }

    /// The scalar function overloads named `name` (partiql-lang-kotlin's legacy lookup by text).
    fn get_functions(&self, _session: &Session, _name: &str) -> Vec<Arc<dyn FnOverload>> {
        vec![]
    }

    /// The aggregation overloads named `name` (partiql-lang-kotlin's legacy lookup by text).
    fn get_aggregations(&self, _session: &Session, _name: &str) -> Vec<Arc<dyn AggOverload>> {
        vec![]
    }

    /// This catalog as a [`RoutineCatalog`], if it is one; partiql-lang-kotlin resolves routines
    /// in a `RoutineCatalog` by identifier and in any other catalog by text.
    fn as_routine_catalog(&self) -> Option<&dyn RoutineCatalog> {
        None
    }
}

/// A catalog that resolves routines by identifier (partiql-lang-kotlin `RoutineCatalog`).
///
/// An implementor must also return `Some(self)` from [`TypedCatalog::as_routine_catalog`];
/// otherwise its routines are looked up only by text, through [`TypedCatalog::get_functions`]
/// and [`TypedCatalog::get_aggregations`].
pub trait RoutineCatalog: TypedCatalog {
    /// The scalar function bindings that `identifier` names.
    fn resolve_functions(
        &self,
        session: &Session,
        identifier: &Identifier,
    ) -> Vec<RoutineBinding<dyn FnOverload>>;

    /// The aggregation bindings that `identifier` names.
    fn resolve_aggregations(
        &self,
        session: &Session,
        identifier: &Identifier,
    ) -> Vec<RoutineBinding<dyn AggOverload>>;
}

/// The system catalog, named [`SYSTEM_CATALOG_NAME`]: it has no tables, and is where the
/// builtin routines are served from. It serves no routines yet. Build it with
/// [`SystemCatalog::default`].
#[derive(Debug, Clone, Default)]
#[non_exhaustive]
pub struct SystemCatalog;

impl TypedCatalog for SystemCatalog {
    fn name(&self) -> &str {
        SYSTEM_CATALOG_NAME
    }

    fn as_routine_catalog(&self) -> Option<&dyn RoutineCatalog> {
        Some(self)
    }
}

impl RoutineCatalog for SystemCatalog {
    fn resolve_functions(
        &self,
        _session: &Session,
        _identifier: &Identifier,
    ) -> Vec<RoutineBinding<dyn FnOverload>> {
        vec![]
    }

    fn resolve_aggregations(
        &self,
        _session: &Session,
        _identifier: &Identifier,
    ) -> Vec<RoutineBinding<dyn AggOverload>> {
        vec![]
    }
}

/// The error of [`Catalogs::get_catalog`] when a name that ignores case matches several
/// catalogs. The names are sorted by exact name.
#[derive(thiserror::Error, Debug, Clone, PartialEq, Eq)]
#[error("Catalog name is ambiguous; matched {}.", .catalog_names.join(", "))]
pub struct CatalogNameAmbiguous {
    catalog_names: Vec<String>,
}

impl CatalogNameAmbiguous {
    #[must_use]
    pub fn new(mut catalog_names: Vec<String>) -> Self {
        // Java's String order: by UTF-16 code unit.
        catalog_names.sort_by(|a, b| a.encode_utf16().cmp(b.encode_utf16()));
        CatalogNameAmbiguous { catalog_names }
    }

    #[must_use]
    pub fn catalog_names(&self) -> &[String] {
        &self.catalog_names
    }
}

/// The catalogs of a session, by name (partiql-lang-kotlin `Catalogs`). Its `Debug` output lists
/// the catalogs' names only.
#[derive(Clone, Default)]
pub struct Catalogs {
    catalogs: Vec<Arc<dyn TypedCatalog>>,
}

impl Catalogs {
    /// The catalogs; a later catalog replaces an earlier one with the same name, in its place.
    pub fn of(catalogs: impl IntoIterator<Item = Arc<dyn TypedCatalog>>) -> Self {
        let mut out = Catalogs::default();
        for catalog in catalogs {
            out.add(catalog);
        }
        out
    }

    pub(crate) fn add(&mut self, catalog: Arc<dyn TypedCatalog>) {
        match self
            .catalogs
            .iter_mut()
            .find(|c| c.name() == catalog.name())
        {
            Some(slot) => *slot = catalog,
            None => self.catalogs.push(catalog),
        }
    }

    /// The catalog named `name`: exactly, or, if `ignore_case`, ignoring case (see
    /// [`super::equals_ignore_case`]), in which case more than one match is an error.
    pub fn get_catalog(
        &self,
        name: &str,
        ignore_case: bool,
    ) -> Result<Option<Arc<dyn TypedCatalog>>, CatalogNameAmbiguous> {
        if !ignore_case {
            return Ok(self.catalogs.iter().find(|c| c.name() == name).cloned());
        }
        let mut matches = self
            .catalogs
            .iter()
            .filter(|c| super::equals_ignore_case(c.name(), name));
        match (matches.next(), matches.next()) {
            (None, _) => Ok(None),
            (Some(c), None) => Ok(Some(c.clone())),
            (Some(a), Some(b)) => {
                let names = [a, b]
                    .into_iter()
                    .chain(matches)
                    .map(|c| c.name().to_string())
                    .collect();
                Err(CatalogNameAmbiguous::new(names))
            }
        }
    }

    /// The catalogs, in the order they were first added.
    pub fn iter(&self) -> impl Iterator<Item = &Arc<dyn TypedCatalog>> {
        self.catalogs.iter()
    }
}

impl std::fmt::Debug for Catalogs {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_list()
            .entries(self.catalogs.iter().map(|c| c.name()))
            .finish()
    }
}
