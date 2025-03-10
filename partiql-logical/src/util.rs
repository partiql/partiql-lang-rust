use crate::Lit;

use partiql_value::{Bag, List, Tuple, Value};

impl From<Value> for Lit {
    fn from(value: Value) -> Self {
        match value {
            Value::Null => Lit::Null,
            Value::Missing => Lit::Missing,
            Value::Boolean(b) => Lit::Bool(b),
            Value::Integer(n) => Lit::Int64(n),
            Value::Real(f) => Lit::Double(f),
            Value::Decimal(d) => Lit::Decimal(*d),
            Value::String(s) => Lit::String(*s),
            Value::Blob(_bytes) => {
                todo!("Value to Lit: Blob")
            }
            Value::DateTime(_dt) => {
                todo!("Value to Lit: DateTime")
            }
            Value::List(list) => (*list).into(),
            Value::Bag(bag) => (*bag).into(),
            Value::Tuple(tuple) => (*tuple).into(),
            Value::Graph(_) => todo!("Value to Lit: Graph"),
            Value::Variant(_) => {
                todo!("Value to Lit: Variant")
            }
        }
    }
}

impl From<List> for Lit {
    fn from(list: List) -> Self {
        Lit::List(list.into_iter().map(Lit::from).collect())
    }
}

impl From<Bag> for Lit {
    fn from(bag: Bag) -> Self {
        Lit::Bag(bag.into_iter().map(Lit::from).collect())
    }
}

impl From<Tuple> for Lit {
    fn from(tuple: Tuple) -> Self {
        Lit::Struct(tuple.into_iter().map(|(k, v)| (k, Lit::from(v))).collect())
    }
}

impl From<bool> for Lit {
    #[inline]
    fn from(b: bool) -> Self {
        Lit::Bool(b)
    }
}

impl From<String> for Lit {
    #[inline]
    fn from(s: String) -> Self {
        Lit::String(s)
    }
}

impl From<&str> for Lit {
    #[inline]
    fn from(s: &str) -> Self {
        Lit::String(s.to_string())
    }
}

impl From<i64> for Lit {
    #[inline]
    fn from(n: i64) -> Self {
        Lit::Int64(n)
    }
}

impl From<i32> for Lit {
    #[inline]
    fn from(n: i32) -> Self {
        i64::from(n).into()
    }
}

impl From<i16> for Lit {
    #[inline]
    fn from(n: i16) -> Self {
        i64::from(n).into()
    }
}

impl From<i8> for Lit {
    #[inline]
    fn from(n: i8) -> Self {
        i64::from(n).into()
    }
}

impl From<usize> for Lit {
    #[inline]
    fn from(n: usize) -> Self {
        // TODO overflow to bigint/decimal
        Lit::Int64(n as i64)
    }
}

impl From<u8> for Lit {
    #[inline]
    fn from(n: u8) -> Self {
        (n as usize).into()
    }
}

impl From<u16> for Lit {
    #[inline]
    fn from(n: u16) -> Self {
        (n as usize).into()
    }
}

impl From<u32> for Lit {
    #[inline]
    fn from(n: u32) -> Self {
        (n as usize).into()
    }
}

impl From<u64> for Lit {
    #[inline]
    fn from(n: u64) -> Self {
        (n as usize).into()
    }
}
