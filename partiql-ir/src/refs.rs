//! References from a plan to catalog objects (tables and routines).
//!
//! The plan refers to catalog objects by name and, for routines, by the resolved signature, so
//! that this crate does not depend on a catalog API. partiql-lang-kotlin embeds the catalog's
//! objects (`Table`, `Fn`, `Agg`) in its plan and adds a `RoutineRef` for their identity
//! (`partiql-plan/src/main/java/org/partiql/plan/RoutineRef.java:30`).

use partiql_types::PartiqlShape;
use std::fmt::{Display, Formatter};

/// The name of a catalog object: the catalog it was resolved in, then the namespace path and
/// the object's own name, all as stored by the catalog (exact, not case-folded).
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct QualifiedName {
    catalog: String,
    namespace: Vec<String>,
    name: String,
}

impl QualifiedName {
    pub fn new<I, S>(catalog: impl Into<String>, namespace: I, name: impl Into<String>) -> Self
    where
        I: IntoIterator<Item = S>,
        S: Into<String>,
    {
        QualifiedName {
            catalog: catalog.into(),
            namespace: namespace.into_iter().map(Into::into).collect(),
            name: name.into(),
        }
    }

    /// The catalog the object was resolved in.
    pub fn catalog(&self) -> &str {
        &self.catalog
    }

    /// The namespace path inside the catalog, outermost first.
    pub fn namespace(&self) -> &[String] {
        &self.namespace
    }

    /// The object's own name.
    pub fn name(&self) -> &str {
        &self.name
    }
}

impl Display for QualifiedName {
    /// `"catalog"."ns1"."ns2"."name"`: every part double-quoted, a `"` in a part doubled, so
    /// distinct names print distinctly (e.g. namespace `["s.x"]` vs `["s", "x"]`).
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", quote(&self.catalog))?;
        for part in &self.namespace {
            write!(f, ".{}", quote(part))?;
        }
        write!(f, ".{}", quote(&self.name))
    }
}

/// `part` double-quoted, a `"` in it doubled.
pub(crate) fn quote(part: &str) -> String {
    format!("\"{}\"", part.replace('"', "\"\""))
}

/// A resolved routine (scalar function or aggregation) together with the signature it was
/// resolved to.
///
/// partiql-lang-kotlin's `RoutineRef` is `(catalog, name)`; the parameter and return types come
/// from the embedded `Fn`/`Agg`'s signature. Here both are carried as data.
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct RoutineRef {
    name: QualifiedName,
    specific: String,
    params: Vec<PartiqlShape>,
    returns: PartiqlShape,
}

impl RoutineRef {
    /// `specific` is the catalog's unique name for the chosen overload.
    pub fn new(
        name: QualifiedName,
        specific: impl Into<String>,
        params: Vec<PartiqlShape>,
        returns: PartiqlShape,
    ) -> Self {
        RoutineRef {
            name,
            specific: specific.into(),
            params,
            returns,
        }
    }

    pub fn name(&self) -> &QualifiedName {
        &self.name
    }

    /// The catalog's unique name for this overload.
    pub fn specific(&self) -> &str {
        &self.specific
    }

    /// The overload's parameter types.
    pub fn params(&self) -> &[PartiqlShape] {
        &self.params
    }

    /// The overload's return type.
    pub fn returns(&self) -> &PartiqlShape {
        &self.returns
    }

    pub fn into_parts(self) -> (QualifiedName, String, Vec<PartiqlShape>, PartiqlShape) {
        (self.name, self.specific, self.params, self.returns)
    }
}
