mod common;

use partiql_ast::experimental::ast::*;

#[test]
fn test_ast_init() {
    common::setup();

    let _i = Item {
        kind: ItemKind::Query(Query {
            expr: Expr {
                kind: ExprKind::Lit(Lit {
                    kind: LitKind::NumericLit(NumericLit {
                        kind: NumericLitKind::Int32(Int32 { value: 12 }),
                    }),
                }),
            },
        }),
    };

    // TODO Add assertion once we have tree traversals
}
