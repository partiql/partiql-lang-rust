//! The rewriter's defaults rebuild a plan unchanged; a pass changes only what it overrides.
mod common;

use common::*;
use partiql_ir::rewrite::walk_rex as rewrite_rex;
use partiql_ir::visit::walk_rex;
use partiql_ir::{render_shape, Plan, Rewriter, Rex, RexOp, Visitor};
use partiql_types::PartiqlShape;
use partiql_value::Value;

struct Identity;
impl Rewriter for Identity {}

fn assert_identity(plan: Plan) {
    let out = Identity.rewrite_plan(plan.clone());
    assert_eq!(out, plan);
    // `PartialEq` on shapes ignores `TypeParams`; `Debug` and the dump do not.
    assert_eq!(format!("{out:?}"), format!("{plan:?}"));
    assert_eq!(out.dump(), plan.dump());
}

#[test]
fn identity_round_trip_select_from_t() {
    assert_identity(select_from_t());
}

#[test]
fn identity_round_trip_every_node() {
    assert_identity(every_node());
}

/// Collects every attached shape, in walk order.
#[derive(Default)]
struct VisitShapes(Vec<String>);
impl Visitor for VisitShapes {
    fn visit_shape(&mut self, shape: &PartiqlShape) {
        self.0.push(render_shape(shape));
    }
}

#[derive(Default)]
struct RewriteShapes(Vec<String>);
impl Rewriter for RewriteShapes {
    fn rewrite_shape(&mut self, shape: PartiqlShape) -> PartiqlShape {
        self.0.push(render_shape(&shape));
        shape
    }
}

/// The visitor and the rewriter reach the same shape slots in the same order.
#[test]
fn visitor_and_rewriter_see_the_same_shapes() {
    let plan = every_node();
    let mut v = VisitShapes::default();
    v.visit_plan(&plan);
    let mut r = RewriteShapes::default();
    r.rewrite_plan(plan);
    assert_eq!(v.0, r.0);
    assert!(v.0.len() > 100, "{} shapes", v.0.len());
}

/// A pass that overrides one node kind changes only those nodes.
struct BumpInts;
impl Rewriter for BumpInts {
    fn rewrite_rex(&mut self, rex: Rex) -> Rex {
        match rex.op() {
            RexOp::Lit {
                value: Value::Integer(i),
                ..
            } => Rex::lit(rex.ty().clone(), Value::from(i + 100)),
            _ => rewrite_rex(self, rex),
        }
    }
}

#[derive(Default)]
struct Ints(Vec<i64>);
impl Visitor for Ints {
    fn visit_rex(&mut self, rex: &Rex) {
        if let RexOp::Lit {
            value: Value::Integer(i),
            ..
        } = rex.op()
        {
            self.0.push(*i);
        }
        walk_rex(self, rex)
    }
}

#[test]
fn a_pass_changes_only_the_nodes_it_overrides() {
    let plan = every_node();
    let mut before = Ints::default();
    before.visit_plan(&plan);
    let out = BumpInts.rewrite_plan(plan.clone());
    let mut after = Ints::default();
    after.visit_plan(&out);
    assert!(!before.0.is_empty());
    assert_eq!(
        after.0,
        before.0.iter().map(|i| i + 100).collect::<Vec<_>>()
    );
    // Undoing the change gives back the input.
    struct Unbump;
    impl Rewriter for Unbump {
        fn rewrite_rex(&mut self, rex: Rex) -> Rex {
            match rex.op() {
                RexOp::Lit {
                    value: Value::Integer(i),
                    ..
                } => Rex::lit(rex.ty().clone(), Value::from(i - 100)),
                _ => rewrite_rex(self, rex),
            }
        }
    }
    assert_eq!(Unbump.rewrite_plan(out), plan);
}
