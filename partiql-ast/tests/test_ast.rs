mod common;

use partiql_ast::experimental::ast::*;

#[test]
fn test_ast_init() {
    common::setup();

    let _i = Item {
        kind: ItemKind::Query(Query {
            expr: Box::new(Expr {
                kind: ExprKind::Lit(Lit::Int32Lit(23)),
            }),
        }),
    };

    // TODO Add assertion once we have tree traversals
}
