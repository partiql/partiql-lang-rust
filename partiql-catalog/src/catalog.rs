use crate::call_defs::ScalarFnCallSpecs;
use crate::scalar_fn::ScalarFunction;
use crate::table_fn::TableFunction;
use delegate::delegate;
use partiql_common::catalog::{CatalogId, EntryId, ObjectId};
use partiql_types::PartiqlShape;
use rustc_hash::FxHashMap;
use std::fmt::Debug;
use std::sync::atomic::{AtomicU64, Ordering};
use thiserror::Error;
use unicase::UniCase;
/// Contains the errors that occur during Catalog related operations
#[derive(Error, Debug, Clone, PartialEq)]
#[error("Catalog error: encountered errors")]
pub struct CatalogError {
    pub errors: Vec<CatalogErrorKind>,
}

impl CatalogError {
    #[must_use]
    pub fn new(errors: Vec<CatalogErrorKind>) -> Self {
        CatalogError { errors }
    }
}

/// Catalog Error kind
///
/// ### Notes
/// This is marked `#[non_exhaustive]`, to reserve the right to add more variants in the future.
#[derive(Error, Debug, Clone, PartialEq)]
#[non_exhaustive]
pub enum CatalogErrorKind {
    /// Entry exists error.
    #[error("Catalog error: entry already exists for `{0}`")]
    EntryExists(String),

    /// Entry error.
    #[error("Catalog error: `{0}`")]
    EntryError(String),

    /// Any other catalog error.
    #[error("Catalog error: unknown error")]
    Unknown,
}

pub trait MutableCatalog: Debug {
    fn add_table_function(&mut self, info: TableFunction) -> Result<ObjectId, CatalogError>;
    fn add_scalar_function(&mut self, info: ScalarFunction) -> Result<ObjectId, CatalogError>;
    fn add_type_entry(&mut self, entry: TypeEnvEntry<'_>) -> Result<ObjectId, CatalogError>;
}

pub trait ReadOnlyCatalog: Debug {
    fn name(&self) -> &str;
    fn get_function(&self, name: &str) -> Option<FunctionEntry<'_>>;
    fn get_function_by_id(&self, id: ObjectId) -> Option<FunctionEntry<'_>>;
    fn resolve_type(&self, name: &str) -> Option<TypeEntry>;
}

pub trait SharedCatalog: ReadOnlyCatalog + Send + Sync {}

pub trait Catalog: MutableCatalog + ReadOnlyCatalog {}

#[derive(Debug)]
pub struct TypeEnvEntry<'a> {
    name: UniCase<String>,
    aliases: Vec<&'a str>,
    ty: PartiqlShape,
}

impl<'a> TypeEnvEntry<'a> {
    #[must_use]
    pub fn new(name: &str, aliases: &[&'a str], ty: PartiqlShape) -> Self {
        TypeEnvEntry {
            name: UniCase::from(name.to_string()),
            aliases: aliases.to_vec(),
            ty,
        }
    }
}

#[derive(Debug, Clone)]
pub struct TypeEntry {
    id: ObjectId,
    ty: PartiqlShape,
}

impl TypeEntry {
    #[must_use]
    pub fn id(&self) -> &ObjectId {
        &self.id
    }

    #[must_use]
    pub fn ty(&self) -> &PartiqlShape {
        &self.ty
    }
}

#[derive(Debug)]
#[allow(dead_code)]
pub struct FunctionEntry<'a> {
    id: ObjectId,
    function: &'a FunctionEntryFunction,
}

#[derive(Debug)]
pub enum FunctionEntryFunction {
    Table(TableFunction),
    Scalar(ScalarFnCallSpecs),
    Aggregate(),
}

impl<'a> FunctionEntry<'a> {
    pub fn id(&self) -> &ObjectId {
        &self.id
    }

    #[must_use]
    pub fn entry(&'a self) -> &'a FunctionEntryFunction {
        self.function
    }
}

#[derive(Debug)]
pub struct PartiqlCatalog {
    name: String,
    functions: CatalogEntrySet<FunctionEntryFunction>,
    types: CatalogEntrySet<PartiqlShape>,
    id: CatalogId,
}

#[derive(Debug)]
pub struct PartiqlSharedCatalog(PartiqlCatalog);

impl Default for PartiqlCatalog {
    fn default() -> Self {
        PartiqlCatalog {
            name: "default".to_string(),
            functions: Default::default(),
            types: Default::default(),
            id: 1.into(),
        }
    }
}

impl PartiqlCatalog {
    pub fn to_shared_catalog(self) -> PartiqlSharedCatalog {
        PartiqlSharedCatalog(self)
    }
}

impl Catalog for PartiqlCatalog {}

impl MutableCatalog for PartiqlCatalog {
    fn add_table_function(&mut self, info: TableFunction) -> Result<ObjectId, CatalogError> {
        let call_def = info.call_def();
        let names = call_def.names.clone();
        if let Some((name, aliases)) = names.split_first() {
            let eid = self
                .functions
                .add(name, aliases, FunctionEntryFunction::Table(info))?;
            Ok((self.id, eid).into())
        } else {
            Err(CatalogError::new(vec![CatalogErrorKind::EntryError(
                "Function definition has no name".into(),
            )]))
        }
    }

    fn add_scalar_function(&mut self, info: ScalarFunction) -> Result<ObjectId, CatalogError> {
        let id = self.id;
        let call_def = info.into_call_def();
        let names = call_def.names;
        if let Some((name, aliases)) = names.split_first() {
            self.functions
                .add(
                    name,
                    aliases,
                    FunctionEntryFunction::Scalar(call_def.overloads),
                )
                .map(|eid| ObjectId::new(id, eid))
        } else {
            Err(CatalogError::new(vec![CatalogErrorKind::EntryError(
                "Function definition has no name".into(),
            )]))
        }
    }

    fn add_type_entry(&mut self, entry: TypeEnvEntry<'_>) -> Result<ObjectId, CatalogError> {
        let eid = self
            .types
            .add(entry.name.as_ref(), entry.aliases.as_slice(), entry.ty);

        match eid {
            Ok(eid) => Ok((self.id, eid).into()),
            Err(e) => Err(e),
        }
    }
}

impl ReadOnlyCatalog for PartiqlCatalog {
    fn name(&self) -> &str {
        &self.name
    }

    fn get_function(&self, name: &str) -> Option<FunctionEntry<'_>> {
        self.functions
            .find_by_name(name)
            .map(|(e, f)| self.to_function_entry(e, f))
    }

    fn get_function_by_id(&self, id: ObjectId) -> Option<FunctionEntry<'_>> {
        assert_eq!(self.id, id.catalog_id());
        self.functions
            .find_by_id(&id.entry_id())
            .map(|(e, f)| self.to_function_entry(e, f))
    }

    fn resolve_type(&self, name: &str) -> Option<TypeEntry> {
        self.types.find_by_name(name).map(|(eid, entry)| TypeEntry {
            id: (self.id, eid).into(),
            ty: entry.clone(),
        })
    }
}

impl ReadOnlyCatalog for PartiqlSharedCatalog {
    delegate! {
        to self.0 {
            fn name(&self) -> &str;
            fn get_function(&self, name: &str) -> Option<FunctionEntry<'_>>;
            fn get_function_by_id(&self, id: ObjectId) -> Option<FunctionEntry<'_>>;
            fn resolve_type(&self, name: &str) -> Option<TypeEntry>;
        }
    }
}

impl SharedCatalog for PartiqlSharedCatalog {}

impl PartiqlCatalog {
    fn to_function_entry<'a>(
        &'a self,
        eid: EntryId,
        entry: &'a FunctionEntryFunction,
    ) -> FunctionEntry<'a> {
        FunctionEntry {
            id: (self.id, eid).into(),
            function: entry,
        }
    }
}

#[derive(Debug)]
struct CatalogEntrySet<T> {
    entries: FxHashMap<EntryId, T>,
    by_name: FxHashMap<UniCase<String>, EntryId>,
    by_alias: FxHashMap<UniCase<String>, EntryId>,

    next_id: AtomicU64,
}

impl<T> Default for CatalogEntrySet<T> {
    fn default() -> Self {
        CatalogEntrySet {
            entries: Default::default(),
            by_name: Default::default(),
            by_alias: Default::default(),
            next_id: 1.into(),
        }
    }
}

impl<T> CatalogEntrySet<T> {
    fn add(&mut self, name: &str, aliases: &[&str], info: T) -> Result<EntryId, CatalogError> {
        let mut errors = vec![];
        let name = UniCase::from(name);
        let aliases: Vec<UniCase<String>> = aliases
            .iter()
            .map(|a| UniCase::from((*a).to_string()))
            .collect();

        for a in &aliases {
            if self.by_alias.contains_key(a) {
                errors.push(CatalogErrorKind::EntryExists(a.as_ref().to_string()));
            }
        }

        if self.by_name.contains_key(&name) {
            errors.push(CatalogErrorKind::EntryExists(name.to_string()));
        }

        let id = self.next_id.fetch_add(1, Ordering::SeqCst).into();

        if let Some(_old_val) = self.entries.insert(id, info) {
            errors.push(CatalogErrorKind::Unknown);
        }

        match errors.is_empty() {
            true => {
                self.by_name.insert(name, id);

                for a in aliases {
                    self.by_alias.insert(a, id);
                }

                Ok(id)
            }
            _ => Err(CatalogError::new(errors)),
        }
    }

    fn find_by_id(&self, eid: &EntryId) -> Option<(EntryId, &T)> {
        self.entries.get(eid).map(|e| (*eid, e))
    }

    fn find_by_name(&self, name: &str) -> Option<(EntryId, &T)> {
        let name = UniCase::from(name);
        let eid = self.by_name.get(&name).or(self.by_alias.get(&name));

        eid.and_then(|eid| self.find_by_id(eid))
    }
}

#[cfg(test)]
mod tests {
    #[test]
    fn todo() {}
}
