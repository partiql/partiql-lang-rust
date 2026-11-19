//! A [`TypedCatalog`] over an existing [`SharedCatalog`], so that catalogs built for the logical
//! planner and evaluator can be used for typed planning unchanged.

use super::catalog::{Table, TypedCatalog};
use super::identifier::{Identifier, Name};
use super::session::Session;
use crate::catalog::SharedCatalog;
use partiql_value::BindingsName;
use std::fmt;
use std::sync::Arc;

/// A [`TypedCatalog`] over a [`SharedCatalog`]'s type entries.
///
/// - The catalog's name is the inner catalog's [`crate::catalog::ReadOnlyCatalog::name`].
/// - A [`SharedCatalog`] has no namespaces, so, like partiql-lang-kotlin's standard catalog,
///   [`TypedCatalog::resolve_table`] matches only the identifier's first part, and the rest is a
///   path into the table. The part is looked up with
///   [`crate::catalog::ReadOnlyCatalog::resolve_binding_type`]: a regular part as
///   [`BindingsName::CaseInsensitive`] and a delimited one as [`BindingsName::CaseSensitive`],
///   so it matches by the inner catalog's case rules, not by [`super::equals_ignore_case`]
///   ([`crate::catalog::PartiqlCatalog`] ignores case with full Unicode case folding, for both).
///   [`TypedCatalog::has_namespaces`] is `false`, so [`super::resolve_table`] does not prepend
///   a session namespace to identifiers looked up in it: under any namespace, `orders` and
///   `default.orders` both find the table `orders` (see there).
/// - A [`SharedCatalog`] does not report the name an entry was registered under, so the resolved
///   [`Name`] is the part's text as written, and [`TypedCatalog::get_table`] looks a name up as
///   written ([`BindingsName::CaseSensitive`]) and, failing that, ignoring case
///   ([`BindingsName::CaseInsensitive`]); a name with a namespace has no table. So that the
///   resolved name always gets back the entry it was resolved to, a regular part that matches
///   one entry ignoring case while its exact text names a different entry resolves to nothing.
/// - The inner catalog's functions are untyped, so this catalog has no routines.
pub struct LegacyCatalogAdapter<C: ?Sized = dyn SharedCatalog> {
    inner: Arc<C>,
}

impl<C: ?Sized> LegacyCatalogAdapter<C> {
    pub fn new(inner: Arc<C>) -> Self {
        LegacyCatalogAdapter { inner }
    }

    /// The inner catalog.
    #[must_use]
    pub fn inner(&self) -> &Arc<C> {
        &self.inner
    }
}

impl<C: ?Sized> Clone for LegacyCatalogAdapter<C> {
    fn clone(&self) -> Self {
        LegacyCatalogAdapter {
            inner: self.inner.clone(),
        }
    }
}

impl<C: SharedCatalog + ?Sized> fmt::Debug for LegacyCatalogAdapter<C> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        // The inner catalog by name only, not its entries.
        f.debug_struct("LegacyCatalogAdapter")
            .field("name", &self.inner.name())
            .finish_non_exhaustive()
    }
}

impl<C: SharedCatalog + ?Sized> TypedCatalog for LegacyCatalogAdapter<C> {
    fn name(&self) -> &str {
        self.inner.name()
    }

    fn get_table(&self, _session: &Session, name: &Name) -> Option<Table> {
        if name.has_namespace() {
            return None;
        }
        let text = name.name();
        self.inner
            .resolve_binding_type(&BindingsName::CaseSensitive(text.into()))
            .or_else(|| {
                self.inner
                    .resolve_binding_type(&BindingsName::CaseInsensitive(text.into()))
            })
            .map(|entry| Table::new(name.clone(), entry.ty().clone()))
    }

    fn has_namespaces(&self) -> bool {
        false
    }

    fn resolve_table(&self, _session: &Session, identifier: &Identifier) -> Option<Name> {
        let first = identifier.parts().next()?;
        let text = first.text();
        let exact = self
            .inner
            .resolve_binding_type(&BindingsName::CaseSensitive(text.into()));
        if first.is_regular() {
            let entry = self
                .inner
                .resolve_binding_type(&BindingsName::CaseInsensitive(text.into()))?;
            if exact.is_some_and(|exact| exact.id() != entry.id()) {
                // `get_table` of the text as written would find the exact entry instead.
                return None;
            }
        } else {
            exact?;
        }
        Name::of([text])
    }
}
