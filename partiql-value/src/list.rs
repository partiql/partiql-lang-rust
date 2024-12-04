use std::cmp::Ordering;

use std::fmt::{Debug, Formatter};
use std::hash::{Hash, Hasher};

use std::{slice, vec};

use crate::{Bag, EqualityValue, NullSortedValue, NullableEq, Value};
#[cfg(feature = "serde")]
use serde::{Deserialize, Serialize};

#[derive(Default, Eq, Clone)]
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
/// Represents a `PartiQL` List value, e.g. [1, 2, 'one']
pub struct List(Vec<Value>);

impl List {
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
    pub fn get(&self, idx: i64) -> Option<&Value> {
        self.0.get(idx as usize)
    }

    #[inline]
    pub fn get_mut(&mut self, idx: i64) -> Option<&mut Value> {
        self.0.get_mut(idx as usize)
    }

    #[inline]
    #[must_use]
    pub fn take_val(self, idx: i64) -> Option<Value> {
        self.0.into_iter().nth(idx as usize)
    }

    #[inline]
    #[must_use]
    pub fn iter(&self) -> ListIter<'_> {
        ListIter(self.0.iter())
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

impl Extend<Value> for List {
    #[inline]
    fn extend<Iter: IntoIterator<Item = Value>>(&mut self, iter: Iter) {
        let iterator = iter.into_iter();
        let (lower_bound, _) = iterator.size_hint();
        self.reserve(lower_bound);
        iterator.for_each(move |v| self.push(v));
    }
}

impl From<Vec<Value>> for List {
    #[inline]
    fn from(values: Vec<Value>) -> Self {
        List(values)
    }
}

impl From<Bag> for List {
    #[inline]
    fn from(bag: Bag) -> Self {
        List(bag.to_vec())
    }
}

impl<T> FromIterator<T> for List
where
    T: Into<Value>,
{
    #[inline]
    fn from_iter<I: IntoIterator<Item = T>>(iter: I) -> List {
        let iterator = iter.into_iter().map(Into::into);
        iterator.collect::<Vec<_>>().into()
    }
}

#[macro_export]
macro_rules! list {
    () => (
        $crate::List::from(vec![])
    );
    ($elem:expr; $n:expr) => (
        $crate::List::from(vec![Value::from($elem); $n])
    );
    ($($x:expr),+ $(,)?) => (
        $crate::List::from(vec![$(Value::from($x)),+])
    );
}

impl<'a> IntoIterator for &'a List {
    type Item = &'a Value;
    type IntoIter = ListIter<'a>;

    fn into_iter(self) -> Self::IntoIter {
        ListIter(self.0.iter())
    }
}

#[derive(Debug, Clone)]
pub struct ListIter<'a>(slice::Iter<'a, Value>);

impl<'a> Iterator for ListIter<'a> {
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

impl IntoIterator for List {
    type Item = Value;
    type IntoIter = ListIntoIterator;

    fn into_iter(self) -> ListIntoIterator {
        ListIntoIterator(self.0.into_iter())
    }
}

pub struct ListIntoIterator(vec::IntoIter<Value>);

impl Iterator for ListIntoIterator {
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

impl Debug for List {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        f.debug_list().entries(&self.0).finish()
    }
}

impl PartialEq for List {
    fn eq(&self, other: &Self) -> bool {
        if self.len() != other.len() {
            return false;
        }
        for (v1, v2) in self.0.iter().zip(other.0.iter()) {
            let wrap = EqualityValue::<true, Value>;
            if NullableEq::eq(&wrap(v1), &wrap(v2)) != Value::Boolean(true) {
                return false;
            }
        }
        true
    }
}

impl PartialOrd for List {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}

impl<const NULLS_FIRST: bool> Ord for NullSortedValue<'_, NULLS_FIRST, List> {
    fn cmp(&self, other: &Self) -> Ordering {
        let wrap = NullSortedValue::<{ NULLS_FIRST }, _>;

        let mut l = self.0 .0.iter();
        let mut r = other.0 .0.iter();

        loop {
            match (l.next(), r.next()) {
                (None, None) => return Ordering::Equal,
                (Some(_), None) => return Ordering::Greater,
                (None, Some(_)) => return Ordering::Less,
                (Some(lv), Some(rv)) => match wrap(lv).cmp(&wrap(rv)) {
                    Ordering::Less => return Ordering::Less,
                    Ordering::Greater => return Ordering::Greater,
                    Ordering::Equal => continue,
                },
            }
        }
    }
}

impl Ord for List {
    fn cmp(&self, other: &Self) -> Ordering {
        let mut l = self.0.iter();
        let mut r = other.0.iter();

        loop {
            match (l.next(), r.next()) {
                (None, None) => return Ordering::Equal,
                (Some(_), None) => return Ordering::Greater,
                (None, Some(_)) => return Ordering::Less,
                (Some(lv), Some(rv)) => match lv.cmp(rv) {
                    Ordering::Less => return Ordering::Less,
                    Ordering::Greater => return Ordering::Greater,
                    Ordering::Equal => continue,
                },
            }
        }
    }
}

impl Hash for List {
    fn hash<H: Hasher>(&self, state: &mut H) {
        for v in &self.0 {
            v.hash(state);
        }
    }
}
