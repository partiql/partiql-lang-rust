//! The validator accepts typed-planner output and rejects id'd, union, undefined and
//! non-nullable shapes wherever they are attached or nested.
mod common;

use common::*;
use partiql_ir::{
    validate, Exclusion, ExclusionItem, Plan, Rel, Rewriter, Rex, RoutineRef, ViolationKind,
    Visitor,
};
use partiql_types::{
    PartiqlShape, PartiqlShapeBuilder, Static, StructConstraint, StructField, StructType,
};
use partiql_value::Value;

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
            "action.rex.type.element.fields[1:b]".to_string(),
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
    let cast = Rex::cast(any_of(), int_lit(1), any_of());
    let plan = Plan::query(Rex::select(any_of(), scan, cast));
    assert_eq!(
        kinds(&plan),
        [
            ("action.rex.type".to_string(), ViolationKind::AnyOf),
            (
                "action.rex.input.type.fields[0:t]".to_string(),
                ViolationKind::AnyOf
            ),
            (
                "action.rex.constructor.type".to_string(),
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
        vec![Exclusion::new(
            str_lit("t"),
            vec![ExclusionItem::struct_symbol("a", vec![])],
        )],
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
    let with_id = violations
        .iter()
        .filter(|v| matches!(v.kind(), ViolationKind::ShapeWithId { .. }))
        .count();
    assert_eq!(with_id, count.0);
}

#[test]
fn rejects_shapes_without_a_plk_counterpart() {
    let optional = b().new_struct(StructType::new_row([StructField::new_optional("o", int())]));
    let closed_set = b().new_struct(StructType::new(
        [
            StructConstraint::Open(false),
            StructConstraint::Fields([StructField::new("a", int())].into_iter().collect()),
        ]
        .into_iter()
        .collect(),
    ));
    let open_with_fields = b().new_struct(StructType::new(
        [StructConstraint::Fields(
            [StructField::new("a", int())].into_iter().collect(),
        )]
        .into_iter()
        .collect(),
    ));
    let plan = Plan::query(Rex::bag(
        bag(dynamic()),
        vec![
            Rex::lit(s(Static::Int), Value::from(1)),
            Rex::error(s(Static::Graph())),
            Rex::error(optional),
            Rex::error(closed_set),
            Rex::error(open_with_fields),
        ],
    ));
    assert_eq!(
        kinds(&plan),
        [
            (
                "action.rex.values[0].type".to_string(),
                ViolationKind::NoPlkEquivalent
            ),
            (
                "action.rex.values[1].type".to_string(),
                ViolationKind::NoPlkEquivalent
            ),
            (
                "action.rex.values[2].type.fields[0:o]".to_string(),
                ViolationKind::NoPlkEquivalent
            ),
            (
                "action.rex.values[3].type".to_string(),
                ViolationKind::NoPlkEquivalent
            ),
            (
                "action.rex.values[4].type".to_string(),
                ViolationKind::NoPlkEquivalent
            ),
        ]
    );
}

#[test]
fn rejects_a_cast_or_map_whose_type_disagrees_with_its_declared_types() {
    let map = b().new_map(string(), int()).unwrap();
    let plan = Plan::query(Rex::array(
        dynamic(),
        vec![
            Rex::cast(s(Static::Int64), int_lit(1), string()),
            Rex::map(map.clone(), string(), string(), vec![]),
            Rex::map(map, string(), int(), vec![]),
        ],
    ));
    assert_eq!(
        kinds(&plan),
        [
            (
                "action.rex.values[0].type".to_string(),
                ViolationKind::CastTypeNotTarget
            ),
            (
                "action.rex.values[1].type".to_string(),
                ViolationKind::MapTypeNotDeclared
            ),
        ]
    );
}

/// A row may repeat a field name; the path tells the fields apart by position.
#[test]
fn paths_name_duplicate_fields_by_position() {
    let idd = PartiqlShapeBuilder::default().new_static(Static::Int32);
    let plan = Plan::query(Rex::error(row(&[("a", int()), ("a", idd)])));
    let paths: Vec<String> = kinds(&plan).into_iter().map(|(p, _)| p).collect();
    assert_eq!(paths, ["action.rex.type.fields[1:a]"]);
}

#[test]
fn rejects_an_exclusion_without_items() {
    let scan = || Rel::scan(rel_type(&[("t", dynamic())]), Rex::var(dynamic(), 0, 0));
    let exclude = Rel::exclude(
        rel_type(&[("t", dynamic())]),
        scan(),
        vec![Exclusion::new(Rex::var(dynamic(), 0, 0), vec![])],
    );
    let plan = Plan::query(Rex::select(
        bag(dynamic()),
        exclude,
        Rex::var(dynamic(), 0, 0),
    ));
    assert_eq!(
        kinds(&plan),
        [(
            "action.rex.input.exclusions[0]".to_string(),
            ViolationKind::ExclusionWithoutItems
        )]
    );
}

/// PLK names window functions after the last fields of the window's input, possibly fewer
/// names than functions (`PlanTransform.kt:280`); such a window is valid.
#[test]
fn accepts_window_bindings_as_plk_derives_them() {
    let scan = Rel::scan(rel_type(&[("t", dynamic())]), Rex::var(dynamic(), 0, 0));
    let rank = || {
        partiql_ir::WindowFunction::new(
            partiql_ir::WindowSignature::new("rank", vec![], int(), false),
            vec![],
        )
    };
    let window = Rel::window(
        rel_type(&[("t", dynamic()), ("r1", int()), ("r2", int())]),
        None,
        scan,
        vec!["t".to_string()],
        vec![rank(), rank()],
        vec![],
        vec![],
    );
    let plan = Plan::query(Rex::select(
        bag(dynamic()),
        window,
        Rex::var(dynamic(), 0, 0),
    ));
    assert_eq!(validate(&plan), Ok(()));
}

#[test]
fn cast_check_sees_unspecified_flags_and_ignores_ids() {
    use partiql_types::TypeParams;
    let varchar = b().new_static_with_params(
        Static::StringVarying(1),
        TypeParams::default().with_unspecified_length(true),
    );
    let varchar_1 = s(Static::StringVarying(1));
    let plan = Plan::query(Rex::cast(varchar_1, str_lit("x"), varchar));
    assert_eq!(
        kinds(&plan),
        [(
            "action.rex.type".to_string(),
            ViolationKind::CastTypeNotTarget
        )]
    );
    // Equal kinds with node ids: only the ids are reported.
    let mut ids = PartiqlShapeBuilder::default();
    let plan = Plan::query(Rex::cast(
        ids.new_static(Static::Int64),
        int_lit(1),
        ids.new_static(Static::Int64),
    ));
    assert!(kinds(&plan)
        .iter()
        .all(|(_, k)| matches!(k, ViolationKind::ShapeWithId { .. })));
}

#[test]
fn rejects_a_dynamic_map_key() {
    let plan = Plan::query(Rex::map(dynamic(), dynamic(), int(), vec![]));
    assert_eq!(
        kinds(&plan),
        [(
            "action.rex.key_type".to_string(),
            ViolationKind::DynamicMapKey
        )]
    );
}

/// Shapes that are `==` but built differently (constraint order, variant encoding) agree.
#[test]
fn cast_check_has_no_false_positive_for_equal_shapes() {
    use partiql_types::VariantType;
    let ion = s(Static::Variant(VariantType::new("ion")));
    let json = s(Static::Variant(VariantType::new("json")));
    let rows = |a: &str, b: &str| row(&[(a, int()), (b, string())]);
    let plan = Plan::query(Rex::array(
        dynamic(),
        vec![
            Rex::cast(ion, str_lit("x"), json),
            Rex::cast(rows("a", "b"), str_lit("x"), rows("a", "b")),
        ],
    ));
    assert_eq!(validate(&plan), Ok(()));
}

/// Only `StructType::new_any()` stands for PLK's `STRUCT`.
#[test]
fn only_the_canonical_open_struct_is_accepted() {
    let open = b().new_struct(StructType::new(
        [StructConstraint::Open(true)].into_iter().collect(),
    ));
    let any = b().new_struct(StructType::new_any());
    let plan = Plan::query(Rex::array(
        dynamic(),
        vec![Rex::error(open), Rex::error(any)],
    ));
    assert_eq!(
        kinds(&plan),
        [(
            "action.rex.values[0].type".to_string(),
            ViolationKind::NoPlkEquivalent
        )]
    );
}

/// A closed, ordered struct with extra constraints is not a PLK ROW.
#[test]
fn rejects_a_row_with_extra_constraints() {
    let mixed = b().new_struct(StructType::new(
        [
            StructConstraint::Open(false),
            StructConstraint::Ordered(true),
            StructConstraint::FieldList(vec![StructField::new("a", int())]),
            StructConstraint::Fields([StructField::new("b", string())].into_iter().collect()),
        ]
        .into_iter()
        .collect(),
    ));
    let plan = Plan::query(Rex::error(mixed));
    assert!(kinds(&plan).contains(&(
        "action.rex.type".to_string(),
        ViolationKind::NoPlkEquivalent
    )));
}

/// Members of a union are checked too.
#[test]
fn reports_problems_inside_any_of_members() {
    let mut ids = PartiqlShapeBuilder::default();
    let (x, y) = (
        ids.new_static(Static::Int32),
        ids.new_static(Static::String),
    );
    let u = ids.any_of([x, y]);
    let kinds = kinds(&Plan::query(Rex::error(u)));
    assert_eq!(
        kinds[0],
        ("action.rex.type".to_string(), ViolationKind::AnyOf)
    );
    assert!(kinds[1..]
        .iter()
        .all(|(p, k)| p.starts_with("action.rex.type.any_of[")
            && matches!(k, ViolationKind::ShapeWithId { .. })));
    assert_eq!(kinds.len(), 3);
}
