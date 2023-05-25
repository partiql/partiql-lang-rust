use crate::call_defs::CallDef;
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

/// Catalog Errors.
///
/// ### Notes
/// This is marked `#[non_exhaustive]`, to reserve the right to add more variants in the future.
#[derive(Error, Debug, Clone, PartialEq)]
#[non_exhaustive]
pub enum CatalogError {
    /// Entry exists error.
    #[error("Catalog error: entry already exists for `{}`", .0)]
    EntryExists(String),

    /// Entry error.
    #[error("Catalog error: `{}`", .0)]
    EntryError(String),

    /// Any other catalog error.
    #[error("Catalog error: unknown error")]
    Unknown,
}

pub trait Catalog: Debug {
    fn add_table_function(&mut self, info: TableFunction) -> Result<ObjectId, CatalogError>;

    fn get_function(&self, name: &str) -> Option<FunctionEntry>;
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

    id: CatalogId,
}

impl Default for PartiqlCatalog {
    fn default() -> Self {
        PartiqlCatalog {
            functions: Default::default(),

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
            Err(CatalogError::EntryError(
                "Function definition has no name".into(),
            ))
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
}

#[derive(Debug)]
struct CatalogEntrySet<T> {
    entries: HashMap<EntryId, T>,
    by_name: HashMap<UniCase<String>, EntryId>,

    next_id: AtomicU64,
}

impl<T> Default for CatalogEntrySet<T> {
    fn default() -> Self {
        CatalogEntrySet {
            entries: Default::default(),
            by_name: Default::default(),
            next_id: 1.into(),
        }
    }
}

impl<T> CatalogEntrySet<T> {
    fn add(&mut self, name: &str, _aliases: &[&str], info: T) -> Result<EntryId, CatalogError> {
        let name = UniCase::from(name);
        if self.by_name.contains_key(&name) {
            return Err(CatalogError::EntryExists(name.to_string()));
        }

        let id = self.next_id.fetch_add(1, Ordering::SeqCst).into();
        if let Some(_old_val) = self.entries.insert(id, info) {
            return Err(CatalogError::Unknown);
        }

        self.by_name.insert(name, id);

        Ok(id)
    }

    fn find_by_name(&self, name: &str) -> Option<(EntryId, &T)> {
        let name = UniCase::from(name);
        if let Some(eid) = self.by_name.get(&name) {
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
