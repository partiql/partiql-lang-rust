use std::collections::HashSet;
use std::fmt::Debug;

pub trait Type {}

impl Type for PartiqlType {}

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
    Bool,
    Decimal,

    Float64,
    String,

    // Container Type
    Struct(StructType),
    Bag(BagType),
    Array(ArrayType),
    // TODO Add Sexp, TIMESTAMP
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
    pub fn unconstrained() -> Self {
        StructType {
            constraints: vec![],
        }
    }

    pub fn constrained(constraints: Vec<StructConstraint>) -> Self {
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
    pub fn bag() -> Self {
        BagType::bag_of(Box::new(PartiqlType {
            kind: TypeKind::Any,
        }))
    }

    pub fn bag_of(typ: Box<PartiqlType>) -> Self {
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
    pub fn array() -> Self {
        ArrayType::array_of(Box::new(PartiqlType {
            kind: TypeKind::Any,
        }))
    }

    pub fn array_of(typ: Box<PartiqlType>) -> Self {
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
