mod common;

use partiql_ast::ast::*;

#[test]
fn test_ast_init() {
    common::setup();

    let _i = Item::Query(Query {
        with: None,
        set: AstNode {
            id: NodeId(2),
            node: QuerySet::Expr(Box::new(Expr::Lit(AstNode {
                id: NodeId(1),
                node: Lit::Int32Lit(23),
            }))),
        },
        order_by: None,
        limit_offset: None,
    });
}
