use crate::call_defs::CallDef;

use partiql_types::PartiqlType;
use partiql_value::Value;
use std::borrow::Cow;

use std::collections::HashMap;
use std::error::Error;
use std::fmt::Debug;
use std::sync::atomic::{AtomicU64, Ordering};
use thiserror::Error;
use unicase::UniCase;

pub mod call_defs;

pub trait Extension: Debug {
    fn name(&self) -> String;
    fn load(&self, catalog: &mut dyn Catalog) -> Result<(), Box<dyn Error>>;
}

#[derive(Debug, Ord, PartialOrd, Eq, PartialEq, Copy, Clone, Hash)]
struct CatalogId(u64);

impl From<u64> for CatalogId {
    fn from(value: u64) -> Self {
        CatalogId(value)
    }
}

#[derive(Debug, Ord, PartialOrd, Eq, PartialEq, Copy, Clone, Hash)]
struct EntryId(u64);

impl From<u64> for EntryId {
    fn from(value: u64) -> Self {
        EntryId(value)
    }
}

#[derive(Debug, Ord, PartialOrd, Eq, PartialEq, Copy, Clone, Hash)]
pub struct ObjectId {
    catalog_id: CatalogId,
    entry_id: EntryId,
}

pub type BaseTableExprResultError = Box<dyn Error>;
pub type BaseTableExprResultValueIter<'a> =
    Box<dyn 'a + Iterator<Item = Result<Value, BaseTableExprResultError>>>;
pub type BaseTableExprResult<'a> =
    Result<BaseTableExprResultValueIter<'a>, BaseTableExprResultError>;

pub trait BaseTableExpr: Debug {
    fn evaluate(&self, args: &[Cow<Value>]) -> BaseTableExprResult;
}

pub trait BaseTableFunctionInfo: Debug {
    fn call_def(&self) -> &CallDef;
    fn plan_eval(&self) -> Box<dyn BaseTableExpr>;
}

#[derive(Debug)]
pub struct TableFunction {
    info: Box<dyn BaseTableFunctionInfo>,
}

impl TableFunction {
    pub fn new(info: Box<dyn BaseTableFunctionInfo>) -> Self {
        TableFunction { info }
    }
}

/// Contains the errors that occur during Catalog related operations
#[derive(Error, Debug, Clone, PartialEq)]
#[error("Catalog error: encountered errors")]
pub struct CatalogError {
    pub errors: Vec<CatalogErrorKind>,
}

impl CatalogError {
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

pub trait Catalog: Debug {
    fn add_table_function(&mut self, info: TableFunction) -> Result<ObjectId, CatalogError>;

    fn add_type_entry(&mut self, entry: TypeEnvEntry) -> Result<ObjectId, CatalogError>;

    fn get_function(&self, name: &str) -> Option<FunctionEntry>;

    fn resolve_type(&self, name: &str) -> Option<TypeEntry>;
}

#[derive(Debug)]
pub struct TypeEnvEntry<'a> {
    name: UniCase<String>,
    aliases: Vec<&'a str>,
    ty: PartiqlType,
}

impl<'a> TypeEnvEntry<'a> {
    pub fn new(name: &str, aliases: &[&'a str], ty: PartiqlType) -> Self {
        TypeEnvEntry {
            name: UniCase::from(name.to_string()),
            aliases: aliases.to_vec(),
            ty,
        }
    }
}

#[derive(Debug)]
pub struct TypeEntry {
    id: ObjectId,
    ty: PartiqlType,
}

impl TypeEntry {
    pub fn id(&self) -> &ObjectId {
        &self.id
    }

    pub fn ty(&self) -> &PartiqlType {
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
    Scalar(),
    Aggregate(),
}

impl<'a> FunctionEntry<'a> {
    pub fn call_def(&'a self) -> &'a CallDef {
        match &self.function {
            FunctionEntryFunction::Table(tf) => tf.info.call_def(),
            FunctionEntryFunction::Scalar() => todo!(),
            FunctionEntryFunction::Aggregate() => todo!(),
        }
    }

    pub fn plan_eval(&'a self) -> Box<dyn BaseTableExpr> {
        match &self.function {
            FunctionEntryFunction::Table(tf) => tf.info.plan_eval(),
            FunctionEntryFunction::Scalar() => todo!(),
            FunctionEntryFunction::Aggregate() => todo!(),
        }
    }
}

#[derive(Debug)]
pub struct PartiqlCatalog {
    functions: CatalogEntrySet<FunctionEntryFunction>,
    types: CatalogEntrySet<PartiqlType>,
    id: CatalogId,
}

impl Default for PartiqlCatalog {
    fn default() -> Self {
        PartiqlCatalog {
            functions: Default::default(),
            types: Default::default(),
            id: CatalogId(1),
        }
    }
}

impl PartiqlCatalog {}

impl Catalog for PartiqlCatalog {
    fn add_table_function(&mut self, info: TableFunction) -> Result<ObjectId, CatalogError> {
        let call_def = info.info.call_def();
        let names = call_def.names.clone();
        if let Some((name, aliases)) = names.split_first() {
            let id = self
                .functions
                .add(name, aliases, FunctionEntryFunction::Table(info))?;
            Ok(ObjectId {
                catalog_id: self.id,
                entry_id: id,
            })
        } else {
            Err(CatalogError::new(vec![CatalogErrorKind::EntryError(
                "Function definition has no name".into(),
            )]))
        }
    }

    fn add_type_entry(&mut self, entry: TypeEnvEntry) -> Result<ObjectId, CatalogError> {
        let id = self
            .types
            .add(entry.name.as_ref(), entry.aliases.as_slice(), entry.ty);

        match id {
            Ok(id) => Ok(ObjectId {
                catalog_id: self.id,
                entry_id: id,
            }),
            Err(e) => Err(e),
        }
    }

    fn get_function(&self, name: &str) -> Option<FunctionEntry> {
        self.functions
            .find_by_name(name)
            .map(|(eid, entry)| FunctionEntry {
                id: ObjectId {
                    catalog_id: self.id,
                    entry_id: eid,
                },
                function: entry,
            })
    }

    fn resolve_type(&self, name: &str) -> Option<TypeEntry> {
        self.types.find_by_name(name).map(|(eid, entry)| TypeEntry {
            id: ObjectId {
                catalog_id: self.id,
                entry_id: eid,
            },
            ty: entry.clone(),
        })
    }
}

#[derive(Debug)]
struct CatalogEntrySet<T> {
    entries: HashMap<EntryId, T>,
    by_name: HashMap<UniCase<String>, EntryId>,
    by_alias: HashMap<UniCase<String>, EntryId>,

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
            .map(|a| UniCase::from(a.to_string()))
            .collect();

        aliases.iter().for_each(|a| {
            if self.by_alias.contains_key(a) {
                errors.push(CatalogErrorKind::EntryExists(a.as_ref().to_string()))
            }
        });

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

                for a in aliases.into_iter() {
                    self.by_alias.insert(a, id);
                }

                Ok(id)
            }
            _ => Err(CatalogError::new(errors)),
        }
    }

    fn find_by_name(&self, name: &str) -> Option<(EntryId, &T)> {
        let name = UniCase::from(name);

        let eid = self.by_name.get(&name).or(self.by_alias.get(&name));

        if let Some(eid) = eid {
            self.entries.get(eid).map(|e| (*eid, e))
        } else {
            None
        }
    }
}

#[cfg(test)]
mod tests {
    #[test]
    fn todo() {}
}
