use ordered_float::OrderedFloat;
use partiql_value::Value;
use std::ops::Deref;

pub(super) use super::internal::ValueRef;

/// Newtype wrapper around Value that implements Send + Sync.
///
/// # Safety
/// TODO: TEMPORARY - ValueOwned wraps Value which contains Rc<SimpleGraph> in Graph variant.
/// This impl allows CompiledPlan to be Send + Sync for thread-safe query compilation.
/// MUST replace with a thread-safe value type before using in actual multi-threaded execution.
/// DO NOT send ValueOwned containing Value::Graph across threads - will cause undefined behavior.
#[repr(transparent)]
#[derive(Clone, Debug, PartialEq, Eq, Hash)]
pub struct ValueOwned(Value);

// Safety: See struct-level safety comment.
// This is a temporary workaround until Value is replaced with a thread-safe alternative.
unsafe impl Send for ValueOwned {}
unsafe impl Sync for ValueOwned {}

impl Deref for ValueOwned {
    type Target = Value;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl From<Value> for ValueOwned {
    fn from(v: Value) -> Self {
        ValueOwned(v)
    }
}

impl From<ValueOwned> for Value {
    fn from(v: ValueOwned) -> Self {
        v.0
    }
}

impl AsRef<Value> for ValueOwned {
    fn as_ref(&self) -> &Value {
        &self.0
    }
}

// TODO: Eventually delete this, or add a different way to introspect the dom.
#[allow(dead_code)]
#[derive(Clone, Copy, Debug)]
pub enum ValueView<'a> {
    Missing,
    Null,
    Bool(bool),
    I64(i64),
    F64(f64),
    Str(&'a str),
    Bytes(&'a [u8]),
    Owned(&'a ValueOwned),
}

#[allow(dead_code)]
impl<'a> ValueView<'a> {
    pub fn from_owned(value: &'a ValueOwned) -> Self {
        ValueRef::from_owned(value).into()
    }

    pub fn as_i64(&self) -> Option<i64> {
        match *self {
            ValueView::I64(v) => Some(v),
            _ => None,
        }
    }

    pub fn as_bool(&self) -> Option<bool> {
        match *self {
            ValueView::Bool(v) => Some(v),
            _ => None,
        }
    }

    pub fn as_str(&self) -> Option<&'a str> {
        match *self {
            ValueView::Str(v) => Some(v),
            _ => None,
        }
    }

    pub fn to_owned(self) -> ValueOwned {
        ValueOwned(match self {
            ValueView::Missing => Value::Missing,
            ValueView::Null => Value::Null,
            ValueView::Bool(v) => Value::Boolean(v),
            ValueView::I64(v) => Value::Integer(v),
            ValueView::F64(v) => Value::Real(OrderedFloat(v)),
            ValueView::Str(v) => Value::String(Box::new(v.to_string())),
            ValueView::Bytes(v) => Value::Blob(Box::new(v.to_vec())),
            ValueView::Owned(v) => return v.clone(),
        })
    }
}

impl<'a> From<ValueRef<'a>> for ValueView<'a> {
    fn from(value: ValueRef<'a>) -> Self {
        match value {
            ValueRef::Missing => ValueView::Missing,
            ValueRef::Null => ValueView::Null,
            ValueRef::Bool(v) => ValueView::Bool(v),
            ValueRef::I64(v) => ValueView::I64(v),
            ValueRef::F64(v) => ValueView::F64(v),
            ValueRef::Str(v) => ValueView::Str(v),
            ValueRef::Bytes(v) => ValueView::Bytes(v),
            ValueRef::Owned(v) => ValueView::Owned(v),
        }
    }
}
