//! The set of table functions a pqlite session can call.
//!
//! A table function needs three registrations: a lowering stub so the logical
//! planner accepts the call, compile-time metadata so the VM compiler can build
//! a `ScanLayout`, and a runtime factory the VM invokes by name. A
//! `TableFnDef` bundles all three, and a `TableFnRegistry` is the one list the
//! session iterates for each of them.

use std::ops::RangeInclusive;
use std::sync::Arc;

use partiql_catalog::call_defs::CallSpecArg;
use partiql_catalog::catalog::{MutableCatalog, PartiqlCatalog, SharedCatalog};
use partiql_catalog::table_fn::TableFunction as CatalogTableFunction;
use partiql_vm::source::{DataSourceMetadata, TableFunction, TableFunctionHandle};
use partiql_vm::ExecutionContext;

use crate::common;

/// One table function: lowering arity, compile-time metadata, runtime factory.
pub struct TableFnDef {
    /// Must be `'static` because `CallDef` names are. Dynamically loaded
    /// functions leak their name; they live for the whole process anyway.
    pub name: &'static str,
    /// Accepted positional-argument counts. One planner overload per count.
    pub arities: RangeInclusive<usize>,
    pub metadata: Arc<dyn DataSourceMetadata>,
    pub factory: Arc<dyn TableFunction>,
    /// One-line usage text for `.help`, e.g. `mem(rows, cols) — sequential integer data`.
    pub usage: String,
}

/// An ordered set of table functions with unique (case-insensitive) names.
#[derive(Clone, Default)]
pub struct TableFnRegistry {
    fns: Vec<Arc<TableFnDef>>,
}

impl TableFnRegistry {
    /// A registry with no functions.
    pub fn empty() -> Self {
        TableFnRegistry::default()
    }

    /// pqlite's built-in functions: `mem`, `rand` and `scan_ion`.
    pub fn builtin() -> Self {
        let mut reg = TableFnRegistry::empty();
        let int_cols = || -> Arc<dyn DataSourceMetadata> {
            Arc::new(common::ColumnarIntMetadata::new(vec![
                "a".to_string(),
                "b".to_string(),
            ]))
        };
        let defs = [
            TableFnDef {
                name: "rand",
                arities: 2..=2,
                metadata: int_cols(),
                factory: Arc::new(common::RandTableFunction),
                usage: "rand(rows, cols)      — random integer data".to_string(),
            },
            TableFnDef {
                name: "mem",
                arities: 2..=2,
                metadata: int_cols(),
                factory: Arc::new(common::MemTableFunction),
                usage: "mem(rows, cols)       — sequential integer data".to_string(),
            },
            TableFnDef {
                name: "scan_ion",
                arities: 1..=1,
                metadata: Arc::new(common::DynamicSchemaMetadata),
                factory: Arc::new(common::ScanIonTableFunction),
                usage: "scan_ion(path)        — read Ion file".to_string(),
            },
        ];
        for def in defs {
            reg.add(def)
                .expect("built-in table function names are unique");
        }
        reg
    }

    /// Add a function. Fails if a function with the same name (ignoring ASCII
    /// case) is already registered.
    pub fn add(&mut self, def: TableFnDef) -> Result<(), String> {
        if self.get(def.name).is_some() {
            return Err(format!(
                "table function '{}' is already registered",
                def.name
            ));
        }
        self.fns.push(Arc::new(def));
        Ok(())
    }

    /// Look a function up by name, ignoring ASCII case.
    pub fn get(&self, name: &str) -> Option<&TableFnDef> {
        self.fns
            .iter()
            .map(|f| f.as_ref())
            .find(|f| f.name.eq_ignore_ascii_case(name))
    }

    pub fn iter(&self) -> impl Iterator<Item = &TableFnDef> {
        self.fns.iter().map(|f| f.as_ref())
    }

    /// Frontend catalog holding one lowering stub per function.
    pub(crate) fn frontend_catalog(&self) -> Box<dyn SharedCatalog> {
        let mut catalog = PartiqlCatalog::default();
        for f in self.iter() {
            let overloads = f
                .arities
                .clone()
                .map(|n| vec![CallSpecArg::Positional; n])
                .collect();
            catalog
                .add_table_function(CatalogTableFunction::new(Box::new(
                    common::StubTableFn::new(f.name, overloads),
                )))
                .unwrap_or_else(|e| panic!("failed to add table function {}: {e:?}", f.name));
        }
        Box::new(catalog.to_shared_catalog())
    }

    /// Compile-time metadata lookup for `CompilationCatalog::get_table_function`.
    pub(crate) fn table_function_handle(&self, name: &str) -> Option<TableFunctionHandle> {
        self.get(name)
            .map(|f| TableFunctionHandle::new(Arc::clone(&f.metadata)))
    }

    /// Register every runtime factory with `ctx`.
    pub(crate) fn register_all(&self, ctx: &mut ExecutionContext) {
        for f in self.iter() {
            ctx.register_table_function(f.name, Arc::clone(&f.factory));
        }
    }
}
