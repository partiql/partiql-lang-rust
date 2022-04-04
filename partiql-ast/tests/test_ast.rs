mod common;

use partiql_ast::experimental::ast;
use partiql_ast::experimental::ast::*;
use partiql_source_map::location::BytePosition;

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

    let span_only = ast::SymbolPrimitive {
        value: "symbol1".to_string(),
    }
    .to_node()
    .span(Span {
        begin: BytePosition::from(12),
        end: BytePosition::from(1),
    })
    .build()
    .expect("Could not retrieve ast node");

    assert_eq!(
        Some(Span {
            begin: BytePosition::from(12),
            end: BytePosition::from(1),
        }),
        span_only.span
    );
}
