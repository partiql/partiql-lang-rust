use crate::datum::OwnedFieldView;
use crate::{PairsIntoIter, PairsIter, Value};
#[cfg(feature = "serde")]
use serde::{Deserialize, Serialize};
use std::borrow::Cow;
use std::iter::Once;
use unicase::UniCase;

#[derive(Clone, Hash, Debug, Eq, PartialEq)]
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
pub enum BindingsName<'s> {
    CaseSensitive(Cow<'s, str>),
    CaseInsensitive(Cow<'s, str>),
}

impl<'s> BindingsName<'s> {
    pub fn matcher(&'s self) -> BindingsMatcher<'s> {
        BindingsMatcher::from(self)
    }
}

#[derive(Clone, Hash, Debug, Eq, PartialEq)]
pub enum BindingsMatcher<'s> {
    CaseSensitive(&'s str),
    CaseInsensitive(UniCase<&'s str>),
}

impl<'s> BindingsMatcher<'s> {
    pub fn matches(&'s self, candidate: &str) -> bool {
        match self {
            BindingsMatcher::CaseSensitive(target) => *target == candidate,
            BindingsMatcher::CaseInsensitive(target) => *target == UniCase::new(candidate),
        }
    }
}

impl<'s> From<&'s BindingsName<'s>> for BindingsMatcher<'s> {
    fn from(name: &'s BindingsName<'_>) -> Self {
        match name {
            BindingsName::CaseSensitive(s) => BindingsMatcher::CaseSensitive(s.as_ref()),
            BindingsName::CaseInsensitive(s) => {
                BindingsMatcher::CaseInsensitive(UniCase::new(s.as_ref()))
            }
        }
    }
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

pub enum BindingIntoIter {
    Tuple(PairsIntoIter),
    Single(Once<Value>),
    Empty,
    DynTuple(Box<dyn Iterator<Item = OwnedFieldView<Value>>>),
}

impl Iterator for BindingIntoIter {
    type Item = (Option<String>, Value);

    #[inline]
    fn next(&mut self) -> Option<Self::Item> {
        match self {
            BindingIntoIter::Tuple(t) => t.next().map(|(k, v)| (Some(k), v)),
            BindingIntoIter::Single(single) => single.next().map(|v| (None, v)),
            BindingIntoIter::Empty => None,
            BindingIntoIter::DynTuple(d) => d.next().map(|f| (Some(f.name), f.value)),
        }
    }

    #[inline]
    fn size_hint(&self) -> (usize, Option<usize>) {
        match self {
            BindingIntoIter::Tuple(t) => t.size_hint(),
            BindingIntoIter::Single(_single) => (1, Some(1)),
            BindingIntoIter::Empty => (0, Some(0)),
            BindingIntoIter::DynTuple(d) => d.size_hint(),
        }
    }
}
