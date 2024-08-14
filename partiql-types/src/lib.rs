#![deny(rust_2018_idioms)]
#![deny(clippy::all)]

use derivative::Derivative;
use indexmap::IndexSet;
use itertools::Itertools;
use miette::Diagnostic;
use partiql_common::node::{AutoNodeIdGenerator, NodeId, NodeIdGenerator};
use std::collections::HashMap;
use std::fmt::{Debug, Display, Formatter};
use std::hash::{Hash, Hasher};
use std::sync::OnceLock;
use thiserror::Error;

#[derive(Debug, Clone, Eq, PartialEq, Hash, Error, Diagnostic)]
#[error("ShapeResult Error")]
#[non_exhaustive]
pub enum ShapeResultError {
    #[error("Unexpected type `{0:?}` for static type bool")]
    UnexpectedType(String),
}

/// Result of attempts to encode to Ion.
pub type ShapeResult<T> = Result<T, ShapeResultError>;

pub trait Type {}

impl Type for StaticType {}

fn indexset_hash<H, T>(set: &IndexSet<T>, state: &mut H)
where
    H: Hasher,
    T: Hash,
{
    for val in set {
        val.hash(state)
    }
}

#[macro_export]
macro_rules! type_dynamic {
    () => {
        $crate::PartiqlShapeBuilder::init_or_get().new_dynamic()
    };
}

#[macro_export]
macro_rules! type_int {
    () => {
        $crate::PartiqlShapeBuilder::init_or_get().new_static($crate::Static::Int)
    };
}

#[macro_export]
macro_rules! type_int8 {
    () => {
        $crate::PartiqlShapeBuilder::init_or_get().new_static($crate::Static::Int8)
    };
}

#[macro_export]
macro_rules! type_int16 {
    () => {
        $crate::PartiqlShapeBuilder::init_or_get().new_static($crate::Static::Int16)
    };
}

#[macro_export]
macro_rules! type_int32 {
    () => {
        $crate::PartiqlShapeBuilder::init_or_get().new_static($crate::Static::Int32)
    };
}

#[macro_export]
macro_rules! type_int64 {
    () => {
        $crate::PartiqlShapeBuilder::init_or_get().new_static($crate::Static::Int64)
    };
}

#[macro_export]
macro_rules! type_decimal {
    () => {
        $crate::PartiqlShapeBuilder::init_or_get().new_static($crate::Static::Decimal)
    };
}

// TODO add macro_rule for Decimal with precision and scale

#[macro_export]
macro_rules! type_float32 {
    () => {
        $crate::PartiqlShapeBuilder::init_or_get().new_static($crate::Static::Float32)
    };
}

#[macro_export]
macro_rules! type_float64 {
    () => {
        $crate::PartiqlShapeBuilder::init_or_get().new_static($crate::Static::Float64)
    };
}

#[macro_export]
macro_rules! type_string {
    () => {
        $crate::PartiqlShapeBuilder::init_or_get().new_static($crate::Static::String)
    };
}

#[macro_export]
macro_rules! type_bool {
    () => {
        $crate::PartiqlShapeBuilder::init_or_get().new_static($crate::Static::Bool)
    };
}

#[macro_export]
macro_rules! type_numeric {
    () => {
        [
            $crate::PartiqlShapeBuilder::init_or_get().new_static($crate::Static::Int),
            $crate::PartiqlShapeBuilder::init_or_get().new_static($crate::Static::Float32),
            $crate::PartiqlShapeBuilder::init_or_get().new_static($crate::Static::Float64),
            $crate::PartiqlShapeBuilder::init_or_get().new_static($crate::Static::Decimal),
        ]
    };
}

#[macro_export]
macro_rules! type_datetime {
    () => {
        $crate::PartiqlShapeBuilder::init_or_get().new_static($crate::Static::DateTime)
    };
}

#[macro_export]
macro_rules! type_struct {
    () => {
        $crate::PartiqlShapeBuilder::init_or_get().new_struct(StructType::new_any())
    };
    ($elem:expr) => {
        $crate::PartiqlShapeBuilder::init_or_get().new_struct(StructType::new($elem))
    };
}

#[macro_export]
macro_rules! struct_fields {
    ($(($x:expr, $y:expr)),+ $(,)?) => (
        $crate::StructConstraint::Fields([$(($x, $y).into()),+].into())
    );
}

#[macro_export]
macro_rules! type_bag {
    () => {
        $crate::PartiqlShapeBuilder::init_or_get().new_bag(BagType::new_any());
    };
    ($elem:expr) => {
        $crate::PartiqlShapeBuilder::init_or_get().new_bag(BagType::new(Box::new($elem)))
    };
}

#[macro_export]
macro_rules! type_array {
    () => {
        $crate::PartiqlShapeBuilder::init_or_get().new_array(ArrayType::new_any());
    };
    ($elem:expr) => {
        $crate::PartiqlShapeBuilder::init_or_get().new_array(ArrayType::new(Box::new($elem)))
    };
}

#[macro_export]
macro_rules! type_undefined {
    () => {
        $crate::PartiqlShape::Undefined
    };
}

// Types with constant `NodeId`, e.g., `NodeId(1)` convenient for testing or use-cases with no
// requirement for unique node ids.

#[macro_export]
macro_rules! type_int_with_const_id {
    () => {
        $crate::PartiqlShapeBuilder::init_or_get().new_static_with_const_id($crate::Static::Int)
    };
}

#[macro_export]
macro_rules! type_float32_with_const_id {
    () => {
        $crate::PartiqlShapeBuilder::init_or_get().new_static_with_const_id($crate::Static::Float32)
    };
}

#[macro_export]
macro_rules! type_string_with_const_id {
    () => {
        $crate::PartiqlShapeBuilder::init_or_get().new_static_with_const_id($crate::Static::String)
    };
}

#[macro_export]
macro_rules! type_struct_with_const_id {
    () => {
        $crate::PartiqlShapeBuilder::init_or_get()
            .new_static_with_const_id(Static::Struct(StructType::new_any()))
    };
    ($elem:expr) => {
        $crate::PartiqlShapeBuilder::init_or_get()
            .new_static_with_const_id(Static::Struct(StructType::new($elem)))
    };
}

/// Represents a PartiQL Shape
#[derive(Debug, Clone, Eq, PartialEq, Hash)]
// With this implementation `Dynamic` and `AnyOf` cannot have `nullability`; this does not mean their
// `null` value at runtime cannot belong to their domain.
// TODO adopt the correct model Pending PartiQL Types semantics finalization: https://github.com/partiql/partiql-lang/issues/18
pub enum PartiqlShape {
    Dynamic,
    AnyOf(AnyOf),
    Static(StaticType),
    Undefined,
}

#[allow(dead_code)]
impl PartiqlShape {
    #[must_use]
    pub fn union_with(self, other: PartiqlShape) -> PartiqlShape {
        match (self, other) {
            (PartiqlShape::Dynamic, _) | (_, PartiqlShape::Dynamic) => PartiqlShape::Dynamic,
            (PartiqlShape::AnyOf(lhs), PartiqlShape::AnyOf(rhs)) => {
                PartiqlShapeBuilder::init_or_get().any_of(lhs.types.into_iter().chain(rhs.types))
            }
            (PartiqlShape::AnyOf(anyof), other) | (other, PartiqlShape::AnyOf(anyof)) => {
                let mut types = anyof.types;
                types.insert(other);
                PartiqlShapeBuilder::init_or_get().any_of(types)
            }
            (l, r) => {
                let types = [l, r];
                PartiqlShapeBuilder::init_or_get().any_of(types)
            }
        }
    }

    #[must_use]
    pub fn static_type_id(&self) -> Option<NodeId> {
        if let PartiqlShape::Static(StaticType { id, .. }) = self {
            Some(*id)
        } else {
            None
        }
    }

    #[must_use]
    pub fn is_string(&self) -> bool {
        matches!(
            &self,
            PartiqlShape::Static(StaticType {
                ty: Static::String,
                nullable: true,
                ..
            })
        )
    }

    #[must_use]
    pub fn is_struct(&self) -> bool {
        matches!(
            *self,
            PartiqlShape::Static(StaticType {
                ty: Static::Struct(_),
                nullable: true,
                ..
            })
        )
    }

    #[must_use]
    pub fn is_collection(&self) -> bool {
        matches!(
            *self,
            PartiqlShape::Static(StaticType {
                ty: Static::Bag(_),
                nullable: true,
                ..
            })
        ) || matches!(
            *self,
            PartiqlShape::Static(StaticType {
                ty: Static::Array(_),
                nullable: true,
                ..
            })
        )
    }

    #[must_use]
    pub fn is_unordered_collection(&self) -> bool {
        !self.is_ordered_collection()
    }

    #[must_use]
    pub fn is_ordered_collection(&self) -> bool {
        // TODO Add Sexp when added
        matches!(
            *self,
            PartiqlShape::Static(StaticType {
                ty: Static::Array(_),
                nullable: true,
                ..
            })
        )
    }

    #[must_use]
    pub fn is_bag(&self) -> bool {
        matches!(
            *self,
            PartiqlShape::Static(StaticType {
                ty: Static::Bag(_),
                nullable: true,
                ..
            })
        )
    }

    #[must_use]
    pub fn is_array(&self) -> bool {
        matches!(
            *self,
            PartiqlShape::Static(StaticType {
                ty: Static::Array(_),
                nullable: true,
                ..
            })
        )
    }

    #[must_use]
    pub fn is_dynamic(&self) -> bool {
        matches!(*self, PartiqlShape::Dynamic)
    }

    #[must_use]
    pub fn is_undefined(&self) -> bool {
        matches!(*self, PartiqlShape::Undefined)
    }

    pub fn expect_bool(&self) -> ShapeResult<StaticType> {
        if let PartiqlShape::Static(StaticType {
            id,
            ty: Static::Bool,
            nullable: n,
        }) = self
        {
            Ok(StaticType {
                id: *id,
                ty: Static::Bool,
                nullable: *n,
            })
        } else {
            Err(ShapeResultError::UnexpectedType(format!("{self}")))
        }
    }

    pub fn expect_bag(&self) -> ShapeResult<BagType> {
        if let PartiqlShape::Static(StaticType {
            ty: Static::Bag(bag),
            ..
        }) = self
        {
            Ok(bag.clone())
        } else {
            Err(ShapeResultError::UnexpectedType(format!("{self}")))
        }
    }

    pub fn expect_struct(&self) -> ShapeResult<StructType> {
        if let PartiqlShape::Static(StaticType {
            ty: Static::Struct(stct),
            ..
        }) = self
        {
            Ok(stct.clone())
        } else {
            Err(ShapeResultError::UnexpectedType(format!("{self}")))
        }
    }

    pub fn expect_static(&self) -> ShapeResult<StaticType> {
        if let PartiqlShape::Static(s) = self {
            Ok(s.clone())
        } else {
            Err(ShapeResultError::UnexpectedType(format!("{self}")))
        }
    }

    pub fn expect_dynamic_type(&self) -> ShapeResult<PartiqlShape> {
        if let PartiqlShape::Dynamic = self {
            Ok(PartiqlShape::Dynamic)
        } else {
            Err(ShapeResultError::UnexpectedType(format!("{self}")))
        }
    }

    pub fn expect_undefined(&self) -> ShapeResult<PartiqlShape> {
        if let PartiqlShape::Undefined = self {
            Ok(PartiqlShape::Undefined)
        } else {
            Err(ShapeResultError::UnexpectedType(format!("{self}")))
        }
    }
}

impl Display for PartiqlShape {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        let x = match self {
            PartiqlShape::Dynamic => "Dynamic".to_string(),
            PartiqlShape::AnyOf(anyof) => {
                format!("AnyOf({})", anyof.types.iter().cloned().join(","))
            }
            PartiqlShape::Static(s) => format!("{s}"),
            PartiqlShape::Undefined => "Undefined".to_string(),
        };
        write!(f, "{x}")
    }
}

#[derive(Default)]
pub struct PartiqlShapeBuilder {
    id_gen: AutoNodeIdGenerator,
}

impl PartiqlShapeBuilder {
    /// A thread-safe method for creating PartiQL shapes with guaranteed uniqueness over
    /// generated `NodeId`s.
    #[track_caller]
    pub fn init_or_get() -> &'static PartiqlShapeBuilder {
        static SHAPE_BUILDER: OnceLock<PartiqlShapeBuilder> = OnceLock::new();
        SHAPE_BUILDER.get_or_init(PartiqlShapeBuilder::default)
    }

    #[must_use]
    pub fn new_static(&self, ty: Static) -> PartiqlShape {
        let id = self.id_gen.id();
        let id = id.read().expect("NodeId read lock");
        PartiqlShape::Static(StaticType {
            id: *id,
            ty,
            nullable: true,
        })
    }

    #[must_use]
    pub fn new_static_with_const_id(&self, ty: Static) -> PartiqlShape {
        PartiqlShape::Static(StaticType {
            id: NodeId(1),
            ty,
            nullable: true,
        })
    }

    #[must_use]
    pub fn new_non_nullable_static(&self, ty: Static) -> PartiqlShape {
        let id = self.id_gen.id();
        let id = id.read().expect("NodeId read lock");
        PartiqlShape::Static(StaticType {
            id: *id,
            ty,
            nullable: false,
        })
    }

    #[must_use]
    pub fn new_non_nullable_static_with_const_id(&self, ty: Static) -> PartiqlShape {
        PartiqlShape::Static(StaticType {
            id: NodeId(1),
            ty,
            nullable: false,
        })
    }

    #[must_use]
    pub fn new_dynamic(&self) -> PartiqlShape {
        PartiqlShape::Dynamic
    }

    #[must_use]
    pub fn new_undefined(&self) -> PartiqlShape {
        PartiqlShape::Dynamic
    }

    #[must_use]
    pub fn new_struct(&self, s: StructType) -> PartiqlShape {
        self.new_static(Static::Struct(s))
    }

    #[must_use]
    pub fn new_bag(&self, b: BagType) -> PartiqlShape {
        self.new_static(Static::Bag(b))
    }

    #[must_use]
    pub fn new_array(&self, a: ArrayType) -> PartiqlShape {
        self.new_static(Static::Array(a))
    }

    // The AnyOf::from_iter(types) uses an IndexSet internally to
    // deduplicate types, thus the match on any_of.types.len() could
    // "flatten" AnyOfs that had duplicates.
    // With the addition of IDs, this deduplication no longer happens.
    // TODO revisit the current implementaion and consider an implementation
    // that allows merging of the `metas` for the same type, e.g., with a
    // user-defined control.
    pub fn any_of<I>(&self, types: I) -> PartiqlShape
    where
        I: IntoIterator<Item = PartiqlShape>,
    {
        let any_of = AnyOf::from_iter(types);
        match any_of.types.len() {
            0 => type_dynamic!(),
            1 => {
                let AnyOf { types } = any_of;
                types.into_iter().next().unwrap()
            }
            // TODO figure out what does it mean for a Union to be nullable or not
            _ => PartiqlShape::AnyOf(any_of),
        }
    }

    #[must_use]
    pub fn as_non_nullable(&self, shape: &PartiqlShape) -> Option<PartiqlShape> {
        if let PartiqlShape::Static(stype) = shape {
            Some(self.new_non_nullable_static(stype.ty.clone()))
        } else {
            None
        }
    }
}

#[derive(Derivative, Eq, Debug, Clone)]
#[derivative(PartialEq, Hash)]
#[allow(dead_code)]
pub struct AnyOf {
    #[derivative(Hash(hash_with = "indexset_hash"))]
    types: IndexSet<PartiqlShape>,
}

impl AnyOf {
    #[must_use]
    pub const fn new(types: IndexSet<PartiqlShape>) -> Self {
        AnyOf { types }
    }

    pub fn types(&self) -> impl Iterator<Item = &PartiqlShape> {
        self.types.iter()
    }
}

impl FromIterator<PartiqlShape> for AnyOf {
    fn from_iter<T: IntoIterator<Item = PartiqlShape>>(iter: T) -> Self {
        AnyOf {
            types: iter.into_iter().collect(),
        }
    }
}

#[derive(Debug, Clone, Eq, PartialEq, Hash)]
pub struct StaticType {
    id: NodeId,
    ty: Static,
    nullable: bool,
}

impl StaticType {
    #[must_use]
    pub fn ty(&self) -> &Static {
        &self.ty
    }

    pub fn ty_id(&self) -> &NodeId {
        &self.id
    }

    #[must_use]
    pub fn is_nullable(&self) -> bool {
        self.nullable
    }

    #[must_use]
    pub fn is_not_nullable(&self) -> bool {
        !self.nullable
    }

    pub fn is_scalar(&self) -> bool {
        self.ty.is_scalar()
    }

    pub fn is_sequence(&self) -> bool {
        self.ty.is_sequence()
    }

    pub fn is_struct(&self) -> bool {
        self.ty.is_struct()
    }
}

impl Display for StaticType {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        let nullable = if self.nullable {
            "nullable"
        } else {
            "non_nullable"
        };
        write!(f, "({}, {})", self.ty, nullable)
    }
}

pub type StaticTypeMetas = HashMap<String, String>;

#[derive(Debug, Clone, Eq, PartialEq, Hash)]
pub enum Static {
    // Scalar Types
    Int,
    Int8,
    Int16,
    Int32,
    Int64,
    Bool,
    Decimal,
    DecimalP(usize, usize),

    Float32,
    Float64,

    String,
    StringFixed(usize),
    StringVarying(usize),

    DateTime,

    // Container Types
    Struct(StructType),
    Bag(BagType),
    Array(ArrayType),
    // TODO Add BitString, ByteString, Blob, Clob, and Graph types
}

impl Static {
    pub fn is_scalar(&self) -> bool {
        !matches!(self, Static::Struct(_) | Static::Bag(_) | Static::Array(_))
    }

    pub fn is_sequence(&self) -> bool {
        matches!(self, Static::Bag(_) | Static::Array(_))
    }

    pub fn is_struct(&self) -> bool {
        matches!(self, Static::Struct(_))
    }
}

impl Display for Static {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        let x = match self {
            Static::Int => "Int".to_string(),
            Static::Int8 => "Int8".to_string(),
            Static::Int16 => "Int16".to_string(),
            Static::Int32 => "Int32".to_string(),
            Static::Int64 => "Int64".to_string(),
            Static::Bool => "Bool".to_string(),
            Static::Decimal => "Decimal".to_string(),
            Static::DecimalP(_, _) => {
                todo!()
            }
            Static::Float32 => "Float32".to_string(),
            Static::Float64 => "Float64".to_string(),
            Static::String => "String".to_string(),
            Static::StringFixed(_) => {
                todo!()
            }
            Static::StringVarying(_) => {
                todo!()
            }
            Static::DateTime => "DateTime".to_string(),
            Static::Struct(_) => "Struct".to_string(),
            Static::Bag(_) => "Bag".to_string(),
            Static::Array(_) => "Array".to_string(),
        };
        write!(f, "{x}")
    }
}

pub const TYPE_DYNAMIC: PartiqlShape = PartiqlShape::Dynamic;

#[derive(Derivative, Eq, Debug, Clone)]
#[derivative(PartialEq, Hash)]
#[allow(dead_code)]
pub struct StructType {
    #[derivative(Hash(hash_with = "indexset_hash"))]
    constraints: IndexSet<StructConstraint>,
}

impl StructType {
    #[must_use]
    pub fn new(constraints: IndexSet<StructConstraint>) -> Self {
        StructType { constraints }
    }

    #[must_use]
    pub fn new_any() -> Self {
        StructType {
            constraints: Default::default(),
        }
    }

    pub fn fields_set(&self) -> IndexSet<StructField> {
        self.constraints
            .iter()
            .flat_map(|c| {
                if let StructConstraint::Fields(fields) = c.clone() {
                    fields
                } else {
                    Default::default()
                }
            })
            .collect()
    }

    pub fn fields(&self) -> impl Iterator<Item = &StructField> {
        self.constraints
            .iter()
            .filter_map(|c| {
                if let StructConstraint::Fields(fields) = c {
                    Some(fields)
                } else {
                    None
                }
            })
            .flat_map(|f| f.iter())
    }

    #[must_use]
    pub fn is_partial(&self) -> bool {
        !self.is_closed()
    }

    #[must_use]
    pub fn is_closed(&self) -> bool {
        self.constraints.contains(&StructConstraint::Open(false))
    }
}

#[derive(Derivative, Eq, Debug, Clone)]
#[derivative(PartialEq, Hash)]
#[allow(dead_code)]
#[non_exhaustive]
pub enum StructConstraint {
    Open(bool),
    Ordered(bool),
    DuplicateAttrs(bool),
    Fields(#[derivative(Hash(hash_with = "indexset_hash"))] IndexSet<StructField>),
}

#[derive(Debug, Clone, Hash, Eq, PartialEq)]
#[allow(dead_code)]
pub struct StructField {
    optional: bool,
    name: String,
    ty: PartiqlShape,
}

impl StructField {
    #[must_use]
    pub fn new(name: &str, ty: PartiqlShape) -> Self {
        StructField {
            name: name.to_string(),
            ty,
            optional: false,
        }
    }

    #[must_use]
    pub fn new_optional(name: &str, ty: PartiqlShape) -> Self {
        StructField {
            name: name.to_string(),
            ty,
            optional: true,
        }
    }

    #[must_use]
    pub fn name(&self) -> &str {
        self.name.as_str()
    }

    #[must_use]
    pub fn ty(&self) -> &PartiqlShape {
        &self.ty
    }

    #[must_use]
    pub fn is_optional(&self) -> bool {
        self.optional
    }
}

impl From<(&str, PartiqlShape)> for StructField {
    fn from(value: (&str, PartiqlShape)) -> Self {
        StructField {
            name: value.0.to_string(),
            ty: value.1,
            optional: false,
        }
    }
}

impl From<(&str, PartiqlShape, bool)> for StructField {
    fn from(value: (&str, PartiqlShape, bool)) -> Self {
        StructField {
            name: value.0.to_string(),
            ty: value.1,
            optional: value.2,
        }
    }
}

#[derive(Derivative, Eq, Debug, Clone)]
#[derivative(PartialEq, Hash)]
#[allow(dead_code)]
pub struct BagType {
    element_type: Box<PartiqlShape>,
}

impl BagType {
    #[must_use]
    pub fn new_any() -> Self {
        BagType::new(Box::new(PartiqlShape::Dynamic))
    }

    #[must_use]
    pub fn new(typ: Box<PartiqlShape>) -> Self {
        BagType { element_type: typ }
    }

    #[must_use]
    pub fn element_type(&self) -> &PartiqlShape {
        &self.element_type
    }
}

#[derive(Derivative, Eq, Debug, Clone)]
#[derivative(PartialEq, Hash)]
#[allow(dead_code)]
pub struct ArrayType {
    element_type: Box<PartiqlShape>,
    // TODO Add Array constraint once we have Schema Specification:
    // https://github.com/partiql/partiql-spec/issues/49
}

impl ArrayType {
    #[must_use]
    pub fn new_any() -> Self {
        ArrayType::new(Box::new(PartiqlShape::Dynamic))
    }

    #[must_use]
    pub fn new(typ: Box<PartiqlShape>) -> Self {
        ArrayType { element_type: typ }
    }

    #[must_use]
    pub fn element_type(&self) -> &PartiqlShape {
        &self.element_type
    }
}

#[cfg(test)]
mod tests {
    use crate::{
        BagType, PartiqlShape, PartiqlShapeBuilder, Static, StructConstraint, StructField,
        StructType,
    };
    use indexmap::IndexSet;

    #[test]
    fn union() {
        let expect_int = type_int_with_const_id!();
        assert_eq!(
            expect_int,
            type_int_with_const_id!().union_with(type_int_with_const_id!())
        );

        let expect_nums = PartiqlShapeBuilder::init_or_get()
            .any_of([type_int_with_const_id!(), type_float32_with_const_id!()]);
        assert_eq!(
            expect_nums,
            type_int_with_const_id!().union_with(type_float32_with_const_id!())
        );
        assert_eq!(
            expect_nums,
            PartiqlShapeBuilder::init_or_get().any_of([
                type_int_with_const_id!().union_with(type_float32_with_const_id!()),
                type_int_with_const_id!().union_with(type_float32_with_const_id!())
            ])
        );
        assert_eq!(
            expect_nums,
            PartiqlShapeBuilder::init_or_get().any_of([
                type_int_with_const_id!().union_with(type_float32_with_const_id!()),
                type_int_with_const_id!().union_with(type_float32_with_const_id!()),
                PartiqlShapeBuilder::init_or_get().any_of([
                    type_int_with_const_id!().union_with(type_float32_with_const_id!()),
                    type_int_with_const_id!().union_with(type_float32_with_const_id!())
                ])
            ])
        );
    }

    #[test]
    fn unique_node_ids() {
        let age_field = struct_fields![("age", type_int!())];
        let details = type_struct![IndexSet::from([age_field])];

        let fields = [
            StructField::new("id", type_int!()),
            StructField::new("name", type_string!()),
            StructField::new("details", details.clone()),
        ];

        let row = type_struct![IndexSet::from([
            StructConstraint::Fields(IndexSet::from(fields)),
            StructConstraint::Open(false)
        ])];

        let shape = type_bag![row.clone()];

        let mut ids = collect_ids(shape);
        ids.sort_unstable();
        assert!(ids.windows(2).all(|w| w[0] != w[1]));
    }

    fn collect_ids(row: PartiqlShape) -> Vec<u32> {
        let mut out = vec![];
        match row {
            PartiqlShape::Dynamic => {}
            PartiqlShape::AnyOf(anyof) => {
                for shape in anyof.types {
                    out.push(collect_ids(shape));
                }
            }
            PartiqlShape::Static(static_type) => {
                out.push(vec![static_type.id.0]);
                match static_type.ty {
                    Static::Struct(struct_type) => {
                        for f in struct_type.fields() {
                            out.push(collect_ids(f.ty.clone()));
                        }
                    }
                    Static::Bag(bag_type) => out.push(collect_ids(*bag_type.element_type)),
                    Static::Array(array_type) => out.push(collect_ids(*array_type.element_type)),
                    _ => {}
                }
            }
            PartiqlShape::Undefined => {}
        }
        out.into_iter().flatten().collect()
    }
}
