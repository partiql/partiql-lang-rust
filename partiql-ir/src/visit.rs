//! A read-only visitor over a plan.

use crate::plan::{Action, Plan};
use crate::refs::RoutineRef;
use crate::rel::{Rel, RelOp};
use crate::rex::{Rex, RexOp};
use partiql_types::PartiqlShape;

/// Visits a plan's expressions, operators and attached shapes.
///
/// Each method's default calls the matching `walk_*` function, which visits the node's children
/// in field order. Override a method to act on a node (match on [`Rex::op`] / [`Rel::op`]) and
/// call the `walk_*` function to keep descending.
pub trait Visitor {
    fn visit_plan(&mut self, plan: &Plan) {
        walk_plan(self, plan)
    }

    fn visit_rex(&mut self, rex: &Rex) {
        walk_rex(self, rex)
    }

    fn visit_rel(&mut self, rel: &Rel) {
        walk_rel(self, rel)
    }

    /// Called for every shape attached to the plan: each [`Rex::ty`], each field of each
    /// [`Rel::ty`], cast targets, map key/value types and routine signatures. Not called for
    /// the shapes nested inside a shape.
    fn visit_shape(&mut self, _shape: &PartiqlShape) {}
}

pub fn walk_plan<V: Visitor + ?Sized>(v: &mut V, plan: &Plan) {
    match plan.action() {
        Action::Query { rex } => v.visit_rex(rex),
    }
}

fn walk_routine<V: Visitor + ?Sized>(v: &mut V, routine: &RoutineRef) {
    routine.params().iter().for_each(|p| v.visit_shape(p));
    v.visit_shape(routine.returns());
}

fn walk_all<V: Visitor + ?Sized>(v: &mut V, rexes: &[Rex]) {
    rexes.iter().for_each(|r| v.visit_rex(r));
}

/// Visits `rex`'s shape, then its children in field order.
pub fn walk_rex<V: Visitor + ?Sized>(v: &mut V, rex: &Rex) {
    v.visit_shape(rex.ty());
    match rex.op() {
        RexOp::Array { values } | RexOp::Bag { values } => walk_all(v, values),
        RexOp::Call { function, args } => {
            walk_routine(v, function);
            walk_all(v, args);
        }
        RexOp::Case {
            match_,
            branches,
            default,
        } => {
            if let Some(m) = match_ {
                v.visit_rex(m);
            }
            for b in branches {
                v.visit_rex(b.condition());
                v.visit_rex(b.result());
            }
            if let Some(d) = default {
                v.visit_rex(d);
            }
        }
        RexOp::Cast { operand, target } => {
            v.visit_rex(operand);
            v.visit_shape(target);
        }
        RexOp::Coalesce { args } | RexOp::Spread { args } => walk_all(v, args),
        RexOp::Dispatch {
            functions, args, ..
        } => {
            functions.iter().for_each(|f| walk_routine(v, f));
            walk_all(v, args);
        }
        RexOp::Error {}
        | RexOp::Lit { .. }
        | RexOp::Table { .. }
        | RexOp::TableRef { .. }
        | RexOp::Var { .. } => {}
        RexOp::Map {
            key_type,
            value_type,
            entries,
        } => {
            v.visit_shape(key_type);
            v.visit_shape(value_type);
            for e in entries {
                v.visit_rex(e.key());
                v.visit_rex(e.value());
            }
        }
        RexOp::MapDynamic { entries } => {
            for e in entries {
                v.visit_rex(e.key());
                v.visit_rex(e.value());
            }
        }
        RexOp::NullIf { v1, v2 } => {
            v.visit_rex(v1);
            v.visit_rex(v2);
        }
        RexOp::PathIndex { operand, index } => {
            v.visit_rex(operand);
            v.visit_rex(index);
        }
        RexOp::PathKey { operand, key } => {
            v.visit_rex(operand);
            v.visit_rex(key);
        }
        RexOp::PathSymbol { operand, .. } => v.visit_rex(operand),
        RexOp::Pivot { input, key, value } => {
            v.visit_rel(input);
            v.visit_rex(key);
            v.visit_rex(value);
        }
        RexOp::Select { input, constructor }
        | RexOp::Subquery {
            input, constructor, ..
        } => {
            v.visit_rel(input);
            v.visit_rex(constructor);
        }
        RexOp::Struct { fields } => {
            for f in fields {
                v.visit_rex(f.key());
                v.visit_rex(f.value());
            }
        }
        RexOp::SubqueryComp { input, args, .. } | RexOp::SubqueryIn { input, args } => {
            v.visit_rel(input);
            walk_all(v, args);
        }
        RexOp::SubqueryTest { input, .. } => v.visit_rel(input),
    }
}

/// Visits each field shape of `rel`'s schema, then its children in field order.
pub fn walk_rel<V: Visitor + ?Sized>(v: &mut V, rel: &Rel) {
    rel.ty().fields().iter().for_each(|f| v.visit_shape(f.ty()));
    match rel.op() {
        RelOp::Aggregate {
            input,
            measures,
            groups,
        } => {
            v.visit_rel(input);
            for m in measures {
                walk_routine(v, m.agg());
                walk_all(v, m.args());
            }
            walk_all(v, groups);
        }
        RelOp::Correlate { left, right, .. }
        | RelOp::Except { left, right, .. }
        | RelOp::Intersect { left, right, .. }
        | RelOp::Union { left, right, .. } => {
            v.visit_rel(left);
            v.visit_rel(right);
        }
        RelOp::Distinct { input } => v.visit_rel(input),
        RelOp::Exclude { input, exclusions } => {
            v.visit_rel(input);
            exclusions.iter().for_each(|e| v.visit_rex(e.var()));
        }
        RelOp::Filter { input, predicate } => {
            v.visit_rel(input);
            v.visit_rex(predicate);
        }
        RelOp::Iterate { rex } | RelOp::Scan { rex } | RelOp::Unpivot { rex } => v.visit_rex(rex),
        RelOp::Join {
            left,
            right,
            condition,
            ..
        } => {
            v.visit_rel(left);
            v.visit_rel(right);
            v.visit_rex(condition);
        }
        RelOp::Limit { input, limit: rex } | RelOp::Offset { input, offset: rex } => {
            v.visit_rel(input);
            v.visit_rex(rex);
        }
        RelOp::Project { input, projections } => {
            v.visit_rel(input);
            walk_all(v, projections);
        }
        RelOp::Sort { input, collations } => {
            v.visit_rel(input);
            collations.iter().for_each(|c| v.visit_rex(c.column()));
        }
        RelOp::Window {
            input,
            functions,
            collations,
            partitions,
            ..
        } => {
            v.visit_rel(input);
            for f in functions {
                let sig = f.signature();
                sig.params().iter().for_each(|p| v.visit_shape(p));
                v.visit_shape(sig.returns());
                walk_all(v, f.args());
            }
            collations.iter().for_each(|c| v.visit_rex(c.column()));
            walk_all(v, partitions);
        }
        RelOp::With { input, elements } => {
            v.visit_rel(input);
            elements
                .iter()
                .for_each(|e| v.visit_rex(e.representation()));
        }
    }
}
