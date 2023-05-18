extern crate core;

use std::borrow::Borrow;
use std::collections::HashMap;

// What needs to be done:
// 1. define type interface, constructor and built-in type
// 2. Define the Typing environment (catalog or static environment)
// 3. Create a pass for typing the AST
// 4. Integrate the types to Logical Plan


// TODOs
// - Union type
// - Nullability and MISSINGness
// - Schema
// - Namespaces
// - Value integration

pub trait Typ {
    fn as_str(&self) -> &'static str {
        std::any::type_name::<Self>()
    }

    fn as_any(&self) -> &dyn std::any::Any;
}

impl<T: 'static> Typ for T {
    fn as_any(&self) -> &dyn std::any::Any {
        self
    }
}

trait  TypConstraint {}
impl<T> TypConstraint for T {}

// trait TypConstraint:Borrow<dyn Typ> {}
//
// impl PartialEq for dyn TypConstraint + '_ {
//     fn eq(&self, that: &dyn TypConstraint) -> bool {
//         self.borrow() == that.borrow()
//     }
// }
//
// impl PartialEq<dyn TypConstraint> for Box<dyn TypConstraint + '_> {
//     fn eq(&self, that: &dyn TypConstraint) -> bool {
//         let this = &**self;
//         this.borrow() == that.borrow()
//     }
// }

enum TupleConstraint {
    Open(bool),
    Ordered(bool),
    DuplicateAttrs(bool),
}

enum CollectionConstraint {
    Ordered(bool),
    CollectionTupleConstraint(CollectionTupleConstraint),
}

enum CollectionTupleConstraint {
    PrimaryKey(Vec<String>),
    PartitionKey(Vec<String>)
}

enum CollectionHomogeneity {
    Homogenous(Box<dyn Typ>),
    Heterogeneous,
}

// #[derive(PartialEq)]
enum IntConstraint {
    Int8,
    Int16,
    Int32,
    Int64,
    Unconstrained,
}

// #[derive(PartialEq)]
struct  BoolConstraint;

// #[derive(PartialEq)]
enum DecimalConstraint {
    // Constraint decimal bound by the given precision and scale
    Constraint(usize, usize),
    Unconstrained,
}

enum FloatConstraint {
    Float32,
    Float64,
}

enum StringConstraint {
    ConstraintFixed(usize),
    ConstraintVarying(usize),
    Unconstrained,
}


// #[derive(PartialEq)]
pub struct TypLit {
    constraints: Vec<Box<dyn TypConstraint>>
}

impl TypLit {
    pub fn int() -> Self {
        TypLit {
            constraints: vec![Box::new(IntConstraint::Unconstrained)]
        }
    }

    pub fn int8() -> Self {
        TypLit {
            constraints: vec![Box::new(IntConstraint::Int8)]
        }
    }

    pub fn int16() -> Self {
        TypLit {
            constraints: vec![Box::new(IntConstraint::Int8)]
        }
    }

    pub fn int32() -> Self {
        TypLit {
            constraints: vec![Box::new(IntConstraint::Int32)]
        }
    }

    pub fn int64() -> Self {
        TypLit {
            constraints: vec![Box::new(IntConstraint::Int64)]
        }
    }

    pub fn bool() -> Self {
        TypLit {
            constraints: vec![Box::new(BoolConstraint)]
        }
    }

    pub fn decimal() -> Self {
        TypLit {
            constraints: vec![Box::new(DecimalConstraint::Unconstrained)]
        }
    }

    pub fn decimal_bounded(precision: usize, scale: usize) -> Self {
        TypLit {
            constraints: vec![Box::new(DecimalConstraint::Constraint(precision, scale))]
        }
    }

    pub fn float32() -> Self {
        TypLit {
            constraints: vec![Box::new(FloatConstraint::Float32)]
        }
    }

    pub fn float64() -> Self {
        TypLit {
            constraints: vec![Box::new(FloatConstraint::Float64)]
        }
    }

    pub fn string() -> Self {
        TypLit {
            constraints: vec![Box::new(StringConstraint::Unconstrained)],
        }
    }

    pub fn string_fixed(n: usize) -> Self {
        TypLit {
            constraints: vec![Box::new(StringConstraint::ConstraintFixed(n))],
        }
    }

    pub fn string_varying(n: usize) -> Self {
        TypLit {
            constraints: vec![Box::new(StringConstraint::ConstraintVarying(n))],
        }
    }
}

pub struct TypTuple {
    // TODO move to multi-map or similar
    fields: Option<HashMap<String, Box<dyn Typ>>>,
    constraints: Vec<TupleConstraint>,
}

impl TypTuple {
    fn unconstrained(fields: Option<HashMap<String, Box<dyn Typ>>>) -> Self {
        TypTuple {
            fields,
            constraints: vec![]
        }
    }

    fn constrained(fields: Option<HashMap<String, Box<dyn Typ>>>, constraints: Vec<TupleConstraint>) -> Self {
        TypTuple {
            fields,
            constraints
        }
    }
}

pub struct CollectionTyp {
    homogeneity: CollectionHomogeneity,
    constraints: Vec<CollectionConstraint>
}

impl CollectionTyp {
    fn bag() -> Self {
        CollectionTyp {
            homogeneity: CollectionHomogeneity::Heterogeneous,
            constraints: vec![CollectionConstraint::Ordered(false)]
        }
    }

    fn list() -> Self {
        CollectionTyp {
            homogeneity: CollectionHomogeneity::Heterogeneous,
            constraints: vec![CollectionConstraint::Ordered(true)]
        }
    }

    fn bag_of(typ: Box<dyn Typ>) -> Self {
        CollectionTyp {
            homogeneity: CollectionHomogeneity::Homogenous(typ),
            constraints: vec![CollectionConstraint::Ordered(false)]
        }
    }

    fn list_of(typ: Box<dyn Typ>) -> Self {
        CollectionTyp {
            homogeneity: CollectionHomogeneity::Homogenous(typ),
            constraints: vec![CollectionConstraint::Ordered(true)]
        }
    }

    fn schema_of(typ: Box<dyn Typ>, constraints: Vec<CollectionConstraint>) -> Self {
        let has_tuple_constraint = constraints.into_iter()
            .any(|c| matches!(c, CollectionConstraint::CollectionTupleConstraint(_)));
        match (typ.as_any().downcast_ref::<TypTuple>(), has_tuple_constraint) {
            (None, true) => panic!("Schema of non-tuple cannot have tuple constraints"),
            _ => CollectionTyp {
                homogeneity: CollectionHomogeneity::Homogenous(typ),
                constraints: vec![CollectionConstraint::Ordered(true)]
            }
        }
    }
}

struct Any;

#[cfg(test)]
mod tests {
    use std::collections::HashMap;
    use crate::{CollectionConstraint, CollectionTupleConstraint, CollectionTyp, IntConstraint, TupleConstraint, Typ, TypLit, TypTuple};

    #[test]
    fn create_int_type() {
        let my_int: Box<dyn Typ> = Box::new(TypLit::int());
        let my_int8: Box<dyn Typ> = Box::new(TypLit::int8());
        let my_int16: Box<dyn Typ> = Box::new(TypLit::int16());
        let f1: HashMap<String, Box<dyn Typ>> = HashMap::from([("a".to_string(), my_int8)]);
        let f2: HashMap<String, Box<dyn Typ>> = HashMap::from([("a".to_string(), my_int16)]);
        let my_struct = TypTuple::unconstrained(Some(f1));
        let my_struct_constrained = TypTuple::constrained(
            Some(f2),
            vec![TupleConstraint::Open(true), TupleConstraint::Ordered(true)]
        );

        struct SomeNode {
            id: i32,
            typ: Box<dyn Typ>
        }

        let node = SomeNode {
            id: 10,
            typ: my_int,
        };
    }

    #[test]
    #[should_panic]
    fn create_schema_fails() {
        let my_schema = CollectionTyp::schema_of(
            Box::new(TypLit::int16()),
            vec![CollectionConstraint::CollectionTupleConstraint(CollectionTupleConstraint::PartitionKey(vec!["h".to_string()]))]
        );
    }
}

