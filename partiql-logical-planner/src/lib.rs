#![deny(rust_2018_idioms)]
#![deny(clippy::all)]

use crate::lower::AstToLogical;

use partiql_ast::ast;
use partiql_ast_passes::error::{AstTransformError, AstTransformationError};
use partiql_ast_passes::name_resolver::NameResolver;
use partiql_common::node::NodeId;
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

/// The uniform "DDL not yet lowerable" error, shared by every DDL-rejection site.
#[inline]
fn ddl_not_yet_implemented() -> AstTransformationError {
    AstTransformationError {
        errors: vec![AstTransformError::NotYetImplemented(
            "DDL statement lowering".to_string(),
        )],
    }
}

impl<'c> LogicalPlanner<'c> {
    pub fn new(catalog: &'c dyn SharedCatalog) -> Self {
        LogicalPlanner { catalog }
    }

    /// Lower a single parsed statement into a top-level [`logical::LogicalStatement`].
    ///
    /// This is the full-fidelity entry point: it preserves the statement
    /// category (query vs. DDL). It takes a single `AstNode<Statement>` rather
    /// than a bare `Statement` because query-bearing statements (a top-level
    /// query, or a CTAS source) are lowered through the two-pass pipeline, which
    /// keys name resolution and lowering on the node's parse-time `NodeId`.
    /// Iterating a multi-statement parse is the caller's concern. [`Self::lower`]
    /// is a back-compat shim over this.
    #[inline]
    pub fn lower_statement(
        &self,
        stmt: &ast::AstNode<ast::Statement>,
    ) -> Result<logical::LogicalStatement, AstTransformationError> {
        match &stmt.node {
            ast::Statement::Query(q) => {
                let plan = self.lower_query(q, stmt.id)?;
                Ok(logical::LogicalStatement::Query(plan))
            }
            ast::Statement::Ddl(ast::DdlOp::CreateTable(ct)) => {
                let table_name = AstToLogical::symprim_to_binding(&ct.table_name);
                match &ct.as_query {
                    None => Ok(logical::LogicalStatement::CreateTable { table_name }),
                    Some(inner) => {
                        // `as_query` is a `TopLevelQuery` (it may carry a `WITH` clause),
                        // so it feeds the existing pipeline directly — no wrapping needed.
                        let plan = self.lower_query(&inner.node, stmt.id)?;
                        Ok(logical::LogicalStatement::CreateTableAs {
                            table_name,
                            query: plan,
                        })
                    }
                }
            }
            ast::Statement::Ddl(_) => Err(ddl_not_yet_implemented()),
            ast::Statement::Dml(dml) => match &dml.op {
                ast::DmlOp::Insert(insert) => {
                    // The spec-general Expr target is narrowed to a bare table-name
                    // VarRef here; qualified/path targets are spec-legal but not yet
                    // lowered.
                    let table_name = match &*insert.target {
                        ast::Expr::VarRef(v) => AstToLogical::symprim_to_binding(&v.node.name),
                        _ => {
                            return Err(AstTransformationError {
                                errors: vec![AstTransformError::NotYetImplemented(
                                    "INSERT target other than a bare table name".to_string(),
                                )],
                            })
                        }
                    };
                    // `Expr::Query` holds a bare `Query`, but `lower_query` needs a
                    // `TopLevelQuery`; `with: None` is inert (a bare Query carries no CTE).
                    let query = match &*insert.values {
                        ast::Expr::Query(q) => {
                            let wrapped = ast::TopLevelQuery {
                                with: None,
                                query: q.clone(),
                            };
                            self.lower_query(&wrapped, stmt.id)?
                        }
                        _ => {
                            return Err(AstTransformationError {
                                errors: vec![AstTransformError::NotYetImplemented(
                                    "INSERT source must be a query".to_string(),
                                )],
                            })
                        }
                    };
                    Ok(logical::LogicalStatement::InsertInto { table_name, query })
                }
                _ => Err(AstTransformationError {
                    errors: vec![AstTransformError::NotYetImplemented(
                        "non-INSERT DML".to_string(),
                    )],
                }),
            },
        }
    }

    /// Lower a parsed statement into a relational plan.
    ///
    /// Back-compat shim over [`Self::lower_statement`]: returns the inner
    /// [`logical::LogicalPlan`] for a query, and rejects DDL/DML so existing
    /// callers (which only handle queries) keep their current contract.
    #[inline]
    pub fn lower(
        &self,
        parsed: &Parsed<'_>,
    ) -> Result<logical::LogicalPlan<logical::BindingsOp>, AstTransformationError> {
        // This shim only handles a single statement; multi-statement iteration is
        // not its job (the relational-plan return type can carry just one query).
        let [stmt] = parsed.statements.as_slice() else {
            return Err(AstTransformationError {
                errors: vec![AstTransformError::NotYetImplemented(
                    "multi-statement input".to_string(),
                )],
            });
        };
        // Reject DDL/DML at the front door so this shim returns a uniform category
        // error rather than leaking an inner-query error. These are surfaced as
        // plans via `lower_statement`.
        match stmt.node {
            ast::Statement::Ddl(_) => return Err(ddl_not_yet_implemented()),
            ast::Statement::Dml(_) => {
                return Err(AstTransformationError {
                    errors: vec![AstTransformError::NotYetImplemented(
                        "DML statement lowering".to_string(),
                    )],
                })
            }
            ast::Statement::Query(_) => {}
        }
        match self.lower_statement(stmt)? {
            logical::LogicalStatement::Query(plan) => Ok(plan),
            // Non-query statements are rejected at the front door above; this arm
            // is a defensive fallback.
            _ => Err(ddl_not_yet_implemented()),
        }
    }

    /// Shared two-pass lowering of a `TopLevelQuery` (name resolution + visitor).
    #[inline]
    fn lower_query(
        &self,
        query: &ast::TopLevelQuery,
        stmt_id: NodeId,
    ) -> Result<logical::LogicalPlan<logical::BindingsOp>, AstTransformationError> {
        let mut resolver = NameResolver::new(self.catalog);
        let registry = resolver.resolve(query, stmt_id)?;
        let planner = AstToLogical::new(self.catalog, registry);
        planner.lower_query(query, stmt_id)
    }
}
