use ordered_float::OrderedFloat;
use rust_decimal::Decimal as RustDecimal;
use std::fmt;

/// Owned tuple representation for arena-allocated value storage
#[derive(Clone, Debug, PartialEq, Eq, Hash)]
pub(crate) struct TupleOwned {
    pub fields: Vec<TupleFieldOwned>,
}

/// A single owned field in a tuple - name and value
#[derive(Clone, Debug, PartialEq, Eq, Hash)]
pub(crate) struct TupleFieldOwned {
    pub name: String,
    pub value: ValueOwned,
}

/// Owned value representation that mirrors ValueRef but with owned data
///
/// This type owns all its data (no pointers/references) and is naturally Send + Sync.
/// ValueRef can borrow from ValueOwned to provide zero-copy access patterns.
#[derive(Clone, Debug, PartialEq, Eq, Hash)]
pub(crate) enum ValueOwned {
    Missing,
    Null,
    Bool(bool),
    I64(i64),
    F64(OrderedFloat<f64>),
    Decimal(RustDecimal),
    String(String),
    #[allow(dead_code)]
    Bytes(Vec<u8>),
    Tuple(TupleOwned),
    List(Vec<ValueOwned>),
    Bag(Vec<ValueOwned>),
    /// Ion variant literal: (raw_bytes, type_name)
    Variant(Vec<u8>, String),
}

impl fmt::Display for ValueOwned {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            ValueOwned::Missing => write!(f, "Missing"),
            ValueOwned::Null => write!(f, "Null"),
            ValueOwned::Bool(b) => write!(f, "Bool({b})"),
            ValueOwned::I64(n) => write!(f, "I64({n})"),
            ValueOwned::F64(n) => write!(f, "F64({n})"),
            ValueOwned::Decimal(d) => write!(f, "Decimal({d})"),
            ValueOwned::String(s) => write!(f, "String(\"{}\")", s.escape_default()),
            ValueOwned::Bytes(b) => write!(f, "Bytes(<{} bytes>)", b.len()),
            ValueOwned::Tuple(t) => {
                write!(f, "{{ ")?;
                for (i, field) in t.fields.iter().enumerate() {
                    if i > 0 {
                        write!(f, ", ")?;
                    }
                    write!(f, "'{}': {}", field.name, field.value)?;
                }
                write!(f, " }}")
            }
            ValueOwned::List(items) => {
                write!(f, "[")?;
                for (i, item) in items.iter().enumerate() {
                    if i > 0 {
                        write!(f, ", ")?;
                    }
                    write!(f, "{item}")?;
                }
                write!(f, "]")
            }
            ValueOwned::Bag(items) => {
                write!(f, "<< ")?;
                for (i, item) in items.iter().enumerate() {
                    if i > 0 {
                        write!(f, ", ")?;
                    }
                    write!(f, "{item}")?;
                }
                write!(f, " >>")
            }
            ValueOwned::Variant(bytes, type_name) => {
                let text = std::str::from_utf8(bytes).unwrap_or("<invalid utf8>");
                write!(f, "Variant({type_name}::{text})")
            }
        }
    }
}
