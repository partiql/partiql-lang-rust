//! Resolving a table identifier against a session's catalogs.

use super::catalog::{CatalogNameAmbiguous, Table, TypedCatalog};
use super::identifier::{IdentPart, Identifier, Name};
use super::session::Session;
use std::sync::Arc;

/// The errors of [`resolve_table`].
#[derive(thiserror::Error, Debug, Clone, PartialEq, Eq)]
#[non_exhaustive]
pub enum ResolveError {
    /// The session's current catalog is not one of its catalogs.
    #[error("Default catalog does not exist: {0}")]
    MissingDefaultCatalog(String),
    #[error(transparent)]
    CatalogNameAmbiguous(#[from] CatalogNameAmbiguous),
}

/// A resolved table: the catalog it is in, the table, and the identifier's parts after the
/// table's name, which are a path into the table.
#[derive(Debug, Clone)]
pub struct ResolvedTable {
    catalog: String,
    table: Table,
    path: Vec<IdentPart>,
}

impl ResolvedTable {
    /// The name of the catalog the table is in.
    #[must_use]
    pub fn catalog(&self) -> &str {
        &self.catalog
    }

    #[must_use]
    pub fn table(&self) -> &Table {
        &self.table
    }

    /// The parts of the identifier after the table's name.
    #[must_use]
    pub fn path(&self) -> &[IdentPart] {
        &self.path
    }
}

/// The table that `identifier` names in `session`, or `None`. The lookups are, in order:
///
/// 1. in the current catalog, with the session namespace prepended to `identifier` (as
///    delimited parts), or, if the current catalog has no namespaces
///    ([`TypedCatalog::has_namespaces`]), as written, so the namespace is never matched against
///    a table name;
/// 2. if `identifier` is qualified, in the current catalog as written (schema-qualified);
/// 3. if `identifier` is qualified, in the catalog its first part names, matched as that part
///    is written (regular: ignoring case, an error if that matches several; delimited:
///    exactly), with the remaining parts (catalog-qualified).
///
/// The first lookup whose catalog resolves a name that it also has a table for wins, so a table
/// in the current catalog named like a catalog shadows that catalog's qualifier. The path is the parts of the looked-up identifier after as many parts
/// as the table's name has (none if the name has as many or more).
pub fn resolve_table(
    session: &Session,
    identifier: &Identifier,
) -> Result<Option<ResolvedTable>, ResolveError> {
    let default = session
        .catalogs()
        .get_catalog(session.catalog(), false)?
        .ok_or_else(|| ResolveError::MissingDefaultCatalog(session.catalog().to_string()))?;

    // 1. The current catalog and namespace; a catalog without namespaces gets the identifier
    // as written.
    let path = match session.namespace().as_identifier() {
        Some(namespace) if default.has_namespaces() => namespace.append(identifier),
        _ => identifier.clone(),
    };
    if let Some(found) = lookup(session, &default, &path) {
        return Ok(Some(found));
    }
    if !identifier.has_qualifier() {
        return Ok(None);
    }

    // 2. A schema-qualified identifier; without a prepended namespace, the same lookup as 1.
    if !session.namespace().is_empty() && default.has_namespaces() {
        if let Some(found) = lookup(session, &default, identifier) {
            return Ok(Some(found));
        }
    }

    // 3. A catalog-qualified identifier.
    let head = &identifier.qualifier()[0];
    let Some(catalog) = session
        .catalogs()
        .get_catalog(head.text(), head.is_regular())?
    else {
        return Ok(None);
    };
    let tail = Identifier::of(identifier.parts().skip(1).cloned())
        .expect("a qualified identifier has a part after its first");
    Ok(lookup(session, &catalog, &tail))
}

fn lookup(
    session: &Session,
    catalog: &Arc<dyn TypedCatalog>,
    path: &Identifier,
) -> Option<ResolvedTable> {
    let name = catalog.resolve_table(session, path)?;
    let table = catalog.get_table(session, &name)?;
    let path = unmatched(path, table.name());
    Some(ResolvedTable {
        catalog: catalog.name().to_string(),
        table,
        path,
    })
}

/// The parts of `path` after as many parts as `name` has (`Env.calculateMatched`).
fn unmatched(path: &Identifier, name: &Name) -> Vec<IdentPart> {
    path.parts().skip(name.len()).cloned().collect()
}
