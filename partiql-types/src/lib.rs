#![deny(rust_2018_idioms)]
#![deny(clippy::all)]

use itertools::Itertools;
use std::collections::BTreeSet;
use std::fmt::{Debug, Display, Formatter};
use std::hash::Hash;

pub trait Type {}

impl Type for PartiqlShape {}

#[macro_export]
macro_rules! any {
    () => {
        $crate::PartiqlShape::new($crate::PartiqlType::Any)
    };
}

#[macro_export]
macro_rules! int {
    () => {
        $crate::PartiqlShape::new($crate::PartiqlType::Int)
    };
}

#[macro_export]
macro_rules! int8 {
    () => {
        $crate::PartiqlShape::new($crate::PartiqlType::Int8)
    };
}

#[macro_export]
macro_rules! int16 {
    () => {
        $crate::PartiqlShape::new($crate::PartiqlType::Int16)
    };
}

#[macro_export]
macro_rules! int32 {
    () => {
        $crate::PartiqlShape::new($crate::PartiqlType::Int32)
    };
}

#[macro_export]
macro_rules! int64 {
    () => {
        $crate::PartiqlShape::new($crate::PartiqlType::Int64)
    };
}

#[macro_export]
macro_rules! dec {
    () => {
        $crate::PartiqlShape::new($crate::PartiqlType::Decimal)
    };
}

// TODO add macro_rule for Decimal with precision and scale

#[macro_export]
macro_rules! f32 {
    () => {
        $crate::PartiqlShape::new($crate::PartiqlType::Float32)
    };
}

#[macro_export]
macro_rules! f64 {
    () => {
        $crate::PartiqlShape::new($crate::PartiqlType::Float64)
    };
}

#[macro_export]
macro_rules! str {
    () => {
        $crate::PartiqlShape::new($crate::PartiqlType::String)
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
        $crate::PartiqlShape::new($crate::PartiqlType::Undefined)
    };
}

#[derive(Debug, Clone, Eq, PartialEq, Hash, Ord, PartialOrd)]
pub struct PartiqlShape {
    ty: PartiqlType,
    nullable: bool
}

#[derive(Debug, Clone, Eq, PartialEq, Hash, Ord, PartialOrd)]
#[non_exhaustive]
pub enum PartiqlType {
    Any,
    AnyOf(AnyOf),

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
    // Serves as Bottom Type
    Undefined,
    // TODO Add BitString, ByteString, Blob, Clob, and Graph types
}

impl Display for PartiqlType {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        let x = match self {
            PartiqlType::Any => "Any".to_string(),
            PartiqlType::AnyOf(anyof) => {
                format!(
                    "AnyOf({})",
                    anyof.types.iter().map(|shape| shape.ty()).join(",")
                )
            }
            PartiqlType::Int => "Int".to_string(),
            PartiqlType::Int8 => "Int8".to_string(),
            PartiqlType::Int16 => "Int16".to_string(),
            PartiqlType::Int32 => "Int32".to_string(),
            PartiqlType::Int64 => "Int64".to_string(),
            PartiqlType::Bool => "Bool".to_string(),
            PartiqlType::Decimal => "Decimal".to_string(),
            PartiqlType::DecimalP(_, _) => {
                todo!()
            }
            PartiqlType::Float32 => "Float32".to_string(),
            PartiqlType::Float64 => "Float64".to_string(),
            PartiqlType::String => "String".to_string(),
            PartiqlType::StringFixed(_) => {
                todo!()
            }
            PartiqlType::StringVarying(_) => {
                todo!()
            }
            PartiqlType::DateTime => "DateTime".to_string(),
            PartiqlType::Struct(_) => "Struct".to_string(),
            PartiqlType::Bag(_) => "Bag".to_string(),
            PartiqlType::Array(_) => "Array".to_string(),
            PartiqlType::Undefined => "Undefined".to_string(),
        };
        write!(f, "{x}")
    }
}

pub const TYPE_ANY: PartiqlShape = PartiqlShape::new(PartiqlType::Any);
pub const TYPE_BOOL: PartiqlShape = PartiqlShape::new(PartiqlType::Bool);
pub const TYPE_INT: PartiqlShape = PartiqlShape::new(PartiqlType::Int);
pub const TYPE_INT8: PartiqlShape = PartiqlShape::new(PartiqlType::Int8);
pub const TYPE_INT16: PartiqlShape = PartiqlShape::new(PartiqlType::Int16);
pub const TYPE_INT32: PartiqlShape = PartiqlShape::new(PartiqlType::Int32);
pub const TYPE_INT64: PartiqlShape = PartiqlShape::new(PartiqlType::Int64);
pub const TYPE_REAL: PartiqlShape = PartiqlShape::new(PartiqlType::Float32);
pub const TYPE_DOUBLE: PartiqlShape = PartiqlShape::new(PartiqlType::Float64);
pub const TYPE_DECIMAL: PartiqlShape = PartiqlShape::new(PartiqlType::Decimal);
pub const TYPE_STRING: PartiqlShape = PartiqlShape::new(PartiqlType::String);
pub const TYPE_DATETIME: PartiqlShape = PartiqlShape::new(PartiqlType::DateTime);
pub const TYPE_NUMERIC_TYPES: [PartiqlShape; 4] = [TYPE_INT, TYPE_REAL, TYPE_DOUBLE, TYPE_DECIMAL];

#[allow(dead_code)]
impl PartiqlShape {
    #[must_use]
    pub const fn new(ty: PartiqlType) -> PartiqlShape {
        let nullable = match &ty {
            PartiqlType::Any => false,
            PartiqlType::AnyOf(_) => false,
            PartiqlType::Int => true,
            PartiqlType::Int8 => true,
            PartiqlType::Int16 => true,
            PartiqlType::Int32 => true,
            PartiqlType::Int64 => true,
            PartiqlType::Bool => true,
            PartiqlType::Decimal => true,
            PartiqlType::DecimalP(_, _) => true,
            PartiqlType::Float32 => true,
            PartiqlType::Float64 => true,
            PartiqlType::String => true,
            PartiqlType::StringFixed(_) => true,
            PartiqlType::StringVarying(_) => true,
            PartiqlType::DateTime => true,
            PartiqlType::Struct(_) => true,
            PartiqlType::Bag(_) => true,
            PartiqlType::Array(_) => true,
            PartiqlType::Undefined => false,
        };

        PartiqlShape {
            ty,
            nullable
        }
    }

    #[must_use]
    pub const fn new_non_nullable(ty: PartiqlType) -> PartiqlShape {
        PartiqlShape {
            ty,
            nullable: false
        }
    }

    #[must_use]
    pub fn new_any() -> PartiqlShape {
        PartiqlShape::new(PartiqlType::Any)
    }

    #[must_use]
    pub fn new_struct(s: StructType) -> PartiqlShape {
        PartiqlShape::new(PartiqlType::Struct(s))
    }

    #[must_use]
    pub fn new_bag(b: BagType) -> PartiqlShape {
        PartiqlShape::new(PartiqlType::Bag(b))
    }

    #[must_use]
    pub fn new_array(a: ArrayType) -> PartiqlShape {
        PartiqlShape::new(PartiqlType::Array(a))
    }

    pub fn any_of<I>(types: I) -> PartiqlShape
    where
        I: IntoIterator<Item = PartiqlShape>,
    {
        let any_of = AnyOf::from_iter(types);
        match any_of.types.len() {
            0 => TYPE_ANY,
            1 => {
                let AnyOf { types } = any_of;
                types.into_iter().next().unwrap()
            }
            // TODO figure out what does it mean for a Union to be nullable or not
            _ => PartiqlShape::new(PartiqlType::AnyOf(any_of)),
        }
    }

    #[must_use]
    pub fn union_with(self, other: PartiqlShape) -> PartiqlShape {
        match (self.ty(), other.ty()) {
            (PartiqlType::Any, _) | (_, PartiqlType::Any) => PartiqlShape::new(PartiqlType::Any),
            (PartiqlType::AnyOf(lhs), PartiqlType::AnyOf(rhs)) => {
                PartiqlShape::any_of(lhs.types.into_iter().chain(rhs.types))
            }
            (PartiqlType::AnyOf(anyof), other) | (other, PartiqlType::AnyOf(anyof)) => {
                let mut types = anyof.types;
                types.insert(PartiqlShape::new(other));
                PartiqlShape::any_of(types)
            }
            (l, r) => {
                let types = [PartiqlShape::new(l), PartiqlShape::new(r)];
                PartiqlShape::any_of(types)
            }
        }
    }

    #[must_use]
    pub fn is_string(&self) -> bool {
        matches!(&self, PartiqlShape { ty: PartiqlType::String, nullable: true })
    }

    #[must_use]
    pub fn ty(&self) -> PartiqlType {
        self.ty.clone()
    }

    #[must_use]
    pub fn is_struct(&self) -> bool {
        matches!(*self, PartiqlShape { ty: PartiqlType::Struct(_), nullable: true })
    }

    #[must_use]
    pub fn is_collection(&self) -> bool {
        matches!(*self, PartiqlShape { ty: PartiqlType::Bag(_), nullable: true })
            || matches!(*self, PartiqlShape { ty: PartiqlType::Array(_), nullable: true })
    }

    #[must_use]
    pub fn is_unordered_collection(&self) -> bool {
        !self.is_ordered_collection()
    }

    #[must_use]
    pub fn is_ordered_collection(&self) -> bool {
        // TODO Add Sexp when added
        matches!(*self, PartiqlShape { ty: PartiqlType::Array(_), nullable: true })
    }

    #[must_use]
    pub fn is_bag(&self) -> bool {
        matches!(*self, PartiqlShape { ty: PartiqlType::Bag(_), nullable: true })
    }

    #[must_use]
    pub fn is_array(&self) -> bool {
        matches!(*self, PartiqlShape { ty: PartiqlType::Array(_), nullable: true })
    }

    #[must_use]
    pub fn is_any(&self) -> bool {
        matches!(*self, PartiqlShape { ty: PartiqlType::Any, nullable: true })
    }

    #[must_use]
    pub fn is_undefined(&self) -> bool {
        matches!(*self, PartiqlShape { ty: PartiqlType::Undefined, nullable: true })
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
    name: String,
    ty: PartiqlShape,
}

impl StructField {
    #[must_use]
    pub fn new(name: &str, ty: PartiqlShape) -> Self {
        StructField {
            name: name.to_string(),
            ty,
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
}

impl From<(&str, PartiqlShape)> for StructField {
    fn from(value: (&str, PartiqlShape)) -> Self {
        StructField {
            name: value.0.to_string(),
            ty: value.1,
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
        BagType::new(Box::new(PartiqlShape::new(PartiqlType::Any)))
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
        ArrayType::new(Box::new(PartiqlShape::new(PartiqlType::Any)))
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
