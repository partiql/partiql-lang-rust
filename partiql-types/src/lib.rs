use std::collections::HashSet;

pub trait Type {}

impl Type for StaticType {}

#[derive(Debug, Clone)]
pub struct StaticType {
    kind: StaticTypeKind,
}

#[allow(dead_code)]
impl StaticType {
    pub fn new(kind: StaticTypeKind) -> StaticType {
        StaticType { kind }
    }

    pub fn new_struct(s: StructType) -> StaticType {
        StaticType {
            kind: StaticTypeKind::Struct(s),
        }
    }

    pub fn new_bag(b: BagType) -> StaticType {
        StaticType {
            kind: StaticTypeKind::Bag(b),
        }
    }

    pub fn new_array(a: ArrayType) -> StaticType {
        StaticType {
            kind: StaticTypeKind::Array(a),
        }
    }

    pub fn union_of(types: HashSet<StaticType>) -> StaticType {
        StaticType {
            kind: StaticTypeKind::AnyOf(AnyOf::new(types)),
        }
    }

    pub fn is_string(&self) -> bool {
        matches!(
            &self,
            StaticType {
                kind: StaticTypeKind::String
            }
        )
    }

    pub fn kind(&self) -> &StaticTypeKind {
        &self.kind
    }
}

#[derive(Debug, Clone)]
pub enum StaticTypeKind {
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

    // Container Type
    Struct(StructType),
    Bag(BagType),
    Array(ArrayType),
    // TODO Add Sexp
}

#[derive(Debug, Clone)]
#[allow(dead_code)]
pub struct AnyOf {
    types: HashSet<StaticType>,
}

impl AnyOf {
    pub fn new(types: HashSet<StaticType>) -> Self {
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
    value: StaticType,
}

impl<T> From<(String, T)> for StructField
where
    T: Into<StaticType>,
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
    element_type: Box<StaticType>,
    constraints: Vec<CollectionConstraint>,
}

impl BagType {
    pub fn bag() -> Self {
        BagType::bag_of(Box::new(StaticType {
            kind: StaticTypeKind::Any,
        }))
    }

    pub fn bag_of(typ: Box<StaticType>) -> Self {
        BagType {
            element_type: typ,
            constraints: vec![CollectionConstraint::Ordered(false)],
        }
    }
}

#[derive(Debug, Clone)]
#[allow(dead_code)]
pub struct ArrayType {
    element_type: Box<StaticType>,
    constraints: Vec<CollectionConstraint>,
}

impl ArrayType {
    pub fn array() -> Self {
        ArrayType::array_of(Box::new(StaticType {
            kind: StaticTypeKind::Any,
        }))
    }

    pub fn array_of(typ: Box<StaticType>) -> Self {
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
