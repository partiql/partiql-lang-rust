use ordered_float::OrderedFloat;
use std::borrow::Cow;
use std::hash::Hash;

use rust_decimal::Decimal as RustDecimal;

use crate::{Bag, BindingIntoIter, BindingIter, DateTime, List, Tuple};
#[cfg(feature = "serde")]
use serde::{Deserialize, Serialize};

mod iter;
mod logic;
mod math;

pub use iter::*;
pub use logic::*;
pub use math::*;

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
    // TODO: add other supported PartiQL values -- sexp
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
    #[must_use]
    pub fn is_sequence(&self) -> bool {
        self.is_bag() || self.is_list()
    }

    #[inline]
    /// Returns true if and only if Value is an integer, real, or decimal
    #[must_use]
    pub fn is_number(&self) -> bool {
        matches!(self, Value::Integer(_) | Value::Real(_) | Value::Decimal(_))
    }
    #[inline]
    /// Returns true if and only if Value is null or missing
    #[must_use]
    pub fn is_absent(&self) -> bool {
        matches!(self, Value::Missing | Value::Null)
    }

    #[inline]
    /// Returns true if Value is neither null nor missing
    #[must_use]
    pub fn is_present(&self) -> bool {
        !self.is_absent()
    }

    #[inline]
    #[must_use]
    pub fn is_ordered(&self) -> bool {
        self.is_list()
    }

    #[inline]
    #[must_use]
    pub fn coerce_into_tuple(self) -> Tuple {
        match self {
            Value::Tuple(t) => *t,
            _ => self
                .into_bindings()
                .map(|(k, v)| (k.unwrap_or_else(|| "_1".to_string()), v))
                .collect(),
        }
    }

    #[inline]
    #[must_use]
    pub fn coerce_to_tuple(&self) -> Tuple {
        match self {
            Value::Tuple(t) => t.as_ref().clone(),
            _ => {
                let fresh = "_1".to_string();
                self.as_bindings()
                    .map(|(k, v)| (k.unwrap_or(&fresh), v.clone()))
                    .collect()
            }
        }
    }

    #[inline]
    #[must_use]
    pub fn as_tuple_ref(&self) -> Cow<'_, Tuple> {
        if let Value::Tuple(t) = self {
            Cow::Borrowed(t)
        } else {
            Cow::Owned(self.coerce_to_tuple())
        }
    }

    #[inline]
    #[must_use]
    pub fn as_bindings(&self) -> BindingIter<'_> {
        match self {
            Value::Tuple(t) => BindingIter::Tuple(t.pairs()),
            Value::Missing => BindingIter::Empty,
            _ => BindingIter::Single(std::iter::once(self)),
        }
    }

    #[inline]
    #[must_use]
    pub fn into_bindings(self) -> BindingIntoIter {
        match self {
            Value::Tuple(t) => BindingIntoIter::Tuple(t.into_pairs()),
            Value::Missing => BindingIntoIter::Empty,
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
