use crate::{Bag, BindingsName, EmbeddedDoc, List, Tuple, Value, ValueIntoIterator};
use std::fmt::Debug;
// TODO [EMBDOC] pub type DatumIterator = dyn Iterator<Item = Value>;

pub type DatumLowerError = ();
pub type DatumLowerResult<T> = Result<T, DatumLowerError>;

pub trait Datum<D: Datum<D>> {
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

    // #[must_use]
    // fn into_iter(self) -> I;

    fn lower(self) -> DatumLowerResult<D>;
}

pub trait DatumCattt<'a> {
    fn category(&'a self) -> DatumCategoryRef<'a>;
    fn into_category(self) -> DatumCategoryOwned;
}

pub enum DatumCategoryRef<'a> {
    Null,
    Missing,
    Tuple(DatumTupleRef<'a>),
    Sequence(DatumSeqRef<'a>),
    Value(DatumValueRef<'a>),
}

pub enum DatumCategoryOwned {
    Null,
    Missing,
    Tuple(DatumTupleOwned),
    Sequence(DatumSeqOwned),
    Value(DatumValueOwned),
}

pub enum DatumTupleRef<'a> {
    Tuple(&'a Tuple),
    Dynamic(Box<dyn RefTupleView<'a, Value>>),
}

pub enum DatumSeqRef<'a> {
    List(&'a List),
    Bag(&'a Bag),
}

pub enum DatumValueRef<'a> {
    Value(&'a Value),
}

pub enum DatumTupleOwned {
    Tuple(Box<Tuple>),
    Dynamic(Box<dyn OwnedTupleView<Value>>),
}

pub enum DatumSeqOwned {
    List(Box<List>),
    Bag(Box<Bag>),
}

pub enum DatumValueOwned {
    Value(Value),
}

impl<'a> DatumCattt<'a> for Value {
    fn category(&'a self) -> DatumCategoryRef<'a> {
        match self {
            Value::Null => DatumCategoryRef::Null,
            Value::Missing => DatumCategoryRef::Missing,
            Value::List(list) => DatumCategoryRef::Sequence(DatumSeqRef::List(list)),
            Value::Bag(bag) => DatumCategoryRef::Sequence(DatumSeqRef::Bag(bag)),
            Value::Tuple(tuple) => DatumCategoryRef::Tuple(DatumTupleRef::Tuple(tuple.as_ref())),
            Value::EmbeddedDoc(doc) => doc.category(),
            val => DatumCategoryRef::Value(DatumValueRef::Value(val)),
        }
    }

    fn into_category(self) -> DatumCategoryOwned {
        match self {
            Value::Null => DatumCategoryOwned::Null,
            Value::Missing => DatumCategoryOwned::Missing,
            Value::List(list) => DatumCategoryOwned::Sequence(DatumSeqOwned::List(list)),
            Value::Bag(bag) => DatumCategoryOwned::Sequence(DatumSeqOwned::Bag(bag)),
            Value::Tuple(tuple) => DatumCategoryOwned::Tuple(DatumTupleOwned::Tuple(tuple)),
            Value::EmbeddedDoc(doc) => doc.into_category(),
            val => DatumCategoryOwned::Value(DatumValueOwned::Value(val)),
        }
    }
}

pub trait RefTupleView<'a, D: Datum<D>> {
    fn get(&self, k: &BindingsName<'_>) -> Option<&'a D>;
}

pub trait OwnedTupleView<D: Datum<D>> {
    fn take_val(self, k: &BindingsName<'_>) -> Option<D>;
    fn take_val_boxed(self: Box<Self>, k: &BindingsName<'_>) -> Option<D>;
}

impl<'a> RefTupleView<'a, Value> for DatumTupleRef<'a> {
    fn get(&self, k: &BindingsName<'_>) -> Option<&'a Value> {
        match self {
            DatumTupleRef::Tuple(tuple) => Tuple::get(tuple, k),
            DatumTupleRef::Dynamic(dynamic) => dynamic.get(k),
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

pub trait RefSequenceView<'a, D: Datum<D>> {
    fn is_ordered(&self) -> bool;
    fn get(self, k: i64) -> Option<&'a D>;
}

pub trait OwnedSequenceView<D: Datum<D>> {
    fn is_ordered(&self) -> bool;
    fn take_val(self, k: i64) -> Option<D>;
}

impl<'a> RefSequenceView<'a, Value> for DatumSeqRef<'a> {
    fn is_ordered(&self) -> bool {
        match self {
            DatumSeqRef::List(_) => true,
            DatumSeqRef::Bag(_) => false,
        }
    }

    fn get(self, k: i64) -> Option<&'a Value> {
        match self {
            DatumSeqRef::List(l) => List::get(l, k),
            DatumSeqRef::Bag(b) => {
                todo!("TODO [EMBDOC]: Bag::get")
            }
        }
    }
}

impl OwnedSequenceView<Value> for DatumSeqOwned {
    fn is_ordered(&self) -> bool {
        match self {
            DatumSeqOwned::List(_) => true,
            DatumSeqOwned::Bag(_) => false,
        }
    }

    fn take_val(self, k: i64) -> Option<Value> {
        match self {
            DatumSeqOwned::List(l) => l.take_val(k),
            DatumSeqOwned::Bag(b) => todo!("TODO [EMBDOC]: Bag::get"),
        }
    }
}
