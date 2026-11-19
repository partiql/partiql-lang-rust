#![deny(rust_2018_idioms)]
#![deny(clippy::all)]

//! A typed `PartiQL` plan.
//!
//! A plan is a tree of scalar expressions ([`Rex`]) and relational operators ([`Rel`]). Every
//! `Rex` carries the [`partiql_types::PartiqlShape`] of the value it produces, and every `Rel`
//! the ordered schema ([`RelType`]) of the rows it produces. The node set follows the public
//! plan of partiql-lang-kotlin v1.6.1 (`org.partiql.plan`), one [`RexOp`] / [`RelOp`] variant
//! per `Rex*` / `Rel*` class.
//!
//! This plan sits beside the untyped `partiql-logical` plan, which evaluation keeps using.
//!
//! - Nodes are immutable: private fields, constructors, getters; build a changed copy with
//!   `into_parts` / `from_parts` or a [`Rewriter`].
//! - [`Visitor`] walks a plan; [`Rewriter`] rebuilds one.
//! - [`Plan::dump`] prints a plan as an indented tree; [`tree`] gives the same tree as data.
//! - [`validate`] checks the invariants of a typed planner's output (no-id, non-union,
//!   nullable shapes).
//!
//! Catalog objects are referenced by name ([`QualifiedName`]) and, for routines, by the
//! resolved signature ([`RoutineRef`]), so this crate does not depend on a catalog API.

mod plan;
mod print;
mod refs;
mod rel;
mod rewrite;
mod rex;
mod validate;
mod visit;

pub use plan::{Action, Plan, Version};
pub use print::{rel_tree, render_shape, rex_tree, tree, Attr, NodeType, TreeNode};
pub use refs::{QualifiedName, RoutineRef};
pub use rel::{
    Collation, Exclusion, ExclusionItem, JoinType, Measure, Nulls, Order, Rel, RelField, RelOp,
    RelType, WindowFunction, WindowSignature, WithElement,
};
pub use rewrite::Rewriter;
pub use rex::{
    CaseBranch, Comparison, MapEntry, Quantifier, Rex, RexOp, StructEntry, SubqueryTest,
};
pub use validate::{validate, Violation, ViolationKind};
pub use visit::Visitor;

/// The default `walk_*` functions of [`Visitor`] and [`Rewriter`], for overriding methods to
/// call.
pub mod walk {
    pub use crate::rewrite::{
        walk_plan as rewrite_plan, walk_rel as rewrite_rel, walk_rex as rewrite_rex,
    };
    pub use crate::visit::{walk_plan, walk_rel, walk_rex};
}
