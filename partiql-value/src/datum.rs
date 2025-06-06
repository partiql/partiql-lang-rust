use crate::{
    Bag, BagIntoIterator, BindingsName, Graph, List, ListIntoIterator, PairsIntoIter, PairsIter,
    Tuple, Value,
};
use std::borrow::Cow;
use std::collections::HashMap;
use std::error::Error;

use std::fmt::Debug;

pub type DatumLowerError = Box<dyn Error>;
pub type DatumLowerResult<T> = Result<T, DatumLowerError>;

pub trait Datum<D>
where
    D: Datum<D>,
{
    /// Returns true if and only if Value is to be interpreted as `NULL`
    #[must_use]
    fn is_null(&self) -> bool;

    /// Returns true if and only if Value is to be interpreted as `MISSING`
    #[must_use]
    fn is_missing(&self) -> bool;

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

pub trait DatumValue<D: Datum<D>>: Datum<D> + Clone + Debug {}

pub trait DatumLower<D: DatumValue<D>>: Datum<D> + Debug {
    fn into_lower(self) -> DatumLowerResult<D>;

    fn into_lower_boxed(self: Box<Self>) -> DatumLowerResult<D>;
    fn lower(&self) -> DatumLowerResult<Cow<'_, D>>;
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
    Graph(DatumGraphRef<'a>),
}

#[derive(Debug)]
pub enum DatumCategoryOwned {
    Null,
    Missing,
    Tuple(DatumTupleOwned),
    Sequence(DatumSeqOwned),
    Scalar(DatumValueOwned),
    Graph(DatumGraphOwned),
}

#[derive(Debug)]
pub enum DatumTupleRef<'a> {
    /// A Tuple with no bindings
    Empty,
    /// Holds a reference to a [`Tuple`]
    Tuple(&'a Tuple),
    /// Holds a reference to a [`RefTupleView`]
    Dynamic(&'a dyn RefTupleView<'a, Value>),
    /// Holds a reference to a value which is not a tuple, but has been coerced (e.g., for `9`: `{'_1': 9}`)
    CoercedValue(usize, &'a Value),
    /// A reference to a value stored under a single key
    SingleKey(Cow<'a, str>, &'a Value),
    /// A reference to a value stored under a single key
    Bindings(&'a HashMap<Cow<'a, str>, &'a Value>),
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
    Dynamic(&'a dyn DatumLower<Value>),
}

#[derive(Debug)]
pub enum DatumGraphRef<'a> {
    Graph(&'a Graph),
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

#[derive(Debug)]
pub enum DatumGraphOwned {
    Graph(Box<Graph>),
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
            Value::Graph(graph) => DatumCategoryRef::Graph(DatumGraphRef::Graph(graph.as_ref())),
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
            Value::Graph(graph) => DatumCategoryOwned::Graph(DatumGraphOwned::Graph(graph)),
            val => DatumCategoryOwned::Scalar(DatumValueOwned::Value(val)),
        }
    }
}

pub struct RefFieldView<'a, DV: DatumValue<DV>> {
    pub name: Option<&'a str>,
    pub value: Cow<'a, DV>,
}

pub trait TupleDatum {
    fn len(&self) -> usize;
    fn is_empty(&self) -> bool {
        self.len() == 0
    }
}

pub trait RefTupleView<'a, DV: DatumValue<DV>>: TupleDatum + Debug {
    fn get_val(&self, k: &BindingsName<'_>) -> Option<Cow<'a, DV>>;
    fn tuple_fields_iter(&'a self) -> Box<dyn Iterator<Item = RefFieldView<'a, DV>> + 'a>;
}

pub struct OwnedFieldView<D: Datum<D>> {
    pub name: String,
    pub value: D,
}

pub trait OwnedTupleView<D: Datum<D>>: TupleDatum + Debug {
    fn take_val(self, k: &BindingsName<'_>) -> Option<D>;
    fn take_val_boxed(self: Box<Self>, k: &BindingsName<'_>) -> Option<D>;
    fn into_iter_boxed(self: Box<Self>) -> Box<dyn Iterator<Item = OwnedFieldView<D>>>;
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
    fn into_iter(self) -> Box<dyn Iterator<Item = Cow<'a, DV>> + 'a>;
}

pub trait OwnedSequenceView<D: Datum<D>>: SequenceDatum + Debug {
    fn take_val(self, k: i64) -> Option<D>;
    fn take_val_boxed(self: Box<Self>, k: i64) -> Option<D>;
    fn into_iter_boxed(self: Box<Self>) -> Box<dyn Iterator<Item = D>>;
}

impl TupleDatum for DatumTupleRef<'_> {
    fn len(&self) -> usize {
        match self {
            DatumTupleRef::Tuple(tuple) => tuple.len(),
            DatumTupleRef::Dynamic(dynamic) => dynamic.len(),
            DatumTupleRef::Empty => 0,
            DatumTupleRef::CoercedValue(_, _) => 1,
            DatumTupleRef::SingleKey(_, _) => 1,
            DatumTupleRef::Bindings(map) => map.len(),
        }
    }
}

impl<'a> RefTupleView<'a, Value> for DatumTupleRef<'a> {
    fn get_val(&self, k: &BindingsName<'_>) -> Option<Cow<'a, Value>> {
        match self {
            DatumTupleRef::Tuple(tuple) => Tuple::get(tuple, k).map(Cow::Borrowed),
            DatumTupleRef::Dynamic(dynamic) => dynamic.get_val(k),
            DatumTupleRef::Empty => None,
            DatumTupleRef::CoercedValue(idx, v) => {
                if k.matcher().matches(&format!("_{idx}")) {
                    Some(Cow::Borrowed(v))
                } else {
                    None
                }
            }
            DatumTupleRef::SingleKey(key, v) => {
                if k.matcher().matches(key.as_ref()) {
                    Some(Cow::Borrowed(v))
                } else {
                    None
                }
            }
            DatumTupleRef::Bindings(map) => {
                let m = k.matcher();
                map.iter()
                    .find_map(|(k, &v)| m.matches(k).then_some(Cow::Borrowed(v)))
            }
        }
    }

    fn tuple_fields_iter(&'a self) -> Box<dyn Iterator<Item = RefFieldView<'a, Value>> + 'a> {
        Box::new(IntoIterator::into_iter(self))
    }
}

impl<'a> IntoIterator for &'a DatumTupleRef<'a> {
    type Item = RefFieldView<'a, Value>;
    type IntoIter = DatumTupleRefIterator<'a>;

    fn into_iter(self) -> Self::IntoIter {
        match self {
            DatumTupleRef::Tuple(t) => DatumTupleRefIterator::Tuple(t.pairs()),
            DatumTupleRef::Empty => DatumTupleRefIterator::Empty,
            DatumTupleRef::Dynamic(d) => DatumTupleRefIterator::Dynamic(d.tuple_fields_iter()),
            DatumTupleRef::CoercedValue(idx, value) => {
                DatumTupleRefIterator::CoercedValue(Some((*idx, value)))
            }
            DatumTupleRef::SingleKey(key, value) => {
                DatumTupleRefIterator::SingleKey(Some((key.as_ref(), value)))
            }
            DatumTupleRef::Bindings(m) => {
                DatumTupleRefIterator::Dynamic(Box::new(m.iter().map(|(k, &v)| RefFieldView {
                    name: Some(k.as_ref()),
                    value: Cow::Borrowed(v),
                })))
            }
        }
    }
}

pub enum DatumTupleRefIterator<'a> {
    Empty,
    Tuple(PairsIter<'a>),
    Dynamic(Box<dyn Iterator<Item = RefFieldView<'a, Value>> + 'a>),
    CoercedValue(Option<(usize, &'a Value)>),
    SingleKey(Option<(&'a str, &'a Value)>),
}

impl<'a> Iterator for DatumTupleRefIterator<'a> {
    type Item = RefFieldView<'a, Value>;

    fn next(&mut self) -> Option<Self::Item> {
        match self {
            DatumTupleRefIterator::Tuple(t) => t.next().map(|(name, value)| RefFieldView {
                name: Some(name),
                value: Cow::Borrowed(value),
            }),
            DatumTupleRefIterator::Empty => None,
            DatumTupleRefIterator::CoercedValue(payload) => {
                payload.take().map(|(_, value)| RefFieldView {
                    name: None,
                    value: Cow::Borrowed(value),
                })
            }
            DatumTupleRefIterator::Dynamic(d) => d.next(),
            DatumTupleRefIterator::SingleKey(payload) => {
                payload.take().map(|(key, value)| RefFieldView {
                    name: Some(key),
                    value: Cow::Borrowed(value),
                })
            }
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

    fn into_iter_boxed(self: Box<Self>) -> Box<dyn Iterator<Item = OwnedFieldView<Value>>> {
        Box::new((*self).into_iter())
    }
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
            DatumSeqRef::Bag(_) => None,
            DatumSeqRef::Dynamic(boxed) => boxed.get_val(k),
        }
    }

    fn into_iter(self) -> Box<dyn Iterator<Item = Cow<'a, Value>> + 'a> {
        match self {
            DatumSeqRef::List(l) => Box::new(l.iter().map(Cow::Borrowed)),
            DatumSeqRef::Bag(b) => Box::new(b.iter().map(Cow::Borrowed)),
            DatumSeqRef::Dynamic(_boxed) => todo!("&dyn RefSequenceView into_iter"),
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
        match self {
            DatumSeqOwned::List(l) => l.len(),
            DatumSeqOwned::Bag(b) => b.len(),
            DatumSeqOwned::Dynamic(boxed) => boxed.len(),
        }
    }
}

impl OwnedSequenceView<Value> for DatumSeqOwned {
    fn take_val(self, k: i64) -> Option<Value> {
        match self {
            DatumSeqOwned::List(l) => l.take_val(k),
            DatumSeqOwned::Bag(_) => None,
            DatumSeqOwned::Dynamic(boxed) => boxed.take_val_boxed(k),
        }
    }

    fn take_val_boxed(self: Box<Self>, k: i64) -> Option<Value> {
        self.take_val(k)
    }

    fn into_iter_boxed(self: Box<Self>) -> Box<dyn Iterator<Item = Value>> {
        Box::new((*self).into_iter())
    }
}

impl IntoIterator for DatumTupleOwned {
    type Item = OwnedFieldView<Value>;
    type IntoIter = DatumTupleOwnedIterator;

    fn into_iter(self) -> Self::IntoIter {
        match self {
            DatumTupleOwned::Tuple(t) => DatumTupleOwnedIterator::Tuple(t.into_pairs()),
            DatumTupleOwned::Dynamic(d) => DatumTupleOwnedIterator::Dynamic(d.into_iter_boxed()),
        }
    }
}

pub enum DatumTupleOwnedIterator {
    Tuple(PairsIntoIter),
    Dynamic(Box<dyn Iterator<Item = OwnedFieldView<Value>>>),
}

impl Iterator for DatumTupleOwnedIterator {
    type Item = OwnedFieldView<Value>;

    fn next(&mut self) -> Option<Self::Item> {
        match self {
            DatumTupleOwnedIterator::Tuple(t) => {
                t.next().map(|(name, value)| OwnedFieldView { name, value })
            }
            DatumTupleOwnedIterator::Dynamic(d) => d.next(),
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
            DatumSeqOwned::Dynamic(d) => DatumSeqOwnedIterator::Dynamic(d.into_iter_boxed()),
        }
    }
}

pub enum DatumSeqOwnedIterator {
    List(ListIntoIterator),
    Bag(BagIntoIterator),
    Dynamic(Box<dyn Iterator<Item = Value>>),
}

impl Iterator for DatumSeqOwnedIterator {
    type Item = Value;

    fn next(&mut self) -> Option<Self::Item> {
        match self {
            DatumSeqOwnedIterator::List(l) => l.next(),
            DatumSeqOwnedIterator::Bag(b) => b.next(),
            DatumSeqOwnedIterator::Dynamic(d) => d.next(),
        }
    }
}

impl Datum<Value> for DatumValueRef<'_> {
    #[inline]
    fn is_null(&self) -> bool {
        match self {
            DatumValueRef::Value(v) => v.is_null(),
            DatumValueRef::Dynamic(d) => d.is_null(),
        }
    }

    #[inline]
    fn is_missing(&self) -> bool {
        match self {
            DatumValueRef::Value(v) => v.is_missing(),
            DatumValueRef::Dynamic(d) => d.is_missing(),
        }
    }

    #[inline]
    fn is_sequence(&self) -> bool {
        match self {
            DatumValueRef::Value(v) => v.is_sequence(),
            DatumValueRef::Dynamic(d) => d.is_sequence(),
        }
    }

    #[inline]
    fn is_ordered(&self) -> bool {
        match self {
            DatumValueRef::Value(v) => v.is_ordered(),
            DatumValueRef::Dynamic(d) => d.is_ordered(),
        }
    }
}

impl Datum<Value> for DatumValueOwned {
    #[inline]
    fn is_null(&self) -> bool {
        match self {
            DatumValueOwned::Value(v) => v.is_null(),
        }
    }

    #[inline]
    fn is_missing(&self) -> bool {
        match self {
            DatumValueOwned::Value(v) => v.is_missing(),
        }
    }

    #[inline]
    fn is_sequence(&self) -> bool {
        match self {
            DatumValueOwned::Value(v) => v.is_sequence(),
        }
    }

    #[inline]
    fn is_ordered(&self) -> bool {
        match self {
            DatumValueOwned::Value(v) => v.is_ordered(),
        }
    }
}

impl DatumLower<Value> for DatumValueOwned {
    #[inline]
    fn into_lower(self) -> DatumLowerResult<Value> {
        match self {
            DatumValueOwned::Value(v) => v.into_lower(),
        }
    }

    #[inline]
    fn into_lower_boxed(self: Box<Self>) -> DatumLowerResult<Value> {
        match *self {
            DatumValueOwned::Value(v) => v.into_lower(),
        }
    }

    #[inline]
    fn lower(&self) -> DatumLowerResult<Cow<'_, Value>> {
        match self {
            DatumValueOwned::Value(v) => v.lower(),
        }
    }
}
