mod common;

use partiql_ast::experimental::ast;
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

    let p = ast::SymbolPrimitive { value: "hello".to_string() };
    p.new();

    // TODO Add assertion once we have tree traversals
}
