use itertools::Itertools;
use std::collections::BTreeSet;
use std::fmt::{Debug, Display, Formatter};
use std::hash::Hash;

pub trait Type {}

impl Type for PartiqlType {}

#[macro_export]
macro_rules! any {
    () => {
        $crate::PartiqlType::new($crate::TypeKind::Any)
    };
}

#[macro_export]
macro_rules! null {
    () => {
        $crate::PartiqlType::new($crate::TypeKind::Null)
    };
}

#[macro_export]
macro_rules! missing {
    () => {
        $crate::PartiqlType::new($crate::TypeKind::Missing)
    };
}

#[macro_export]
macro_rules! int {
    () => {
        $crate::PartiqlType::new($crate::TypeKind::Int)
    };
}

#[macro_export]
macro_rules! int8 {
    () => {
        $crate::PartiqlType::new($crate::TypeKind::Int8)
    };
}

#[macro_export]
macro_rules! int16 {
    () => {
        $crate::PartiqlType::new($crate::TypeKind::Int16)
    };
}

#[macro_export]
macro_rules! int32 {
    () => {
        $crate::PartiqlType::new($crate::TypeKind::Int32)
    };
}

#[macro_export]
macro_rules! int64 {
    () => {
        $crate::PartiqlType::new($crate::TypeKind::Int64)
    };
}

#[macro_export]
macro_rules! dec {
    () => {
        $crate::PartiqlType::new($crate::TypeKind::Decimal)
    };
}

// TODO add macro_rule for Decimal with precision and scale

#[macro_export]
macro_rules! f32 {
    () => {
        $crate::PartiqlType::new($crate::TypeKind::Float32)
    };
}

#[macro_export]
macro_rules! f64 {
    () => {
        $crate::PartiqlType::new($crate::TypeKind::Float64)
    };
}

#[macro_export]
macro_rules! str {
    () => {
        $crate::PartiqlType::new($crate::TypeKind::String)
    };
}

#[macro_export]
macro_rules! r#struct {
    () => {
        $crate::PartiqlType::new_struct(StructType::new_any())
    };
    ($elem:expr) => {
        $crate::PartiqlType::new_struct(StructType::new($elem))
    };
}

#[macro_export]
macro_rules! struct_fields {
    ($(($x:expr, $y:expr)),+ $(,)?) => (
        $crate::StructConstraint::Fields(vec![$(($x, $y).into()),+])
    );
}

#[macro_export]
macro_rules! r#bag {
    () => {
        $crate::PartiqlType::new_bag(BagType::new_any());
    };
    ($elem:expr) => {
        $crate::PartiqlType::new_bag(BagType::new(Box::new($elem)))
    };
}

#[macro_export]
macro_rules! r#array {
    () => {
        $crate::PartiqlType::new_array(ArrayType::new_any());
    };
    ($elem:expr) => {
        $crate::PartiqlType::new_array(ArrayType::new(Box::new($elem)))
    };
}

#[macro_export]
macro_rules! undefined {
    () => {
        $crate::PartiqlType::new($crate::TypeKind::Undefined)
    };
}

#[derive(Debug, Clone, Eq, PartialEq, Hash, Ord, PartialOrd)]
pub struct PartiqlType(TypeKind);

#[derive(Debug, Clone, Eq, PartialEq, Hash, Ord, PartialOrd)]
#[non_exhaustive]
pub enum TypeKind {
    Any,
    AnyOf(AnyOf),

    // Absent Types
    Null,
    Missing,

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

impl Display for TypeKind {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        let x = match self {
            TypeKind::Any => "Any".to_string(),
            TypeKind::AnyOf(anyof) => {
                format!(
                    "AnyOf({})",
                    anyof.types.iter().map(|pt| pt.kind()).join(",")
                )
            }
            TypeKind::Null => "Null".to_string(),
            TypeKind::Missing => "Missing".to_string(),
            TypeKind::Int => "Int".to_string(),
            TypeKind::Int8 => "Int8".to_string(),
            TypeKind::Int16 => "Int16".to_string(),
            TypeKind::Int32 => "Int32".to_string(),
            TypeKind::Int64 => "Int64".to_string(),
            TypeKind::Bool => "Bool".to_string(),
            TypeKind::Decimal => "Decimal".to_string(),
            TypeKind::DecimalP(_, _) => {
                todo!()
            }
            TypeKind::Float32 => "Float32".to_string(),
            TypeKind::Float64 => "Float64".to_string(),
            TypeKind::String => "String".to_string(),
            TypeKind::StringFixed(_) => {
                todo!()
            }
            TypeKind::StringVarying(_) => {
                todo!()
            }
            TypeKind::DateTime => "DateTime".to_string(),
            TypeKind::Struct(_) => "Struct".to_string(),
            TypeKind::Bag(_) => "Bag".to_string(),
            TypeKind::Array(_) => "Array".to_string(),
            TypeKind::Undefined => "Undefined".to_string(),
        };
        write!(f, "{}", x)
    }
}

pub const TYPE_ANY: PartiqlType = PartiqlType::new(TypeKind::Any);
pub const TYPE_NULL: PartiqlType = PartiqlType::new(TypeKind::Null);
pub const TYPE_MISSING: PartiqlType = PartiqlType::new(TypeKind::Missing);
pub const TYPE_BOOL: PartiqlType = PartiqlType::new(TypeKind::Bool);
pub const TYPE_INT: PartiqlType = PartiqlType::new(TypeKind::Int);
pub const TYPE_INT8: PartiqlType = PartiqlType::new(TypeKind::Int8);
pub const TYPE_INT16: PartiqlType = PartiqlType::new(TypeKind::Int16);
pub const TYPE_INT32: PartiqlType = PartiqlType::new(TypeKind::Int32);
pub const TYPE_INT64: PartiqlType = PartiqlType::new(TypeKind::Int64);
pub const TYPE_REAL: PartiqlType = PartiqlType::new(TypeKind::Float32);
pub const TYPE_DOUBLE: PartiqlType = PartiqlType::new(TypeKind::Float64);
pub const TYPE_DECIMAL: PartiqlType = PartiqlType::new(TypeKind::Decimal);
pub const TYPE_STRING: PartiqlType = PartiqlType::new(TypeKind::String);
pub const TYPE_DATETIME: PartiqlType = PartiqlType::new(TypeKind::DateTime);
pub const TYPE_NUMERIC_TYPES: [PartiqlType; 4] = [TYPE_INT, TYPE_REAL, TYPE_DOUBLE, TYPE_DECIMAL];

#[allow(dead_code)]
impl PartiqlType {
    pub const fn new(kind: TypeKind) -> PartiqlType {
        PartiqlType(kind)
    }

    pub fn new_any() -> PartiqlType {
        PartiqlType(TypeKind::Any)
    }

    pub fn new_struct(s: StructType) -> PartiqlType {
        PartiqlType(TypeKind::Struct(s))
    }

    pub fn new_bag(b: BagType) -> PartiqlType {
        PartiqlType(TypeKind::Bag(b))
    }

    pub fn new_array(a: ArrayType) -> PartiqlType {
        PartiqlType(TypeKind::Array(a))
    }

    pub fn any_of<I>(types: I) -> PartiqlType
    where
        I: IntoIterator<Item = PartiqlType>,
    {
        let any_of = AnyOf::from_iter(types);
        match any_of.types.len() {
            0 => TYPE_ANY,
            1 => {
                let AnyOf { types } = any_of;
                types.into_iter().next().unwrap()
            }
            _ => PartiqlType(TypeKind::AnyOf(any_of)),
        }
    }

    pub fn union_with(self, other: PartiqlType) -> PartiqlType {
        match (self.0, other.0) {
            (TypeKind::Any, _) | (_, TypeKind::Any) => PartiqlType::new(TypeKind::Any),
            (TypeKind::AnyOf(lhs), TypeKind::AnyOf(rhs)) => {
                PartiqlType::any_of(lhs.types.into_iter().chain(rhs.types))
            }
            (TypeKind::AnyOf(anyof), other) | (other, TypeKind::AnyOf(anyof)) => {
                let mut types = anyof.types;
                types.insert(PartiqlType::new(other));
                PartiqlType::any_of(types)
            }
            (l, r) => {
                let types = [PartiqlType::new(l), PartiqlType::new(r)];
                PartiqlType::any_of(types)
            }
        }
    }

    pub fn is_string(&self) -> bool {
        matches!(&self, PartiqlType(TypeKind::String))
    }

    pub fn kind(&self) -> &TypeKind {
        &self.0
    }

    pub fn is_struct(&self) -> bool {
        matches!(*self, PartiqlType(TypeKind::Struct(_)))
    }

    pub fn is_collection(&self) -> bool {
        matches!(*self, PartiqlType(TypeKind::Bag(_)))
            || matches!(*self, PartiqlType(TypeKind::Array(_)))
    }

    pub fn is_unordered_collection(&self) -> bool {
        !self.is_ordered_collection()
    }

    pub fn is_ordered_collection(&self) -> bool {
        // TODO Add Sexp when added
        matches!(*self, PartiqlType(TypeKind::Array(_)))
    }

    pub fn is_bag(&self) -> bool {
        matches!(*self, PartiqlType(TypeKind::Bag(_)))
    }

    pub fn is_array(&self) -> bool {
        matches!(*self, PartiqlType(TypeKind::Array(_)))
    }

    pub fn is_any(&self) -> bool {
        matches!(*self, PartiqlType(TypeKind::Any))
    }

    pub fn is_undefined(&self) -> bool {
        matches!(*self, PartiqlType(TypeKind::Undefined))
    }
 }

#[derive(Hash, Eq, PartialEq, Debug, Clone, Ord, PartialOrd)]
#[allow(dead_code)]
pub struct AnyOf {
    types: BTreeSet<PartiqlType>,
}

impl AnyOf {
    pub const fn new(types: BTreeSet<PartiqlType>) -> Self {
        AnyOf { types }
    }

    pub fn types(&self) -> impl Iterator<Item = &PartiqlType> {
        self.types.iter()
    }
}

impl FromIterator<PartiqlType> for AnyOf {
    fn from_iter<T: IntoIterator<Item = PartiqlType>>(iter: T) -> Self {
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
    pub fn new(constraints: BTreeSet<StructConstraint>) -> Self {
        StructType { constraints }
    }

    pub fn new_any() -> Self {
        StructType {
            constraints: Default::default(),
        }
    }

    pub fn fields(&self) -> Vec<StructField> {
        self.constraints
            .iter()
            .flat_map(|c| {
                if let StructConstraint::Fields(fields) = c.clone() {
                    fields
                } else {
                    vec![]
                }
            })
            .collect()
    }

    pub fn is_partial(&self) -> bool {
        !self.is_closed()
    }

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
    Fields(Vec<StructField>),
}

#[derive(Debug, Clone, Eq, PartialEq, Hash, Ord, PartialOrd)]
#[allow(dead_code)]
pub struct StructField {
    name: String,
    ty: PartiqlType,
}

impl StructField {
    pub fn new(name: &str, ty: PartiqlType) -> Self {
        StructField {
            name: name.to_string(),
            ty,
        }
    }

    pub fn name(&self) -> &str {
        self.name.as_str()
    }

    pub fn ty(&self) -> &PartiqlType {
        &self.ty
    }
}

impl From<(&str, PartiqlType)> for StructField {
    fn from(value: (&str, PartiqlType)) -> Self {
        StructField {
            name: value.0.to_string(),
            ty: value.1,
        }
    }
}

#[derive(Debug, Clone, Eq, PartialEq, Hash, Ord, PartialOrd)]
#[allow(dead_code)]
pub struct BagType {
    element_type: Box<PartiqlType>,
}

impl BagType {
    pub fn new_any() -> Self {
        BagType::new(Box::new(PartiqlType(TypeKind::Any)))
    }

    pub fn new(typ: Box<PartiqlType>) -> Self {
        BagType { element_type: typ }
    }

    pub fn element_type(&self) -> &PartiqlType {
        &self.element_type
    }
}

#[derive(Debug, Clone, Eq, PartialEq, Hash, Ord, PartialOrd)]
#[allow(dead_code)]
pub struct ArrayType {
    element_type: Box<PartiqlType>,
    constraints: Vec<ArrayConstraint>,
}

impl ArrayType {
    pub fn new_any() -> Self {
        ArrayType::new(Box::new(PartiqlType(TypeKind::Any)))
    }

    pub fn new(typ: Box<PartiqlType>) -> Self {
        ArrayType {
            element_type: typ,
            constraints: vec![],
        }
    }

    pub fn new_constrained(typ: Box<PartiqlType>, constraints: Vec<ArrayConstraint>) -> Self {
        ArrayType {
            element_type: typ,
            constraints,
        }
    }

    pub fn element_type(&self) -> &PartiqlType {
        &self.element_type
    }

    pub fn constraints(&self) -> &Vec<ArrayConstraint> {
        &self.constraints
    }
}

#[derive(Debug, Clone, Eq, PartialEq, Hash, Ord, PartialOrd)]
#[non_exhaustive]
pub enum ArrayConstraint {
    // TODO Add Array constraint once we have Schema Specification:
    // https://github.com/partiql/partiql-spec/issues/49
}

#[cfg(test)]
mod tests {
    use crate::{PartiqlType, TYPE_INT, TYPE_REAL};

    #[test]
    fn union() {
        let expect_int = TYPE_INT;
        assert_eq!(expect_int, TYPE_INT.union_with(TYPE_INT));

        let expect_nums = PartiqlType::any_of([TYPE_INT, TYPE_REAL]);
        assert_eq!(expect_nums, TYPE_INT.union_with(TYPE_REAL));
        assert_eq!(
            expect_nums,
            PartiqlType::any_of([
                TYPE_INT.union_with(TYPE_REAL),
                TYPE_INT.union_with(TYPE_REAL)
            ])
        );
        assert_eq!(
            expect_nums,
            PartiqlType::any_of([
                TYPE_INT.union_with(TYPE_REAL),
                TYPE_INT.union_with(TYPE_REAL),
                PartiqlType::any_of([
                    TYPE_INT.union_with(TYPE_REAL),
                    TYPE_INT.union_with(TYPE_REAL)
                ])
            ])
        );
    }
}
