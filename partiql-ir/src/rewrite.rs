//! A fold-style rewriter: builds a new plan from an old one.

use crate::plan::{Action, Plan};
use crate::refs::RoutineRef;
use crate::rel::{
    Collation, Exclusion, Measure, Rel, RelField, RelOp, RelType, WindowFunction, WindowSignature,
    WithElement,
};
use crate::rex::{CaseBranch, MapEntry, Rex, RexOp, StructEntry};
use partiql_types::PartiqlShape;

/// Rewrites a plan by value.
///
/// Each method's default calls the matching `walk_*` function, which rewrites the node's
/// children (in field order) and rebuilds the node from them, keeping every other field. So a
/// pass overrides only the nodes it changes; the defaults alone return a plan equal to the
/// input. Types are not recomputed: a pass that changes a node's value changes its type too.
pub trait Rewriter {
    fn rewrite_plan(&mut self, plan: Plan) -> Plan {
        walk_plan(self, plan)
    }

    fn rewrite_rex(&mut self, rex: Rex) -> Rex {
        walk_rex(self, rex)
    }

    fn rewrite_rel(&mut self, rel: Rel) -> Rel {
        walk_rel(self, rel)
    }

    /// Called for every shape attached to the plan (as [`crate::Visitor::visit_shape`]).
    fn rewrite_shape(&mut self, shape: PartiqlShape) -> PartiqlShape {
        shape
    }
}

pub fn walk_plan<R: Rewriter + ?Sized>(r: &mut R, plan: Plan) -> Plan {
    let (version, action) = plan.into_parts();
    let action = match action {
        Action::Query { rex } => Action::Query {
            rex: r.rewrite_rex(rex),
        },
    };
    Plan::new(version, action)
}

fn all<R: Rewriter + ?Sized>(r: &mut R, rexes: Vec<Rex>) -> Vec<Rex> {
    rexes.into_iter().map(|x| r.rewrite_rex(x)).collect()
}

fn boxed<R: Rewriter + ?Sized>(r: &mut R, rex: Box<Rex>) -> Box<Rex> {
    Box::new(r.rewrite_rex(*rex))
}

fn boxed_rel<R: Rewriter + ?Sized>(r: &mut R, rel: Box<Rel>) -> Box<Rel> {
    Box::new(r.rewrite_rel(*rel))
}

fn shapes<R: Rewriter + ?Sized>(r: &mut R, shapes: Vec<PartiqlShape>) -> Vec<PartiqlShape> {
    shapes.into_iter().map(|s| r.rewrite_shape(s)).collect()
}

fn routine<R: Rewriter + ?Sized>(r: &mut R, routine: RoutineRef) -> RoutineRef {
    let (name, specific, params, returns) = routine.into_parts();
    let params = shapes(r, params);
    RoutineRef::new(name, specific, params, r.rewrite_shape(returns))
}

fn map_entries<R: Rewriter + ?Sized>(r: &mut R, entries: Vec<MapEntry>) -> Vec<MapEntry> {
    entries
        .into_iter()
        .map(|e| {
            let (k, v) = e.into_parts();
            MapEntry::new(r.rewrite_rex(k), r.rewrite_rex(v))
        })
        .collect()
}

fn collations<R: Rewriter + ?Sized>(r: &mut R, collations: Vec<Collation>) -> Vec<Collation> {
    collations
        .into_iter()
        .map(|c| {
            let (column, order, nulls) = c.into_parts();
            Collation::new(r.rewrite_rex(column), order, nulls)
        })
        .collect()
}

/// Rewrites `rex`'s shape and children, then rebuilds it.
pub fn walk_rex<R: Rewriter + ?Sized>(r: &mut R, rex: Rex) -> Rex {
    let (ty, op) = rex.into_parts();
    let ty = r.rewrite_shape(ty);
    let op = match op {
        RexOp::Array { values } => RexOp::Array {
            values: all(r, values),
        },
        RexOp::Bag { values } => RexOp::Bag {
            values: all(r, values),
        },
        RexOp::Call { function, args } => RexOp::Call {
            function: routine(r, function),
            args: all(r, args),
        },
        RexOp::Case {
            match_,
            branches,
            default,
        } => RexOp::Case {
            match_: match_.map(|m| boxed(r, m)),
            branches: branches
                .into_iter()
                .map(|b| {
                    let (c, v) = b.into_parts();
                    CaseBranch::new(r.rewrite_rex(c), r.rewrite_rex(v))
                })
                .collect(),
            default: default.map(|d| boxed(r, d)),
        },
        RexOp::Cast { operand, target } => RexOp::Cast {
            operand: boxed(r, operand),
            target: r.rewrite_shape(target),
        },
        RexOp::Coalesce { args } => RexOp::Coalesce { args: all(r, args) },
        RexOp::Dispatch {
            name,
            functions,
            args,
        } => RexOp::Dispatch {
            name,
            functions: functions.into_iter().map(|f| routine(r, f)).collect(),
            args: all(r, args),
        },
        op @ (RexOp::Error {}
        | RexOp::Lit { .. }
        | RexOp::Table { .. }
        | RexOp::TableRef { .. }
        | RexOp::Var { .. }) => op,
        RexOp::Map {
            key_type,
            value_type,
            entries,
        } => RexOp::Map {
            key_type: r.rewrite_shape(key_type),
            value_type: r.rewrite_shape(value_type),
            entries: map_entries(r, entries),
        },
        RexOp::MapDynamic { entries } => RexOp::MapDynamic {
            entries: map_entries(r, entries),
        },
        RexOp::NullIf { v1, v2 } => RexOp::NullIf {
            v1: boxed(r, v1),
            v2: boxed(r, v2),
        },
        RexOp::PathIndex { operand, index } => RexOp::PathIndex {
            operand: boxed(r, operand),
            index: boxed(r, index),
        },
        RexOp::PathKey { operand, key } => RexOp::PathKey {
            operand: boxed(r, operand),
            key: boxed(r, key),
        },
        RexOp::PathSymbol { operand, symbol } => RexOp::PathSymbol {
            operand: boxed(r, operand),
            symbol,
        },
        RexOp::Pivot { input, key, value } => RexOp::Pivot {
            input: boxed_rel(r, input),
            key: boxed(r, key),
            value: boxed(r, value),
        },
        RexOp::Select { input, constructor } => RexOp::Select {
            input: boxed_rel(r, input),
            constructor: boxed(r, constructor),
        },
        RexOp::Spread { args } => RexOp::Spread { args: all(r, args) },
        RexOp::Struct { fields } => RexOp::Struct {
            fields: fields
                .into_iter()
                .map(|f| {
                    let (k, v) = f.into_parts();
                    StructEntry::new(r.rewrite_rex(k), r.rewrite_rex(v))
                })
                .collect(),
        },
        RexOp::Subquery {
            input,
            constructor,
            scalar,
        } => RexOp::Subquery {
            input: boxed_rel(r, input),
            constructor: boxed(r, constructor),
            scalar,
        },
        RexOp::SubqueryComp {
            input,
            args,
            comparison,
            quantifier,
        } => RexOp::SubqueryComp {
            input: boxed_rel(r, input),
            args: all(r, args),
            comparison,
            quantifier,
        },
        RexOp::SubqueryIn { input, args } => RexOp::SubqueryIn {
            input: boxed_rel(r, input),
            args: all(r, args),
        },
        RexOp::SubqueryTest { input, test } => RexOp::SubqueryTest {
            input: boxed_rel(r, input),
            test,
        },
    };
    Rex::from_parts(ty, op)
}

/// Rewrites `rel`'s schema shapes and children, then rebuilds it.
pub fn walk_rel<R: Rewriter + ?Sized>(r: &mut R, rel: Rel) -> Rel {
    let (ty, op) = rel.into_parts();
    let (fields, ordered) = ty.into_parts();
    let fields = fields
        .into_iter()
        .map(|f| {
            let (name, ty) = f.into_parts();
            RelField::new(name, r.rewrite_shape(ty))
        })
        .collect();
    let ty = RelType::new(fields, ordered);
    let op = match op {
        RelOp::Aggregate {
            input,
            measures,
            groups,
        } => RelOp::Aggregate {
            input: boxed_rel(r, input),
            measures: measures
                .into_iter()
                .map(|m| {
                    let (agg, args, distinct) = m.into_parts();
                    Measure::new(routine(r, agg), all(r, args), distinct)
                })
                .collect(),
            groups: all(r, groups),
        },
        RelOp::Correlate {
            left,
            right,
            join_type,
        } => RelOp::Correlate {
            left: boxed_rel(r, left),
            right: boxed_rel(r, right),
            join_type,
        },
        RelOp::Distinct { input } => RelOp::Distinct {
            input: boxed_rel(r, input),
        },
        RelOp::Except { left, right, all } => RelOp::Except {
            left: boxed_rel(r, left),
            right: boxed_rel(r, right),
            all,
        },
        RelOp::Exclude { input, exclusions } => RelOp::Exclude {
            input: boxed_rel(r, input),
            exclusions: exclusions
                .into_iter()
                .map(|e| {
                    let (var, items) = e.into_parts();
                    Exclusion::new(r.rewrite_rex(var), items)
                })
                .collect(),
        },
        RelOp::Filter { input, predicate } => RelOp::Filter {
            input: boxed_rel(r, input),
            predicate: r.rewrite_rex(predicate),
        },
        RelOp::Intersect { left, right, all } => RelOp::Intersect {
            left: boxed_rel(r, left),
            right: boxed_rel(r, right),
            all,
        },
        RelOp::Iterate { rex } => RelOp::Iterate {
            rex: r.rewrite_rex(rex),
        },
        RelOp::Join {
            left,
            right,
            condition,
            join_type,
        } => RelOp::Join {
            left: boxed_rel(r, left),
            right: boxed_rel(r, right),
            condition: r.rewrite_rex(condition),
            join_type,
        },
        RelOp::Limit { input, limit } => RelOp::Limit {
            input: boxed_rel(r, input),
            limit: r.rewrite_rex(limit),
        },
        RelOp::Offset { input, offset } => RelOp::Offset {
            input: boxed_rel(r, input),
            offset: r.rewrite_rex(offset),
        },
        RelOp::Project { input, projections } => RelOp::Project {
            input: boxed_rel(r, input),
            projections: all(r, projections),
        },
        RelOp::Scan { rex } => RelOp::Scan {
            rex: r.rewrite_rex(rex),
        },
        RelOp::Sort {
            input,
            collations: c,
        } => RelOp::Sort {
            input: boxed_rel(r, input),
            collations: collations(r, c),
        },
        RelOp::Union { left, right, all } => RelOp::Union {
            left: boxed_rel(r, left),
            right: boxed_rel(r, right),
            all,
        },
        RelOp::Unpivot { rex } => RelOp::Unpivot {
            rex: r.rewrite_rex(rex),
        },
        RelOp::Window {
            input,
            name,
            bindings,
            functions,
            collations: c,
            partitions,
        } => RelOp::Window {
            input: boxed_rel(r, input),
            name,
            bindings,
            functions: functions
                .into_iter()
                .map(|f| {
                    let (sig, args) = f.into_parts();
                    let (name, params, returns, ignore_nulls) = sig.into_parts();
                    let params = shapes(r, params);
                    let returns = r.rewrite_shape(returns);
                    let sig = WindowSignature::new(name, params, returns, ignore_nulls);
                    WindowFunction::new(sig, all(r, args))
                })
                .collect(),
            collations: collations(r, c),
            partitions: all(r, partitions),
        },
        RelOp::With { input, elements } => RelOp::With {
            input: boxed_rel(r, input),
            elements: elements
                .into_iter()
                .map(|e| {
                    let (name, rep) = e.into_parts();
                    WithElement::new(name, r.rewrite_rex(rep))
                })
                .collect(),
        },
    };
    Rel::from_parts(ty, op)
}
