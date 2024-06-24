#![deny(rust_2018_idioms)]
#![deny(clippy::all)]

use itertools::Itertools;
use miette::Diagnostic;
use std::collections::BTreeSet;
use std::fmt::{Debug, Display, Formatter};
use std::hash::Hash;
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

#[macro_export]
macro_rules! dynamic {
    () => {
        $crate::PartiqlShape::Dynamic
    };
}

#[macro_export]
macro_rules! int {
    () => {
        $crate::PartiqlShape::new($crate::Static::Int)
    };
}

#[macro_export]
macro_rules! int8 {
    () => {
        $crate::PartiqlShape::new($crate::Static::Int8)
    };
}

#[macro_export]
macro_rules! int16 {
    () => {
        $crate::PartiqlShape::new($crate::Static::Int16)
    };
}

#[macro_export]
macro_rules! int32 {
    () => {
        $crate::PartiqlShape::new($crate::Static::Int32)
    };
}

#[macro_export]
macro_rules! int64 {
    () => {
        $crate::PartiqlShape::new($crate::Static::Int64)
    };
}

#[macro_export]
macro_rules! dec {
    () => {
        $crate::PartiqlShape::new($crate::Static::Decimal)
    };
}

// TODO add macro_rule for Decimal with precision and scale

#[macro_export]
macro_rules! f32 {
    () => {
        $crate::PartiqlShape::new($crate::Static::Float32)
    };
}

#[macro_export]
macro_rules! f64 {
    () => {
        $crate::PartiqlShape::new($crate::Static::Float64)
    };
}

#[macro_export]
macro_rules! str {
    () => {
        $crate::PartiqlShape::new($crate::Static::String)
    };
}

#[macro_export]
macro_rules! r#struct {
    () => {
        $crate::PartiqlShape::new_struct(StructType::new_any())
    };
    ($elem:expr) => {
        $crate::PartiqlShape::new_struct(StructType::new($elem))
    };
}

#[macro_export]
macro_rules! struct_fields {
    ($(($x:expr, $y:expr)),+ $(,)?) => (
        $crate::StructConstraint::Fields([$(($x, $y).into()),+].into())
    );
}

#[macro_export]
macro_rules! r#bag {
    () => {
        $crate::PartiqlShape::new_bag(BagType::new_any());
    };
    ($elem:expr) => {
        $crate::PartiqlShape::new_bag(BagType::new(Box::new($elem)))
    };
}

#[macro_export]
macro_rules! r#array {
    () => {
        $crate::PartiqlShape::new_array(ArrayType::new_any());
    };
    ($elem:expr) => {
        $crate::PartiqlShape::new_array(ArrayType::new(Box::new($elem)))
    };
}

#[macro_export]
macro_rules! undefined {
    () => {
        $crate::PartiqlShape::Undefined
    };
}

/// Represents a PartiQL Shape
#[derive(Debug, Clone, Eq, PartialEq, Hash, Ord, PartialOrd)]
// With this implementation `Dynamic` and `AnyOf` cannot have `nullability`; this does not mean their
// `null` value at runtime cannot belong to their domain.
// TODO adopt the correct model Pending PartiQL Types semantics finalization: https://github.com/partiql/partiql-lang/issues/18
pub enum PartiqlShape {
    Dynamic,
    AnyOf(AnyOf),
    Static(StaticType),
    Undefined,
}

#[derive(Debug, Clone, Eq, PartialEq, Hash, Ord, PartialOrd)]
pub struct StaticType {
    ty: Static,
    nullable: bool,
}

#[derive(Debug, Clone, Eq, PartialEq, Hash, Ord, PartialOrd)]
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

impl StaticType {
    #[must_use]
    pub fn new(&self, ty: Static) -> StaticType {
        StaticType { ty, nullable: true }
    }

    #[must_use]
    pub fn new_non_nullable(&self, ty: Static) -> StaticType {
        StaticType {
            ty,
            nullable: false,
        }
    }

    #[must_use]
    pub fn ty(&self) -> Static {
        self.ty.clone()
    }

    #[must_use]
    pub fn is_nullable(&self) -> bool {
        self.nullable
    }

    #[must_use]
    pub fn is_not_nullable(&self) -> bool {
        !self.nullable
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
pub const TYPE_BOOL: PartiqlShape = PartiqlShape::new(Static::Bool);
pub const TYPE_INT: PartiqlShape = PartiqlShape::new(Static::Int);
pub const TYPE_INT8: PartiqlShape = PartiqlShape::new(Static::Int8);
pub const TYPE_INT16: PartiqlShape = PartiqlShape::new(Static::Int16);
pub const TYPE_INT32: PartiqlShape = PartiqlShape::new(Static::Int32);
pub const TYPE_INT64: PartiqlShape = PartiqlShape::new(Static::Int64);
pub const TYPE_REAL: PartiqlShape = PartiqlShape::new(Static::Float32);
pub const TYPE_DOUBLE: PartiqlShape = PartiqlShape::new(Static::Float64);
pub const TYPE_DECIMAL: PartiqlShape = PartiqlShape::new(Static::Decimal);
pub const TYPE_STRING: PartiqlShape = PartiqlShape::new(Static::String);
pub const TYPE_DATETIME: PartiqlShape = PartiqlShape::new(Static::DateTime);
pub const TYPE_NUMERIC_TYPES: [PartiqlShape; 4] = [TYPE_INT, TYPE_REAL, TYPE_DOUBLE, TYPE_DECIMAL];

#[allow(dead_code)]
impl PartiqlShape {
    #[must_use]
    pub const fn new(ty: Static) -> PartiqlShape {
        PartiqlShape::Static(StaticType { ty, nullable: true })
    }
    #[must_use]
    pub const fn new_non_nullable(ty: Static) -> PartiqlShape {
        PartiqlShape::Static(StaticType {
            ty,
            nullable: false,
        })
    }

    #[must_use]
    pub fn as_non_nullable(&self) -> Option<PartiqlShape> {
        if let PartiqlShape::Static(stype) = self {
            Some(PartiqlShape::Static(StaticType {
                ty: stype.ty.clone(),
                nullable: false,
            }))
        } else {
            None
        }
    }

    #[must_use]
    pub fn new_dynamic() -> PartiqlShape {
        PartiqlShape::Dynamic
    }

    #[must_use]
    pub fn new_struct(s: StructType) -> PartiqlShape {
        PartiqlShape::new(Static::Struct(s))
    }

    #[must_use]
    pub fn new_bag(b: BagType) -> PartiqlShape {
        PartiqlShape::new(Static::Bag(b))
    }

    #[must_use]
    pub fn new_array(a: ArrayType) -> PartiqlShape {
        PartiqlShape::new(Static::Array(a))
    }

    pub fn any_of<I>(types: I) -> PartiqlShape
    where
        I: IntoIterator<Item = PartiqlShape>,
    {
        let any_of = AnyOf::from_iter(types);
        match any_of.types.len() {
            0 => TYPE_DYNAMIC,
            1 => {
                let AnyOf { types } = any_of;
                types.into_iter().next().unwrap()
            }
            // TODO figure out what does it mean for a Union to be nullable or not
            _ => PartiqlShape::AnyOf(any_of),
        }
    }

    #[must_use]
    pub fn union_with(self, other: PartiqlShape) -> PartiqlShape {
        match (self, other) {
            (PartiqlShape::Dynamic, _) | (_, PartiqlShape::Dynamic) => PartiqlShape::new_dynamic(),
            (PartiqlShape::AnyOf(lhs), PartiqlShape::AnyOf(rhs)) => {
                PartiqlShape::any_of(lhs.types.into_iter().chain(rhs.types))
            }
            (PartiqlShape::AnyOf(anyof), other) | (other, PartiqlShape::AnyOf(anyof)) => {
                let mut types = anyof.types;
                types.insert(other);
                PartiqlShape::any_of(types)
            }
            (l, r) => {
                let types = [l, r];
                PartiqlShape::any_of(types)
            }
        }
    }

    #[must_use]
    pub fn is_string(&self) -> bool {
        matches!(
            &self,
            PartiqlShape::Static(StaticType {
                ty: Static::String,
                nullable: true
            })
        )
    }

    #[must_use]
    pub fn is_struct(&self) -> bool {
        matches!(
            *self,
            PartiqlShape::Static(StaticType {
                ty: Static::Struct(_),
                nullable: true
            })
        )
    }

    #[must_use]
    pub fn is_collection(&self) -> bool {
        matches!(
            *self,
            PartiqlShape::Static(StaticType {
                ty: Static::Bag(_),
                nullable: true
            })
        ) || matches!(
            *self,
            PartiqlShape::Static(StaticType {
                ty: Static::Array(_),
                nullable: true
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
                nullable: true
            })
        )
    }

    #[must_use]
    pub fn is_bag(&self) -> bool {
        matches!(
            *self,
            PartiqlShape::Static(StaticType {
                ty: Static::Bag(_),
                nullable: true
            })
        )
    }

    #[must_use]
    pub fn is_array(&self) -> bool {
        matches!(
            *self,
            PartiqlShape::Static(StaticType {
                ty: Static::Array(_),
                nullable: true
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
            ty: Static::Bool,
            nullable: n,
        }) = self
        {
            Ok(StaticType {
                ty: Static::Bool,
                nullable: *n,
            })
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

#[derive(Hash, Eq, PartialEq, Debug, Clone, Ord, PartialOrd)]
#[allow(dead_code)]
pub struct AnyOf {
    types: BTreeSet<PartiqlShape>,
}

impl AnyOf {
    #[must_use]
    pub const fn new(types: BTreeSet<PartiqlShape>) -> Self {
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

#[derive(Debug, Clone, Hash, PartialEq, Eq, Ord, PartialOrd)]
#[allow(dead_code)]
pub struct StructType {
    constraints: BTreeSet<StructConstraint>,
}

impl StructType {
    #[must_use]
    pub fn new(constraints: BTreeSet<StructConstraint>) -> Self {
        StructType { constraints }
    }

    #[must_use]
    pub fn new_any() -> Self {
        StructType {
            constraints: Default::default(),
        }
    }

    #[must_use]
    pub fn fields(&self) -> BTreeSet<StructField> {
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

    #[must_use]
    pub fn is_partial(&self) -> bool {
        !self.is_closed()
    }

    #[must_use]
    pub fn is_closed(&self) -> bool {
        self.constraints.contains(&StructConstraint::Open(false))
    }
}

#[derive(Debug, Clone, Eq, PartialEq, Hash, Ord, PartialOrd)]
#[allow(dead_code)]
#[non_exhaustive]
pub enum StructConstraint {
    Open(bool),
    Ordered(bool),
    DuplicateAttrs(bool),
    Fields(BTreeSet<StructField>),
}

#[derive(Debug, Clone, Eq, PartialEq, Hash, Ord, PartialOrd)]
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

#[derive(Debug, Clone, Eq, PartialEq, Hash, Ord, PartialOrd)]
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

#[derive(Debug, Clone, Eq, PartialEq, Hash, Ord, PartialOrd)]
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
    use crate::{PartiqlShape, TYPE_INT, TYPE_REAL};

    #[test]
    fn union() {
        let expect_int = TYPE_INT;
        assert_eq!(expect_int, TYPE_INT.union_with(TYPE_INT));

        let expect_nums = PartiqlShape::any_of([TYPE_INT, TYPE_REAL]);
        assert_eq!(expect_nums, TYPE_INT.union_with(TYPE_REAL));
        assert_eq!(
            expect_nums,
            PartiqlShape::any_of([
                TYPE_INT.union_with(TYPE_REAL),
                TYPE_INT.union_with(TYPE_REAL)
            ])
        );
        assert_eq!(
            expect_nums,
            PartiqlShape::any_of([
                TYPE_INT.union_with(TYPE_REAL),
                TYPE_INT.union_with(TYPE_REAL),
                PartiqlShape::any_of([
                    TYPE_INT.union_with(TYPE_REAL),
                    TYPE_INT.union_with(TYPE_REAL)
                ])
            ])
        );
    }
}
