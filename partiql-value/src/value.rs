use ordered_float::OrderedFloat;
use std::borrow::Cow;
use std::fmt::{Debug, Display, Formatter};
use std::hash::Hash;

use rust_decimal::Decimal as RustDecimal;

use crate::variant::Variant;
use crate::{tuple, Bag, BindingIntoIter, BindingIter, DateTime, Graph, List, Tuple};
use rust_decimal::prelude::FromPrimitive;
#[cfg(feature = "serde")]
use serde::{Deserialize, Serialize};

mod iter;
mod logic;
mod math;

use crate::datum::{
    Datum, DatumCategory, DatumCategoryOwned, DatumCategoryRef, DatumLower, DatumLowerResult,
    DatumTupleOwned, DatumTupleRef, DatumValue,
};
pub use iter::*;
pub use logic::*;
pub use math::*;
use partiql_common::pretty::ToPretty;
use std::cmp::Ordering;

#[derive(Hash, PartialEq, Eq, Clone, Default)]
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
pub enum Value {
    Null,
    #[default]
    Missing,
    Boolean(bool),
    Integer(i64),
    Real(OrderedFloat<f64>),
    Decimal(Box<RustDecimal>),
    String(Box<String>),
    Blob(Box<Vec<u8>>),
    DateTime(Box<DateTime>),
    List(Box<List>),
    Bag(Box<Bag>),
    Tuple(Box<Tuple>),
    Graph(Box<Graph>),
    Variant(Box<Variant>),
}

impl Value {
    #[inline]
    #[must_use]
    pub fn is_tuple(&self) -> bool {
        matches!(self, Value::Tuple(_))
    }

    #[inline]
    #[must_use]
    pub fn is_list(&self) -> bool {
        matches!(self, Value::List(_))
    }

    #[inline]
    #[must_use]
    pub fn is_bag(&self) -> bool {
        matches!(self, Value::Bag(_))
    }

    #[inline]
    /// Returns true if and only if Value is an integer, real, or decimal
    #[must_use]
    pub fn is_number(&self) -> bool {
        matches!(self, Value::Integer(_) | Value::Real(_) | Value::Decimal(_))
    }

    #[inline]
    #[must_use]
    pub fn coerce_into_tuple(self) -> Tuple {
        let coerce = |non_tuple: Value| {
            non_tuple
                .into_bindings()
                .map(|(k, v)| (k.unwrap_or_else(|| "_1".to_string()), v))
                .collect()
        };

        match self {
            Value::Tuple(t) => *t,
            Value::Variant(ref v) => match v.category() {
                DatumCategoryRef::Tuple(_) => self
                    .into_lower()
                    .expect("variant lower")
                    .coerce_into_tuple(),
                _ => coerce(self),
            },
            _ => coerce(self),
        }
    }

    #[inline]
    #[must_use]
    pub fn coerce_to_tuple(&self) -> Tuple {
        self.clone().coerce_into_tuple()
    }

    #[inline]
    #[must_use]
    pub fn as_datum_tuple_ref(&self) -> DatumTupleRef<'_> {
        match self.category() {
            DatumCategoryRef::Tuple(t) => t,
            _ => DatumTupleRef::CoercedValue(1, self),
        }
    }

    #[inline]
    #[must_use]
    pub fn as_tuple_ref(&self) -> Cow<'_, Tuple> {
        match self.category() {
            DatumCategoryRef::Tuple(t) => match t {
                DatumTupleRef::Tuple(t) => Cow::Borrowed(t),
                DatumTupleRef::Dynamic(_) => match self.lower().expect("variant lower") {
                    Cow::Borrowed(Value::Tuple(t)) => Cow::Borrowed(t.as_ref()),
                    Cow::Owned(Value::Tuple(t)) => Cow::Owned(*t),
                    _ => unreachable!(),
                },
                DatumTupleRef::Empty => Cow::Owned(tuple![]),
                DatumTupleRef::CoercedValue(_, v) => Cow::Owned(tuple![("_1", v.clone())]),
                DatumTupleRef::SingleKey(k, v) => Cow::Owned(tuple![(k.as_ref(), v.clone())]),
                DatumTupleRef::Bindings(map) => Cow::Owned(Tuple::from_iter(
                    map.iter().map(|(k, &v)| (k.as_ref(), v.clone())),
                )),
            },
            _ => Cow::Owned(self.coerce_to_tuple()),
        }
    }

    #[inline]
    #[must_use]
    pub fn as_bindings(&self) -> BindingIter<'_> {
        match self.category() {
            DatumCategoryRef::Missing => BindingIter::Empty,
            DatumCategoryRef::Tuple(t) => match t {
                DatumTupleRef::Tuple(t) => BindingIter::Tuple(t.pairs()),
                DatumTupleRef::Dynamic(_) => unreachable!(),
                DatumTupleRef::Empty => BindingIter::Empty,
                DatumTupleRef::CoercedValue(_, value) => {
                    BindingIter::Single(std::iter::once(value))
                }
                DatumTupleRef::SingleKey(_, value) => BindingIter::Single(std::iter::once(value)),
                DatumTupleRef::Bindings(map) => {
                    let entries = map.iter().map(|(k, &v)| (Some(k.as_ref()), v));
                    BindingIter::Dynamic(Box::new(entries))
                }
            },
            _ => BindingIter::Single(std::iter::once(self)),
        }
    }

    #[inline]
    #[must_use]
    pub fn into_bindings(self) -> BindingIntoIter {
        match self {
            Value::Tuple(t) => BindingIntoIter::Tuple(t.into_pairs()),
            Value::Missing => BindingIntoIter::Empty,
            Value::Variant(v) => match v.category() {
                DatumCategoryRef::Missing => BindingIntoIter::Empty,
                DatumCategoryRef::Tuple(_) => match v.into_category() {
                    DatumCategoryOwned::Tuple(DatumTupleOwned::Tuple(t)) => {
                        BindingIntoIter::Tuple(t.into_pairs())
                    }
                    DatumCategoryOwned::Tuple(DatumTupleOwned::Dynamic(d)) => {
                        BindingIntoIter::DynTuple(d.into_iter_boxed())
                    }
                    _ => unreachable!(),
                },
                _ => BindingIntoIter::Single(std::iter::once(Value::Variant(v))),
            },
            _ => BindingIntoIter::Single(std::iter::once(self)),
        }
    }

    #[inline]
    #[must_use]
    pub fn coerce_into_bag(self) -> Bag {
        if let Value::Bag(b) = self {
            *b
        } else {
            Bag::from(vec![self])
        }
    }

    #[inline]
    #[must_use]
    pub fn as_bag_ref(&self) -> Cow<'_, Bag> {
        if let Value::Bag(b) = self {
            Cow::Borrowed(b)
        } else {
            Cow::Owned(self.clone().coerce_into_bag())
        }
    }

    #[inline]
    #[must_use]
    pub fn coerce_into_list(self) -> List {
        if let Value::List(b) = self {
            *b
        } else {
            List::from(vec![self])
        }
    }

    #[inline]
    #[must_use]
    pub fn as_list_ref(&self) -> Cow<'_, List> {
        if let Value::List(l) = self {
            Cow::Borrowed(l)
        } else {
            Cow::Owned(self.clone().coerce_into_list())
        }
    }

    #[inline]
    #[must_use]
    pub fn iter(&self) -> ValueIter<'_> {
        match self {
            Value::Null | Value::Missing => ValueIter::Single(None),
            Value::List(list) => ValueIter::List(list.iter()),
            Value::Bag(bag) => ValueIter::Bag(bag.iter()),
            other => ValueIter::Single(Some(other)),
        }
    }

    #[inline]
    #[must_use]
    pub fn sequence_iter(&self) -> Option<ValueIter<'_>> {
        if self.is_sequence() {
            Some(self.iter())
        } else {
            None
        }
    }
}

impl DatumValue<Value> for Value {}

impl DatumLower<Value> for Value {
    fn into_lower(self) -> DatumLowerResult<Value> {
        match self {
            Value::Variant(variant) => variant.into_lower(),
            _ => Ok(self),
        }
    }

    fn into_lower_boxed(self: Box<Self>) -> DatumLowerResult<Value> {
        self.into_lower()
    }

    fn lower(&self) -> DatumLowerResult<Cow<'_, Value>> {
        match self {
            Value::Variant(variant) => variant.lower(),
            _ => Ok(Cow::Borrowed(self)),
        }
    }
}

impl Datum<Value> for Value {
    #[inline]
    fn is_null(&self) -> bool {
        match self {
            Value::Null => true,
            Value::Variant(variant) => variant.is_null(),
            _ => false,
        }
    }

    #[inline]
    fn is_missing(&self) -> bool {
        match self {
            Value::Missing => true,
            Value::Variant(variant) => variant.is_missing(),
            _ => false,
        }
    }

    #[inline]
    fn is_sequence(&self) -> bool {
        match self {
            Value::List(_) => true,
            Value::Bag(_) => true,
            Value::Variant(variant) => variant.is_sequence(),
            _ => false,
        }
    }

    #[inline]
    fn is_ordered(&self) -> bool {
        match self {
            Value::List(_) => true,
            Value::Variant(variant) => variant.is_ordered(),
            _ => false,
        }
    }
}

impl Display for Value {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        match self.to_pretty_string(f.width().unwrap_or(80)) {
            Ok(pretty) => f.write_str(&pretty),
            Err(_) => f.write_str("<internal value error occurred>"),
        }
    }
}

impl Debug for Value {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        match self {
            Value::Null => write!(f, "NULL"),
            Value::Missing => write!(f, "MISSING"),
            Value::Boolean(b) => write!(f, "{b}"),
            Value::Integer(i) => write!(f, "{i}"),
            Value::Real(r) => write!(f, "{}", r.0),
            Value::Decimal(d) => write!(f, "{d}"),
            Value::String(s) => write!(f, "'{s}'"),
            Value::Blob(blob) => {
                write!(f, "x'")?;
                for byte in blob.as_ref() {
                    f.write_str(&format!("{byte:02x}"))?;
                }
                write!(f, "'")
            }
            Value::DateTime(t) => Debug::fmt(&t, f),
            Value::List(l) => Debug::fmt(&l, f),
            Value::Bag(b) => Debug::fmt(&b, f),
            Value::Tuple(t) => Debug::fmt(&t, f),
            Value::Graph(g) => Debug::fmt(&g, f),
            Value::Variant(v) => Debug::fmt(&v, f),
        }
    }
}

impl PartialOrd for Value {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}

/// Implementation of spec's `order-by less-than` assuming nulls first.
/// TODO: more tests for Ord on Value
impl Ord for Value {
    fn cmp(&self, other: &Self) -> Ordering {
        // **NOTE** The Order of these match arms defines the proper comparisons; Do not reorder
        match (self, other) {
            (Value::Null, Value::Null) => Ordering::Equal,
            (Value::Missing, Value::Null) => Ordering::Equal,

            (Value::Null, Value::Missing) => Ordering::Equal,
            (Value::Null, _) => Ordering::Less,
            (_, Value::Null) => Ordering::Greater,

            (Value::Missing, Value::Missing) => Ordering::Equal,
            (Value::Missing, _) => Ordering::Less,
            (_, Value::Missing) => Ordering::Greater,

            (Value::Boolean(l), Value::Boolean(r)) => match (l, r) {
                (false, true) => Ordering::Less,
                (true, false) => Ordering::Greater,
                (_, _) => Ordering::Equal,
            },
            (Value::Boolean(_), _) => Ordering::Less,
            (_, Value::Boolean(_)) => Ordering::Greater,

            // TODO: `OrderedFloat`'s implementation of `Ord` slightly differs from what we want in
            //  the PartiQL spec. See https://partiql.org/assets/PartiQL-Specification.pdf#subsection.12.2
            //  point 3. In PartiQL, `nan`, comes before `-inf` which comes before all numeric
            //  values, which are followed by `+inf`. `OrderedFloat` places `NaN` as greater than
            //  all other `OrderedFloat` values. We could consider creating our own float type
            //  to get around this annoyance.
            (Value::Real(l), Value::Real(r)) => {
                if l.is_nan() {
                    if r.is_nan() {
                        Ordering::Equal
                    } else {
                        Ordering::Less
                    }
                } else if r.is_nan() {
                    Ordering::Greater
                } else {
                    l.cmp(r)
                }
            }
            (Value::Integer(l), Value::Integer(r)) => l.cmp(r),
            (Value::Decimal(l), Value::Decimal(r)) => l.cmp(r),
            (Value::Integer(l), Value::Real(_)) => {
                Value::Real(ordered_float::OrderedFloat(*l as f64)).cmp(other)
            }
            (Value::Real(_), Value::Integer(r)) => {
                self.cmp(&Value::Real(ordered_float::OrderedFloat(*r as f64)))
            }
            (Value::Integer(l), Value::Decimal(r)) => RustDecimal::from(*l).cmp(r),
            (Value::Decimal(l), Value::Integer(r)) => l.as_ref().cmp(&RustDecimal::from(*r)),
            (Value::Real(l), Value::Decimal(r)) => {
                if l.is_nan() || l.0 == f64::NEG_INFINITY {
                    Ordering::Less
                } else if l.0 == f64::INFINITY {
                    Ordering::Greater
                } else {
                    match RustDecimal::from_f64(l.0) {
                        Some(l_d) => l_d.cmp(r),
                        None => todo!(
                            "Decide default behavior when f64 can't be converted to RustDecimal"
                        ),
                    }
                }
            }
            (Value::Decimal(l), Value::Real(r)) => {
                if r.is_nan() || r.0 == f64::NEG_INFINITY {
                    Ordering::Greater
                } else if r.0 == f64::INFINITY {
                    Ordering::Less
                } else {
                    match RustDecimal::from_f64(r.0) {
                        Some(r_d) => l.as_ref().cmp(&r_d),
                        None => todo!(
                            "Decide default behavior when f64 can't be converted to RustDecimal"
                        ),
                    }
                }
            }
            (Value::Integer(_), _) => Ordering::Less,
            (Value::Real(_), _) => Ordering::Less,
            (Value::Decimal(_), _) => Ordering::Less,
            (_, Value::Integer(_)) => Ordering::Greater,
            (_, Value::Real(_)) => Ordering::Greater,
            (_, Value::Decimal(_)) => Ordering::Greater,

            (Value::DateTime(l), Value::DateTime(r)) => l.cmp(r),
            (Value::DateTime(_), _) => Ordering::Less,
            (_, Value::DateTime(_)) => Ordering::Greater,

            (Value::String(l), Value::String(r)) => l.cmp(r),
            (Value::String(_), _) => Ordering::Less,
            (_, Value::String(_)) => Ordering::Greater,

            (Value::Blob(l), Value::Blob(r)) => l.cmp(r),
            (Value::Blob(_), _) => Ordering::Less,
            (_, Value::Blob(_)) => Ordering::Greater,

            (Value::List(l), Value::List(r)) => l.cmp(r),
            (Value::List(_), _) => Ordering::Less,
            (_, Value::List(_)) => Ordering::Greater,

            (Value::Tuple(l), Value::Tuple(r)) => l.cmp(r),
            (Value::Tuple(_), _) => Ordering::Less,
            (_, Value::Tuple(_)) => Ordering::Greater,

            (Value::Bag(l), Value::Bag(r)) => l.cmp(r),
            (Value::Bag(_), _) => Ordering::Less,
            (_, Value::Bag(_)) => Ordering::Greater,

            // TODO need to RFC graph comparison behavior
            (Value::Graph(l), Value::Graph(r)) => l.cmp(r),
            (Value::Graph(_), _) => Ordering::Less,
            (_, Value::Graph(_)) => Ordering::Greater,

            (Value::Variant(l), Value::Variant(r)) => l.cmp(r),
        }
    }
}

impl From<bool> for Value {
    #[inline]
    fn from(b: bool) -> Self {
        Value::Boolean(b)
    }
}

impl From<String> for Value {
    #[inline]
    fn from(s: String) -> Self {
        Value::String(Box::new(s))
    }
}

impl From<&str> for Value {
    #[inline]
    fn from(s: &str) -> Self {
        Value::String(Box::new(s.to_string()))
    }
}

impl From<i128> for Value {
    #[inline]
    fn from(n: i128) -> Self {
        Value::from(RustDecimal::from(n))
    }
}

impl From<i64> for Value {
    #[inline]
    fn from(n: i64) -> Self {
        Value::Integer(n)
    }
}

impl From<i32> for Value {
    #[inline]
    fn from(n: i32) -> Self {
        i64::from(n).into()
    }
}

impl From<i16> for Value {
    #[inline]
    fn from(n: i16) -> Self {
        i64::from(n).into()
    }
}

impl From<i8> for Value {
    #[inline]
    fn from(n: i8) -> Self {
        i64::from(n).into()
    }
}

impl From<usize> for Value {
    #[inline]
    fn from(n: usize) -> Self {
        if n > i64::MAX as usize {
            Value::from(RustDecimal::from(n))
        } else {
            Value::Integer(n as i64)
        }
    }
}

impl From<u8> for Value {
    #[inline]
    fn from(n: u8) -> Self {
        (n as usize).into()
    }
}

impl From<u16> for Value {
    #[inline]
    fn from(n: u16) -> Self {
        (n as usize).into()
    }
}

impl From<u32> for Value {
    #[inline]
    fn from(n: u32) -> Self {
        (n as usize).into()
    }
}

impl From<u64> for Value {
    #[inline]
    fn from(n: u64) -> Self {
        (n as usize).into()
    }
}

impl From<u128> for Value {
    #[inline]
    fn from(n: u128) -> Self {
        Value::from(RustDecimal::from(n))
    }
}

impl From<f64> for Value {
    #[inline]
    fn from(f: f64) -> Self {
        Value::from(OrderedFloat(f))
    }
}

impl From<OrderedFloat<f64>> for Value {
    #[inline]
    fn from(f: OrderedFloat<f64>) -> Self {
        Value::Real(f)
    }
}

impl From<RustDecimal> for Value {
    #[inline]
    fn from(d: RustDecimal) -> Self {
        Value::Decimal(Box::new(d))
    }
}

impl From<DateTime> for Value {
    #[inline]
    fn from(t: DateTime) -> Self {
        Value::DateTime(Box::new(t))
    }
}

impl From<List> for Value {
    #[inline]
    fn from(v: List) -> Self {
        Value::List(Box::new(v))
    }
}

impl From<Tuple> for Value {
    #[inline]
    fn from(v: Tuple) -> Self {
        Value::Tuple(Box::new(v))
    }
}

impl From<Bag> for Value {
    #[inline]
    fn from(v: Bag) -> Self {
        Value::Bag(Box::new(v))
    }
}

impl From<Graph> for Value {
    #[inline]
    fn from(g: Graph) -> Self {
        Value::Graph(Box::new(g))
    }
}

impl From<Variant> for Value {
    #[inline]
    fn from(v: Variant) -> Self {
        Value::Variant(Box::new(v))
    }
}
