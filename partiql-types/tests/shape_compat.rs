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

#[test]
fn equal_structs_hash_equally_whatever_the_constraint_order() {
    use std::collections::hash_map::DefaultHasher;
    use std::hash::{Hash, Hasher};
    let hash = |s: &StructType| {
        let mut h = DefaultHasher::new();
        s.hash(&mut h);
        h.finish()
    };
    let mut bld = PartiqlNoIdShapeBuilder::default();
    let fields = IndexSet::from([
        StructField::new("a", type_int32!(bld)),
        StructField::new("b", type_string!(bld)),
    ]);
    let reversed: IndexSet<_> = fields.iter().rev().cloned().collect();
    let one = StructType::new(IndexSet::from([
        StructConstraint::Fields(fields),
        StructConstraint::Open(false),
    ]));
    let other = StructType::new(IndexSet::from([
        StructConstraint::Open(false),
        StructConstraint::Fields(reversed),
    ]));
    // `IndexSet` equality ignores order, so the hash must too.
    assert_eq!(one, other);
    assert_eq!(hash(&one), hash(&other));
}

#[test]
fn equal_unions_hash_equally_whatever_the_member_order() {
    use partiql_types::AnyOf;
    use std::collections::hash_map::DefaultHasher;
    use std::collections::HashSet;
    use std::hash::{Hash, Hasher};
    let hash = |s: &PartiqlShape| {
        let mut h = DefaultHasher::new();
        s.hash(&mut h);
        h.finish()
    };
    let mut bld = PartiqlNoIdShapeBuilder::default();
    let ab = PartiqlShape::AnyOf(AnyOf::new(IndexSet::from([
        type_int32!(bld),
        type_string!(bld),
    ])));
    let ba = PartiqlShape::AnyOf(AnyOf::new(IndexSet::from([
        type_string!(bld),
        type_int32!(bld),
    ])));
    assert_eq!(ab, ba);
    assert_eq!(hash(&ab), hash(&ba));
    let set: HashSet<_> = [ab.clone(), ba.clone()].into();
    assert_eq!(set.len(), 1);

    // The same unions as field types inside struct field sets.
    let s1 = StructType::new(IndexSet::from([StructConstraint::Fields(IndexSet::from(
        [StructField::new("u", ab)],
    ))]));
    let s2 = StructType::new(IndexSet::from([StructConstraint::Fields(IndexSet::from(
        [StructField::new("u", ba)],
    ))]));
    assert_eq!(s1, s2);
}
