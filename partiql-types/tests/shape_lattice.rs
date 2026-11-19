//! Tests for `ShapeKind` and the `lattice` tables, ported from partiql-lang-kotlin (PLK) v1.6.1.
//!
//! The expected values are transcribed from PLK's sources (named at each table) independently of
//! the implementation, one row per kind.

use partiql_types::lattice::{
    can_cast, can_coerce, coercion_family, compare_fn_precedence, fn_precedence, type_precedence,
    CoercionFamily as F, SqlTypeFamily, FN_PRECEDENCE, TYPE_PRECEDENCE,
};
use partiql_types::{
    ArrayType, BagType, IntervalQualifier, IntervalType, MapType, PartiqlNoIdShapeBuilder,
    PartiqlShape, PartiqlShapeBuilder, ShapeKind as K, Static, StructConstraint, StructField,
    StructType, VariantType,
};
use std::cmp::Ordering;
use std::collections::HashSet;

/// Every kind except `excluded`, in `ShapeKind::ALL` order.
fn all_except(excluded: &[K]) -> Vec<K> {
    K::ALL
        .iter()
        .copied()
        .filter(|k| !excluded.contains(k))
        .collect()
}

struct KindRow {
    kind: K,
    /// `PType.name()`, `PType.java:232-297`.
    name: &'static str,
    /// `CoercionFamily.family`, `partiql-planner/.../CoercionFamily.kt:39-72`.
    family: F,
    /// The index in `FnComparator.precedence`, `FnComparator.kt:45-73`.
    fn_precedence: Option<usize>,
    /// The index in `TypePrecedence.TYPE_PRECEDENCE`, `TypePrecedence.kt:11-39`.
    type_precedence: Option<usize>,
    /// The kinds `graph[kind]` marks `YES`, `CastTable.kt:54-311`.
    casts: Vec<K>,
}

fn rows() -> Vec<KindRow> {
    use K::*;
    let row = |kind, name, family, fp, tp, casts: Vec<K>| KindRow {
        kind,
        name,
        family,
        fn_precedence: fp,
        type_precedence: tp,
        casts,
    };
    let numbers = || {
        vec![
            Bool, TinyInt, SmallInt, Integer, BigInt, Numeric, Decimal, Real, Double, Char, String,
            Varchar,
        ]
    };
    let text = || all_except(&[Blob, Unknown, Dynamic, Array, Bag, Row]);
    let time = || {
        vec![
            TimestampTz,
            Timestamp,
            TimeTz,
            Time,
            String,
            Varchar,
            Char,
            Clob,
        ]
    };
    let timestamp = || {
        vec![
            TimestampTz,
            Timestamp,
            Date,
            TimeTz,
            Time,
            String,
            Varchar,
            Char,
            Clob,
        ]
    };
    vec![
        row(
            Dynamic,
            "DYNAMIC",
            F::Dynamic,
            Some(26),
            Some(26),
            K::ALL.to_vec(),
        ),
        row(Bool, "BOOL", F::Boolean, Some(0), Some(1), numbers()),
        row(TinyInt, "TINYINT", F::Number, Some(1), Some(2), numbers()),
        row(SmallInt, "SMALLINT", F::Number, Some(2), Some(3), numbers()),
        row(Integer, "INTEGER", F::Number, Some(3), Some(4), numbers()),
        row(BigInt, "BIGINT", F::Number, Some(4), Some(5), numbers()),
        row(Numeric, "NUMERIC", F::Number, Some(5), Some(6), numbers()),
        row(Decimal, "DECIMAL", F::Number, Some(6), Some(7), numbers()),
        row(Real, "REAL", F::Number, Some(7), Some(8), numbers()),
        row(Double, "DOUBLE", F::Number, Some(8), Some(9), numbers()),
        row(Char, "CHAR", F::String, Some(9), Some(10), text()),
        row(Varchar, "VARCHAR", F::String, Some(10), Some(11), text()),
        row(String, "STRING", F::String, Some(11), Some(12), text()),
        row(Blob, "BLOB", F::Binary, Some(13), Some(14), vec![]),
        row(Clob, "CLOB", F::String, Some(12), Some(13), text()),
        row(
            Date,
            "DATE",
            F::DateTimestamp,
            Some(14),
            Some(15),
            vec![
                TimestampTz,
                Timestamp,
                TimeTz,
                Date,
                String,
                Varchar,
                Char,
                Clob,
            ],
        ),
        row(Time, "TIME", F::Time, Some(15), Some(16), time()),
        row(TimeTz, "TIMEZ", F::Time, Some(16), Some(17), time()),
        row(
            Timestamp,
            "TIMESTAMP",
            F::DateTimestamp,
            Some(17),
            Some(18),
            timestamp(),
        ),
        row(
            TimestampTz,
            "TIMESTAMPZ",
            F::DateTimestamp,
            Some(18),
            Some(19),
            timestamp(),
        ),
        row(
            Array,
            "ARRAY",
            F::Collection,
            Some(21),
            Some(22),
            vec![Bag, Array],
        ),
        row(
            Bag,
            "BAG",
            F::Collection,
            Some(22),
            Some(23),
            vec![Bag, Array],
        ),
        row(Row, "ROW", F::Structure, Some(23), Some(24), vec![Row, Map]),
        row(
            Struct,
            "STRUCT",
            F::Structure,
            Some(24),
            Some(25),
            vec![Struct, Map],
        ),
        row(
            Unknown,
            "UNKNOWN",
            F::Unknown,
            None,
            Some(0),
            K::ALL.to_vec(),
        ),
        row(Variant, "VARIANT", F::Unknown, None, None, K::ALL.to_vec()),
        row(
            IntervalYm,
            "INTERVAL_YM",
            F::IntervalYm,
            Some(19),
            Some(20),
            vec![IntervalYm],
        ),
        row(
            IntervalDt,
            "INTERVAL_DT",
            F::IntervalDt,
            Some(20),
            Some(21),
            vec![IntervalDt],
        ),
        row(Map, "MAP", F::Map, Some(25), None, vec![Map]),
    ]
}

#[test]
fn rows_cover_every_kind_in_plk_code_order() {
    let kinds: Vec<K> = rows().iter().map(|r| r.kind).collect();
    assert_eq!(kinds, K::ALL.to_vec());
    assert_eq!(K::ALL.iter().collect::<HashSet<_>>().len(), 29);
}

#[test]
fn name_and_display_per_kind() {
    for r in rows() {
        assert_eq!(r.kind.name(), r.name);
        assert_eq!(r.kind.to_string(), r.name);
    }
}

#[test]
fn coercion_family_per_kind() {
    for r in rows() {
        assert_eq!(coercion_family(r.kind), r.family, "{}", r.name);
    }
}

#[test]
fn precedence_per_kind() {
    for r in rows() {
        assert_eq!(fn_precedence(r.kind), r.fn_precedence, "fn {}", r.name);
        assert_eq!(
            type_precedence(r.kind),
            r.type_precedence,
            "type {}",
            r.name
        );
    }
    // Each list is a strict order: no kind twice.
    assert_eq!(FN_PRECEDENCE.iter().collect::<HashSet<_>>().len(), 27);
    assert_eq!(TYPE_PRECEDENCE.iter().collect::<HashSet<_>>().len(), 27);
}

#[test]
fn cast_row_per_kind() {
    for r in rows() {
        for target in K::ALL.iter().copied() {
            assert_eq!(
                can_cast(r.kind, target),
                r.casts.contains(&target),
                "CAST({} AS {target})",
                r.name
            );
        }
    }
}

/// Port of PLK's `TypeLatticeTest.latticeAsciidocDump` (`TypeLatticeTest.kt:9-14`), its only
/// test: `@Disabled`, with no assertions, it exists to print the cast table. PLK's `CastTable`
/// has no `toString`, so it prints an object reference; this port prints the table as Asciidoc
/// (operand rows, target columns), as the PLK test's name intends. Ignored like the original;
/// run it with `cargo test -p partiql-types --test shape_lattice -- --ignored --nocapture`.
/// The table's content is checked by `cast_row_per_kind`.
#[test]
#[ignore = "dump only, as PLK's @Disabled TypeLatticeTest"]
fn lattice_asciidoc_dump() {
    let mut out = String::from("|===\n|");
    for target in K::ALL {
        out.push_str(&format!(" |{target}"));
    }
    out.push('\n');
    for operand in K::ALL.iter().copied() {
        out.push_str(&format!("|{operand}"));
        for target in K::ALL.iter().copied() {
            out.push_str(if can_cast(operand, target) {
                " |Y"
            } else {
                " |N"
            });
        }
        out.push('\n');
    }
    out.push_str("|===\n");
    println!("{out}");
}

#[test]
fn cast_table_quirks_are_kept() {
    // `DATE` reaches `TIMEZ` but not `TIME`.
    assert!(can_cast(K::Date, K::TimeTz));
    assert!(!can_cast(K::Date, K::Time));
    // `BLOB` casts to nothing, not even itself; character strings do not cast to `DYNAMIC`.
    assert!(!can_cast(K::Blob, K::Blob));
    assert!(!can_cast(K::Varchar, K::Dynamic));
    assert!(can_cast(K::Varchar, K::Struct));
    // A number does not cast to `CLOB`; `BOOL` and the numbers cast to each other.
    assert!(!can_cast(K::Integer, K::Clob));
    assert!(can_cast(K::Double, K::Bool));
    // Rows and structs cast to maps, maps only to maps.
    assert!(can_cast(K::Row, K::Map));
    assert!(!can_cast(K::Map, K::Struct));
    assert!(!can_cast(K::Row, K::Struct));
}

#[test]
fn coercion_rules() {
    // No direction within a family (`CoercionFamily.kt:82-90`).
    assert!(can_coerce(K::Double, K::TinyInt));
    assert!(can_coerce(K::TinyInt, K::Double));
    assert!(can_coerce(K::Clob, K::Char));
    assert!(can_coerce(K::Date, K::TimestampTz));
    assert!(can_coerce(K::Row, K::Struct));
    assert!(can_coerce(K::Array, K::Bag));
    // Different families.
    assert!(!can_coerce(K::Time, K::Timestamp));
    assert!(!can_coerce(K::Blob, K::Clob));
    assert!(!can_coerce(K::IntervalYm, K::IntervalDt));
    assert!(!can_coerce(K::Bool, K::Integer));
    // `UNKNOWN` coerces to anything and anything coerces to `DYNAMIC`, but not the reverse.
    for k in K::ALL.iter().copied() {
        assert!(can_coerce(K::Unknown, k));
        assert!(can_coerce(k, K::Dynamic));
    }
    assert!(!can_coerce(K::Dynamic, K::Integer));
    assert!(!can_coerce(K::Integer, K::Unknown));
    // `VARIANT` shares `UNKNOWN`'s family but not its "from" rule.
    assert!(can_coerce(K::Variant, K::Unknown));
    assert!(!can_coerce(K::Variant, K::Integer));
    assert!(!can_coerce(K::Integer, K::Variant));
}

#[test]
fn fn_precedence_orders_overloads_as_plk() {
    // A call with a DYNAMIC or VARIANT argument dispatches over every candidate, sorted by
    // `FnComparator` (`FnResolver.kt:47-52`); for `abs`, the numbers and intervals in this order.
    let mut candidates = vec![
        K::IntervalDt,
        K::Double,
        K::TinyInt,
        K::IntervalYm,
        K::Real,
        K::Decimal,
        K::BigInt,
        K::Numeric,
        K::Integer,
        K::SmallInt,
    ];
    candidates.sort_by(|a, b| compare_fn_precedence(*a, *b).unwrap());
    assert_eq!(
        candidates,
        vec![
            K::TinyInt,
            K::SmallInt,
            K::Integer,
            K::BigInt,
            K::Numeric,
            K::Decimal,
            K::Real,
            K::Double,
            K::IntervalYm,
            K::IntervalDt,
        ]
    );
    assert_eq!(
        compare_fn_precedence(K::Unknown, K::Unknown),
        Some(Ordering::Equal)
    );
    assert_eq!(compare_fn_precedence(K::Unknown, K::Integer), None);
    assert_eq!(compare_fn_precedence(K::Integer, K::Variant), None);
    assert_eq!(
        compare_fn_precedence(K::Map, K::Dynamic),
        Some(Ordering::Less)
    );
}

#[test]
fn sql_type_family_membership() {
    use SqlTypeFamily as S;
    let groups: [(S, Vec<K>); 10] = [
        (S::BOOLEAN, vec![K::Bool]),
        (S::TEXT, vec![K::Char, K::Varchar, K::String, K::Clob]),
        (S::COLLECTION, vec![K::Array, K::Bag]),
        (
            S::NUMBER,
            vec![
                K::TinyInt,
                K::SmallInt,
                K::Integer,
                K::BigInt,
                K::Numeric,
                K::Real,
                K::Double,
                K::Decimal,
            ],
        ),
        (S::DATE, vec![K::Date]),
        (S::TIME, vec![K::Time, K::TimeTz]),
        (S::TIMESTAMP, vec![K::Timestamp, K::TimestampTz]),
        (
            S::DATE_TIMESTAMP,
            vec![K::Date, K::Timestamp, K::TimestampTz],
        ),
        (
            S::DATETIME,
            vec![K::Date, K::Time, K::TimeTz, K::Timestamp, K::TimestampTz],
        ),
        (S::INTERVAL, vec![K::IntervalYm, K::IntervalDt]),
    ];
    for (family, members) in groups {
        assert_eq!(family.members(), members.as_slice());
        for k in K::ALL.iter().copied() {
            assert_eq!(family.contains(k), members.contains(&k), "{k}");
        }
    }
}

#[test]
fn sql_type_family_of_per_kind() {
    use K::*;
    // `SqlTypeFamily.kt:40-45`.
    let number = [
        Integer, TinyInt, SmallInt, BigInt, Real, Double, Decimal, Numeric,
    ];
    let text = [Char, Varchar, String, Clob];
    let collection = [Bag, Array];
    for k in K::ALL.iter().copied() {
        let family = SqlTypeFamily::of(k);
        if number.contains(&k) {
            assert_eq!(family, SqlTypeFamily::NUMBER, "{k}");
        } else if text.contains(&k) {
            assert_eq!(family, SqlTypeFamily::TEXT, "{k}");
        } else if collection.contains(&k) {
            assert_eq!(family, SqlTypeFamily::COLLECTION, "{k}");
        } else {
            assert_eq!(family.members(), &[k], "{k}");
        }
    }
    // `DiadicOperator`'s check (`DiadicOperator.kt:60`): `DATE` and `TIMESTAMP` are not one
    // family there, while `DATE_TIMESTAMP` groups them.
    assert!(!SqlTypeFamily::of(K::Date).contains(K::Timestamp));
    assert!(SqlTypeFamily::of(K::Integer).contains(K::Double));
}

fn shape(ty: Static) -> PartiqlShape {
    PartiqlNoIdShapeBuilder::default().new_static(ty)
}

#[test]
fn kind_per_static_type() {
    let row = StructType::new_row([StructField::new("a", shape(Static::Int32))]);
    let closed_set = StructType::new(
        [
            StructConstraint::Open(false),
            StructConstraint::Ordered(true),
            StructConstraint::Fields([StructField::new("a", shape(Static::Int32))].into()),
        ]
        .into(),
    );
    let map = MapType::new(shape(Static::String), shape(Static::Int32)).unwrap();
    let cases: Vec<(Static, Option<K>)> = vec![
        (Static::Int, None),
        (Static::Int8, Some(K::TinyInt)),
        (Static::Int16, Some(K::SmallInt)),
        (Static::Int32, Some(K::Integer)),
        (Static::Int64, Some(K::BigInt)),
        (Static::Bool, Some(K::Bool)),
        (Static::Decimal, None),
        (Static::DecimalP(10, 2), Some(K::Decimal)),
        (Static::Float32, Some(K::Real)),
        (Static::Float64, Some(K::Double)),
        (Static::String, Some(K::String)),
        (Static::StringFixed(3), Some(K::Char)),
        (Static::StringVarying(3), Some(K::Varchar)),
        (Static::DateTime, None),
        (Static::NumericP(10, 2), Some(K::Numeric)),
        (Static::Blob(8), Some(K::Blob)),
        (Static::Clob(8), Some(K::Clob)),
        (Static::Date, Some(K::Date)),
        (Static::Time(3), Some(K::Time)),
        (Static::TimeTz(3), Some(K::TimeTz)),
        (Static::Timestamp(6), Some(K::Timestamp)),
        (Static::TimestampTz(6), Some(K::TimestampTz)),
        (Static::Null, Some(K::Unknown)),
        (Static::Missing, Some(K::Unknown)),
        (Static::Variant(VariantType::new("ion")), Some(K::Variant)),
        (Static::Struct(row), Some(K::Row)),
        (Static::Struct(closed_set), Some(K::Row)),
        (Static::Struct(StructType::new_any()), Some(K::Struct)),
        (Static::Bag(BagType::new_any()), Some(K::Bag)),
        (Static::Array(ArrayType::new_any()), Some(K::Array)),
        (Static::Map(map), Some(K::Map)),
        (Static::Graph(), None),
    ];
    for (ty, kind) in cases {
        assert_eq!(ty.kind(), kind, "{ty:?}");
        let s = shape(ty.clone());
        assert_eq!(s.kind(), kind, "{ty:?}");
        // Nullability and node ids are not part of a kind.
        let non_null = PartiqlShapeBuilder::default().new_non_nullable_static(ty.clone());
        assert_eq!(non_null.kind(), kind, "{ty:?}");
        assert_eq!(s.expect_static().unwrap().kind(), kind, "{ty:?}");
    }
}

#[test]
fn kind_of_interval_follows_its_family() {
    use IntervalQualifier as Q;
    let ym = [Q::Year, Q::Month, Q::YearToMonth];
    let dt = [
        Q::Day,
        Q::Hour,
        Q::Minute,
        Q::Second,
        Q::DayToHour,
        Q::DayToMinute,
        Q::DayToSecond,
        Q::HourToMinute,
        Q::HourToSecond,
        Q::MinuteToSecond,
    ];
    for q in ym {
        let ty = Static::Interval(IntervalType::new(q, 2, 0));
        assert_eq!(ty.kind(), Some(K::IntervalYm), "{q:?}");
    }
    for q in dt {
        let ty = Static::Interval(IntervalType::new(q, 2, 6));
        assert_eq!(ty.kind(), Some(K::IntervalDt), "{q:?}");
    }
}

#[test]
fn kind_of_non_static_shapes() {
    let mut b = PartiqlNoIdShapeBuilder::default();
    assert_eq!(b.new_dynamic().kind(), Some(K::Dynamic));
    assert_eq!(b.new_undefined().kind(), None);
    let union = b.any_of([shape(Static::Int32), shape(Static::String)]);
    assert!(matches!(union, PartiqlShape::AnyOf(_)));
    assert_eq!(union.kind(), None);
}

/// PLK types every closed struct as `ROW`, also when its fields are unordered or there are none,
/// and only an open struct as `STRUCT` (`PlanTyper.kt:1434-1437`, `:1576`, `:1687-1691`;
/// `PType.struct()` carries no fields, `PType.java:1182-1184`).
#[test]
fn closed_structs_are_rows_open_structs_are_structs() {
    let a = || StructField::new("a", shape(Static::Int32));
    let closed_unordered = StructType::new(
        [
            StructConstraint::Open(false),
            StructConstraint::Fields([a()].into()),
        ]
        .into(),
    );
    let closed_empty = StructType::new([StructConstraint::Open(false)].into());
    let open_with_fields = StructType::new(
        [
            StructConstraint::Open(true),
            StructConstraint::Fields([a()].into()),
        ]
        .into(),
    );
    let no_open_constraint = StructType::new([StructConstraint::Fields([a()].into())].into());
    let cases = [
        (closed_unordered, K::Row),
        (closed_empty, K::Row),
        (StructType::new_row([]), K::Row),
        (open_with_fields, K::Struct),
        (no_open_constraint, K::Struct),
        (StructType::new_any(), K::Struct),
    ];
    for (s, kind) in cases {
        let ty = Static::Struct(s);
        assert_eq!(ty.kind(), Some(kind), "{ty:?}");
    }
    // So a closed table schema casts as PLK's ROW does.
    let closed = Static::Struct(StructType::new(
        [
            StructConstraint::Open(false),
            StructConstraint::Fields([a()].into()),
        ]
        .into(),
    ));
    let k = closed.kind().unwrap();
    assert!(can_cast(k, K::Row));
    assert!(!can_cast(k, K::Struct));
}

/// The properties a sort with `compare_fn_precedence` relies on, over every pair and triple of
/// kinds.
#[test]
fn compare_fn_precedence_is_a_partial_order() {
    let unranked = |k: K| matches!(k, K::Unknown | K::Variant);
    for a in K::ALL.iter().copied() {
        assert_eq!(compare_fn_precedence(a, a), Some(Ordering::Equal), "{a}");
        for b in K::ALL.iter().copied() {
            let ab = compare_fn_precedence(a, b);
            // `None` exactly when the kinds differ and one has no precedence.
            assert_eq!(
                ab.is_none(),
                a != b && (unranked(a) || unranked(b)),
                "{a} {b}"
            );
            // Antisymmetric, and `Equal` only for equal kinds.
            assert_eq!(
                ab,
                compare_fn_precedence(b, a).map(Ordering::reverse),
                "{a} {b}"
            );
            if a != b {
                assert_ne!(ab, Some(Ordering::Equal), "{a} {b}");
            }
            for c in K::ALL.iter().copied() {
                // Transitive.
                if ab == Some(Ordering::Less) && compare_fn_precedence(b, c) == Some(Ordering::Less)
                {
                    assert_eq!(
                        compare_fn_precedence(a, c),
                        Some(Ordering::Less),
                        "{a} {b} {c}"
                    );
                }
            }
        }
    }
}
