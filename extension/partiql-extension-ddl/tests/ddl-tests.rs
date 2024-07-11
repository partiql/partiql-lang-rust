use indexmap::IndexSet;
use partiql_extension_ddl::ddl::{DdlFormat, PartiqlBasicDdlEncoder, PartiqlDdlEncoder};
use partiql_types::{bag, int, r#struct, str, struct_fields, StructConstraint, StructField};
use partiql_types::{BagType, PartiqlShape, Static, StructType};

#[test]
fn basic_ddl_test() {
    let details_fields = struct_fields![("age", int!())];
    let details = r#struct![IndexSet::from([details_fields])];
    let fields = [
        StructField::new("id", int!()),
        StructField::new("name", str!()),
        StructField::new("address", PartiqlShape::new_non_nullable(Static::String)),
        StructField::new_optional("details", details.clone()),
    ]
    .into();
    let shape = bag![r#struct![IndexSet::from([
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
