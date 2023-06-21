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

pub trait Schema {
    fn relation(&self) -> Vec<Attr>;
}

#[derive(Debug, Clone)]
pub struct Attr {
    name: String,
    ty: PartiqlType,
}

impl Attr {
    fn new(name: &str, ty: PartiqlType) -> Self {
        Attr {
            name: name.to_string(),
            ty,
        }
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

impl StructType {
    pub fn fields(&self) -> Vec<StructField> {
        self.constraints.iter().map(|c|
        {
            if let StructConstraint::Fields(fields) = c.clone() {
                fields
            } else {
                vec![]
            }
        }
        ).flatten().collect()
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
    Fields(Vec<StructField>),
}

#[derive(Debug, Clone)]
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
}

trait Collection {}

impl Collection for BagType {}
impl Collection for ArrayType {}

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

impl Schema for BagType {
    fn relation(&self) -> Vec<Attr> {
        let kind = self.element_type.kind();
        match kind {
            TypeKind::Any |
            TypeKind::AnyOf(_) |
            TypeKind::Missing => vec![],
            TypeKind::Struct(s) => {
                s.fields().into_iter().map(|f| Attr::new(f.name.as_str(), f.ty)).collect()
            },
            _ => {
                let key = "_1";
                let ty = PartiqlType::new(kind.clone());
                vec![Attr::new(key, ty)]
            }
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
    use super::*;
    #[test]
    fn todo() {
        let bag1 = bag!(PartiqlType::new_struct(StructType::new(vec![StructConstraint::Fields(vec![StructField::new("a", PartiqlType::new(TypeKind::Int))])])));
        let bag2 = BagType::new(Box::new(PartiqlType::new(TypeKind::Int)));
        // dbg!(bag1.kind().relation());
        // dbg!(bag2.relation());
    }
}
