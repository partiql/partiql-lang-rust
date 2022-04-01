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
    let m = NodeMetaData::from([("test", NodeMetaDataValue::Bool(true))]);
    let n = p.to_node()
        .with_span("1".to_string(), "2".to_string())
        .with_meta(m)
        .build();
    assert_eq!(Some(Span { begin: "1".to_string(), end: "2".to_string()}), n.span);
    assert_eq!(Some(NodeMetaData::from([("test", NodeMetaDataValue::Bool(true))])),
               n.meta);
}
