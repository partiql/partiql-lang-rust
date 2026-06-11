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
        if parsed.statements.len() != 1 {
            return Err(AstTransformationError {
                errors: vec![AstTransformError::NotYetImplemented(
                    "multi-statement input".to_string(),
                )],
            });
        }
        let stmt = &parsed.statements[0];
        match &stmt.node {
            ast::Statement::Query(q) => {
                let mut resolver = NameResolver::new(self.catalog);
                let registry = resolver.resolve(q, stmt.id)?;
                let planner = AstToLogical::new(self.catalog, registry);
                planner.lower_query(q, stmt.id)
            }
            ast::Statement::Ddl(_) => Err(AstTransformationError {
                errors: vec![AstTransformError::NotYetImplemented(
                    "DDL statement lowering".to_string(),
                )],
            }),
            ast::Statement::Dml(_) => Err(AstTransformationError {
                errors: vec![AstTransformError::NotYetImplemented(
                    "DML statement lowering".to_string(),
                )],
            }),
        }
    }
}
