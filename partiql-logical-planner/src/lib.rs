#![deny(rust_2018_idioms)]
#![deny(clippy::all)]

use crate::lower::AstToLogical;

use partiql_ast::ast;
use partiql_ast_passes::error::{AstTransformError, AstTransformationError};
use partiql_ast_passes::name_resolver::NameResolver;
use partiql_logical as logical;
use partiql_parser::Parsed;

use partiql_catalog::catalog::SharedCatalog;

mod builtins;
mod functions;
mod graph;
mod lower;
mod typer;

pub struct LogicalPlanner<'c> {
    catalog: &'c dyn SharedCatalog,
}

impl<'c> LogicalPlanner<'c> {
    pub fn new(catalog: &'c dyn SharedCatalog) -> Self {
        LogicalPlanner { catalog }
    }

    #[inline]
    pub fn lower(
        &self,
        parsed: &Parsed<'_>,
    ) -> Result<logical::LogicalPlan<logical::BindingsOp>, AstTransformationError> {
        match &parsed.ast.node {
            // Data-retrieval queries flow through the existing lowering pipeline unchanged.
            ast::Item::Query(q) => {
                let mut resolver = NameResolver::new(self.catalog);
                let registry = resolver.resolve(q)?;
                let planner = AstToLogical::new(self.catalog, registry);
                planner.lower_query(q)
            }
            // DDL/DML lowering is not yet implemented; surface a clear error rather
            // than silently producing an empty plan.
            ast::Item::Ddl(_) => Err(AstTransformationError {
                errors: vec![AstTransformError::NotYetImplemented(
                    "DDL statement lowering".to_string(),
                )],
            }),
            ast::Item::Dml(_) => Err(AstTransformationError {
                errors: vec![AstTransformError::NotYetImplemented(
                    "DML statement lowering".to_string(),
                )],
            }),
        }
    }
}
