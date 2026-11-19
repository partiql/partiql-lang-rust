//! The validator accepts typed-planner output and rejects id'd, union, undefined and
//! non-nullable shapes wherever they are attached or nested.
mod common;

use common::*;
use partiql_ir::{
    validate, Exclusion, Plan, Rel, Rewriter, Rex, RoutineRef, ViolationKind, Visitor,
};
use partiql_types::{PartiqlShape, PartiqlShapeBuilder, Static};

fn kinds(plan: &Plan) -> Vec<(String, ViolationKind)> {
    validate(plan)
        .expect_err("violations")
        .into_iter()
        .map(|v| (v.path().to_string(), v.kind().clone()))
        .collect()
}

#[test]
fn accepts_hand_built_plans() {
    assert_eq!(validate(&select_from_t()), Ok(()));
    assert_eq!(validate(&every_node()), Ok(()));
}

#[test]
fn rejects_a_shape_with_a_node_id() {
    let ty = PartiqlShapeBuilder::default().new_static(Static::Int32);
    let id = ty.static_type_id().expect("static").0;
    assert_ne!(id, 0);
    let plan = Plan::query(Rex::var(ty, 0, 0));
    assert_eq!(
        kinds(&plan),
        [(
            "action.rex.type".to_string(),
            ViolationKind::ShapeWithId { id }
        )]
    );
}

#[test]
fn rejects_an_id_nested_in_a_row_in_a_bag() {
    let idd = PartiqlShapeBuilder::default().new_static(Static::String);
    let id = idd.static_type_id().unwrap().0;
    let ty = bag(row(&[("a", int()), ("b", idd)]));
    let plan = Plan::query(Rex::bag(ty, vec![]));
    assert_eq!(
        kinds(&plan),
        [(
            "action.rex.type.element.fields[b]".to_string(),
            ViolationKind::ShapeWithId { id }
        )]
    );
}

fn any_of() -> PartiqlShape {
    let u = b().any_of([int(), string()]);
    assert!(matches!(u, PartiqlShape::AnyOf(_)));
    u
}

#[test]
fn rejects_any_of_on_a_rex_a_rel_field_and_a_cast_target() {
    let scan = Rel::scan(rel_type(&[("t", any_of())]), Rex::var(dynamic(), 0, 0));
    let cast = Rex::cast(dynamic(), int_lit(1), any_of());
    let plan = Plan::query(Rex::select(any_of(), scan, cast));
    assert_eq!(
        kinds(&plan),
        [
            ("action.rex.type".to_string(), ViolationKind::AnyOf),
            (
                "action.rex.input.type.fields[t]".to_string(),
                ViolationKind::AnyOf
            ),
            (
                "action.rex.constructor.target".to_string(),
                ViolationKind::AnyOf
            ),
        ]
    );
}

#[test]
fn rejects_any_of_in_a_routine_signature_and_a_map_value() {
    let f = RoutineRef::new(
        partiql_ir::QualifiedName::new("$system", Vec::<String>::new(), "f"),
        "f__1",
        vec![int(), any_of()],
        int(),
    );
    let map = b().new_map(string(), any_of()).unwrap();
    let plan = Plan::query(Rex::call(map, f, vec![]));
    assert_eq!(
        kinds(&plan),
        [
            ("action.rex.type.value".to_string(), ViolationKind::AnyOf),
            (
                "action.rex.function.params[1]".to_string(),
                ViolationKind::AnyOf
            ),
        ]
    );
}

#[test]
fn rejects_undefined_and_non_nullable_shapes() {
    let not_null = b().new_non_nullable_static(Static::Int32);
    let plan = Plan::query(Rex::array(
        b().new_array_of(not_null),
        vec![Rex::error(PartiqlShape::Undefined)],
    ));
    assert_eq!(
        kinds(&plan),
        [
            (
                "action.rex.type.element".to_string(),
                ViolationKind::NotNullable
            ),
            (
                "action.rex.values[0].type".to_string(),
                ViolationKind::Undefined
            ),
        ]
    );
}

#[test]
fn rejects_an_exclusion_whose_variable_is_not_a_var() {
    let scan = || Rel::scan(rel_type(&[("t", dynamic())]), Rex::var(dynamic(), 0, 0));
    let exclude = Rel::exclude(
        rel_type(&[("t", dynamic())]),
        scan(),
        vec![Exclusion::new(str_lit("t"), vec![])],
    );
    let plan = Plan::query(Rex::select(
        bag(dynamic()),
        exclude,
        Rex::var(dynamic(), 0, 0),
    ));
    assert_eq!(
        kinds(&plan),
        [(
            "action.rex.input.exclusions[0].var".to_string(),
            ViolationKind::ExclusionVarNotVar
        )]
    );
}

/// Every shape slot the visitor reaches is checked: replacing each attached shape by an id'd
/// one yields one violation per slot.
#[test]
fn checks_every_attached_shape() {
    struct Count(usize);
    impl Visitor for Count {
        fn visit_shape(&mut self, _: &PartiqlShape) {
            self.0 += 1;
        }
    }
    struct WithIds(PartiqlShapeBuilder);
    impl Rewriter for WithIds {
        fn rewrite_shape(&mut self, _: PartiqlShape) -> PartiqlShape {
            self.0.new_static(Static::Bool)
        }
    }
    let plan = every_node();
    let mut count = Count(0);
    count.visit_plan(&plan);
    let bad = WithIds(PartiqlShapeBuilder::default()).rewrite_plan(plan);
    let violations = validate(&bad).expect_err("violations");
    assert_eq!(violations.len(), count.0);
    assert!(violations
        .iter()
        .all(|v| matches!(v.kind(), ViolationKind::ShapeWithId { .. })));
}
