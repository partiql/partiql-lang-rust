use std::collections::HashSet;
use std::fmt::Debug;

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

#[derive(Debug, Clone)]
pub struct PartiqlType {
    kind: TypeKind,
}

#[derive(Debug, Clone)]
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

    // Container Types
    Struct(StructType),
    Bag(BagType),
    Array(ArrayType),
    // TODO Add Sexp, TIMESTAMP, BitString, ByteString, Blob, Clob, and Graph types
}

#[allow(dead_code)]
impl PartiqlType {
    pub fn new(kind: TypeKind) -> PartiqlType {
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

    pub fn union_of(types: HashSet<PartiqlType>) -> PartiqlType {
        PartiqlType {
            kind: TypeKind::AnyOf(AnyOf::new(types)),
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

#[derive(Debug, Clone)]
#[allow(dead_code)]
pub struct AnyOf {
    types: HashSet<PartiqlType>,
}

impl AnyOf {
    pub fn new(types: HashSet<PartiqlType>) -> Self {
        AnyOf { types }
    }
}

#[derive(Debug, Clone)]
#[allow(dead_code)]
pub struct StructType {
    constraints: Vec<StructConstraint>,
}

#[derive(Debug, Clone)]
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

#[derive(Debug, Clone)]
pub enum StructConstraint {
    Open(bool),
    Ordered(bool),
    DuplicateAttrs(bool),
    Fields(StructField),
}

#[derive(Debug, Clone)]
#[allow(dead_code)]
pub struct BagType {
    element_type: Box<PartiqlType>,
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

#[derive(Debug, Clone)]
#[allow(dead_code)]
pub struct ArrayType {
    element_type: Box<PartiqlType>,
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

#[derive(Debug, Clone)]
enum CollectionConstraint {
    Ordered(bool),
}

#[cfg(test)]
mod tests {
    #[test]
    fn todo() {}
}
