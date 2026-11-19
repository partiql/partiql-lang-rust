//! Compatibility tests that use only the `partiql-types` API that existed before the new
//! static kinds, row and map types and unspecified-parameter flags were added, so that they run
//! unchanged against the previous release as well.

use indexmap::IndexSet;
use partiql_types::{
    type_bag, type_int, type_int32, type_string, type_struct, PartiqlNoIdShapeBuilder,
    PartiqlShape, PartiqlShapeBuilder, Static, StructConstraint, StructField, StructType,
};

#[test]
fn debug_text_of_existing_shapes_is_unchanged() {
    let mut bld = PartiqlNoIdShapeBuilder::default();
    assert_eq!(
        format!("{:?}", type_int!(bld)),
        "Static(StaticType { id: NodeId(0), ty: Int, nullable: true })"
    );
    assert_eq!(
        format!("{:?}", bld.new_non_nullable_static(Static::String)),
        "Static(StaticType { id: NodeId(0), ty: String, nullable: false })"
    );
    assert_eq!(
        format!("{:?}", bld.new_static(Static::DecimalP(10, 2))),
        "Static(StaticType { id: NodeId(0), ty: DecimalP(10, 2), nullable: true })"
    );

    let a = type_int32!(bld);
    let row = type_struct![
        bld,
        IndexSet::from([
            StructConstraint::Fields(IndexSet::from([StructField::new("a", a)])),
            StructConstraint::Open(false),
        ])
    ];
    let bag = type_bag![bld, row];
    assert_eq!(
        format!("{bag:?}"),
        "Static(StaticType { id: NodeId(0), ty: Bag(BagType { element_type: \
         Static(StaticType { id: NodeId(0), ty: Struct(StructType { constraints: \
         {Fields({StructField { optional: false, name: \"a\", ty: Static(StaticType { \
         id: NodeId(0), ty: Int32, nullable: true }) }}), Open(false)} }), nullable: true }) }), \
         nullable: true })"
    );

    let mut auto = PartiqlShapeBuilder::default();
    let s = type_string!(auto);
    let PartiqlShape::Static(st) = &s else {
        panic!("expected a static shape")
    };
    assert_eq!(
        format!("{s:?}"),
        format!(
            "Static(StaticType {{ id: {:?}, ty: String, nullable: true }})",
            st.ty_id()
        )
    );
}

#[test]
fn display_of_char_and_varchar_does_not_panic() {
    assert_eq!(format!("{}", Static::StringFixed(3)), "Char(3)");
    assert_eq!(format!("{}", Static::StringVarying(5)), "Varchar(5)");
    let mut bld = PartiqlNoIdShapeBuilder::default();
    assert_eq!(
        format!("{}", bld.new_non_nullable_static(Static::StringVarying(5))),
        "NOT NULL Varchar(5)"
    );
}

#[test]
fn equality_and_hash_of_existing_shapes_is_unchanged() {
    use std::collections::HashSet;

    // No-id shapes compare structurally.
    let mut bld = PartiqlNoIdShapeBuilder::default();
    assert_eq!(type_int32!(bld), type_int32!(bld));
    let set: HashSet<_> = [type_int32!(bld), type_int32!(bld), type_string!(bld)].into();
    assert_eq!(set.len(), 2);

    // Shapes with generated ids differ by id.
    let mut auto = PartiqlShapeBuilder::default();
    assert_ne!(type_int32!(auto), type_int32!(auto));
}
