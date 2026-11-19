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
//!   `into_parts` and `Rex::from_parts` / `Rel::from_parts` (`Plan::new`), or a [`Rewriter`].
//! - [`Visitor`] walks a plan; [`Rewriter`] rebuilds one. An overriding method calls the
//!   default traversal as [`visit::walk_rex`] / [`rewrite::walk_rex`] (and `walk_rel`,
//!   `walk_plan`).
//! - [`Plan::dump`] prints a plan as an indented tree; [`tree`] gives the same tree as data.
//!   EXPERIMENTAL: the dump format, the [`TreeNode`] shape and the text helpers ([`name_text`],
//!   [`value_text`], [`render_shape`]) are a debugging and comparison aid and may change; the
//!   dump is order-preserving, not canonical (see the `print` module docs).
//! - [`validate`] checks the invariants of a typed planner's output: no-id, non-union,
//!   nullable shapes with a partiql-lang-kotlin counterpart; `Undefined` (the bottom type) only
//!   where absent values flow; failed expressions typed `Dynamic`.
//!
//! Plans are not `Send`/`Sync`: a literal holds a `partiql_value::Value`, which is not (the
//! same holds for `partiql-logical`'s plan).
//!
//! Catalog objects are referenced by name ([`QualifiedName`]) and, for routines, by the
//! resolved signature ([`RoutineRef`]), so this crate does not depend on a catalog API.

mod plan;
mod print;
mod refs;
mod rel;
pub mod rewrite;
mod rex;
mod validate;
pub mod visit;

pub use plan::{Action, Plan, Version};
pub use print::{
    name_text, rel_tree, render_shape, rex_tree, tree, value_text, Attr, NodeType, Source, TreeNode,
};
pub use refs::{QualifiedName, RoutineRef};
pub use rel::{
    Collation, Exclusion, ExclusionItem, JoinType, Measure, Nulls, Order, Rel, RelField, RelOp,
    RelType, WindowFunction, WindowSignature, WithElement,
};
pub use rewrite::Rewriter;
pub use rex::{
    CaseBranch, Comparison, MapEntry, Quantifier, Rex, RexOp, StructEntry, SubqueryTest,
};
pub use validate::{validate, validate_with_max_depth, Violation, ViolationKind, MAX_DEPTH};
pub use visit::Visitor;
