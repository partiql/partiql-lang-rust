use itertools::Itertools;

use std::cmp::Ordering;

use std::collections::HashSet;
use std::fmt::{Debug, Formatter};
use std::hash::{Hash, Hasher};

use std::{slice, vec};

use crate::sort::NullSortedValue;
use crate::{EqualityValue, List, NullableEq, Value};
#[cfg(feature = "serde")]
use serde::{Deserialize, Serialize};

#[derive(Default, Eq, Clone)]
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
/// Represents a `PartiQL` BAG value, e.g.: <<1, 'two', 4>>
pub struct Bag(Vec<Value>);

impl Bag {
    #[inline]
    pub fn push(&mut self, value: Value) {
        self.0.push(value);
    }

    #[inline]
    #[must_use]
    pub fn len(&self) -> usize {
        self.0.len()
    }

    #[inline]
    #[must_use]
    pub fn is_empty(&self) -> bool {
        self.len() == 0
    }

    #[inline]
    #[must_use]
    pub fn iter(&self) -> BagIter<'_> {
        BagIter(self.0.iter())
    }

    #[inline]
    pub fn reserve(&mut self, additional: usize) {
        self.0.reserve(additional);
    }

    #[inline]
    #[must_use]
    pub fn to_vec(self) -> Vec<Value> {
        self.0
    }
}

impl Extend<Value> for Bag {
    #[inline]
    fn extend<Iter: IntoIterator<Item = Value>>(&mut self, iter: Iter) {
        let iterator = iter.into_iter();
        let (lower_bound, _) = iterator.size_hint();
        self.reserve(lower_bound);
        iterator.for_each(move |v| self.push(v));
    }
}

impl From<Vec<Value>> for Bag {
    #[inline]
    fn from(values: Vec<Value>) -> Self {
        Bag(values)
    }
}

impl From<HashSet<Value>> for Bag {
    #[inline]
    fn from(values: HashSet<Value>) -> Self {
        Bag(values.into_iter().collect())
    }
}

impl From<List> for Bag {
    #[inline]
    fn from(list: List) -> Self {
        Bag(list.to_vec())
    }
}

impl<T> FromIterator<T> for Bag
where
    T: Into<Value>,
{
    #[inline]
    fn from_iter<I: IntoIterator<Item = T>>(iter: I) -> Bag {
        let iterator = iter.into_iter().map(Into::into);
        iterator.collect::<Vec<_>>().into()
    }
}

#[macro_export]
macro_rules! bag {
    () => (
        $crate::Bag::from(vec![])
    );
    ($elem:expr; $n:expr) => (
        $crate::Bag::from(vec![Value::from($elem); $n])
    );
    ($($x:expr),+ $(,)?) => (
        $crate::Bag::from(vec![$(Value::from($x)),+])
    );
}

impl<'a> IntoIterator for &'a Bag {
    type Item = &'a Value;
    type IntoIter = BagIter<'a>;

    #[inline]
    fn into_iter(self) -> Self::IntoIter {
        BagIter(self.0.iter())
    }
}

#[derive(Debug, Clone)]
pub struct BagIter<'a>(slice::Iter<'a, Value>);

impl<'a> Iterator for BagIter<'a> {
    type Item = &'a Value;

    #[inline]
    fn next(&mut self) -> Option<Self::Item> {
        self.0.next()
    }

    #[inline]
    fn size_hint(&self) -> (usize, Option<usize>) {
        self.0.size_hint()
    }
}

impl IntoIterator for Bag {
    type Item = Value;
    type IntoIter = BagIntoIterator;

    fn into_iter(self) -> BagIntoIterator {
        BagIntoIterator(self.0.into_iter())
    }
}

pub struct BagIntoIterator(vec::IntoIter<Value>);

impl Iterator for BagIntoIterator {
    type Item = Value;

    #[inline]
    fn next(&mut self) -> Option<Self::Item> {
        self.0.next()
    }

    #[inline]
    fn size_hint(&self) -> (usize, Option<usize>) {
        self.0.size_hint()
    }
}

impl Debug for Bag {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f, "<<")?;
        let mut iter = self.iter().peekable();
        while let Some(v) = iter.next() {
            if iter.peek().is_some() {
                write!(f, "{v:?}, ")?;
            } else {
                write!(f, "{v:?}")?;
            }
        }
        write!(f, ">>")
    }
}

impl PartialEq for Bag {
    fn eq(&self, other: &Self) -> bool {
        let wrap = EqualityValue::<true, false, _>;
        NullableEq::eq(&wrap(self), &wrap(other)) == Value::Boolean(true)
    }
}

impl<const NULLS_EQUAL: bool, const NAN_EQUAL: bool> NullableEq
    for EqualityValue<'_, NULLS_EQUAL, NAN_EQUAL, Bag>
{
    #[inline(always)]
    fn eq(&self, other: &Self) -> Value {
        let ord_wrap = NullSortedValue::<'_, false, _>;
        let (l, r) = (self.0, other.0);
        if l.len() != r.len() {
            return Value::Boolean(false);
        }

        let li = l.iter().map(ord_wrap).sorted().map(|nsv| nsv.0);
        let ri = r.iter().map(ord_wrap).sorted().map(|nsv| nsv.0);

        for (v1, v2) in li.zip(ri) {
            let wrap = EqualityValue::<{ NULLS_EQUAL }, { NAN_EQUAL }, Value>;
            if NullableEq::eqg(&wrap(v1), &wrap(v2)) != Value::Boolean(true) {
                return Value::Boolean(false);
            }
        }
        Value::Boolean(true)
    }

    #[inline(always)]
    fn eqg(&self, rhs: &Self) -> Value {
        let wrap = EqualityValue::<'_, true, { NAN_EQUAL }, _>;
        NullableEq::eq(&wrap(self.0), &wrap(rhs.0))
    }
}

impl PartialOrd for Bag {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}

impl<const NULLS_FIRST: bool> Ord for NullSortedValue<'_, NULLS_FIRST, Bag> {
    fn cmp(&self, other: &Self) -> Ordering {
        let wrap = NullSortedValue::<{ NULLS_FIRST }, List>;

        let mut l = self.0.clone();
        l.0.sort();
        let mut r = other.0.clone();
        r.0.sort();
        wrap(&List::from(l)).cmp(&wrap(&List::from(r)))
    }
}

impl Ord for Bag {
    fn cmp(&self, other: &Self) -> Ordering {
        let mut l = self.0.clone();
        l.sort();
        let mut r = other.0.clone();
        r.sort();
        List::from(l).cmp(&List::from(r))
    }
}

impl Hash for Bag {
    fn hash<H: Hasher>(&self, state: &mut H) {
        for v in self.0.iter().sorted() {
            v.hash(state);
        }
    }
}
