mod common;

use ion_rs::value::AnyInt;
use ion_rs::value::owned::{OwnedValue};
use partiql_ast::experimental::ast::*;

#[test]
fn test_ast_init() {
    common::setup();

    let _i = Item {
        kind: ItemKind::Query(Query {
            expr: Expr {
                kind: ExprKind::Lit(Lit { value: OwnedValue::Integer(AnyInt::I64(1)) })
            }
        })
    };

    // TODO Add assertion once we have tree traversals
}