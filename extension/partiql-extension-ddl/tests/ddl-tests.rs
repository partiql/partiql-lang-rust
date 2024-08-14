use indexmap::IndexSet;
use partiql_extension_ddl::ddl::{DdlFormat, PartiqlBasicDdlEncoder, PartiqlDdlEncoder};
use partiql_types::{
    struct_fields, type_bag, type_int, type_string, type_struct, PartiqlShapeBuilder,
    StructConstraint, StructField,
};
use partiql_types::{BagType, Static, StructType};

#[test]
fn basic_ddl_test() {
    let details_fields = struct_fields![("age", type_int!())];
    let details = type_struct![IndexSet::from([details_fields])];
    let fields = [
        StructField::new("id", type_int!()),
        StructField::new("name", type_string!()),
        StructField::new(
            "address",
            PartiqlShapeBuilder::init_or_get().new_non_nullable_static(Static::String),
        ),
        StructField::new_optional("details", details.clone()),
    ]
    .into();
    let shape = type_bag![type_struct![IndexSet::from([
        StructConstraint::Fields(fields),
        StructConstraint::Open(false)
    ])]];

    let ddl_compact = PartiqlBasicDdlEncoder::new(DdlFormat::Compact);
    let actual = ddl_compact.ddl(&shape).expect("ddl_output");
    let expected = r#""id" INT,"name" VARCHAR,"address" VARCHAR NOT NULL,"details" OPTIONAL STRUCT<"age": INT>"#;

    println!("Actual: {actual}");
    println!("Expected: {expected}");

    assert_eq!(actual, expected);
}
