//! Typed catalogs: tables with [`PartiqlShape`] schemas, routine signatures, the set of catalogs
//! a session can see, and the system catalog.

use super::identifier::{Identifier, Name};
use super::routine::{AggOverload, FnOverload, RoutineBinding};
use super::session::Session;
use partiql_types::PartiqlShape;
use std::fmt::Debug;
use std::sync::Arc;

/// The name of the system catalog, which every [`Session`] has.
pub const SYSTEM_CATALOG_NAME: &str = "$system";

/// A table: its canonical name and its schema, normally a bag of rows.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Table {
    name: Name,
    schema: PartiqlShape,
}

impl Table {
    /// A table named `name` with the schema `schema`.
    #[must_use]
    pub fn new(name: Name, schema: PartiqlShape) -> Self {
        Table { name, schema }
    }

    /// A table whose schema is `DYNAMIC`: its schema is not known.
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

/// A catalog for typed planning: its tables and, by text, its routines.
///
/// EXPERIMENTAL: may change until the typed-plan RFC settles.
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
    /// The contract:
    /// - The longest prefix of `identifier` that names a table wins.
    /// - If two or more tables match a prefix of that length under the catalog's own case rules
    ///   (for example `Foo` and `FOO` for a regular `foo`, where regular parts ignore case), the
    ///   identifier is ambiguous and the result is `None`. [`super::LegacyCatalogAdapter`] cannot
    ///   detect this: its inner catalog answers with at most one entry.
    /// - The returned name has exactly as many parts as the prefix it matched.
    /// - [`TypedCatalog::get_table`] of the returned name returns the table it was resolved to,
    ///   whose [`Table::name`] is the returned name. [`super::resolve_table`] takes the parts of
    ///   `identifier` after as many parts as that name has as the path, so a catalog that resolves
    ///   bare `orders` to the two-part `public.orders` would make `orders.total` resolve with an
    ///   empty path, dropping `.total`.
    ///
    /// The case rules are the implementation's. [`super::LegacyCatalogAdapter`] applies its
    /// inner catalog's rules: over [`crate::catalog::PartiqlCatalog`], delimited parts also
    /// ignore case, by full Unicode case folding.
    fn resolve_table(&self, _session: &Session, _identifier: &Identifier) -> Option<Name> {
        None
    }

    /// Whether the catalog's names can have namespaces. As the current catalog, one without
    /// them (`false`) is searched by [`super::resolve_table`] with the identifier as written
    /// rather than with the session namespace prepended. The default is `true`: the session
    /// namespace is prepended. A catalog that wraps another must forward this.
    ///
    /// This is deliberate: a catalog whose names have no namespaces cannot hold `shop.orders`,
    /// so prepending a namespace `shop` to `orders` and matching only the first part would
    /// resolve the table `shop`, a different object, with the path `orders`.
    fn has_namespaces(&self) -> bool {
        true
    }

    /// The scalar function overloads named `name` (lookup by text, for catalogs that do not
    /// resolve routines by identifier).
    fn get_functions(&self, _session: &Session, _name: &str) -> Vec<Arc<dyn FnOverload>> {
        vec![]
    }

    /// The aggregation overloads named `name` (lookup by text, for catalogs that do not resolve
    /// routines by identifier).
    fn get_aggregations(&self, _session: &Session, _name: &str) -> Vec<Arc<dyn AggOverload>> {
        vec![]
    }

    /// This catalog as a [`RoutineCatalog`], if it is one. Routines are resolved by identifier in
    /// a `RoutineCatalog` and by text in any other catalog.
    fn as_routine_catalog(&self) -> Option<&dyn RoutineCatalog> {
        None
    }
}

/// A catalog that resolves routines by identifier.
///
/// EXPERIMENTAL: may change until the typed-plan RFC settles.
///
/// An implementor must also return `Some(self)` from [`TypedCatalog::as_routine_catalog`];
/// otherwise its routines are looked up only by text, through [`TypedCatalog::get_functions`]
/// and [`TypedCatalog::get_aggregations`].
///
/// The contract of [`RoutineCatalog::resolve_functions`] and
/// [`RoutineCatalog::resolve_aggregations`]:
/// - The identifier is local to the catalog: it never includes the catalog's own name.
/// - It matches a routine at its full depth; a prefix of the identifier does not match.
/// - A regular part matches ignoring case and a delimited part exactly, as
///   [`super::IdentPart::matches`] does.
/// - The result is authoritative: for a `RoutineCatalog` there is no fallback to the by-text
///   lookup, so an empty result means the catalog has no such routine.
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
/// catalogs: the name as written, and the matching catalogs' names, sorted by exact name
/// comparing UTF-16 code units, so the list has a stable, documented order.
#[derive(thiserror::Error, Debug, Clone, PartialEq, Eq)]
#[error(
    "Catalog name '{name}' is ambiguous ignoring case; it matches {}; quote the name to select one",
    quoted(.catalog_names)
)]
pub struct CatalogNameAmbiguous {
    name: String,
    catalog_names: Vec<String>,
}

fn quoted(names: &[String]) -> String {
    names
        .iter()
        .map(|n| format!("\"{n}\""))
        .collect::<Vec<_>>()
        .join(", ")
}

impl CatalogNameAmbiguous {
    /// The error for the name `name`, as written, matching the catalogs `catalog_names`.
    #[must_use]
    pub fn new(name: impl Into<String>, mut catalog_names: Vec<String>) -> Self {
        // By UTF-16 code unit, not by Unicode scalar value (see the type's docs).
        catalog_names.sort_by(|a, b| a.encode_utf16().cmp(b.encode_utf16()));
        CatalogNameAmbiguous {
            name: name.into(),
            catalog_names,
        }
    }

    /// The name as written.
    #[must_use]
    pub fn name(&self) -> &str {
        &self.name
    }

    #[must_use]
    pub fn catalog_names(&self) -> &[String] {
        &self.catalog_names
    }
}

/// The catalogs of a session, by name. Its `Debug` output lists the catalogs' names only.
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
                Err(CatalogNameAmbiguous::new(name, names))
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
