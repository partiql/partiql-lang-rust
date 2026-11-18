use indexmap::IndexSet;
use partiql_extension_ddl::ddl::{DdlFormat, PartiqlBasicDdlEncoder, PartiqlDdlEncoder};
use partiql_types::{
    struct_fields, type_bag, type_int, type_string, type_struct, PartiqlShapeBuilder,
    StructConstraint, StructField,
};
use partiql_types::{Static, StructType, VariantType};

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
        (Static::Variant(VariantType::new("ion")), "VARIANT('ion')"),
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
    assert_eq!(
        column_ddl(bld.new_map(k, v).unwrap()),
        r#""c" MAP<VARCHAR,INT>"#
    );

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
        // partiql-lang-kotlin's planner defaults: BLOB/CLOB length i32::MAX, TIME precision 0.
        (Static::Blob(i32::MAX as usize), length, "BLOB"),
        (Static::Clob(i32::MAX as usize), length, "CLOB"),
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
        (Static::Time(0), precision, "TIME"),
        (Static::TimeTz(0), precision, "TIME WITH TIME ZONE"),
        (Static::Timestamp(6), precision, "TIMESTAMP"),
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
fn ddl_keeps_parameters_whose_flag_does_not_match_the_payload() {
    // A flag on a value that is not the parameter's default (or on a kind without that
    // parameter) is cleared when the shape is built, so the specified value is not lost.
    use partiql_types::{IntervalQualifier, IntervalType, TypeParams};
    let mut bld = PartiqlShapeBuilder::default();
    let all = TypeParams::default()
        .with_unspecified_length(true)
        .with_unspecified_precision(true)
        .with_unspecified_scale(true)
        .with_unspecified_fractional_precision(true);
    let cases = [
        (Static::StringVarying(50), "VARCHAR(50)"),
        (Static::StringFixed(2), "CHAR(2)"),
        (Static::Blob(1), "BLOB(1)"),
        (Static::Clob(16), "CLOB(16)"),
        (Static::DecimalP(10, 2), "DECIMAL(10, 2)"),
        (Static::NumericP(38, 2), "NUMERIC(38, 2)"),
        (Static::Time(6), "TIME(6)"),
        (Static::Timestamp(3), "TIMESTAMP(3)"),
        (Static::TimestampTz(0), "TIMESTAMP(0) WITH TIME ZONE"),
        (
            Static::Interval(IntervalType::new(IntervalQualifier::DayToSecond, 4, 3)),
            "INTERVAL DAY(4) TO SECOND(3)",
        ),
        (Static::Int32, "INTEGER"),
    ];
    for (kind, ddl) in cases {
        let shape = bld.new_static_with_params(kind.clone(), all);
        assert_eq!(column_ddl(shape), format!("\"c\" {ddl}"), "{kind}");
    }
    // Flags a kind does not have are dropped.
    let int = bld.new_static_with_params(Static::Int32, all);
    assert!(int.expect_static().unwrap().params().is_default());
    let year = Static::Interval(IntervalType::new(IntervalQualifier::Year, 2, 0));
    let year = bld.new_static_with_params(year, all);
    let params = *year.expect_static().unwrap().params();
    // As in partiql-lang-kotlin, the fractional flag is kept for any qualifier.
    assert!(params.unspecified_precision && params.unspecified_fractional_precision);
}

#[test]
fn ddl_of_string_and_datetime_is_lossy_as_before() {
    // `String` and `DateTime` print as on main, the same text as unspecified VARCHAR/TIMESTAMP.
    use partiql_types::TypeParams;
    let mut bld = PartiqlShapeBuilder::default();
    let unspecified_varchar = bld.new_static_with_params(
        Static::StringVarying(1),
        TypeParams::default().with_unspecified_length(true),
    );
    assert_eq!(
        column_ddl(type_string!(bld)),
        column_ddl(unspecified_varchar)
    );
    let unspecified_ts = bld.new_static_with_params(
        Static::Timestamp(6),
        TypeParams::default().with_unspecified_precision(true),
    );
    assert_eq!(
        column_ddl(bld.new_static(Static::DateTime)),
        column_ddl(unspecified_ts)
    );
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
    let map = bld.new_map(k, v).unwrap();
    let row = bld.new_row([StructField::new("m", map)]);
    let k = type_string!(bld);
    let outer = bld.new_map(k, row).unwrap();
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
    // Control and Unicode format characters cannot end a quoted name; they are kept.
    for name in ["tab\tname", "new\nline", "rtl\u{202e}x"] {
        assert_eq!(
            try_column_ddl(name, type_int!(bld)).unwrap(),
            format!("\"{name}\" INT")
        );
    }
    // A quote or backslash could end the identifier early under SQL or PartiQL escaping rules
    // (e.g. `a" INT,"b` used to produce two columns).
    for name in [r#"a" INT,"b"#, r"a\", r#"q"uote"#] {
        let err = try_column_ddl(name, type_int!(bld)).expect_err(name);
        assert!(err.to_string().contains("quote or backslash"), "{err}");
        let inner = struct_fields![(name, type_int!(bld))];
        let nested = type_struct![bld, IndexSet::from([inner])];
        let err = try_column_ddl("s", nested).expect_err(name);
        assert!(err.to_string().contains("quote or backslash"), "{err}");
        let i = type_int!(bld);
        let row = bld.new_row([StructField::new(name, i)]);
        let (k, a) = (type_string!(bld), bld.new_array_of(row));
        let err = try_column_ddl("m", bld.new_map(k, a).unwrap()).expect_err(name);
        assert!(err.to_string().contains("quote or backslash"), "{err}");
    }
    for encoding in ["it's", r"x\"] {
        let variant = bld.new_static(Static::Variant(VariantType::new(encoding)));
        let err = try_column_ddl("c", variant).expect_err(encoding);
        assert!(err.to_string().contains("quote or backslash"), "{err}");
    }
}

#[test]
fn ddl_writes_duplicate_column_names_as_main_does() {
    let encoder = PartiqlBasicDdlEncoder::new(DdlFormat::Compact);
    let mut bld = PartiqlShapeBuilder::default();
    // A row with a repeated name (`SELECT a, a`) encodes like a set-based struct does on main.
    let (a1, a2) = (type_int!(bld), type_string!(bld));
    let row = bld.new_row([StructField::new("a", a1), StructField::new("a", a2)]);
    let table = type_bag![bld, row];
    assert_eq!(encoder.ddl(&table).unwrap(), r#""a" INT,"a" VARCHAR"#);

    let (a1, a2) = (type_int!(bld), type_string!(bld));
    let fields = [StructField::new("a", a1), StructField::new("a", a2)].into();
    let set_based = type_bag![
        bld,
        type_struct![
            bld,
            IndexSet::from([
                StructConstraint::Fields(fields),
                StructConstraint::Open(false)
            ])
        ]
    ];
    assert_eq!(encoder.ddl(&set_based).unwrap(), r#""a" INT,"a" VARCHAR"#);
}

#[test]
fn ddl_of_nested_absent_value_types_is_an_error() {
    let mut bld = PartiqlShapeBuilder::default();
    for kind in [Static::Null, Static::Missing] {
        let i = type_int!(bld);
        let absent = bld.new_static(kind.clone());
        let union = bld.any_of([i, absent]);
        let absent = bld.new_static(kind.clone());
        let array = bld.new_array_of(absent);
        let (k, absent) = (type_string!(bld), bld.new_static(kind.clone()));
        let map = bld.new_map(k, absent).unwrap();
        let absent = bld.new_static(kind.clone());
        let nested = type_struct![bld, IndexSet::from([struct_fields![("x", absent)]])];
        for shape in [union, array, map, nested] {
            let err = try_column_ddl("c", shape).expect_err("no DDL");
            assert!(err.to_string().contains("unsupported"), "{err}");
        }
    }
}
