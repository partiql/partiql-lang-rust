use crate::{
    Bag, BagIntoIterator, BagIter, BindingsName, List, ListIntoIterator, ListIter, Tuple, Value,
};
use std::borrow::Cow;
use std::error::Error;
use std::fmt::Debug;
use std::vec;

pub type DatumLowerError = Box<dyn Error>;
pub type DatumLowerResult<T> = Result<T, DatumLowerError>;

pub trait Datum<D>
where
    D: Datum<D>,
{
    #[inline]
    /// Returns true if and only if Value is to be interpreted as `NULL`
    #[must_use]
    fn is_null(&self) -> bool {
        false
    }

    #[inline]
    /// Returns true if and only if Value is to be interpreted as `MISSING`
    #[must_use]
    fn is_missing(&self) -> bool {
        false
    }

    #[inline]
    /// Returns true if and only if Value is null or missing
    #[must_use]
    fn is_absent(&self) -> bool {
        self.is_null() || self.is_missing()
    }

    #[inline]
    /// Returns true if Value is neither null nor missing
    #[must_use]
    fn is_present(&self) -> bool {
        !self.is_absent()
    }

    #[must_use]
    fn is_sequence(&self) -> bool;

    #[must_use]
    fn is_ordered(&self) -> bool;
}

pub trait DatumValue<D>: Clone + Datum<D>
where
    D: Datum<D>,
{
    fn into_lower(self) -> DatumLowerResult<D>;
}

pub trait DatumCategory<'a> {
    fn category(&'a self) -> DatumCategoryRef<'a>;
    fn into_category(self) -> DatumCategoryOwned;
}

#[derive(Debug)]
pub enum DatumCategoryRef<'a> {
    Null,
    Missing,
    Tuple(DatumTupleRef<'a>),
    Sequence(DatumSeqRef<'a>),
    Scalar(DatumValueRef<'a>),
}

#[derive(Debug)]
pub enum DatumCategoryOwned {
    Null,
    Missing,
    Tuple(DatumTupleOwned),
    Sequence(DatumSeqOwned),
    Scalar(DatumValueOwned),
}

#[derive(Debug)]
pub enum DatumTupleRef<'a> {
    Tuple(&'a Tuple),
    Dynamic(&'a dyn RefTupleView<'a, Value>),
}

#[derive(Debug)]
pub enum DatumSeqRef<'a> {
    List(&'a List),
    Bag(&'a Bag),
    Dynamic(&'a dyn RefSequenceView<'a, Value>),
}

#[derive(Debug)]
pub enum DatumValueRef<'a> {
    Value(&'a Value),
}

#[derive(Debug)]
pub enum DatumTupleOwned {
    Tuple(Box<Tuple>),
    Dynamic(Box<dyn OwnedTupleView<Value>>),
}

#[derive(Debug)]
pub enum DatumSeqOwned {
    List(Box<List>),
    Bag(Box<Bag>),
    Dynamic(Box<dyn OwnedSequenceView<Value>>),
}

#[derive(Debug)]
pub enum DatumValueOwned {
    Value(Value),
}

impl<'a> DatumCategory<'a> for Value {
    fn category(&'a self) -> DatumCategoryRef<'a> {
        match self {
            Value::Null => DatumCategoryRef::Null,
            Value::Missing => DatumCategoryRef::Missing,
            Value::List(list) => DatumCategoryRef::Sequence(DatumSeqRef::List(list)),
            Value::Bag(bag) => DatumCategoryRef::Sequence(DatumSeqRef::Bag(bag)),
            Value::Tuple(tuple) => DatumCategoryRef::Tuple(DatumTupleRef::Tuple(tuple.as_ref())),
            Value::Variant(doc) => doc.category(),
            val => DatumCategoryRef::Scalar(DatumValueRef::Value(val)),
        }
    }

    fn into_category(self) -> DatumCategoryOwned {
        match self {
            Value::Null => DatumCategoryOwned::Null,
            Value::Missing => DatumCategoryOwned::Missing,
            Value::List(list) => DatumCategoryOwned::Sequence(DatumSeqOwned::List(list)),
            Value::Bag(bag) => DatumCategoryOwned::Sequence(DatumSeqOwned::Bag(bag)),
            Value::Tuple(tuple) => DatumCategoryOwned::Tuple(DatumTupleOwned::Tuple(tuple)),
            Value::Variant(doc) => doc.into_category(),
            val => DatumCategoryOwned::Scalar(DatumValueOwned::Value(val)),
        }
    }
}

pub trait TupleDatum {
    fn len(&self) -> usize;
    fn is_empty(&self) -> bool {
        self.len() == 0
    }
}

pub trait RefTupleView<'a, DV: DatumValue<DV>>: TupleDatum + Debug {
    fn get_val(&self, k: &BindingsName<'_>) -> Option<Cow<'a, DV>>;
}

pub trait OwnedTupleView<D: Datum<D>>: TupleDatum + Debug {
    fn take_val(self, k: &BindingsName<'_>) -> Option<D>;
    fn take_val_boxed(self: Box<Self>, k: &BindingsName<'_>) -> Option<D>;
}

impl TupleDatum for DatumTupleRef<'_> {
    fn len(&self) -> usize {
        match self {
            DatumTupleRef::Tuple(tuple) => tuple.len(),
            DatumTupleRef::Dynamic(dynamic) => dynamic.len(),
        }
    }
}

impl<'a> RefTupleView<'a, Value> for DatumTupleRef<'a> {
    fn get_val(&self, k: &BindingsName<'_>) -> Option<Cow<'a, Value>> {
        match self {
            DatumTupleRef::Tuple(tuple) => Tuple::get(tuple, k).map(Cow::Borrowed),
            DatumTupleRef::Dynamic(dynamic) => dynamic.get_val(k),
        }
    }
}

impl TupleDatum for DatumTupleOwned {
    fn len(&self) -> usize {
        match self {
            DatumTupleOwned::Tuple(tuple) => tuple.len(),
            DatumTupleOwned::Dynamic(dynamic) => dynamic.len(),
        }
    }
}

impl OwnedTupleView<Value> for DatumTupleOwned {
    fn take_val(self, k: &BindingsName<'_>) -> Option<Value> {
        match self {
            DatumTupleOwned::Tuple(tuple) => Tuple::take_val(*tuple, k),
            DatumTupleOwned::Dynamic(dynamic) => dynamic.take_val_boxed(k),
        }
    }

    fn take_val_boxed(self: Box<Self>, k: &BindingsName<'_>) -> Option<Value> {
        (*self).take_val(k)
    }
}

pub trait SequenceDatum {
    fn is_ordered(&self) -> bool;
    fn len(&self) -> usize;
    fn is_empty(&self) -> bool {
        self.len() == 0
    }
}

pub trait RefSequenceView<'a, DV: DatumValue<DV>>: SequenceDatum + Debug {
    fn get_val(&self, k: i64) -> Option<Cow<'a, DV>>;
}

pub trait OwnedSequenceView<D: Datum<D>>: SequenceDatum + Debug {
    fn take_val(self, k: i64) -> Option<D>;
    fn take_val_boxed(self: Box<Self>, k: i64) -> Option<D>;
}

impl SequenceDatum for DatumSeqRef<'_> {
    fn is_ordered(&self) -> bool {
        match self {
            DatumSeqRef::List(_) => true,
            DatumSeqRef::Bag(_) => false,
            DatumSeqRef::Dynamic(boxed) => boxed.is_ordered(),
        }
    }

    fn len(&self) -> usize {
        match self {
            DatumSeqRef::List(l) => l.len(),
            DatumSeqRef::Bag(b) => b.len(),
            DatumSeqRef::Dynamic(boxed) => boxed.len(),
        }
    }
}

impl<'a> RefSequenceView<'a, Value> for DatumSeqRef<'a> {
    fn get_val(&self, k: i64) -> Option<Cow<'a, Value>> {
        match self {
            DatumSeqRef::List(l) => List::get(l, k).map(Cow::Borrowed),
            DatumSeqRef::Bag(_) => {
                todo!("TODO [EMBDOC]: Bag::get")
            }
            DatumSeqRef::Dynamic(boxed) => boxed.get_val(k),
        }
    }
}

impl SequenceDatum for DatumSeqOwned {
    fn is_ordered(&self) -> bool {
        match self {
            DatumSeqOwned::List(_) => true,
            DatumSeqOwned::Bag(_) => false,
            DatumSeqOwned::Dynamic(boxed) => boxed.is_ordered(),
        }
    }

    fn len(&self) -> usize {
        todo!()
    }
}

impl OwnedSequenceView<Value> for DatumSeqOwned {
    fn take_val(self, k: i64) -> Option<Value> {
        match self {
            DatumSeqOwned::List(l) => l.take_val(k),
            DatumSeqOwned::Bag(_) => todo!("TODO [EMBDOC]: Bag::get"),
            DatumSeqOwned::Dynamic(boxed) => boxed.take_val_boxed(k),
        }
    }

    fn take_val_boxed(self: Box<Self>, k: i64) -> Option<Value> {
        self.take_val(k)
    }
}

impl<'a> IntoIterator for DatumSeqRef<'a> {
    type Item = &'a Value;
    type IntoIter = DatumSeqRefIterator<'a>;

    fn into_iter(self) -> Self::IntoIter {
        match self {
            DatumSeqRef::List(l) => DatumSeqRefIterator::List(l.into_iter()),
            DatumSeqRef::Bag(b) => DatumSeqRefIterator::Bag(b.into_iter()),
            DatumSeqRef::Dynamic(_) => {
                todo!()
            }
        }
    }
}

pub enum DatumSeqRefIterator<'a> {
    List(ListIter<'a>),
    Bag(BagIter<'a>),
}

impl<'a> Iterator for DatumSeqRefIterator<'a> {
    type Item = &'a Value;

    fn next(&mut self) -> Option<Self::Item> {
        match self {
            DatumSeqRefIterator::List(l) => l.next(),
            DatumSeqRefIterator::Bag(b) => b.next(),
        }
    }
}

impl IntoIterator for DatumSeqOwned {
    type Item = Value;
    type IntoIter = DatumSeqOwnedIterator;

    fn into_iter(self) -> Self::IntoIter {
        match self {
            DatumSeqOwned::List(l) => DatumSeqOwnedIterator::List(l.into_iter()),
            DatumSeqOwned::Bag(b) => DatumSeqOwnedIterator::Bag(b.into_iter()),
            DatumSeqOwned::Dynamic(_) => {
                todo!()
            }
        }
    }
}

pub enum DatumSeqOwnedIterator {
    List(ListIntoIterator),
    Bag(BagIntoIterator),
}

impl Iterator for DatumSeqOwnedIterator {
    type Item = Value;

    fn next(&mut self) -> Option<Self::Item> {
        match self {
            DatumSeqOwnedIterator::List(l) => l.next(),
            DatumSeqOwnedIterator::Bag(b) => b.next(),
        }
    }
}
