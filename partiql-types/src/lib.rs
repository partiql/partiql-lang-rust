use std::collections::BTreeSet;
use std::fmt::Debug;
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

// TODO add `DecimalP`

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
macro_rules! unknown {
    () => {
        $crate::PartiqlType::new($crate::TypeKind::Unknown)
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

    // Container Types
    Struct(StructType),
    Bag(BagType),
    Array(ArrayType),
    // Serves as Bottom Type
    Unknown,
    // TODO Add Sexp, TIMESTAMP, BitString, ByteString, Blob, Clob, and Graph types
}

#[allow(dead_code)]
impl PartiqlType {
    pub fn new(kind: TypeKind) -> PartiqlType {
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

    pub fn union_of(types: BTreeSet<PartiqlType>) -> PartiqlType {
        PartiqlType(TypeKind::AnyOf(AnyOf::new(types)))
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

    pub fn is_bag(&self) -> bool {
        matches!(*self, PartiqlType(TypeKind::Bag(_)))
    }

    pub fn is_array(&self) -> bool {
        matches!(*self, PartiqlType(TypeKind::Array(_)))
    }
}

#[derive(Debug, Clone, Eq, PartialEq)]
#[allow(dead_code)]
pub struct Attr {
    name: String,
    ty: PartiqlType,
}

impl Attr {
    pub fn new(name: &str, ty: &PartiqlType) -> Self {
        Attr {
            name: name.to_string(),
            ty: ty.clone(),
        }
    }
}

#[derive(Hash, Eq, PartialEq, Debug, Clone, Ord, PartialOrd)]
#[allow(dead_code)]
pub struct AnyOf {
    types: BTreeSet<PartiqlType>,
}

impl AnyOf {
    pub fn new(types: BTreeSet<PartiqlType>) -> Self {
        AnyOf { types }
    }

    pub fn types(&self) -> &BTreeSet<PartiqlType> {
        &self.types
    }
}

#[derive(Debug, Clone, Eq, PartialEq, Hash, Ord, PartialOrd)]
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

    pub fn is_unordered(&self) -> bool {
        !self.is_ordered()
    }

    pub fn is_ordered(&self) -> bool {
        self.constraints.contains(&StructConstraint::Ordered(true))
    }
}

impl<T> From<(String, T)> for StructField
where
    T: Into<PartiqlType>,
{
    fn from(pair: (String, T)) -> Self {
        StructField {
            name: pair.0,
            ty: pair.1.into(),
        }
    }
}

#[derive(Debug, Clone, Eq, PartialEq, Hash, Ord, PartialOrd)]
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
    constraints: Vec<CollectionConstraint>,
}

impl BagType {
    pub fn new_any() -> Self {
        BagType::new(Box::new(PartiqlType(TypeKind::Any)))
    }

    pub fn new(typ: Box<PartiqlType>) -> Self {
        BagType {
            element_type: typ,
            constraints: vec![CollectionConstraint::Ordered(false)],
        }
    }

    pub fn element_type(&self) -> &PartiqlType {
        &self.element_type
    }

    pub fn constraints(&self) -> &Vec<CollectionConstraint> {
        &self.constraints
    }
}

#[derive(Debug, Clone, Eq, PartialEq, Hash, Ord, PartialOrd)]
#[allow(dead_code)]
pub struct ArrayType {
    element_type: Box<PartiqlType>,
    constraints: Vec<CollectionConstraint>,
}

impl ArrayType {
    pub fn new_any() -> Self {
        ArrayType::new(Box::new(PartiqlType(TypeKind::Any)))
    }

    pub fn new(typ: Box<PartiqlType>) -> Self {
        ArrayType {
            element_type: typ,
            constraints: vec![CollectionConstraint::Ordered(true)],
        }
    }

    pub fn element_type(&self) -> &PartiqlType {
        &self.element_type
    }

    pub fn constraints(&self) -> &Vec<CollectionConstraint> {
        &self.constraints
    }
}

#[derive(Debug, Clone, Eq, PartialEq, Hash, Ord, PartialOrd)]
#[non_exhaustive]
pub enum CollectionConstraint {
    Ordered(bool),
}

#[cfg(test)]
mod tests {
    #[test]
    fn todo() {}
}
