mod common;

use partiql_ast::experimental::ast;
use partiql_ast::experimental::ast::*;
use partiql_common::srcmap::location::BytePosition;

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

    let meta_only = ast::SymbolPrimitive {
        value: "symbol2".to_string(),
    }
    .to_node()
    .meta(NodeMetaData::from([(
        "test",
        NodeMetaDataValue::Bool(true),
    )]))
    .build()
    .expect("Could not retrieve ast node");

    assert_eq!(
        Some(NodeMetaData::from([(
            "test",
            NodeMetaDataValue::Bool(true),
        )])),
        meta_only.meta
    );

    let all_fields = ast::SymbolPrimitive {
        value: "symbol3".to_string(),
    }
    .to_node()
    .span(Span {
        begin: BytePosition::from(12),
        end: BytePosition::from(1),
    })
    .meta(NodeMetaData::from([(
        "test",
        NodeMetaDataValue::Bool(true),
    )]))
    .build()
    .expect("Could not retrieve ast node");

    assert_eq!(
        AstNode {
            node: SymbolPrimitive {
                value: "symbol3".to_string()
            },
            span: Some(Span {
                begin: BytePosition::from(12),
                end: BytePosition::from(1),
            }),
            meta: Some(NodeMetaData::from([(
                "test",
                NodeMetaDataValue::Bool(true),
            )])),
        },
        all_fields
    );
}
