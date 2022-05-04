mod common;

use partiql_ast::ast;
use partiql_ast::ast::*;
use partiql_source_map::location::{BytePosition, Location};

#[test]
fn test_ast_init() {
    common::setup();

    let _i = Item {
        kind: ItemKind::Query(Query {
            set: QuerySet::Expr(Box::new(Expr {
                kind: ExprKind::Lit(Lit::Int32Lit(23).to_ast(BytePosition::from(1)..12.into())),
            }))
            .to_ast(BytePosition::from(1)..12.into()),
            offset: None,
            order_by: None,
            limit: None,
        }),
    };

    let span_only = ast::SymbolPrimitive {
        value: "symbol1".to_string(),
    }
    .to_node()
    .location(Location {
        start: BytePosition::from(12),
        end: BytePosition::from(1),
    })
    .build()
    .expect("Could not retrieve ast node");

    assert_eq!(
        Some(Location {
            start: BytePosition::from(12),
            end: BytePosition::from(1),
        }),
        span_only.location
    );
}
