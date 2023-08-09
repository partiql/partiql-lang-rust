use itertools::Itertools;
use std::collections::BTreeSet;
use std::fmt::{Debug, Display, Formatter};

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
macro_rules! r#bag {
    () => {
        $crate::PartiqlType::new_bag(BagType::new_any());
    };
    ($elem:expr) => {
        $crate::PartiqlType::new_bag(BagType::new($elem))
    };
}

#[macro_export]
macro_rules! r#array {
    () => {
        $crate::PartiqlType::new_array(ArrayType::new_any());
    };
    ($elem:expr) => {
        $crate::PartiqlType::new_bag(ArrayType::new($elem))
    };
}

#[derive(Debug, Clone, PartialEq, Eq, Ord, PartialOrd)]
pub struct PartiqlType {
    kind: TypeKind,
}

#[derive(Debug, Clone, PartialEq, Eq, Ord, PartialOrd)]
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
    // TODO Add Sexp, BitString, ByteString, Blob, Clob, and Graph types
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
        PartiqlType { kind }
    }

    pub fn new_struct(s: StructType) -> PartiqlType {
        PartiqlType {
            kind: TypeKind::Struct(s),
        }
    }

    pub fn new_bag(b: BagType) -> PartiqlType {
        PartiqlType {
            kind: TypeKind::Bag(b),
        }
    }

    pub fn new_array(a: ArrayType) -> PartiqlType {
        PartiqlType {
            kind: TypeKind::Array(a),
        }
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
            _ => PartiqlType {
                kind: TypeKind::AnyOf(any_of),
            },
        }
    }

    pub fn union_with(self, other: PartiqlType) -> PartiqlType {
        match (self.kind, other.kind) {
            (TypeKind::Any, _) | (_, TypeKind::Any) => PartiqlType::new(TypeKind::Any),
            (TypeKind::AnyOf(lhs), TypeKind::AnyOf(rhs)) => {
                PartiqlType::any_of(lhs.types.into_iter().chain(rhs.types.into_iter()))
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
        matches!(
            &self,
            PartiqlType {
                kind: TypeKind::String
            }
        )
    }

    pub fn kind(&self) -> &TypeKind {
        &self.kind
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Ord, PartialOrd)]
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

#[derive(Debug, Clone, PartialEq, Eq, Ord, PartialOrd)]
#[allow(dead_code)]
pub struct StructType {
    constraints: Vec<StructConstraint>,
}

#[derive(Debug, Clone, PartialEq, Eq, Ord, PartialOrd)]
#[allow(dead_code)]
pub struct StructField {
    name: String,
    value: PartiqlType,
}

impl<T> From<(String, T)> for StructField
where
    T: Into<PartiqlType>,
{
    fn from(pair: (String, T)) -> Self {
        StructField {
            name: pair.0,
            value: pair.1.into(),
        }
    }
}

impl StructType {
    pub fn new_any() -> Self {
        StructType {
            constraints: vec![],
        }
    }

    pub fn new(constraints: Vec<StructConstraint>) -> Self {
        StructType { constraints }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Ord, PartialOrd)]
pub enum StructConstraint {
    Open(bool),
    Ordered(bool),
    DuplicateAttrs(bool),
    Fields(StructField),
}

#[derive(Debug, Clone, PartialEq, Eq, Ord, PartialOrd)]
#[allow(dead_code)]
pub struct BagType {
    pub element_type: Box<PartiqlType>,
    constraints: Vec<CollectionConstraint>,
}

impl BagType {
    pub fn new_any() -> Self {
        BagType::new(Box::new(PartiqlType {
            kind: TypeKind::Any,
        }))
    }

    pub fn new(typ: Box<PartiqlType>) -> Self {
        BagType {
            element_type: typ,
            constraints: vec![CollectionConstraint::Ordered(false)],
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Ord, PartialOrd)]
#[allow(dead_code)]
pub struct ArrayType {
    pub element_type: Box<PartiqlType>,
    constraints: Vec<CollectionConstraint>,
}

impl ArrayType {
    pub fn new_any() -> Self {
        ArrayType::new(Box::new(PartiqlType {
            kind: TypeKind::Any,
        }))
    }

    pub fn new(typ: Box<PartiqlType>) -> Self {
        ArrayType {
            element_type: typ,
            constraints: vec![CollectionConstraint::Ordered(true)],
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Ord, PartialOrd)]
enum CollectionConstraint {
    Ordered(bool),
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
