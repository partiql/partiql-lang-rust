//! Tests for `UNKNOWN` (`PartiqlShape::Undefined`, the static type of `NULL` and `MISSING`) as
//! the bottom of the lattice, and for `common_supertype`.

use partiql_types::lattice::{can_coerce, common_supertype, implicit_coercion, Coercion};
use partiql_types::{
    ArrayType, BagType, PartiqlNoIdShapeBuilder, PartiqlShape, ShapeKind as K, Static,
    StructConstraint, StructField, StructType, VariantType,
};

fn s(ty: Static) -> PartiqlShape {
    PartiqlNoIdShapeBuilder::default().new_static(ty)
}

fn undefined() -> PartiqlShape {
    PartiqlShape::Undefined
}

fn row(fields: &[(&str, Static)]) -> PartiqlShape {
    s(Static::Struct(StructType::new_row(
        fields
            .iter()
            .map(|(n, t)| StructField::new(n, s(t.clone()))),
    )))
}

/// The supertype of several types, folded pairwise; `None` when a pair has none. (A typer's
/// `CASE` / `COALESCE` result adds a further step on top, see `common_supertype`.)
fn fold(types: &[PartiqlShape]) -> Option<PartiqlShape> {
    let mut it = types.iter();
    let first = it.next()?.clone();
    it.try_fold(first, |acc, t| common_supertype(&acc, t))
}

fn sample() -> Vec<PartiqlShape> {
    vec![
        PartiqlShape::Dynamic,
        s(Static::Bool),
        s(Static::Int8),
        s(Static::Int32),
        s(Static::Int64),
        s(Static::NumericP(10, 2)),
        s(Static::DecimalP(5, 2)),
        s(Static::Float32),
        s(Static::Float64),
        s(Static::StringFixed(3)),
        s(Static::StringVarying(5)),
        s(Static::String),
        s(Static::Blob(4)),
        s(Static::Clob(8)),
        s(Static::Date),
        s(Static::Time(3)),
        s(Static::TimeTz(3)),
        s(Static::Timestamp(6)),
        s(Static::TimestampTz(6)),
        s(Static::Array(ArrayType::new(Box::new(s(Static::Int32))))),
        s(Static::Bag(BagType::new(Box::new(s(Static::Int32))))),
        row(&[("a", Static::Int32)]),
        s(Static::Struct(StructType::new_any())),
    ]
}

/// An `UNKNOWN` argument coerces to every parameter, with no cast.
#[test]
fn undefined_coerces_to_every_kind_without_a_cast() {
    assert_eq!(undefined().kind(), Some(K::Unknown));
    for k in K::ALL.iter().copied() {
        assert!(can_coerce(K::Unknown, k), "{k}");
        assert_eq!(
            implicit_coercion(K::Unknown, k),
            Some(Coercion::Absent),
            "{k}"
        );
    }
}

#[test]
fn implicit_coercion_rules() {
    assert_eq!(
        implicit_coercion(K::Integer, K::Integer),
        Some(Coercion::Exact)
    );
    assert_eq!(
        implicit_coercion(K::Integer, K::BigInt),
        Some(Coercion::Cast)
    );
    assert_eq!(
        implicit_coercion(K::Double, K::TinyInt),
        Some(Coercion::Cast)
    );
    assert_eq!(
        implicit_coercion(K::Integer, K::Dynamic),
        Some(Coercion::Cast)
    );
    assert_eq!(implicit_coercion(K::Integer, K::String), None);
    assert_eq!(implicit_coercion(K::Dynamic, K::Integer), None);
}

/// `supertype(T, UNKNOWN) = supertype(UNKNOWN, T) = T`.
#[test]
fn undefined_is_the_bottom_of_the_supertype() {
    for t in sample() {
        assert_eq!(common_supertype(&t, &undefined()), Some(t.clone()), "{t}");
        assert_eq!(common_supertype(&undefined(), &t), Some(t.clone()), "{t}");
    }
    assert_eq!(
        common_supertype(&undefined(), &undefined()),
        Some(undefined())
    );
}

/// `CASE WHEN c THEN 1 ELSE NULL END` is `INTEGER`: the literal `1` is `INTEGER` (it fits in 32
/// bits) and `NULL` is `UNKNOWN`, the bottom. (A typer that skips the `NULL` literal reaches the
/// same result.)
#[test]
fn case_then_1_else_null_is_integer() {
    let int = s(Static::Int32);
    assert_eq!(fold(&[int.clone(), undefined()]), Some(int.clone()));
    assert_eq!(fold(&[undefined(), int.clone()]), Some(int.clone()));
    // `COALESCE(NULL, 1, NULL)`.
    assert_eq!(fold(&[undefined(), int.clone(), undefined()]), Some(int));
    // The bottom with itself. (A typer should type a `CASE` whose branches are all absent
    // literals as `DYNAMIC`, in its result step.)
    assert_eq!(fold(&[undefined(), undefined()]), Some(undefined()));
}

#[test]
fn dynamic_absorbs() {
    for t in sample() {
        if t.kind() == Some(K::Dynamic) {
            continue;
        }
        assert_eq!(
            common_supertype(&t, &PartiqlShape::Dynamic),
            Some(PartiqlShape::Dynamic)
        );
        assert_eq!(
            common_supertype(&PartiqlShape::Dynamic, &t),
            Some(PartiqlShape::Dynamic)
        );
    }
}

#[test]
fn widening_table() {
    let cases: Vec<(Static, Static, Option<Static>)> = vec![
        // Numbers.
        (Static::Int8, Static::Int16, Some(Static::Int16)),
        (Static::Int32, Static::Int64, Some(Static::Int64)),
        (Static::Int8, Static::Float64, Some(Static::Float64)),
        (Static::Int64, Static::Float32, Some(Static::Float32)),
        (
            Static::DecimalP(5, 2),
            Static::Int32,
            Some(Static::DecimalP(12, 2)),
        ),
        (
            Static::Int32,
            Static::DecimalP(5, 2),
            Some(Static::DecimalP(12, 2)),
        ),
        (
            Static::Int8,
            Static::NumericP(2, 1),
            Some(Static::NumericP(4, 1)),
        ),
        (
            Static::NumericP(5, 2),
            Static::DecimalP(3, 1),
            Some(Static::DecimalP(5, 2)),
        ),
        (
            Static::DecimalP(10, 2),
            Static::Float32,
            Some(Static::Float32),
        ),
        (Static::Float32, Static::Float64, Some(Static::Float64)),
        // Decimal merge, capped at 38, signed when the scale exceeds the precision.
        (
            Static::DecimalP(38, 0),
            Static::DecimalP(38, 10),
            Some(Static::DecimalP(38, 10)),
        ),
        (
            Static::DecimalP(1, 2),
            Static::DecimalP(1, 2),
            Some(Static::DecimalP(1, 2)),
        ),
        (
            Static::DecimalP(1, 2),
            Static::DecimalP(3, 0),
            Some(Static::DecimalP(5, 2)),
        ),
        // Text.
        (
            Static::StringFixed(3),
            Static::StringVarying(5),
            Some(Static::StringVarying(5)),
        ),
        (
            Static::StringFixed(7),
            Static::StringVarying(5),
            Some(Static::StringVarying(7)),
        ),
        (Static::StringFixed(2), Static::String, Some(Static::String)),
        (
            Static::StringVarying(4),
            Static::Clob(2),
            Some(Static::Clob(4)),
        ),
        (
            Static::String,
            Static::Clob(10),
            Some(Static::Clob(i32::MAX as usize)),
        ),
        (Static::Blob(4), Static::Blob(9), Some(Static::Blob(9))),
        (Static::Blob(4), Static::Clob(9), None),
        // Datetime.
        (
            Static::Date,
            Static::Timestamp(3),
            Some(Static::Timestamp(3)),
        ),
        (
            Static::Date,
            Static::TimestampTz(3),
            Some(Static::TimestampTz(3)),
        ),
        (Static::Time(2), Static::TimeTz(5), Some(Static::TimeTz(5))),
        (
            Static::Timestamp(6),
            Static::TimestampTz(3),
            Some(Static::TimestampTz(6)),
        ),
        (Static::Date, Static::Time(3), None),
        (Static::Time(3), Static::Timestamp(3), None),
        (Static::TimeTz(3), Static::Time(3), Some(Static::TimeTz(3))),
        // No widening across families.
        (Static::Bool, Static::Int32, None),
        (Static::Int32, Static::String, None),
    ];
    for (a, b, expected) in cases {
        let got = common_supertype(&s(a.clone()), &s(b.clone()));
        assert_eq!(got, expected.clone().map(s), "{a:?} with {b:?}");
    }
}

#[test]
fn collections_and_rows() {
    let int = || s(Static::Int32);
    let array = |e: PartiqlShape| s(Static::Array(ArrayType::new(Box::new(e))));
    let bag = |e: PartiqlShape| s(Static::Bag(BagType::new(Box::new(e))));
    // Element-wise.
    assert_eq!(
        common_supertype(&array(int()), &array(s(Static::Int64))),
        Some(array(s(Static::Int64)))
    );
    assert_eq!(
        common_supertype(&array(int()), &array(s(Static::String))),
        None
    );
    // `ARRAY` widens to `BAG<DYNAMIC>`, whose element absorbs.
    assert_eq!(
        common_supertype(&array(int()), &bag(int())),
        Some(bag(PartiqlShape::Dynamic))
    );
    // Rows merge field by field; differing names become `_`.
    assert_eq!(
        common_supertype(&row(&[("a", Static::Int32)]), &row(&[("b", Static::Int64)])),
        Some(row(&[("_", Static::Int64)]))
    );
    assert_eq!(
        common_supertype(
            &row(&[("a", Static::Int32), ("b", Static::String)]),
            &row(&[("a", Static::Int8), ("b", Static::String)])
        ),
        Some(row(&[("a", Static::Int32), ("b", Static::String)]))
    );
    assert_eq!(
        common_supertype(&row(&[("a", Static::Int32)]), &row(&[])),
        None
    );
    // A closed struct with an unordered field set has no field order to merge by: it has a
    // supertype only with an equal struct, also when the sets were built in another order.
    let closed = |fields: Vec<StructField>| {
        s(Static::Struct(StructType::new(
            [
                StructConstraint::Open(false),
                StructConstraint::Fields(fields.into_iter().collect()),
            ]
            .into(),
        )))
    };
    let ab = closed(vec![
        StructField::new("a", int()),
        StructField::new("b", s(Static::String)),
    ]);
    let ba = closed(vec![
        StructField::new("b", s(Static::String)),
        StructField::new("a", int()),
    ]);
    assert_eq!(common_supertype(&ab, &ba), Some(ab.clone()));
    assert_eq!(
        common_supertype(
            &closed(vec![StructField::new("a", int())]),
            &row(&[("a", Static::Int64)])
        ),
        None
    );
    // An optional field stays optional.
    let opt = s(Static::Struct(StructType::new_row([
        StructField::new_optional("a", int()),
    ])));
    let merged = common_supertype(&opt, &row(&[("a", Static::Int64)])).unwrap();
    assert_eq!(
        merged,
        s(Static::Struct(StructType::new_row([
            StructField::new_optional("a", s(Static::Int64))
        ])))
    );
    // `ROW` and `STRUCT` have no supertype; two `STRUCT`s keep the left one.
    let open = s(Static::Struct(StructType::new_any()));
    assert_eq!(common_supertype(&row(&[]), &open), None);
    assert_eq!(common_supertype(&open, &open), Some(open.clone()));
}

#[test]
fn shapes_without_a_kind_have_no_supertype_except_with_the_bottom() {
    let int = s(Static::Int32);
    let union = PartiqlNoIdShapeBuilder::default().any_of([int.clone(), s(Static::String)]);
    assert_eq!(common_supertype(&union, &int), None);
    assert_eq!(common_supertype(&s(Static::Int), &int), None);
    // The bottom holds for every shape.
    assert_eq!(
        common_supertype(&s(Static::Decimal), &undefined()),
        Some(s(Static::Decimal))
    );
    assert_eq!(common_supertype(&undefined(), &union), Some(union.clone()));
}

/// The supertype's kind does not depend on the order of the operands (widening is tried in
/// both directions).
#[test]
fn supertype_kind_is_symmetric() {
    let all = sample();
    for a in &all {
        for b in &all {
            let ab = common_supertype(a, b).map(|t| t.kind());
            let ba = common_supertype(b, a).map(|t| t.kind());
            assert_eq!(ab, ba, "{a} with {b}");
        }
    }
}

/// Every type admits the absent values, so every supertype is nullable, whatever the inputs.
#[test]
fn supertypes_are_nullable() {
    let mut b = PartiqlNoIdShapeBuilder::default();
    let non_null_int = b.new_non_nullable_static(Static::Int32);
    let non_null_bool = b.new_non_nullable_static(Static::Bool);
    let nullable = |t: Option<PartiqlShape>| match t {
        Some(PartiqlShape::Static(st)) => st.is_nullable(),
        other => panic!("{other:?}"),
    };
    assert!(nullable(common_supertype(&non_null_int, &undefined())));
    assert!(nullable(common_supertype(&undefined(), &non_null_int)));
    assert!(nullable(common_supertype(&non_null_bool, &s(Static::Bool))));
    assert!(nullable(common_supertype(&s(Static::Bool), &non_null_bool)));
    assert!(nullable(common_supertype(&non_null_int, &non_null_int)));
    assert!(nullable(common_supertype(&non_null_int, &s(Static::Int64))));
}

/// The encoding is part of a `VARIANT` type, so two encodings have no common supertype; one
/// encoding is its own supertype, and the bottom still gives the other side.
#[test]
fn variant_supertype_follows_its_encoding() {
    let ion = s(Static::Variant(VariantType::new("ion")));
    let json = s(Static::Variant(VariantType::new("json")));
    assert_ne!(ion, json);
    assert_eq!(common_supertype(&ion, &ion), Some(ion.clone()));
    assert_eq!(common_supertype(&ion, &json), None);
    assert_eq!(common_supertype(&json, &ion), None);
    assert_eq!(common_supertype(&ion, &undefined()), Some(ion.clone()));
    assert_eq!(
        common_supertype(&ion, &PartiqlShape::Dynamic),
        Some(PartiqlShape::Dynamic)
    );
}
