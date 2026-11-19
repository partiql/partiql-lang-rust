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
