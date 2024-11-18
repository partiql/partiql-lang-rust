use crate::Lit;
use partiql_extension_ion::decode::{IonDecoderBuilder, IonDecoderConfig};
use partiql_extension_ion::Encoding;
use partiql_value::{Bag, List, Tuple, Value};
use thiserror::Error;

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

impl From<Lit> for Value {
    fn from(lit: Lit) -> Self {
        match lit {
            Lit::Null => Value::Null,
            Lit::Missing => Value::Missing,
            Lit::Int8(n) => Value::Integer(n.into()),
            Lit::Int16(n) => Value::Integer(n.into()),
            Lit::Int32(n) => Value::Integer(n.into()),
            Lit::Int64(n) => Value::Integer(n),
            Lit::Decimal(d) => Value::Decimal(d.into()),
            Lit::Double(f) => Value::Real(f),
            Lit::Bool(b) => Value::Boolean(b),
            Lit::String(s) => Value::String(s.into()),
            Lit::BoxDocument(contents, _typ) => {
                parse_embedded_ion_str(&String::from_utf8_lossy(contents.as_slice()))
                    .expect("TODO ion parsing error")
            }
            Lit::Struct(strct) => Value::from(Tuple::from_iter(
                strct.into_iter().map(|(k, v)| (k, Value::from(v))),
            )),
            Lit::Bag(bag) => Value::from(Bag::from_iter(bag.into_iter().map(Value::from))),
            Lit::List(list) => Value::from(List::from_iter(list.into_iter().map(Value::from))),
        }
    }
}

/// Represents a Literal Value Error
#[derive(Error, Debug, Clone, PartialEq, Eq, Hash)]
#[non_exhaustive]
pub enum LiteralError {
    /// Indicates that there was an error interpreting a literal value.
    #[error("Error with literal: {literal}: {error}")]
    Literal { literal: String, error: String },
}

// TODO remove parsing in favor of embedding
fn parse_embedded_ion_str(contents: &str) -> Result<Value, LiteralError> {
    fn lit_err(literal: &str, err: impl std::error::Error) -> LiteralError {
        LiteralError::Literal {
            literal: literal.into(),
            error: err.to_string(),
        }
    }

    let reader = ion_rs_old::ReaderBuilder::new()
        .build(contents)
        .map_err(|e| lit_err(contents, e))?;
    let mut iter = IonDecoderBuilder::new(IonDecoderConfig::default().with_mode(Encoding::Ion))
        .build(reader)
        .map_err(|e| lit_err(contents, e))?;

    iter.next()
        .ok_or_else(|| LiteralError::Literal {
            literal: contents.into(),
            error: "Contains no value".into(),
        })?
        .map_err(|e| lit_err(contents, e))
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
