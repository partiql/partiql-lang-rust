use crate::{PairsIntoIter, PairsIter, Value};
#[cfg(feature = "serde")]
use serde::{Deserialize, Serialize};
use std::borrow::Cow;
use std::iter::Once;

#[derive(Clone, Hash, Debug, Eq, PartialEq)]
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
pub enum BindingsName<'s> {
    CaseSensitive(Cow<'s, str>),
    CaseInsensitive(Cow<'s, str>),
}

#[derive(Debug, Clone)]
pub enum BindingIter<'a> {
    Tuple(PairsIter<'a>),
    Single(Once<&'a Value>),
    Empty,
}

impl<'a> Iterator for BindingIter<'a> {
    type Item = (Option<&'a String>, &'a Value);

    #[inline]
    fn next(&mut self) -> Option<Self::Item> {
        match self {
            BindingIter::Tuple(t) => t.next().map(|(k, v)| (Some(k), v)),
            BindingIter::Single(single) => single.next().map(|v| (None, v)),
            BindingIter::Empty => None,
        }
    }

    #[inline]
    fn size_hint(&self) -> (usize, Option<usize>) {
        match self {
            BindingIter::Tuple(t) => t.size_hint(),
            BindingIter::Single(_single) => (1, Some(1)),
            BindingIter::Empty => (0, Some(0)),
        }
    }
}

#[derive(Debug)]
pub enum BindingIntoIter {
    Tuple(PairsIntoIter),
    Single(Once<Value>),
    Empty,
}

impl Iterator for BindingIntoIter {
    type Item = (Option<String>, Value);

    #[inline]
    fn next(&mut self) -> Option<Self::Item> {
        match self {
            BindingIntoIter::Tuple(t) => t.next().map(|(k, v)| (Some(k), v)),
            BindingIntoIter::Single(single) => single.next().map(|v| (None, v)),
            BindingIntoIter::Empty => None,
        }
    }

    #[inline]
    fn size_hint(&self) -> (usize, Option<usize>) {
        match self {
            BindingIntoIter::Tuple(t) => t.size_hint(),
            BindingIntoIter::Single(_single) => (1, Some(1)),
            BindingIntoIter::Empty => (0, Some(0)),
        }
    }
}
