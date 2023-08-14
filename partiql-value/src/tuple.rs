use itertools::Itertools;

use std::cmp::Ordering;

use std::fmt::{Debug, Formatter};
use std::hash::{Hash, Hasher};
use std::iter::{zip, Zip};
use std::vec;

use unicase::UniCase;

use crate::{BindingsName, NullSortedValue, Value};
#[cfg(feature = "serde")]
use serde::{Deserialize, Serialize};

#[derive(Default, Eq, Clone)]
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
pub struct Tuple {
    attrs: Vec<String>,
    vals: Vec<Value>,
}

impl Tuple {
    pub fn new() -> Self {
        Tuple {
            attrs: vec![],
            vals: vec![],
        }
    }

    #[inline]
    pub fn insert(&mut self, attr: &str, val: Value) {
        self.attrs.push(attr.to_string());
        self.vals.push(val);
    }

    #[inline]
    pub fn len(&self) -> usize {
        self.attrs.len()
    }

    #[inline]
    pub fn is_empty(&self) -> bool {
        self.len() == 0
    }

    #[inline]
    /// Creates a `Tuple` containing all the attributes and values provided by `self` and `other`
    /// removing duplicate attributes. Assumes that `self` contains unique attributes and `other`
    /// contains unique attributes. In the case of duplicate attributes between `self` and `other`,
    /// the result `Tuple` will contain the attribute provided by `other`. See section 3.4 of the
    /// spec for details: https://partiql.org/assets/PartiQL-Specification.pdf#subsection.3.4.
    pub fn tuple_concat(&self, other: &Tuple) -> Self {
        other
            .pairs()
            .chain(self.pairs())
            .map(|(a, v)| (a, v.clone()))
            .unique_by(|(a, _)| *a)
            .collect()
    }

    #[inline]
    pub fn get(&self, attr: &BindingsName) -> Option<&Value> {
        match attr {
            BindingsName::CaseSensitive(s) => match self.attrs.iter().position(|a| a.as_str() == s)
            {
                Some(i) => Some(&self.vals[i]),
                _ => None,
            },
            BindingsName::CaseInsensitive(s) => match self
                .attrs
                .iter()
                .position(|a| UniCase::<&String>::from(a) == UniCase::<&String>::from(s))
            {
                Some(i) => Some(&self.vals[i]),
                _ => None,
            },
        }
    }

    #[inline]
    pub fn remove(&mut self, attr: &BindingsName) -> Option<Value> {
        match attr {
            BindingsName::CaseSensitive(s) => match self.attrs.iter().position(|a| a.as_str() == s)
            {
                Some(i) => {
                    self.attrs.remove(i);
                    Some(self.vals.remove(i))
                }
                _ => None,
            },
            BindingsName::CaseInsensitive(s) => match self
                .attrs
                .iter()
                .position(|a| UniCase::<&String>::from(a) == UniCase::<&String>::from(s))
            {
                Some(i) => {
                    self.attrs.remove(i);
                    Some(self.vals.remove(i))
                }
                _ => None,
            },
        }
    }

    #[inline]
    pub fn pairs(&self) -> PairsIter {
        let attrs = self.attrs.iter();
        PairsIter(zip(attrs, self.vals.iter()))
    }

    #[inline]
    pub fn into_pairs(self) -> PairsIntoIter {
        PairsIntoIter(zip(self.attrs, self.vals))
    }

    #[inline]
    pub fn values(&self) -> impl Iterator<Item = &Value> + Clone {
        self.vals.iter()
    }

    #[inline]
    pub fn into_values(self) -> impl Iterator<Item = Value> {
        self.vals.into_iter()
    }
}

#[derive(Debug, Clone)]
pub struct PairsIter<'a>(Zip<std::slice::Iter<'a, String>, std::slice::Iter<'a, Value>>);

impl<'a> Iterator for PairsIter<'a> {
    type Item = (&'a String, &'a Value);

    #[inline]
    fn next(&mut self) -> Option<Self::Item> {
        self.0.next()
    }

    #[inline]
    fn size_hint(&self) -> (usize, Option<usize>) {
        self.0.size_hint()
    }
}

#[derive(Debug, Clone)]
pub struct PairsIntoIter(Zip<std::vec::IntoIter<String>, std::vec::IntoIter<Value>>);

impl Iterator for PairsIntoIter {
    type Item = (String, Value);

    #[inline]
    fn next(&mut self) -> Option<Self::Item> {
        self.0.next()
    }

    #[inline]
    fn size_hint(&self) -> (usize, Option<usize>) {
        self.0.size_hint()
    }
}

impl<const N: usize, T> From<[(&str, T); N]> for Tuple
where
    T: Into<Value>,
{
    #[inline]
    fn from(arr: [(&str, T); N]) -> Self {
        arr.into_iter()
            .fold(Tuple::new(), |mut acc: Tuple, (attr, val)| {
                acc.insert(attr, val.into());
                acc
            })
    }
}

impl<S, T> FromIterator<(S, T)> for Tuple
where
    S: Into<String>,
    T: Into<Value>,
{
    #[inline]
    fn from_iter<I: IntoIterator<Item = (S, T)>>(iter: I) -> Tuple {
        let iterator = iter.into_iter();
        let (lower, _) = iterator.size_hint();
        let mut attrs = Vec::with_capacity(lower);
        let mut vals = Vec::with_capacity(lower);
        for (k, v) in iterator {
            attrs.push(k.into());
            vals.push(v.into());
        }
        Tuple { attrs, vals }
    }
}

impl Iterator for Tuple {
    type Item = (String, Value);

    #[inline]
    fn next(&mut self) -> Option<Self::Item> {
        match (self.attrs.pop(), self.vals.pop()) {
            (Some(attr), Some(val)) => Some((attr, val)),
            _ => None,
        }
    }

    #[inline]
    fn size_hint(&self) -> (usize, Option<usize>) {
        (self.attrs.len(), Some(self.attrs.len()))
    }
}

impl PartialEq for Tuple {
    fn eq(&self, other: &Self) -> bool {
        self.pairs().sorted().eq(other.pairs().sorted())
    }
}

impl PartialOrd for Tuple {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}

impl Hash for Tuple {
    fn hash<H: Hasher>(&self, state: &mut H) {
        for (k, v) in self.pairs().sorted() {
            k.hash(state);
            v.hash(state);
        }
    }
}

impl Debug for Tuple {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f, "{{")?;
        let mut iter = self.pairs().peekable();
        while let Some((k, v)) = iter.next() {
            if iter.peek().is_some() {
                write!(f, " {k}: {v:?},")?;
            } else {
                write!(f, " {k}: {v:?} ")?;
            }
        }
        write!(f, "}}")
    }
}

impl<'a, const NULLS_FIRST: bool> Ord for NullSortedValue<'a, NULLS_FIRST, Tuple> {
    fn cmp(&self, other: &Self) -> Ordering {
        let wrap = NullSortedValue::<{ NULLS_FIRST }, Value>;

        let self_pairs = self.0.pairs();
        let other_pairs = other.0.pairs();
        let mut p1 = self_pairs.sorted();
        let mut p2 = other_pairs.sorted();

        loop {
            return match (p1.next(), p2.next()) {
                (None, None) => Ordering::Equal,
                (Some(_), None) => Ordering::Greater,
                (None, Some(_)) => Ordering::Less,
                (Some((ls, lv)), Some((rs, rv))) => match (ls.cmp(rs), wrap(lv).cmp(&wrap(rv))) {
                    (Ordering::Less, _) => Ordering::Less,
                    (Ordering::Greater, _) => Ordering::Greater,
                    (_, Ordering::Less) => Ordering::Less,
                    (_, Ordering::Greater) => Ordering::Greater,
                    (_, Ordering::Equal) => continue,
                },
            };
        }
    }
}

impl Ord for Tuple {
    fn cmp(&self, other: &Self) -> Ordering {
        let self_pairs = self.pairs();
        let other_pairs = other.pairs();
        let mut p1 = self_pairs.sorted();
        let mut p2 = other_pairs.sorted();

        loop {
            return match (p1.next(), p2.next()) {
                (None, None) => Ordering::Equal,
                (Some(_), None) => Ordering::Greater,
                (None, Some(_)) => Ordering::Less,
                (Some(lv), Some(rv)) => match lv.cmp(&rv) {
                    Ordering::Less => Ordering::Less,
                    Ordering::Greater => Ordering::Greater,
                    Ordering::Equal => continue,
                },
            };
        }
    }
}

#[macro_export]
macro_rules! tuple {
    () => (
        $crate::Tuple::new()
    );
    ($(($x:expr, $y:expr)),+ $(,)?) => (
        $crate::Tuple::from([$(($x, Value::from($y))),+])
    );
}
