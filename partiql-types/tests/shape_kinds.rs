//! Tests for the static kinds, the row and map types and the unspecified-parameter flags.

use indexmap::IndexSet;
use partiql_types::{
    type_int32, type_string, type_struct, IntervalFamily, IntervalQualifier, IntervalType, MapType,
    PartiqlNoIdShapeBuilder, PartiqlShape, PartiqlShapeBuilder, ShapeResultError, Static,
    StaticCategory, StructConstraint, StructField, StructType, TypeParams, VariantType,
};
use std::collections::HashSet;

fn new_scalar_kinds() -> Vec<(Static, &'static str)> {
    vec![
        (Static::NumericP(10, 2), "Numeric(10,2)"),
        (Static::Blob(16), "Blob(16)"),
        (Static::Clob(16), "Clob(16)"),
        (Static::Date, "Date"),
        (Static::Time(3), "Time(3)"),
        (Static::TimeTz(3), "TimeTz(3)"),
        (Static::Timestamp(6), "Timestamp(6)"),
        (Static::TimestampTz(6), "TimestampTz(6)"),
        (
            Static::Interval(IntervalType::new(IntervalQualifier::DayToSecond, 2, 6)),
            "Interval(Day(2) To Second(6))",
        ),
        (
            Static::Interval(IntervalType::new(IntervalQualifier::YearToMonth, 2, 0)),
            "Interval(Year(2) To Month)",
        ),
        (Static::Null, "Null"),
        (Static::Missing, "Missing"),
        (Static::Variant(VariantType::new("ion")), "Variant(ion)"),
    ]
}

#[test]
fn each_new_scalar_kind_constructs_displays_and_is_scalar() {
    let mut bld = PartiqlNoIdShapeBuilder::default();
    for (kind, display) in new_scalar_kinds() {
        let shape = bld.new_static(kind.clone());
        assert_eq!(format!("{shape}"), display);
        let st = shape.expect_static().unwrap();
        assert_eq!(st.ty(), &kind);
        assert!(st.is_nullable());
        assert!(st.is_scalar(), "{kind} should be scalar");
        assert!(!st.is_sequence());
        assert!(!st.is_struct());
        assert!(st.params().is_default());
        assert!(matches!(kind.category(), StaticCategory::Scalar(k) if k == &kind));

        if matches!(kind, Static::Null | Static::Missing) {
            continue; // always nullable, see `absent_value_types_are_always_nullable`
        }
        let not_null = bld.new_non_nullable_static(kind.clone());
        assert_eq!(format!("{not_null}"), format!("NOT NULL {display}"));
        assert_ne!(shape, not_null);
    }
}

#[test]
fn new_kinds_are_pairwise_distinct_and_distinct_from_existing_kinds() {
    let mut kinds: Vec<Static> = new_scalar_kinds().into_iter().map(|(k, _)| k).collect();
    kinds.extend([
        Static::DecimalP(10, 2),
        Static::Decimal,
        Static::DateTime,
        Static::String,
        Static::StringFixed(16),
        Static::StringVarying(16),
    ]);
    let set: HashSet<_> = kinds.iter().cloned().collect();
    assert_eq!(set.len(), kinds.len());

    // DECIMAL and NUMERIC are different kinds with the same parameters.
    let mut bld = PartiqlNoIdShapeBuilder::default();
    assert_ne!(
        bld.new_static(Static::DecimalP(10, 2)),
        bld.new_static(Static::NumericP(10, 2))
    );
    // Parameters take part in equality.
    assert_ne!(Static::Time(3), Static::Time(6));
    assert_ne!(Static::Null, Static::Missing);
}

#[test]
fn interval_qualifiers_belong_to_one_family() {
    use IntervalQualifier::*;
    for q in [Year, Month, YearToMonth] {
        assert_eq!(q.family(), IntervalFamily::YearMonth, "{q:?}");
    }
    for q in [
        Day,
        Hour,
        Minute,
        Second,
        DayToHour,
        DayToMinute,
        DayToSecond,
        HourToMinute,
        HourToSecond,
        MinuteToSecond,
    ] {
        assert_eq!(q.family(), IntervalFamily::DayTime, "{q:?}");
    }

    assert!([Second, DayToSecond, HourToSecond, MinuteToSecond]
        .iter()
        .all(IntervalQualifier::has_seconds));
    assert!(![
        Year,
        Month,
        YearToMonth,
        Day,
        Hour,
        Minute,
        DayToHour,
        DayToMinute,
        HourToMinute
    ]
    .iter()
    .any(IntervalQualifier::has_seconds));
    // Without a SECOND field the fractional precision is not part of the type.
    assert_eq!(IntervalType::new(Year, 2, 6), IntervalType::new(Year, 2, 0));
    assert_eq!(IntervalType::new(DayToHour, 2, 6).fractional_precision(), 0);

    let i = IntervalType::new(HourToSecond, 3, 4);
    assert_eq!(i.qualifier(), HourToSecond);
    assert_eq!(i.precision(), 3);
    assert_eq!(i.fractional_precision(), 4);
    assert_eq!(i.family(), IntervalFamily::DayTime);
    assert_ne!(i, IntervalType::new(HourToSecond, 3, 5));
}

#[test]
fn map_is_a_non_scalar_container() {
    let mut bld = PartiqlNoIdShapeBuilder::default();
    let (k, v) = (type_string!(bld), type_int32!(bld));
    let map = bld.new_map(k.clone(), v.clone()).unwrap();
    assert_eq!(format!("{map}"), "Map<String, Int32>");

    let st = map.expect_static().unwrap();
    assert!(!st.is_scalar());
    assert!(!st.is_sequence());
    assert!(!st.is_struct());
    let Static::Map(m) = st.ty() else {
        panic!("expected a map")
    };
    assert_eq!(m.key_type(), &k);
    assert_eq!(m.value_type(), &v);
    assert!(matches!(st.ty().category(), StaticCategory::Map(c) if c == m));
    assert_eq!(m, &MapType::new(k.clone(), v.clone()).unwrap());
    assert_ne!(m, &MapType::new(v, k).unwrap());
}

#[test]
fn row_keeps_order_and_duplicate_names() {
    let mut bld = PartiqlNoIdShapeBuilder::default();
    // SELECT a, a, b: the rows have two fields named `a`.
    let (a1, a2, b) = (type_int32!(bld), type_int32!(bld), type_string!(bld));
    let row = bld.new_row([
        StructField::new("a", a1),
        StructField::new("a", a2),
        StructField::new("b", b),
    ]);
    assert_eq!(format!("{row}"), "Row{a: Int32, a: Int32, b: String}");
    let st = row.expect_struct().unwrap();
    assert!(st.is_row());
    assert!(st.is_closed());
    assert!(!st.is_partial());
    let names: Vec<_> = st.fields().map(|f| f.name().to_string()).collect();
    assert_eq!(names, ["a", "a", "b"]);
    // `fields_set` reads the row's fields too (and, being a set, collapses equal ones).
    assert_eq!(st.fields_set().len(), 2);
    assert_eq!(st, StructType::new_row(st.fields().cloned()));

    // Field order matters for a row.
    let (x, y) = (type_int32!(bld), type_string!(bld));
    let xy = StructType::new_row([
        StructField::new("x", x.clone()),
        StructField::new("y", y.clone()),
    ]);
    let yx = StructType::new_row([StructField::new("y", y), StructField::new("x", x)]);
    assert_ne!(xy, yx);

    // The set-based `Fields` constraint collapses equal no-id fields; this is why rows use
    // `FieldList`.
    let (a1, a2) = (type_int32!(bld), type_int32!(bld));
    let set_based = type_struct![
        bld,
        IndexSet::from([
            StructConstraint::Fields(IndexSet::from([
                StructField::new("a", a1),
                StructField::new("a", a2),
            ])),
            StructConstraint::Open(false),
        ])
    ];
    assert_eq!(set_based.expect_struct().unwrap().fields().count(), 1);
}

#[test]
fn only_closed_ordered_structs_are_rows() {
    let mut bld = PartiqlNoIdShapeBuilder::default();
    assert!(!type_struct![bld].expect_struct().unwrap().is_row());
    assert!(!StructType::new_any().is_row());
    let closed_unordered = StructType::new(IndexSet::from([StructConstraint::Open(false)]));
    assert!(!closed_unordered.is_row());
    let open_ordered = StructType::new(IndexSet::from([
        StructConstraint::Open(true),
        StructConstraint::Ordered(true),
    ]));
    assert!(!open_ordered.is_row());
    assert!(StructType::new_row([]).is_row());
    assert_eq!(StructType::new_row([]).fields().count(), 0);

    // A struct holding both field constraints reads the fields of both, in constraint order.
    let (a, b) = (type_int32!(bld), type_string!(bld));
    let both = StructType::new(IndexSet::from([
        StructConstraint::Fields(IndexSet::from([StructField::new("a", a)])),
        StructConstraint::FieldList(vec![StructField::new("b", b)]),
    ]));
    let names: Vec<_> = both.fields().map(StructField::name).collect();
    assert_eq!(names, ["a", "b"]);
    assert_eq!(both.fields_set().len(), 2);
}

#[test]
fn unspecified_parameter_flags_do_not_take_part_in_equality_or_hash() {
    let mut bld = PartiqlNoIdShapeBuilder::default();
    let unspecified = TypeParams::default().with_unspecified_length(true);
    assert!(!unspecified.is_default());
    let varchar = bld.new_static_with_params(Static::StringVarying(1), unspecified);
    let varchar_1 = bld.new_static(Static::StringVarying(1));
    assert_eq!(varchar, varchar_1);
    let set: HashSet<_> = [varchar.clone(), varchar_1.clone()].into();
    assert_eq!(set.len(), 1);

    assert!(varchar.expect_static().unwrap().params().unspecified_length);
    assert!(
        !varchar_1
            .expect_static()
            .unwrap()
            .params()
            .unspecified_length
    );
    assert!(varchar.expect_static().unwrap().is_nullable());

    // Flags are shown by Debug only when set.
    assert_eq!(
        format!("{varchar:?}"),
        "Static(StaticType { id: NodeId(0), ty: StringVarying(1), nullable: true, params: \
         TypeParams { unspecified_length: true, unspecified_precision: false, \
         unspecified_scale: false, unspecified_fractional_precision: false } })"
    );
    assert_eq!(
        format!("{varchar_1:?}"),
        "Static(StaticType { id: NodeId(0), ty: StringVarying(1), nullable: true })"
    );
    // Display is unaffected by the flags.
    assert_eq!(format!("{varchar}"), format!("{varchar_1}"));

    // Shape collections that de-duplicate keep the first of two shapes differing only in flags.
    let merged = bld.any_of([varchar.clone(), varchar_1.clone()]);
    assert!(merged.expect_static().unwrap().params().unspecified_length);
    let merged = bld.any_of([varchar_1, varchar]);
    assert!(!merged.expect_static().unwrap().params().unspecified_length);

    let all = TypeParams::default()
        .with_unspecified_length(true)
        .with_unspecified_precision(true)
        .with_unspecified_scale(true)
        .with_unspecified_fractional_precision(true);
    assert!(all.unspecified_length && all.unspecified_precision);
    assert!(all.unspecified_scale && all.unspecified_fractional_precision);
    assert!(all.with_unspecified_length(false).unspecified_precision);
    assert!(!all.with_unspecified_length(false).unspecified_length);
}

#[test]
fn as_non_nullable_keeps_parameter_flags() {
    let mut bld = PartiqlNoIdShapeBuilder::default();
    let params = TypeParams::default().with_unspecified_precision(true);
    let ts = bld.new_static_with_params(Static::Timestamp(6), params);
    let not_null = bld.as_non_nullable(&ts).unwrap();
    let st = not_null.expect_static().unwrap();
    assert!(st.is_not_nullable());
    assert_eq!(st.ty(), &Static::Timestamp(6));
    assert_eq!(st.params(), &params);
}

#[test]
fn no_id_builder_makes_nested_new_kinds_structurally_equal() {
    let mk = |b: &mut PartiqlNoIdShapeBuilder| {
        let d = b.new_static(Static::Date);
        let row = b.new_row([StructField::new("x", d)]);
        b.new_bag_of(row)
    };
    let mut none = PartiqlNoIdShapeBuilder::default();
    assert_eq!(mk(&mut none), mk(&mut none));

    let mut auto = PartiqlShapeBuilder::default();
    let (d1, d2) = (auto.new_static(Static::Date), auto.new_static(Static::Date));
    assert_ne!(d1, d2);
    assert!(matches!(d1, PartiqlShape::Static(_)));
}

#[test]
fn map_key_must_not_be_dynamic() {
    // As partiql-lang-kotlin's `PTypeMap`: "MAP key type must not be DYNAMIC".
    let mut bld = PartiqlNoIdShapeBuilder::default();
    let err = MapType::new(PartiqlShape::Dynamic, type_int32!(bld)).expect_err("dynamic key");
    assert_eq!(err, ShapeResultError::DynamicMapKey);
    assert_eq!(err.to_string(), "MAP key type must not be DYNAMIC");
    assert_eq!(
        bld.new_map(PartiqlShape::Dynamic, PartiqlShape::Dynamic),
        Err(ShapeResultError::DynamicMapKey)
    );
    // A dynamic value, or a key that is a union, is fine.
    let k = type_string!(bld);
    assert!(bld.new_map(k, PartiqlShape::Dynamic).is_ok());
    let (a, b) = (type_string!(bld), type_int32!(bld));
    let union = bld.any_of([a, b]);
    assert!(bld.new_map(union, PartiqlShape::Dynamic).is_ok());
}

#[test]
fn variant_equality_ignores_the_encoding() {
    // As partiql-lang-kotlin, where VARIANT compares only its type code.
    let (ion, json) = (VariantType::new("ion"), VariantType::new("json"));
    assert_eq!(ion, json);
    assert_eq!(Static::Variant(ion.clone()), Static::Variant(json.clone()));
    let set: HashSet<_> = [Static::Variant(ion.clone()), Static::Variant(json)].into();
    assert_eq!(set.len(), 1);
    assert_eq!(ion.encoding(), "ion");
    assert_eq!(format!("{}", Static::Variant(ion)), "Variant(ion)");
    assert_ne!(Static::Variant(VariantType::new("ion")), Static::String);
}

#[test]
fn rows_have_their_own_display_and_set_based_structs_are_not_rows() {
    let mut bld = PartiqlNoIdShapeBuilder::default();
    let a = type_int32!(bld);
    let set_based = StructType::new(IndexSet::from([
        StructConstraint::Fields(IndexSet::from([StructField::new("a", a.clone())])),
        StructConstraint::Open(false),
        StructConstraint::Ordered(true),
    ]));
    assert!(!set_based.is_row());
    assert_eq!(format!("{set_based}"), "{a: Int32}");
    // A closed, ordered struct with no field constraint is not a row either.
    let no_fields = StructType::new(IndexSet::from([
        StructConstraint::Open(false),
        StructConstraint::Ordered(true),
    ]));
    assert!(!no_fields.is_row());
    let row = StructType::new_row([StructField::new("a", a)]);
    assert_eq!(format!("{row}"), "Row{a: Int32}");
    assert_eq!(format!("{}", StructType::new_row([])), "Row{}");
}

#[test]
fn equal_rows_hash_equally_whatever_the_constraint_order() {
    use std::hash::{BuildHasher, RandomState};
    let mut bld = PartiqlNoIdShapeBuilder::default();
    let fields = vec![
        StructField::new("a", type_int32!(bld)),
        StructField::new("a", type_string!(bld)),
    ];
    let row = StructType::new_row(fields.clone());
    let reordered = StructType::new(IndexSet::from([
        StructConstraint::FieldList(fields),
        StructConstraint::DuplicateAttrs(true),
        StructConstraint::Ordered(true),
        StructConstraint::Open(false),
    ]));
    assert_eq!(row, reordered);
    let state = RandomState::new();
    assert_eq!(state.hash_one(&row), state.hash_one(&reordered));
    let set: HashSet<_> = [row, reordered].into();
    assert_eq!(set.len(), 1);
}

#[test]
fn absent_value_types_are_always_nullable() {
    let mut bld = PartiqlNoIdShapeBuilder::default();
    for kind in [Static::Null, Static::Missing] {
        let shape = bld.new_non_nullable_static(kind.clone());
        assert!(shape.expect_static().unwrap().is_nullable());
        assert_eq!(format!("{shape}"), format!("{kind}"));
        let nullable = bld.new_static(kind.clone());
        assert_eq!(shape, nullable);
        let copy = bld.as_non_nullable(&nullable).unwrap();
        assert!(copy.expect_static().unwrap().is_nullable());
        assert!(kind.is_scalar());
    }
    // Other kinds are unaffected.
    let d = bld.new_non_nullable_static(Static::Date);
    assert!(d.expect_static().unwrap().is_not_nullable());
}

#[test]
fn flags_are_kept_only_on_default_payloads() {
    let mut bld = PartiqlNoIdShapeBuilder::default();
    let length = TypeParams::default().with_unspecified_length(true);
    let kept = bld.new_static_with_params(Static::StringVarying(1), length);
    assert!(kept.expect_static().unwrap().params().unspecified_length);
    let cleared = bld.new_static_with_params(Static::StringVarying(50), length);
    assert!(cleared.expect_static().unwrap().params().is_default());
    // partiql-lang-kotlin's planner defaults: BLOB/CLOB i32::MAX, TIME 0, TIMESTAMP 6.
    let blob = bld.new_static_with_params(Static::Blob(i32::MAX as usize), length);
    assert!(blob.expect_static().unwrap().params().unspecified_length);
    let blob_1 = bld.new_static_with_params(Static::Blob(1), length);
    assert!(blob_1.expect_static().unwrap().params().is_default());
    let precision = TypeParams::default().with_unspecified_precision(true);
    for (kind, kept) in [
        (Static::Time(0), true),
        (Static::Time(6), false),
        (Static::TimeTz(0), true),
        (Static::Timestamp(6), true),
        (Static::Timestamp(3), false),
        (Static::TimestampTz(6), true),
        (Static::DecimalP(38, 2), true),
        (Static::NumericP(10, 0), false),
    ] {
        let shape = bld.new_static_with_params(kind.clone(), precision);
        assert_eq!(
            shape
                .expect_static()
                .unwrap()
                .params()
                .unspecified_precision,
            kept,
            "{kind}"
        );
    }
}

#[test]
fn interval_display_is_sql_like() {
    use IntervalQualifier::*;
    let cases = [
        (Year, 2, 0, "Interval(Year(2))"),
        (Month, 3, 6, "Interval(Month(3))"),
        (Second, 2, 6, "Interval(Second(2,6))"),
        (DayToHour, 2, 0, "Interval(Day(2) To Hour)"),
        (HourToSecond, 2, 3, "Interval(Hour(2) To Second(3))"),
        (MinuteToSecond, 1, 0, "Interval(Minute(1) To Second(0))"),
    ];
    for (q, p, fp, text) in cases {
        assert_eq!(format!("{}", IntervalType::new(q, p, fp)), text);
    }
}
