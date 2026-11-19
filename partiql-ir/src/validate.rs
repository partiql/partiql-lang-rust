//! Invariants of a typed plan produced by the typed planner.
//!
//! The typed planner matches partiql-lang-kotlin's (v1.6.1) typing exactly, whose types have no
//! unions, no nullability and no identity. So in its output every attached shape, and every
//! shape nested in one, is
//! - built without a node id (`PartiqlNoIdShapeBuilder`, id 0), so that equality is structural;
//! - not an `AnyOf` union (partiql-lang-kotlin collapses unions to `DYNAMIC`);
//! - not `Undefined`;
//! - nullable.
//!
//! and an `Exclusion`'s variable is a `RexVar` (`Exclusion.kt:30`).

use crate::plan::Plan;
use crate::print::{tree, Attr, NodeType, TreeNode};
use partiql_types::{PartiqlShape, Static};
use std::fmt::{Display, Formatter};

/// One broken invariant, at a path from the plan root.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Violation {
    path: String,
    kind: ViolationKind,
}

impl Violation {
    /// Where: the roles from the plan root to the node, then the shape's place in the node
    /// (`.type`, `.attr`, `.type.fields[a]`, `.type.element`, ..). E.g.
    /// `action.rex.input.rex.type.element.fields[b]`.
    pub fn path(&self) -> &str {
        &self.path
    }

    pub fn kind(&self) -> &ViolationKind {
        &self.kind
    }
}

impl Display for Violation {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}: {}", self.path, self.kind)
    }
}

/// What invariant is broken.
#[derive(Debug, Clone, PartialEq, Eq)]
#[non_exhaustive]
pub enum ViolationKind {
    /// A shape built with a node id (not a no-id shape).
    ShapeWithId { id: u32 },
    /// An `AnyOf` union.
    AnyOf,
    /// The `Undefined` shape.
    Undefined,
    /// A non-nullable shape.
    NotNullable,
    /// A shape kind this crate does not know (`PartiqlShape` is non-exhaustive).
    UnsupportedShape,
    /// An `Exclusion` whose variable is not a `RexVar`.
    ExclusionVarNotVar,
}

impl Display for ViolationKind {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        match self {
            ViolationKind::ShapeWithId { id } => write!(f, "shape has node id {id}"),
            ViolationKind::AnyOf => write!(f, "AnyOf union"),
            ViolationKind::Undefined => write!(f, "Undefined shape"),
            ViolationKind::NotNullable => write!(f, "non-nullable shape"),
            ViolationKind::UnsupportedShape => write!(f, "unsupported shape kind"),
            ViolationKind::ExclusionVarNotVar => write!(f, "exclusion variable is not a RexVar"),
        }
    }
}

/// Checks `plan` against the invariants above; returns every violation, in tree order.
pub fn validate(plan: &Plan) -> Result<(), Vec<Violation>> {
    let mut out = vec![];
    check_node(&tree(plan), "", &mut out);
    if out.is_empty() {
        Ok(())
    } else {
        Err(out)
    }
}

fn join(path: &str, role: &str) -> String {
    if path.is_empty() {
        role.to_string()
    } else {
        format!("{path}.{role}")
    }
}

fn check_node(node: &TreeNode<'_>, path: &str, out: &mut Vec<Violation>) {
    match node.ty {
        Some(NodeType::Rex(t)) => check_shape(t, &join(path, "type"), out),
        Some(NodeType::Rel(t)) => {
            for f in t.fields() {
                check_shape(
                    f.ty(),
                    &join(path, &format!("type.fields[{}]", f.name())),
                    out,
                );
            }
        }
        None => {}
    }
    for (name, attr) in &node.attrs {
        if let Attr::Shape(s) = attr {
            check_shape(s, &join(path, name), out);
        }
    }
    if node.kind == "Exclusion" {
        if let Some((_, var)) = node.children.iter().find(|(role, _)| role == "var") {
            if var.kind != "RexVar" {
                out.push(Violation {
                    path: join(path, "var"),
                    kind: ViolationKind::ExclusionVarNotVar,
                });
            }
        }
    }
    for (role, child) in &node.children {
        check_node(child, &join(path, role), out);
    }
}

fn check_shape(shape: &PartiqlShape, path: &str, out: &mut Vec<Violation>) {
    let mut push = |kind| {
        out.push(Violation {
            path: path.to_string(),
            kind,
        })
    };
    match shape {
        PartiqlShape::Dynamic => {}
        PartiqlShape::AnyOf(_) => push(ViolationKind::AnyOf),
        PartiqlShape::Undefined => push(ViolationKind::Undefined),
        PartiqlShape::Static(st) => {
            let id = st.ty_id().0;
            if id != 0 {
                push(ViolationKind::ShapeWithId { id });
            }
            if st.is_not_nullable() {
                push(ViolationKind::NotNullable);
            }
            match st.ty() {
                Static::Struct(s) => {
                    for f in s.fields() {
                        check_shape(f.ty(), &format!("{path}.fields[{}]", f.name()), out);
                    }
                }
                Static::Bag(b) => check_shape(b.element_type(), &format!("{path}.element"), out),
                Static::Array(a) => check_shape(a.element_type(), &format!("{path}.element"), out),
                Static::Map(m) => {
                    check_shape(m.key_type(), &format!("{path}.key"), out);
                    check_shape(m.value_type(), &format!("{path}.value"), out);
                }
                _ => {}
            }
        }
        // `PartiqlShape` is non-exhaustive: a shape kind added later is not one that
        // partiql-lang-kotlin's typing produces.
        _ => push(ViolationKind::UnsupportedShape),
    }
}
