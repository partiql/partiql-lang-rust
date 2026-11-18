use indexmap::IndexSet;
use partiql_extension_ddl::ddl::{DdlFormat, PartiqlBasicDdlEncoder, PartiqlDdlEncoder};
use partiql_types::{
    struct_fields, type_bag, type_int, type_string, type_struct, PartiqlShapeBuilder,
    StructConstraint, StructField,
};
use partiql_types::{Static, StructType};

#[test]
fn basic_ddl_test() {
    let mut bld = PartiqlShapeBuilder::default();
    let details_fields = struct_fields![("age", type_int!(bld))];
    let details = type_struct![bld, IndexSet::from([details_fields])];
    let fields = [
        StructField::new("id", type_int!(bld)),
        StructField::new("name", type_string!(bld)),
        StructField::new("address", bld.new_non_nullable_static(Static::String)),
        StructField::new_optional("details", details.clone()),
    ]
    .into();
    let shape = type_bag![
        bld,
        type_struct![
            bld,
            IndexSet::from([
                StructConstraint::Fields(fields),
                StructConstraint::Open(false)
            ])
        ]
    ];

    let ddl_compact = PartiqlBasicDdlEncoder::new(DdlFormat::Compact);
    let actual = ddl_compact.ddl(&shape).expect("ddl_output");
    let expected = r#""id" INT,"name" VARCHAR,"address" VARCHAR NOT NULL,"details" OPTIONAL STRUCT<"age": INT>"#;

    println!("Actual: {actual}");
    println!("Expected: {expected}");

    assert_eq!(actual, expected);
}

/// DDL of a one-column table whose column has the given shape.
fn column_ddl(column: partiql_types::PartiqlShape) -> String {
    let mut bld = PartiqlShapeBuilder::default();
    let fields = [StructField::new("c", column)].into();
    let shape = type_bag![
        bld,
        type_struct![
            bld,
            IndexSet::from([
                StructConstraint::Fields(fields),
                StructConstraint::Open(false)
            ])
        ]
    ];
    PartiqlBasicDdlEncoder::new(DdlFormat::Compact)
        .ddl(&shape)
        .expect("ddl_output")
}

#[test]
fn ddl_of_new_kinds() {
    use partiql_types::{IntervalQualifier, IntervalType};
    let mut bld = PartiqlShapeBuilder::default();
    let interval = |q, p, fp| Static::Interval(IntervalType::new(q, p, fp));
    let cases = [
        (Static::StringFixed(3), "CHAR(3)"),
        (Static::StringVarying(5), "VARCHAR(5)"),
        (Static::NumericP(10, 2), "NUMERIC(10, 2)"),
        (Static::Blob(16), "BLOB(16)"),
        (Static::Clob(16), "CLOB(16)"),
        (Static::Date, "DATE"),
        (Static::Time(3), "TIME(3)"),
        (Static::TimeTz(3), "TIME(3) WITH TIME ZONE"),
        (Static::Timestamp(6), "TIMESTAMP(6)"),
        (Static::TimestampTz(6), "TIMESTAMP(6) WITH TIME ZONE"),
        (Static::Variant("ion".to_string()), "VARIANT('ion')"),
        (interval(IntervalQualifier::Year, 4, 0), "INTERVAL YEAR(4)"),
        (
            interval(IntervalQualifier::Month, 2, 0),
            "INTERVAL MONTH(2)",
        ),
        (
            interval(IntervalQualifier::YearToMonth, 2, 0),
            "INTERVAL YEAR(2) TO MONTH",
        ),
        (interval(IntervalQualifier::Day, 2, 0), "INTERVAL DAY(2)"),
        (interval(IntervalQualifier::Hour, 2, 0), "INTERVAL HOUR(2)"),
        (
            interval(IntervalQualifier::Minute, 2, 0),
            "INTERVAL MINUTE(2)",
        ),
        (
            interval(IntervalQualifier::Second, 2, 6),
            "INTERVAL SECOND(2, 6)",
        ),
        (
            interval(IntervalQualifier::DayToHour, 2, 0),
            "INTERVAL DAY(2) TO HOUR",
        ),
        (
            interval(IntervalQualifier::DayToMinute, 2, 0),
            "INTERVAL DAY(2) TO MINUTE",
        ),
        (
            interval(IntervalQualifier::DayToSecond, 2, 6),
            "INTERVAL DAY(2) TO SECOND(6)",
        ),
        (
            interval(IntervalQualifier::HourToMinute, 2, 0),
            "INTERVAL HOUR(2) TO MINUTE",
        ),
        (
            interval(IntervalQualifier::HourToSecond, 2, 3),
            "INTERVAL HOUR(2) TO SECOND(3)",
        ),
        (
            interval(IntervalQualifier::MinuteToSecond, 2, 3),
            "INTERVAL MINUTE(2) TO SECOND(3)",
        ),
    ];
    for (kind, ddl) in cases {
        assert_eq!(
            column_ddl(bld.new_static(kind.clone())),
            format!("\"c\" {ddl}")
        );
        assert_eq!(
            column_ddl(bld.new_non_nullable_static(kind)),
            format!("\"c\" {ddl} NOT NULL")
        );
    }

    let (k, v) = (type_string!(bld), type_int!(bld));
    assert_eq!(column_ddl(bld.new_map(k, v)), r#""c" MAP<VARCHAR,INT>"#);

    let (a1, a2) = (type_int!(bld), type_string!(bld));
    let row = bld.new_row([StructField::new("a", a1), StructField::new("a", a2)]);
    assert_eq!(column_ddl(row), r#""c" STRUCT<"a": INT,"a": VARCHAR>"#);
}

#[test]
fn ddl_omits_unspecified_parameters() {
    use partiql_types::{IntervalQualifier, IntervalType, TypeParams};
    let mut bld = PartiqlShapeBuilder::default();
    let length = TypeParams::default().with_unspecified_length(true);
    let precision = TypeParams::default()
        .with_unspecified_precision(true)
        .with_unspecified_scale(true);
    let precision_only = TypeParams::default().with_unspecified_precision(true);
    let scale = TypeParams::default().with_unspecified_scale(true);
    let fraction = TypeParams::default().with_unspecified_fractional_precision(true);
    let cases = [
        (Static::StringVarying(1), length, "VARCHAR"),
        (Static::StringFixed(1), length, "CHAR"),
        (Static::Blob(1), length, "BLOB"),
        (Static::Clob(1), length, "CLOB"),
        (Static::NumericP(38, 0), precision, "NUMERIC"),
        (Static::NumericP(10, 0), scale, "NUMERIC(10)"),
        (Static::NumericP(38, 2), precision_only, "NUMERIC(38, 2)"),
        (Static::DecimalP(38, 0), precision, "DECIMAL"),
        (Static::DecimalP(10, 0), scale, "DECIMAL(10)"),
        (Static::DecimalP(38, 2), precision_only, "DECIMAL(38, 2)"),
        (
            Static::DecimalP(10, 2),
            TypeParams::default(),
            "DECIMAL(10, 2)",
        ),
        (Static::Time(6), precision, "TIME"),
        (
            Static::TimestampTz(6),
            precision,
            "TIMESTAMP WITH TIME ZONE",
        ),
        (
            Static::Interval(IntervalType::new(IntervalQualifier::DayToSecond, 2, 6)),
            fraction,
            "INTERVAL DAY(2) TO SECOND",
        ),
        (
            Static::Interval(IntervalType::new(IntervalQualifier::DayToSecond, 2, 6)),
            precision,
            "INTERVAL DAY TO SECOND(6)",
        ),
        (
            Static::Interval(IntervalType::new(IntervalQualifier::Second, 2, 6)),
            fraction,
            "INTERVAL SECOND(2)",
        ),
        (
            Static::Interval(IntervalType::new(IntervalQualifier::Second, 2, 6)),
            precision.with_unspecified_fractional_precision(true),
            "INTERVAL SECOND",
        ),
        (
            Static::Interval(IntervalType::new(IntervalQualifier::Second, 2, 3)),
            precision_only,
            "INTERVAL SECOND(2, 3)",
        ),
        (
            Static::Interval(IntervalType::new(IntervalQualifier::Year, 2, 0)),
            precision,
            "INTERVAL YEAR",
        ),
    ];
    for (kind, params, ddl) in cases {
        let shape = bld.new_static_with_params(kind, params);
        assert_eq!(column_ddl(shape), format!("\"c\" {ddl}"));
    }
}

#[test]
fn ddl_of_an_unsupported_kind_is_an_error() {
    let mut bld = PartiqlShapeBuilder::default();
    let graph = bld.new_graph();
    let fields = [StructField::new("g", graph)].into();
    let shape = type_bag![
        bld,
        type_struct![
            bld,
            IndexSet::from([
                StructConstraint::Fields(fields),
                StructConstraint::Open(false)
            ])
        ]
    ];
    let err = PartiqlBasicDdlEncoder::new(DdlFormat::Compact)
        .ddl(&shape)
        .expect_err("graph has no DDL");
    assert!(err.to_string().contains("Graph"), "{err}");
}

/// DDL of a one-column table whose column has the given shape, or the encoding error.
fn try_column_ddl(
    name: &str,
    column: partiql_types::PartiqlShape,
) -> Result<String, partiql_extension_ddl::ddl::ShapeEncodingError> {
    let mut bld = PartiqlShapeBuilder::default();
    let fields = [StructField::new(name, column)].into();
    let shape = type_bag![
        bld,
        type_struct![
            bld,
            IndexSet::from([
                StructConstraint::Fields(fields),
                StructConstraint::Open(false)
            ])
        ]
    ];
    PartiqlBasicDdlEncoder::new(DdlFormat::Compact).ddl(&shape)
}

#[test]
fn ddl_of_absent_value_types_is_an_error() {
    // The absent-value types have no DDL form: e.g. `NULL` after a column name would read as a
    // constraint, not a type. This holds wherever they appear, including inside a UNION.
    let mut bld = PartiqlShapeBuilder::default();
    for kind in [Static::Null, Static::Missing] {
        for shape in [
            bld.new_static(kind.clone()),
            bld.new_non_nullable_static(kind.clone()),
        ] {
            let err = try_column_ddl("c", shape).expect_err("no DDL");
            assert!(err.to_string().contains("unsupported"), "{err}");
        }
    }
}

#[test]
fn ddl_of_nested_maps_and_rows() {
    let mut bld = PartiqlShapeBuilder::default();
    let (k, v) = (type_string!(bld), type_int!(bld));
    let map = bld.new_map(k, v);
    let row = bld.new_row([StructField::new("m", map)]);
    let k = type_string!(bld);
    let outer = bld.new_map(k, row);
    let bag = type_bag![bld, outer];
    assert_eq!(
        try_column_ddl("c", bag).unwrap(),
        r#""c" BAG<MAP<VARCHAR,STRUCT<"m": MAP<VARCHAR,INT>>>>"#
    );
}

#[test]
fn ddl_rejects_text_it_cannot_quote_unambiguously() {
    let mut bld = PartiqlShapeBuilder::default();
    // Spaces and commas are fine inside a quoted name.
    assert_eq!(
        try_column_ddl("a b, c", type_int!(bld)).unwrap(),
        r#""a b, c" INT"#
    );
    // A quote, backslash or control character could end the identifier early under SQL or
    // PartiQL escaping rules (e.g. `a" INT,"b` used to produce two columns).
    for name in [r#"a" INT,"b"#, r"a\", "a\nb", "a\u{7}"] {
        let err = try_column_ddl(name, type_int!(bld)).expect_err(name);
        assert!(err.to_string().contains("quote, backslash"), "{err}");
        let inner = struct_fields![(name, type_int!(bld))];
        let nested = type_struct![bld, IndexSet::from([inner])];
        let err = try_column_ddl("s", nested).expect_err(name);
        assert!(err.to_string().contains("quote, backslash"), "{err}");
        let i = type_int!(bld);
        let row = bld.new_row([StructField::new(name, i)]);
        let (k, a) = (type_string!(bld), bld.new_array_of(row));
        let err = try_column_ddl("m", bld.new_map(k, a)).expect_err(name);
        assert!(err.to_string().contains("quote, backslash"), "{err}");
    }
    for encoding in ["it's", r"x\"] {
        let variant = bld.new_static(Static::Variant(encoding.to_string()));
        let err = try_column_ddl("c", variant).expect_err(encoding);
        assert!(err.to_string().contains("quote, backslash"), "{err}");
    }
}

#[test]
fn ddl_rejects_duplicate_column_names() {
    let mut bld = PartiqlShapeBuilder::default();
    let (a1, a2) = (type_int!(bld), type_string!(bld));
    let row = bld.new_row([StructField::new("a", a1), StructField::new("a", a2)]);
    let table = type_bag![bld, row];
    let err = PartiqlBasicDdlEncoder::new(DdlFormat::Compact)
        .ddl(&table)
        .expect_err("duplicate columns");
    assert!(err.to_string().contains("duplicate column name"), "{err}");
}
