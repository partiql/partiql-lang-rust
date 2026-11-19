//! Invariants of a typed plan produced by the typed planner.
//!
//! The typed planner matches partiql-lang-kotlin's (v1.6.1) typing exactly, whose types have no
//! unions, no nullability and no identity. So in its output every attached shape, and every
//! shape nested in one,
//! - is built without a node id (`PartiqlNoIdShapeBuilder`, id 0), so that equality is
//!   structural;
//! - is not an `AnyOf` union (partiql-lang-kotlin collapses unions to `DYNAMIC`);
//! - is nullable;
//! - has a partiql-lang-kotlin counterpart: not `Int` (unbounded), `Decimal` (unparameterized),
//!   `DateTime` or `Graph`; a struct is a row (`ROW`) or `StructType::new_any()` (`STRUCT`),
//!   and has no optional fields. So a typer normalizes catalog schemas (e.g. an open struct
//!   with known fields) to these forms before it puts them in a plan.
//!
//! `Undefined` is the bottom type, "no present value" (partiql-lang-kotlin's `UNKNOWN`). `NULL` and
//! `MISSING` are absent values, not types: the typer types a `NULL` or `MISSING` literal
//! `Undefined`, and a literal typed `Undefined` must hold an absent value. An absent-value literal
//! may also carry a concrete type: partiql-lang-kotlin replaces the `NULL` / `MISSING` branches of
//! `CASE` and `COALESCE` with absent literals typed as the supertype (`DynamicTyper.kt:107-108`).
//! `Undefined` may appear wherever an absent value's type flows: on an expression that can only
//! produce absent values (a variable or path reading one, a supertype of only absent values,
//! `PlanTyper.kt:134-136`), nested in rows, collections and map values (`SELECT NULL AS a`,
//! `[NULL]`; not map keys, see below), in schemas, in resolved routine signatures
//! (partiql-lang-kotlin resolves `upper(NULL)` with an `UNKNOWN` parameter, `FnUpper.kt:49`), and
//! nested in a cast target (a set operation or map constructor coerces `NULL` to the supertype,
//! e.g. `ARRAY<UNKNOWN>`). It may not be a cast target itself, nor the type of a constructor, a
//! table reference, a non-scalar subquery or `EXISTS`/`UNIQUE`, which always produce a present
//! value. An absent value is `Datum::is_absent`, so a null variant counts. A `RexError` (a failed
//! expression) is typed `Dynamic`, never `Undefined`. partiql-lang-kotlin types some errors
//! `UNKNOWN` (`PlanTyper.kt:1014,1099,1111`) and that `UNKNOWN` propagates into parents (e.g. a row
//! field of an unresolved path), so a typer that matches it keeps PLK's type for those parents: a
//! parent's `Undefined` may come from a `Dynamic` error child. A conformance comparison maps the
//! error nodes themselves. Not checked: a literal whose type only nests `Undefined` against the
//! value's elements.
//!
//! Two nodes whose type partiql-lang-kotlin derives in the node itself agree with it, including
//! unspecified-parameter flags: a `RexCast`'s type is its target (`rex/RexCast.java:42`), a
//! `RexMap`'s type is the map of its key and value types (`rex/RexMap.java:63`), whose key is
//! neither `DYNAMIC` nor `Undefined`, nor a union with such a member, as `MapType::new` requires
//! (the MAP type RFC disallows NULL and MISSING keys). This departs from partiql-lang-kotlin, which
//! types `MAP{NULL: 1}` as `MAP<UNKNOWN, INT>`; a typer reports such a map as an error instead.
//! Other types (e.g. a `RexCall`'s, a `RexLit`'s) are set by the typer (partiql-lang-kotlin's
//! `PlanTransform` overrides them with `setType`) and are not compared. Structurally: an
//! `Exclusion` has a `RexVar` variable and at least one item (`Exclusion.kt:15,30`). A
//! `RelWindow`'s `bindings` are not checked: partiql-lang-kotlin takes them as the last names of
//! the window's input schema, possibly fewer than its functions (`PlanTransform.kt:280`).
//!
//! Not checked (left to the typer, whose scoping rules define them): that a `RexVar`'s scope and
//! offset are in range, and that a `Rel`'s schema matches its operator.

use crate::plan::Plan;
use crate::print::{is_plk_struct, name_text};
use crate::print::{tree, Attr, NodeType, Source, TreeNode};
use crate::rel::RelOp;
use crate::rex::RexOp;
use partiql_types::{PartiqlNoIdShapeBuilder, PartiqlShape, ShapeResultError, Static};
use partiql_value::datum::Datum;
use std::fmt::{Display, Formatter};

/// One broken invariant, at a path from the plan root.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Violation {
    path: String,
    kind: ViolationKind,
}

impl Violation {
    /// Where: the roles from the plan root to the node, then the shape's place in the node
    /// (`.type`, `.<attr>`, `.type.fields[<i>:<name>]`, `.element`, `.key`, `.value`). E.g.
    /// `action.rex.input.rex.type.element.fields[1:b]`.
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
    /// `Undefined` where no absent value can be: a cast target, a literal with a present value,
    /// or a constructor, table reference or `EXISTS`/`UNIQUE`.
    Undefined,
    /// A `RexError` not typed `Dynamic`.
    ErrorNotDynamic,
    /// A non-nullable shape.
    NotNullable,
    /// A shape with no partiql-lang-kotlin counterpart (see the module docs).
    NoPlkEquivalent,
    /// A shape kind this crate does not know (`PartiqlShape` and `Static` are non-exhaustive).
    UnsupportedShape,
    /// A `RexCast` whose type is not its target.
    CastTypeNotTarget,
    /// A `RexMap` whose type is not the map of its key and value types.
    MapTypeNotDeclared,
    /// A `RexMap` whose key type is `DYNAMIC` (or a union with that member).
    DynamicMapKey,
    /// A `RexMap` whose key type is `Undefined` (or a union with that member); the MAP type RFC
    /// disallows NULL and MISSING keys.
    UndefinedMapKey,
    /// An `Exclusion` whose variable is not a `RexVar`.
    ExclusionVarNotVar,
    /// An `Exclusion` without items.
    ExclusionWithoutItems,
}

impl Display for ViolationKind {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        match self {
            ViolationKind::ShapeWithId { id } => write!(f, "shape has node id {id}"),
            ViolationKind::AnyOf => write!(f, "AnyOf union"),
            ViolationKind::Undefined => write!(f, "Undefined where no absent value can be"),
            ViolationKind::ErrorNotDynamic => write!(f, "error node is not typed Dynamic"),
            ViolationKind::NotNullable => write!(f, "non-nullable shape"),
            ViolationKind::NoPlkEquivalent => {
                write!(f, "shape has no partiql-lang-kotlin counterpart")
            }
            ViolationKind::UnsupportedShape => write!(f, "unsupported shape kind"),
            ViolationKind::CastTypeNotTarget => write!(f, "cast type is not its target"),
            ViolationKind::MapTypeNotDeclared => {
                write!(f, "map type is not MAP<key_type, value_type>")
            }
            ViolationKind::DynamicMapKey => write!(f, "map key type is DYNAMIC"),
            ViolationKind::UndefinedMapKey => {
                write!(f, "map key type is Undefined")
            }
            ViolationKind::ExclusionVarNotVar => write!(f, "exclusion variable is not a RexVar"),
            ViolationKind::ExclusionWithoutItems => write!(f, "exclusion has no items"),
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

fn push(out: &mut Vec<Violation>, path: String, kind: ViolationKind) {
    out.push(Violation { path, kind });
}

fn check_node(node: &TreeNode<'_>, path: &str, out: &mut Vec<Violation>) {
    match node.ty {
        Some(NodeType::Rex(t)) => check_shape(t, &join(path, "type"), out),
        Some(NodeType::Rel(t)) => {
            for (i, f) in t.fields().iter().enumerate() {
                let p = join(path, &format!("type.fields[{i}:{}]", name_text(f.name())));
                check_shape(f.ty(), &p, out);
            }
        }
        None => {}
    }
    for (name, attr) in &node.attrs {
        if let Attr::Shape(s) = attr {
            check_shape(s, &join(path, name), out);
        }
    }
    // Constructors, table references, non-scalar subqueries and EXISTS/UNIQUE always produce a
    // present value.
    if let Some(Source::Rex(rex)) = node.source {
        let present = matches!(
            rex.op(),
            RexOp::Array { .. }
                | RexOp::Bag { .. }
                | RexOp::Struct { .. }
                | RexOp::Map { .. }
                | RexOp::MapDynamic { .. }
                | RexOp::Spread { .. }
                | RexOp::Select { .. }
                | RexOp::Pivot { .. }
                | RexOp::Table { .. }
                | RexOp::TableRef { .. }
                | RexOp::SubqueryTest { .. }
                // A non-scalar subquery builds a collection; a scalar one yields a field value,
                // which may be absent.
                | RexOp::Subquery { scalar: false, .. }
        );
        if present && rex.ty().is_undefined() {
            push(out, join(path, "type"), ViolationKind::Undefined)
        }
    }
    match node.source {
        Some(Source::Rex(rex)) => match rex.op() {
            RexOp::Lit { value } => {
                if rex.ty().is_undefined() && !value.is_absent() {
                    push(out, join(path, "type"), ViolationKind::Undefined)
                }
            }
            RexOp::Error {} if !rex.ty().is_dynamic() => {
                push(out, join(path, "type"), ViolationKind::ErrorNotDynamic)
            }
            RexOp::Cast { target, .. } => {
                if target.is_undefined() {
                    push(out, join(path, "target"), ViolationKind::Undefined)
                }
                // Shapes with ids are reported above; comparing them would add false mismatches.
                if !has_id(rex.ty()) && !has_id(target) && !same_type(rex.ty(), target) {
                    push(out, join(path, "type"), ViolationKind::CastTypeNotTarget)
                }
            }
            RexOp::Map {
                key_type,
                value_type,
                ..
            } => {
                // `new_map` is the authority on valid keys (partiql-types follows the MAP type
                // RFC): a key that is `DYNAMIC` or `Undefined` (or a union with such a member) has no
                // map type.
                let declared = PartiqlNoIdShapeBuilder::default()
                    .new_map(key_type.clone(), value_type.clone());
                match declared {
                    Err(ShapeResultError::DynamicMapKey) => {
                        push(out, join(path, "key_type"), ViolationKind::DynamicMapKey)
                    }
                    Err(ShapeResultError::UndefinedMapKey) => {
                        push(out, join(path, "key_type"), ViolationKind::UndefinedMapKey)
                    }
                    Ok(d) if has_id(rex.ty()) || has_id(&d) || same_type(&d, rex.ty()) => {}
                    _ => push(out, join(path, "type"), ViolationKind::MapTypeNotDeclared),
                }
            }
            _ => {}
        },
        Some(Source::Rel(rel)) => {
            if let RelOp::Exclude { exclusions, .. } = rel.op() {
                for (i, e) in exclusions.iter().enumerate() {
                    if !matches!(e.var().op(), RexOp::Var { .. }) {
                        let p = join(path, &format!("exclusions[{i}].var"));
                        push(out, p, ViolationKind::ExclusionVarNotVar);
                    }
                    if e.items().is_empty() {
                        let p = join(path, &format!("exclusions[{i}]"));
                        push(out, p, ViolationKind::ExclusionWithoutItems);
                    }
                }
            }
        }
        None => {}
    }
    for (role, child) in &node.children {
        check_node(child, &join(path, role), out);
    }
}

/// Equal (`==`) and with equal unspecified-parameter flags, which `==` ignores.
fn same_type(a: &PartiqlShape, b: &PartiqlShape) -> bool {
    a == b && same_params(a, b)
}

/// Compares the `TypeParams` of two shapes that are `==`, position by position.
fn same_params(a: &PartiqlShape, b: &PartiqlShape) -> bool {
    let (PartiqlShape::Static(x), PartiqlShape::Static(y)) = (a, b) else {
        return true;
    };
    if x.params() != y.params() {
        return false;
    }
    match (x.ty(), y.ty()) {
        // Only rows keep field order; equal field sets may iterate in different orders.
        (Static::Struct(s), Static::Struct(t)) if s.is_row() && t.is_row() => s
            .fields()
            .zip(t.fields())
            .all(|(f, g)| same_params(f.ty(), g.ty())),
        (Static::Bag(s), Static::Bag(t)) => same_params(s.element_type(), t.element_type()),
        (Static::Array(s), Static::Array(t)) => same_params(s.element_type(), t.element_type()),
        (Static::Map(s), Static::Map(t)) => {
            same_params(s.key_type(), t.key_type()) && same_params(s.value_type(), t.value_type())
        }
        _ => true,
    }
}

/// Whether `shape` or a shape nested in it has a node id.
fn has_id(shape: &PartiqlShape) -> bool {
    let st = match shape {
        PartiqlShape::Static(st) => st,
        PartiqlShape::AnyOf(a) => return a.types().any(has_id),
        _ => return false,
    };
    st.ty_id().0 != 0
        || match st.ty() {
            Static::Struct(s) => s.fields().any(|f| has_id(f.ty())),
            Static::Bag(b) => has_id(b.element_type()),
            Static::Array(a) => has_id(a.element_type()),
            Static::Map(m) => has_id(m.key_type()) || has_id(m.value_type()),
            _ => false,
        }
}

fn check_shape(shape: &PartiqlShape, path: &str, out: &mut Vec<Violation>) {
    let here = |out: &mut Vec<Violation>, kind| push(out, path.to_string(), kind);
    match shape {
        PartiqlShape::Dynamic => {}
        PartiqlShape::AnyOf(a) => {
            here(out, ViolationKind::AnyOf);
            for (i, t) in a.types().enumerate() {
                check_shape(t, &format!("{path}.any_of[{i}]"), out);
            }
        }
        // The bottom type; where it may not appear is checked per node (see the module docs).
        PartiqlShape::Undefined => {}
        PartiqlShape::Static(st) => {
            let id = st.ty_id().0;
            if id != 0 {
                here(out, ViolationKind::ShapeWithId { id });
            }
            if st.is_not_nullable() {
                here(out, ViolationKind::NotNullable);
            }
            match st.ty() {
                Static::Int8
                | Static::Int16
                | Static::Int32
                | Static::Int64
                | Static::Bool
                | Static::DecimalP(..)
                | Static::NumericP(..)
                | Static::Float32
                | Static::Float64
                | Static::String
                | Static::StringFixed(_)
                | Static::StringVarying(_)
                | Static::Blob(_)
                | Static::Clob(_)
                | Static::Date
                | Static::Time(_)
                | Static::TimeTz(_)
                | Static::Timestamp(_)
                | Static::TimestampTz(_)
                | Static::Interval(_)
                | Static::Variant(_) => {}
                Static::Int | Static::Decimal | Static::DateTime | Static::Graph() => {
                    here(out, ViolationKind::NoPlkEquivalent)
                }
                Static::Struct(s) => {
                    // PLK's STRUCT is the one canonical open struct, `StructType::new_any()`.
                    let plk_struct = is_plk_struct(s);
                    if !plk_struct {
                        here(out, ViolationKind::NoPlkEquivalent);
                    }
                    for (i, f) in s.fields().enumerate() {
                        let p = format!("{path}.fields[{i}:{}]", name_text(f.name()));
                        if f.is_optional() {
                            push(out, p.clone(), ViolationKind::NoPlkEquivalent);
                        }
                        check_shape(f.ty(), &p, out);
                    }
                }
                Static::Bag(b) => check_shape(b.element_type(), &format!("{path}.element"), out),
                Static::Array(a) => check_shape(a.element_type(), &format!("{path}.element"), out),
                Static::Map(m) => {
                    check_shape(m.key_type(), &format!("{path}.key"), out);
                    check_shape(m.value_type(), &format!("{path}.value"), out);
                }
                // `Static` is non-exhaustive: a kind added later may nest shapes this check
                // cannot see, and is not one partiql-lang-kotlin's typing produces.
                _ => here(out, ViolationKind::UnsupportedShape),
            }
        }
        _ => here(out, ViolationKind::UnsupportedShape),
    }
}
