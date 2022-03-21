mod common;

use partiql_ast::experimental::ast::*;

#[test]
fn test_ast_init() {
    common::setup();

    let _i = Item {
        kind: ItemKind::Query(Query {
            expr: Box::new(Expr {
                kind: ExprKind::Lit(Lit {
                    kind: LitKind::NumericLit(NumericLit {
                        kind: NumericLitKind::Int32Lit(Int32Lit { value: 12 }),
                    }),
                }),
            }),
        }),
    };

    // TODO Add assertion once we have tree traversals
}
