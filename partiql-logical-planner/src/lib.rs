#![deny(rust_2018_idioms)]
#![deny(clippy::all)]

use crate::lower::AstToLogical;

use partiql_ast_passes::error::AstTransformationError;
use partiql_ast_passes::name_resolver::NameResolver;
use partiql_logical as logical;
use partiql_parser::Parsed;

use partiql_catalog::catalog::{Catalog, PartiqlCatalog};

mod builtins;
mod functions;
mod graph;
mod lower;
mod typer;

pub struct LogicalPlanner<'c> {
    catalog: &'c dyn Catalog,
}

impl<'c> LogicalPlanner<'c> {
    pub fn new(catalog: &'c dyn Catalog) -> Self {
        LogicalPlanner { catalog }
    }

    #[inline]
    pub fn lower(
        &self,
        parsed: &Parsed<'_>,
    ) -> Result<logical::LogicalPlan<logical::BindingsOp>, AstTransformationError> {
        let q = &parsed.ast;
        let catalog = PartiqlCatalog::default();
        let mut resolver = NameResolver::new(&catalog);
        let registry = resolver.resolve(q)?;
        let planner = AstToLogical::new(self.catalog, registry);
        planner.lower_query(q)
    }
}
