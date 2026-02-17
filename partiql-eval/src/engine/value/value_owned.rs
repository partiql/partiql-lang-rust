use ordered_float::OrderedFloat;
use rust_decimal::Decimal as RustDecimal;

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
}
