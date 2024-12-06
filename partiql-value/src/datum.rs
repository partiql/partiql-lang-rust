use crate::{Bag, BindingsName, List, Tuple, Value};
use std::borrow::Cow;
use std::error::Error;

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

    /// Returns true if and only if Value is an integer, real, or decimal
    #[must_use]
    fn is_number(&self) -> bool;

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
}

#[derive(Debug)]
pub enum DatumSeqRef<'a> {
    List(&'a List),
    Bag(&'a Bag),
}

#[derive(Debug)]
pub enum DatumValueRef<'a> {
    Value(&'a Value),
}

#[derive(Debug)]
pub enum DatumTupleOwned {
    Tuple(Box<Tuple>),
}

#[derive(Debug)]
pub enum DatumSeqOwned {
    List(Box<List>),
    Bag(Box<Bag>),
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

pub trait RefTupleView<'a, DV: DatumValue<DV>>: TupleDatum {
    fn get_val(&self, k: &BindingsName<'_>) -> Option<Cow<'a, DV>>;
}

pub trait OwnedTupleView<D: Datum<D>>: TupleDatum {
    fn take_val(self, k: &BindingsName<'_>) -> Option<D>;
    fn take_val_boxed(self: Box<Self>, k: &BindingsName<'_>) -> Option<D>;
}

impl TupleDatum for DatumTupleRef<'_> {
    fn len(&self) -> usize {
        match self {
            DatumTupleRef::Tuple(tuple) => tuple.len(),
        }
    }
}

impl<'a> RefTupleView<'a, Value> for DatumTupleRef<'a> {
    fn get_val(&self, k: &BindingsName<'_>) -> Option<Cow<'a, Value>> {
        match self {
            DatumTupleRef::Tuple(tuple) => Tuple::get(tuple, k).map(Cow::Borrowed),
        }
    }
}

impl TupleDatum for DatumTupleOwned {
    fn len(&self) -> usize {
        match self {
            DatumTupleOwned::Tuple(tuple) => tuple.len(),
        }
    }
}

impl OwnedTupleView<Value> for DatumTupleOwned {
    fn take_val(self, k: &BindingsName<'_>) -> Option<Value> {
        match self {
            DatumTupleOwned::Tuple(tuple) => Tuple::take_val(*tuple, k),
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

pub trait RefSequenceView<'a, DV: DatumValue<DV>>: SequenceDatum {
    fn get_val(&self, k: i64) -> Option<Cow<'a, DV>>;
}

pub trait OwnedSequenceView<D: Datum<D>>: SequenceDatum {
    fn take_val(self, k: i64) -> Option<D>;
    fn take_val_boxed(self: Box<Self>, k: i64) -> Option<D>;
}

impl SequenceDatum for DatumSeqRef<'_> {
    fn is_ordered(&self) -> bool {
        match self {
            DatumSeqRef::List(_) => true,
            DatumSeqRef::Bag(_) => false,
        }
    }

    fn len(&self) -> usize {
        match self {
            DatumSeqRef::List(l) => l.len(),
            DatumSeqRef::Bag(b) => b.len(),
        }
    }
}

impl<'a> RefSequenceView<'a, Value> for DatumSeqRef<'a> {
    fn get_val(&self, k: i64) -> Option<Cow<'a, Value>> {
        match self {
            DatumSeqRef::List(l) => List::get(l, k).map(Cow::Borrowed),
            DatumSeqRef::Bag(_) => {
                todo!("TODO RefSequenceView: Bag::get")
            }
        }
    }
}

impl SequenceDatum for DatumSeqOwned {
    fn is_ordered(&self) -> bool {
        match self {
            DatumSeqOwned::List(_) => true,
            DatumSeqOwned::Bag(_) => false,
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
            DatumSeqOwned::Bag(_) => todo!("TODO OwnedSequenceView: Bag::get"),
        }
    }

    fn take_val_boxed(self: Box<Self>, k: i64) -> Option<Value> {
        self.take_val(k)
    }
}
