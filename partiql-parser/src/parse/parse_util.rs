use partiql_ast::ast;

pub(crate) enum CallSite {
    Call(ast::Call),
    CallAgg(ast::CallAgg),
}

// if this is just a parenthesized expr, lift it out of the query AST, otherwise return input
//      e.g. `(1+2)` should be a ExprKind::Expr, not wrapped deep in a ExprKind::Query
pub(crate) fn strip_query(q: Box<ast::Expr>) -> Box<ast::Expr> {
    if let ast::Expr::Query(ast::AstNode {
        node:
            ast::Query {
                with: None,
                set:
                    ast::AstNode {
                        node: ast::QuerySet::Expr(e),
                        ..
                    },
                order_by: None,
                limit: None,
                offset: None,
            },
        ..
    }) = *q
    {
        e
    } else {
        q
    }
}
