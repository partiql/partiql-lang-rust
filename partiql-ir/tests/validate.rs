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
fn rejects_non_nullable_shapes() {
    let not_null = b().new_non_nullable_static(Static::Int32);
    let plan = Plan::query(Rex::array(b().new_array_of(not_null), vec![]));
    assert_eq!(
        kinds(&plan),
        [(
            "action.rex.type.element".to_string(),
            ViolationKind::NotNullable
        )]
    );
}

/// `Undefined` is the bottom type: the type of absent-value literals and of what can only hold
/// them, also nested, in schemas and in resolved signatures.
#[test]
fn accepts_undefined_where_absent_values_flow() {
    use partiql_ir::{QualifiedName, RoutineRef};
    use partiql_value::Value;
    let undef = || PartiqlShape::Undefined;
    let upper = RoutineRef::new(
        QualifiedName::new("$system", Vec::<String>::new(), "upper"),
        "upper__Undefined",
        vec![undef()],
        string(),
    );
    let a = row(&[("a", undef())]);
    let scan = Rel::scan(rel_type(&[("t", a.clone())]), Rex::var(a.clone(), 0, 0));
    let projection = Rex::bag(
        bag(undef()),
        vec![
            Rex::lit(undef(), Value::Null),
            Rex::lit(undef(), Value::Missing),
            Rex::path_key(undef(), Rex::var(a.clone(), 0, 0), str_lit("a")),
            Rex::call(string(), upper, vec![Rex::lit(undef(), Value::Null)]),
        ],
    );
    let plan = Plan::query(Rex::select(bag(bag(undef())), scan, projection));
    assert_eq!(validate(&plan), Ok(()));
}

/// What PLK produces around absent values: a `NULL` typed as the `CASE`/`COALESCE` supertype
/// (`DynamicTyper.kt:107`), and a coercion cast whose target nests `UNKNOWN` (`[NULL]` in a set
/// operation).
#[test]
fn accepts_plk_typed_absent_literals_and_nested_undefined_cast_targets() {
    use partiql_value::Value;
    let undef = || PartiqlShape::Undefined;
    let arr = || b().new_array_of(undef());
    let plan = Plan::query(Rex::coalesce(
        int(),
        vec![
            int_lit(1),
            Rex::lit(int(), Value::Null),
            Rex::cast(arr(), Rex::lit(undef(), Value::Null), arr()),
        ],
    ));
    assert_eq!(validate(&plan), Ok(()));
}

#[test]
fn rejects_undefined_where_no_absent_value_can_be() {
    use partiql_value::Value;
    let undef = || PartiqlShape::Undefined;
    let plan = Plan::query(Rex::array(
        dynamic(),
        vec![
            Rex::lit(undef(), Value::from(1)),
            Rex::from_parts(undef(), partiql_ir::Rex::error().into_parts().1),
            Rex::cast(undef(), int_lit(1), undef()),
            Rex::bag(undef(), vec![int_lit(1)]),
            Rex::struct_(undef(), vec![]),
            // A cast whose target nests Undefined still checks its type against the target.
            Rex::cast(int(), int_lit(1), b().new_array_of(undef())),
        ],
    ));
    assert_eq!(
        kinds(&plan),
        [
            (
                "action.rex.values[0].type".to_string(),
                ViolationKind::Undefined
            ),
            (
                "action.rex.values[1].type".to_string(),
                ViolationKind::ErrorNotDynamic
            ),
            (
                "action.rex.values[2].target".to_string(),
                ViolationKind::Undefined
            ),
            (
                "action.rex.values[3].type".to_string(),
                ViolationKind::Undefined
            ),
            (
                "action.rex.values[4].type".to_string(),
                ViolationKind::Undefined
            ),
            (
                "action.rex.values[5].type".to_string(),
                ViolationKind::CastTypeNotTarget
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
            Rex::var(s(Static::Graph()), 0, 0),
            Rex::var(optional, 0, 0),
            Rex::var(closed_set, 0, 0),
            Rex::var(open_with_fields, 0, 0),
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
    let plan = Plan::query(Rex::var(row(&[("a", int()), ("a", idd)]), 0, 0));
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

/// Shapes that are `==` but built differently (constraint insertion order) agree; a variant
/// cast to another encoding does not (the encoding takes part in equality).
#[test]
fn cast_check_compares_shapes_not_their_construction() {
    use partiql_types::VariantType;
    let fields = || vec![StructField::new("a", int())];
    let row_fwd = b().new_struct(StructType::new_row(fields()));
    let row_rev = b().new_struct(StructType::new(
        [
            StructConstraint::FieldList(fields()),
            StructConstraint::DuplicateAttrs(true),
            StructConstraint::Ordered(true),
            StructConstraint::Open(false),
        ]
        .into_iter()
        .collect(),
    ));
    assert_ne!(format!("{row_fwd:?}"), format!("{row_rev:?}"));
    let ion = || s(Static::Variant(VariantType::new("ion")));
    let json = s(Static::Variant(VariantType::new("json")));
    let plan = Plan::query(Rex::array(
        dynamic(),
        vec![
            Rex::cast(row_fwd, str_lit("x"), row_rev),
            Rex::cast(ion(), str_lit("x"), ion()),
            Rex::cast(ion(), str_lit("x"), json),
        ],
    ));
    assert_eq!(
        kinds(&plan),
        [(
            "action.rex.values[2].type".to_string(),
            ViolationKind::CastTypeNotTarget
        )]
    );
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
        vec![Rex::var(open, 0, 0), Rex::var(any, 0, 0)],
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
    let plan = Plan::query(Rex::var(mixed, 0, 0));
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
    let kinds = kinds(&Plan::query(Rex::var(u, 0, 0)));
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

/// Every node that always produces a present value rejects `Undefined`, and its other checks
/// still run (a map's DYNAMIC key is still reported).
#[test]
fn present_value_nodes_reject_undefined() {
    use partiql_ir::{QualifiedName, SubqueryTest};
    let u = || PartiqlShape::Undefined;
    let scan = || Rel::scan(rel_type(&[("t", dynamic())]), Rex::var(dynamic(), 0, 0));
    let nodes = vec![
        Rex::array(u(), vec![]),
        Rex::bag(u(), vec![]),
        Rex::struct_(u(), vec![]),
        Rex::map(u(), dynamic(), int(), vec![]),
        Rex::map_dynamic(u(), vec![]),
        Rex::spread(u(), vec![]),
        Rex::select(u(), scan(), Rex::var(dynamic(), 0, 0)),
        Rex::pivot(u(), scan(), str_lit("k"), Rex::var(dynamic(), 0, 0)),
        Rex::table(u(), QualifiedName::new("c", Vec::<String>::new(), "t")),
        Rex::table_ref(u(), 0, 0),
        Rex::subquery_test(u(), scan(), SubqueryTest::Exists),
        Rex::subquery(u(), scan(), Rex::var(dynamic(), 0, 0), false),
    ];
    let n = nodes.len();
    let got = kinds(&Plan::query(Rex::array(dynamic(), nodes)));
    let undefined: Vec<String> = got
        .iter()
        .filter(|(_, k)| *k == ViolationKind::Undefined)
        .map(|(p, _)| p.clone())
        .collect();
    let want: Vec<String> = (0..n)
        .map(|i| format!("action.rex.values[{i}].type"))
        .collect();
    assert_eq!(undefined, want);
    assert!(got.contains(&(
        "action.rex.values[3].key_type".to_string(),
        ViolationKind::DynamicMapKey
    )));
}

/// Nodes PLK legitimately types `UNKNOWN`, or whose parts it types `UNKNOWN`, are accepted.
#[test]
fn accepts_plk_legal_undefined_placements() {
    use partiql_value::Value;
    let u = || PartiqlShape::Undefined;
    let null = || Rex::lit(u(), Value::Null);
    let scan = || Rel::scan(rel_type(&[("t", dynamic())]), Rex::var(dynamic(), 0, 0));
    let nodes = vec![
        // A NULL map value: MAP<STRING, UNKNOWN>.
        Rex::map(
            b().new_map(string(), u()).unwrap(),
            string(),
            u(),
            vec![partiql_ir::MapEntry::new(str_lit("k"), null())],
        ),
        Rex::case(
            u(),
            None,
            vec![partiql_ir::CaseBranch::new(
                Rex::lit(bool_(), Value::from(true)),
                null(),
            )],
            Some(null()),
        ),
        Rex::coalesce(u(), vec![null(), null()]),
        Rex::null_if(u(), null(), null()),
        // A MISSING branch typed as the CASE supertype (`DynamicTyper.kt:108`).
        Rex::case(
            int(),
            None,
            vec![partiql_ir::CaseBranch::new(
                Rex::lit(bool_(), Value::from(true)),
                Rex::lit(int(), Value::Missing),
            )],
            Some(Rex::lit(int(), Value::Null)),
        ),
        // A scalar subquery over SELECT NULL.
        Rex::subquery(u(), scan(), null(), true),
    ];
    let plan = Plan::query(Rex::array(dynamic(), nodes));
    assert_eq!(validate(&plan), Ok(()));
}

/// partiql-types follows the MAP type RFC: a key that is or contains `Undefined` (NULL and
/// MISSING keys) has no map type, so a `RexMap` declaring one is rejected (PLK would type
/// `MAP{NULL: 1}` as `MAP<UNKNOWN, INT>`).
#[test]
fn rejects_a_map_with_an_undefined_key() {
    use partiql_value::Value;
    let u = || PartiqlShape::Undefined;
    let plan = Plan::query(Rex::map(
        dynamic(),
        u(),
        int(),
        vec![partiql_ir::MapEntry::new(
            Rex::lit(u(), Value::Null),
            int_lit(1),
        )],
    ));
    assert_eq!(
        kinds(&plan),
        [(
            "action.rex.key_type".to_string(),
            ViolationKind::UndefinedMapKey
        )]
    );
}

/// PLK produces absent literals with a concrete type in several places, so they are accepted
/// anywhere: e.g. the INT length NULL of `x IS VARCHAR` (`RexConverter.kt:1392`), and a CASE
/// folded into its first branch, leaving a retyped NULL with no CASE parent
/// (`PlanTyper.kt:1296`).
#[test]
fn accepts_typed_absent_literals_wherever_plk_makes_them() {
    use partiql_ir::{QualifiedName, RoutineRef};
    use partiql_value::Value;
    let is_varchar = RoutineRef::new(
        QualifiedName::new("$system", Vec::<String>::new(), "is_varchar"),
        "is_varchar__Int32_String",
        vec![int(), string()],
        bool_(),
    );
    let plan = Plan::query(Rex::array(
        dynamic(),
        vec![
            Rex::call(
                bool_(),
                is_varchar,
                vec![Rex::lit(int(), Value::Null), str_lit("x")],
            ),
            Rex::lit(int(), Value::Null),
        ],
    ));
    assert_eq!(validate(&plan), Ok(()));
}

fn chain(depth: usize) -> Plan {
    // `depth` nested Rex nodes: depth - 1 path symbols over a variable.
    let mut rex = Rex::var(dynamic(), 0, 0);
    for _ in 1..depth {
        rex = Rex::path_symbol(dynamic(), rex, "a");
    }
    Plan::query(rex)
}

/// A plan at the depth limit validates and prints on a test thread's stack; a deeper one is
/// rejected without recursing (even far deeper, which would otherwise overflow).
#[test]
fn depth_is_bounded() {
    use partiql_ir::MAX_DEPTH;
    let at = chain(MAX_DEPTH);
    assert_eq!(validate(&at), Ok(()));
    assert!(at.dump().lines().count() > MAX_DEPTH);
    let over = kinds(&chain(MAX_DEPTH + 1));
    assert_eq!(
        over,
        [(
            "action".to_string(),
            ViolationKind::TooDeep { max: MAX_DEPTH }
        )]
    );
    // A caller with more stack can raise the limit.
    assert_eq!(
        partiql_ir::validate_with_max_depth(&chain(MAX_DEPTH + 1), MAX_DEPTH + 1),
        Ok(())
    );
    // A very deep plan, validated and printed on a small (1 MiB) stack: both stop at the limit
    // instead of recursing. The plan is leaked, because dropping it recurses.
    std::thread::Builder::new()
        .stack_size(1 << 20)
        .spawn(|| {
            let deep = std::mem::ManuallyDrop::new(chain(200_000));
            assert!(matches!(
                validate(&deep).unwrap_err()[0].kind(),
                ViolationKind::TooDeep { .. }
            ));
            assert!(deep.dump().starts_with("<plan nested deeper than"));
        })
        .unwrap()
        .join()
        .unwrap();
}

/// Nested shapes and EXCLUDE paths count toward the depth limit, so a plan with few nodes but a
/// very deep type or exclusion path is rejected instead of overflowing the shape and path checks.
#[test]
fn deep_shapes_and_exclusion_paths_are_bounded() {
    use partiql_ir::{ExclusionItem, MAX_DEPTH};
    std::thread::Builder::new()
        .stack_size(1 << 20)
        .spawn(|| {
            let mut ty = int();
            for _ in 0..100_000 {
                ty = b().new_array_of(ty);
            }
            let plan = std::mem::ManuallyDrop::new(Plan::query(Rex::var(ty, 0, 0)));
            assert!(matches!(
                validate(&plan).unwrap_err()[0].kind(),
                ViolationKind::TooDeep { .. }
            ));
            let mut items = vec![];
            for _ in 0..100_000 {
                items = vec![ExclusionItem::struct_symbol("a", items)];
            }
            let scan = Rel::scan(rel_type(&[("t", dynamic())]), Rex::var(dynamic(), 0, 0));
            let exclude = Rel::exclude(
                rel_type(&[("t", dynamic())]),
                scan,
                vec![Exclusion::new(Rex::var(dynamic(), 0, 0), items)],
            );
            let plan = std::mem::ManuallyDrop::new(Plan::query(Rex::select(
                bag(dynamic()),
                exclude,
                Rex::var(dynamic(), 0, 0),
            )));
            assert!(matches!(
                validate(&plan).unwrap_err()[0].kind(),
                ViolationKind::TooDeep { .. }
            ));
        })
        .unwrap()
        .join()
        .unwrap();
    // A shape nested within the limit is fine.
    let mut ty = int();
    for _ in 0..(MAX_DEPTH - 2) {
        ty = b().new_array_of(ty);
    }
    assert_eq!(validate(&Plan::query(Rex::var(ty, 0, 0))), Ok(()));
}

/// Literal values nest too: a deeply nested list literal is rejected by `validate`, and `dump`
/// prints the notice instead of recursing into it (on a 1 MiB stack; the plan is leaked because
/// dropping it recurses).
#[test]
fn deep_literal_values_are_bounded() {
    use partiql_ir::MAX_DEPTH;
    use partiql_value::{List, Value};
    std::thread::Builder::new()
        .stack_size(1 << 20)
        .spawn(|| {
            let mut v = Value::from(1);
            for _ in 0..100_000 {
                v = Value::from(List::from(vec![v]));
            }
            let plan = std::mem::ManuallyDrop::new(Plan::query(Rex::lit(dynamic(), v)));
            assert!(matches!(
                validate(&plan).unwrap_err()[0].kind(),
                ViolationKind::TooDeep { .. }
            ));
            assert!(plan.dump().starts_with("<plan nested deeper than"));
        })
        .unwrap()
        .join()
        .unwrap();
    // A literal nested within the limit validates and prints.
    let mut v = Value::from(1);
    for _ in 0..(MAX_DEPTH - 2) {
        v = Value::from(List::from(vec![v]));
    }
    let plan = Plan::query(Rex::lit(dynamic(), v));
    assert_eq!(validate(&plan), Ok(()));
    assert!(plan.dump().contains("RexLit"));
}

/// `dump_with_max_depth` follows a raised limit, as `validate_with_max_depth` does.
#[test]
fn dump_follows_a_raised_depth_limit() {
    use partiql_ir::{render_shape, validate_with_max_depth, MAX_DEPTH};
    let deeper = chain(MAX_DEPTH + 10);
    assert!(deeper.dump().starts_with("<plan nested deeper than"));
    assert_eq!(validate_with_max_depth(&deeper, MAX_DEPTH + 10), Ok(()));
    let dump = deeper.dump_with_max_depth(MAX_DEPTH + 10, &render_shape);
    assert!(dump.lines().count() > MAX_DEPTH);
}

fn graph_with_node_value(v: partiql_value::Value) -> partiql_value::Value {
    use partiql_value::{Graph, SimpleGraph, Value};
    use std::collections::HashSet;
    let g = SimpleGraph::from_spec(
        (vec!["n".to_string()], vec![HashSet::new()], vec![Some(v)]),
        (vec![], vec![], vec![], vec![]),
    );
    Value::Graph(Box::new(Graph::Simple(std::rc::Rc::new(g))))
}

/// Graph element values nest too: a graph literal whose node holds a deeply nested list is
/// rejected, and `dump` prints the notice instead of recursing into it (1 MiB stack; leaked).
#[test]
fn deep_graph_element_values_are_bounded() {
    use partiql_value::{List, Value};
    std::thread::Builder::new()
        .stack_size(1 << 20)
        .spawn(|| {
            let mut v = Value::from(1);
            for _ in 0..100_000 {
                v = Value::from(List::from(vec![v]));
            }
            let plan = std::mem::ManuallyDrop::new(Plan::query(Rex::lit(
                dynamic(),
                graph_with_node_value(v),
            )));
            assert!(matches!(
                validate(&plan).unwrap_err()[0].kind(),
                ViolationKind::TooDeep { .. }
            ));
            assert!(plan.dump().starts_with("<plan nested deeper than"));
        })
        .unwrap()
        .join()
        .unwrap();
    // A shallow graph literal validates and prints its element values.
    let plan = Plan::query(Rex::lit(dynamic(), graph_with_node_value(Value::from(7))));
    assert_eq!(validate(&plan), Ok(()));
    assert!(
        plan.dump().contains(
            "Graph #0 {nodes: [0 [] Integer 7], edges: [], directed: [], undirected: []}"
        ),
        "{}",
        plan.dump()
    );
}

/// Graph literals that differ only in labels or edge direction dump differently.
#[test]
fn graph_labels_and_edges_are_printed() {
    use partiql_value::{EdgeSpec, Graph, SimpleGraph, Value};
    use std::collections::HashSet;
    let graph = |label: &str, dir: (&str, &str)| {
        let g = SimpleGraph::from_spec(
            (
                vec!["a".to_string(), "b".to_string()],
                vec![HashSet::from([label.to_string()]), HashSet::new()],
                vec![Some(Value::from(1)), Some(Value::from(2))],
            ),
            (
                vec!["e".to_string()],
                vec![HashSet::new()],
                vec![EdgeSpec::Directed(dir.0.to_string(), dir.1.to_string())],
                vec![None],
            ),
        );
        let v = Value::Graph(Box::new(Graph::Simple(std::rc::Rc::new(g))));
        Plan::query(Rex::lit(dynamic(), v)).dump()
    };
    let base = graph("x", ("a", "b"));
    let want = r#"Graph #0 {nodes: [0 ["x"] Integer 1, 1 [] Integer 2], edges: [0 [] _], directed: [(0 -[0]-> 1)], undirected: []}"#;
    assert!(base.contains(want), "{base}");
    assert_ne!(base, graph("y", ("a", "b")));
    assert_ne!(base, graph("x", ("b", "a")));
}

/// Nodes that print an intermediate tree level (a CASE branch) use more stack per level; a
/// CASE chain at the limit still validates and prints on a default (2 MiB) test thread.
#[test]
fn case_chain_at_the_limit_fits() {
    use partiql_ir::{CaseBranch, MAX_DEPTH};
    let mut rex = int_lit(1);
    // Each CASE adds one Rex level (its branch result).
    for _ in 1..MAX_DEPTH {
        rex = Rex::case(
            int(),
            None,
            vec![CaseBranch::new(Rex::var(bool_(), 0, 0), rex)],
            None,
        );
    }
    let plan = Plan::query(rex);
    assert_eq!(validate(&plan), Ok(()));
    assert!(plan.dump().lines().count() > 2 * MAX_DEPTH);
}

/// Undirected edges, and a connection whose edge id is not in the edge list, still print.
#[test]
fn graph_links_print_in_full() {
    use partiql_value::{EdgeSpec, GEdgeId, GNodeId, Graph, SimpleGraph, Value};
    use std::collections::HashSet;
    let build = |extra: bool| {
        let mut g = SimpleGraph::from_spec(
            (
                vec!["a".to_string(), "b".to_string()],
                vec![HashSet::new(), HashSet::new()],
                vec![None, None],
            ),
            (
                vec!["e".to_string()],
                vec![HashSet::new()],
                vec![EdgeSpec::Undirected("a".to_string(), "b".to_string())],
                vec![None],
            ),
        );
        if extra {
            g.g_dir.push((GNodeId(0), GEdgeId(7), GNodeId(1)));
        }
        let v = Value::Graph(Box::new(Graph::Simple(std::rc::Rc::new(g))));
        Plan::query(Rex::lit(dynamic(), v)).dump()
    };
    let plain = build(false);
    assert!(plain.contains("undirected: [(0 -[0]- 1)]"), "{plain}");
    let dangling = build(true);
    assert!(dangling.contains("directed: [(0 -[7]-> 1)]"), "{dangling}");
    assert_ne!(plain, dangling);
}

/// Graph literals share their graphs (`Rc`), so a small plan can hold exponentially many
/// paths. The depth walk and the printer handle each distinct graph once (repeats print as
/// `Graph #n`), so both stay linear: 40 levels of doubly shared graphs, and one large graph
/// shared by thousands of literals, validate and print quickly.
#[test]
fn shared_graph_literals_are_walked_once() {
    use partiql_value::{Graph, List, SimpleGraph, Value};
    use std::collections::HashSet;
    let graph = |values: Vec<Option<Value>>| {
        let names = (0..values.len()).map(|i| format!("n{i}")).collect();
        let labels = values.iter().map(|_| HashSet::new()).collect();
        let g = SimpleGraph::from_spec((names, labels, values), (vec![], vec![], vec![], vec![]));
        Value::Graph(Box::new(Graph::Simple(std::rc::Rc::new(g))))
    };
    let start = std::time::Instant::now();
    let mut v = Value::from(1);
    for _ in 0..40 {
        v = graph(vec![Some(v.clone()), Some(v)]);
    }
    let plan = Plan::query(Rex::lit(dynamic(), v));
    assert_eq!(validate(&plan), Ok(()));
    let dump = plan.dump();
    assert!(dump.len() < 100_000, "{} bytes", dump.len());
    assert!(dump.contains("Graph #1"));
    // One graph of 10000 value-less nodes, in 5000 list elements and 100 literals.
    let big = graph(vec![None; 10_000]);
    let lits = (0..100)
        .map(|_| Rex::lit(dynamic(), Value::from(List::from(vec![big.clone(); 5_000]))))
        .collect();
    let plan = Plan::query(Rex::array(dynamic(), lits));
    assert_eq!(validate(&plan), Ok(()));
    // Linear: the graph once (~150 KB) plus 500000 short back-references; printing the graph
    // at every reference would be ~75 GB.
    let dump = plan.dump();
    assert!(dump.len() < 20_000_000, "{} bytes", dump.len());
    assert_eq!(dump.matches("Graph #0 {").count(), 1);
    assert!(start.elapsed() < std::time::Duration::from_secs(20));
}

/// A large flat literal is fine: no size cap applies, only the depth bound.
#[test]
fn large_flat_literals_validate() {
    use partiql_value::{List, Value};
    let v = Value::from(List::from(
        (0..200_000).map(Value::from).collect::<Vec<_>>(),
    ));
    assert_eq!(validate(&Plan::query(Rex::lit(dynamic(), v))), Ok(()));
}

/// A shared graph is measured once but checked at every occurrence: one first seen at a
/// shallow position and again under a deep expression is rejected when the deep occurrence
/// would exceed the limit (the printer may print it in full there).
#[test]
fn shared_graph_is_checked_at_every_occurrence() {
    use partiql_value::{Graph, List, SimpleGraph, Value};
    use std::collections::HashSet;
    let mut tall = Value::from(1);
    for _ in 0..60 {
        tall = Value::from(List::from(vec![tall]));
    }
    let g = SimpleGraph::from_spec(
        (
            vec!["n".to_string()],
            vec![HashSet::new()],
            vec![Some(tall)],
        ),
        (vec![], vec![], vec![], vec![]),
    );
    let g = Value::Graph(Box::new(Graph::Simple(std::rc::Rc::new(g))));
    let shallow = Rex::lit(dynamic(), g.clone());
    let mut deep = Rex::lit(dynamic(), g);
    for _ in 0..100 {
        deep = Rex::path_symbol(dynamic(), deep, "a");
    }
    // Either occurrence alone fits only if shallow; together the deep one exceeds the limit.
    assert_eq!(validate(&Plan::query(shallow.clone())), Ok(()));
    let plan = Plan::query(Rex::array(dynamic(), vec![shallow, deep]));
    assert!(matches!(
        validate(&plan).unwrap_err()[0].kind(),
        ViolationKind::TooDeep { .. }
    ));
}
