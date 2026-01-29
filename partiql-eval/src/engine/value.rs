use ordered_float::OrderedFloat;
use partiql_value::datum::{DatumCategory, DatumCategoryRef, RefTupleView};
use partiql_value::Value;
use std::borrow::Cow;
use std::ops::Deref;

use crate::engine::error::{EngineError, Result};
use crate::engine::row::Arena;
use partiql_value::BindingsName;

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

#[derive(Clone, Copy, Debug)]
pub enum ValueRef<'a> {
    Missing,
    Null,
    Bool(bool),
    I64(i64),
    F64(f64),
    Str(&'a str),
    Bytes(&'a [u8]),
    Owned(&'a ValueOwned),
}

impl<'a> ValueRef<'a> {
    pub fn from_owned(value: &'a ValueOwned) -> Self {
        match value.deref() {
            Value::Missing => ValueRef::Missing,
            Value::Null => ValueRef::Null,
            Value::Boolean(v) => ValueRef::Bool(*v),
            Value::Integer(v) => ValueRef::I64(*v),
            Value::Real(v) => ValueRef::F64(v.0),
            Value::String(v) => ValueRef::Str(v.as_str()),
            Value::Blob(v) => ValueRef::Bytes(v.as_slice()),
            _ => ValueRef::Owned(value),
        }
    }

    pub fn as_i64(&self) -> Result<i64> {
        match *self {
            ValueRef::I64(v) => Ok(v),
            _ => Err(EngineError::TypeError("expected i64".to_string())),
        }
    }

    pub fn as_bool(&self) -> Result<bool> {
        match *self {
            ValueRef::Bool(v) => Ok(v),
            _ => Err(EngineError::TypeError("expected bool".to_string())),
        }
    }
}

pub fn value_get_field_ref<'a>(value: ValueRef<'a>, key: &str, arena: &'a Arena) -> ValueRef<'a> {
    match value {
        ValueRef::Owned(owned) => match owned.deref() {
            Value::Tuple(tuple) => {
                let name = BindingsName::CaseInsensitive(key.into());
                tuple
                    .get(&name)
                    .map(|v| {
                        ValueRef::from_owned(unsafe {
                            // Safety: ValueOwned is repr(transparent) and has the same layout as Value
                            &*(v as *const Value as *const ValueOwned)
                        })
                    })
                    .unwrap_or(ValueRef::Missing)
            }
            Value::Variant(variant) => match variant.category() {
                DatumCategoryRef::Tuple(tuple_ref) => {
                    let name = BindingsName::CaseInsensitive(key.into());
                    match tuple_ref.get_val(&name) {
                        Some(Cow::Borrowed(v)) => {
                            ValueRef::from_owned(unsafe {
                                // Safety: ValueOwned is repr(transparent) and has the same layout as Value
                                &*(v as *const Value as *const ValueOwned)
                            })
                        }
                        Some(Cow::Owned(v)) => {
                            ValueRef::from_owned(arena.alloc(ValueOwned::from(v)))
                        }
                        None => ValueRef::Missing,
                    }
                }
                _ => ValueRef::Missing,
            },
            _ => ValueRef::Missing,
        },
        _ => ValueRef::Missing,
    }
}

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
