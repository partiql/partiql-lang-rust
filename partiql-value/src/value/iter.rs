use crate::{
    BagIntoIterator, BagIter, ListIntoIterator, ListIter, Value, VariantIntoIterator, VariantIter,
};

pub enum ValueIter<'a> {
    List(ListIter<'a>),
    Bag(BagIter<'a>),
    Variant(VariantIter<'a>),
    Single(Option<&'a Value>),
}

impl<'a> Iterator for ValueIter<'a> {
    type Item = &'a Value;

    #[inline]
    fn next(&mut self) -> Option<Self::Item> {
        match self {
            ValueIter::List(list) => list.next(),
            ValueIter::Bag(bag) => bag.next(),
            ValueIter::Variant(_doc) => {
                todo!("next for ValueIter::Variant")
            }
            ValueIter::Single(v) => v.take(),
        }
    }

    #[inline]
    fn size_hint(&self) -> (usize, Option<usize>) {
        match self {
            ValueIter::List(list) => list.size_hint(),
            ValueIter::Bag(bag) => bag.size_hint(),
            ValueIter::Variant(_doc) => {
                todo!("size_hint for ValueIter::Variant")
            }
            ValueIter::Single(_) => (1, Some(1)),
        }
    }
}

impl IntoIterator for Value {
    type Item = Value;
    type IntoIter = ValueIntoIterator;

    #[inline]
    fn into_iter(self) -> ValueIntoIterator {
        match self {
            Value::List(list) => ValueIntoIterator::List(list.into_iter()),
            Value::Bag(bag) => ValueIntoIterator::Bag(bag.into_iter()),
            Value::Variant(doc) => ValueIntoIterator::Variant(doc.into_iter()),
            other => ValueIntoIterator::Single(Some(other)),
        }
    }
}

pub enum ValueIntoIterator {
    List(ListIntoIterator),
    Bag(BagIntoIterator),
    Variant(VariantIntoIterator),
    Single(Option<Value>),
}

impl Iterator for ValueIntoIterator {
    type Item = Value;

    #[inline]
    fn next(&mut self) -> Option<Self::Item> {
        match self {
            ValueIntoIterator::List(list) => list.next(),
            ValueIntoIterator::Bag(bag) => bag.next(),
            ValueIntoIterator::Variant(doc) => doc
                .next()
                .map(|d| Value::Variant(Box::new(d.expect("Variant iteration")))),
            ValueIntoIterator::Single(v) => v.take(),
        }
    }

    #[inline]
    fn size_hint(&self) -> (usize, Option<usize>) {
        match self {
            ValueIntoIterator::List(list) => list.size_hint(),
            ValueIntoIterator::Bag(bag) => bag.size_hint(),
            ValueIntoIterator::Variant(doc) => doc.size_hint(),
            ValueIntoIterator::Single(_) => (1, Some(1)),
        }
    }
}
