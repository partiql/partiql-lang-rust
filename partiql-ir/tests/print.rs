//! Snapshots of hand-built plans printed by `Plan::dump`.
mod common;

use common::*;
use partiql_ir::{tree, NodeType, Plan};
use partiql_types::PartiqlShape;

#[test]
fn dump_select_from_t() {
    insta::assert_snapshot!(select_from_t().dump());
}

#[test]
fn dump_every_node() {
    insta::assert_snapshot!(every_node().dump());
}

#[test]
fn dump_with_custom_shape_renderer() {
    let kind = |s: &PartiqlShape| match s {
        PartiqlShape::Dynamic => "DYNAMIC".to_string(),
        _ => "T".to_string(),
    };
    insta::assert_snapshot!(Plan::query(partiql_ir::Rex::var(dynamic(), 0, 0)).dump_with(&kind), @"
    Plan {version: V1}
      action: Query
        rex: RexVar :: DYNAMIC {scope: 0, offset: 0}
    ");
}

/// The tree is the data the printer prints: roles and types are reachable without parsing.
#[test]
fn tree_exposes_roles_and_types() {
    let plan = select_from_t();
    let root = tree(&plan);
    let (role, query) = &root.children[0];
    assert_eq!(role, "action");
    let (role, select) = &query.children[0];
    assert_eq!(role, "rex");
    assert_eq!(select.kind, "RexSelect");
    let roles: Vec<&str> = select.children.iter().map(|(r, _)| r.as_str()).collect();
    assert_eq!(roles, ["input", "constructor"]);
    let (_, project) = &select.children[0];
    assert!(matches!(project.ty, Some(NodeType::Rel(t)) if t.degree() == 1));
}

/// Distinct catalog names print distinctly.
#[test]
fn qualified_names_are_quoted() {
    use partiql_ir::QualifiedName;
    let one = QualifiedName::new("default", ["s.x"], "T");
    let two = QualifiedName::new("default", ["s", "x"], "T");
    assert_eq!(one.to_string(), r#""default"."s.x"."T""#);
    assert_eq!(two.to_string(), r#""default"."s"."x"."T""#);
    assert_eq!(
        QualifiedName::new("", Vec::<String>::new(), "a\"b").to_string(),
        r#"""."a""b""#
    );
}

/// Names that could be read as dump syntax are quoted.
#[test]
fn names_are_quoted_when_not_simple() {
    use partiql_ir::{name_text, RelField, RelType};
    assert_eq!(name_text("a_1$"), "a_1$");
    assert_eq!(name_text("a: Int32, b"), r#""a: Int32, b""#);
    assert_eq!(name_text(""), r#""""#);
    let one = RelType::new(vec![RelField::new("a: Int32, b", int())], false);
    let two = RelType::new(
        vec![RelField::new("a", int()), RelField::new("b", int())],
        false,
    );
    let dump = |t: RelType| {
        partiql_ir::Plan::query(partiql_ir::Rex::select(
            bag(dynamic()),
            partiql_ir::Rel::scan(t, partiql_ir::Rex::var(dynamic(), 0, 0)),
            partiql_ir::Rex::var(dynamic(), 0, 0),
        ))
        .dump()
    };
    assert_ne!(dump(one), dump(two));
}

#[test]
fn dispatch_and_signature_names_are_quoted() {
    let r = partiql_ir::Rex::dispatch(dynamic(), "g}, z: w", vec![], vec![]);
    let dump = partiql_ir::Plan::query(r).dump();
    assert!(dump.contains(r#"{name: "g}, z: w"}"#), "{dump}");
}

/// Literals print their kind and escape strings, so unequal literals print differently.
#[test]
fn literals_print_kind_and_escape() {
    use partiql_ir::value_text;
    use partiql_value::Value;
    assert_eq!(value_text(&Value::from(1)), "Integer 1");
    assert_eq!(value_text(&Value::from(1.0)), "Real 1.0");
    assert_eq!(
        value_text(&Value::from("x', b: 'y\nz")),
        r#"String "x', b: 'y\nz""#
    );
}

/// A struct that is neither a row nor `new_any()` prints its constraints.
#[test]
fn non_canonical_structs_print_their_constraints() {
    use partiql_ir::render_shape;
    use partiql_types::{StructConstraint, StructType};
    let open = b().new_struct(StructType::new(
        [StructConstraint::Open(true)].into_iter().collect(),
    ));
    let any = b().new_struct(StructType::new_any());
    assert_eq!(render_shape(&any), "{...}");
    assert_ne!(render_shape(&open), render_shape(&any));
}
