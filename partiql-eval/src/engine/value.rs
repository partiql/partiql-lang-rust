use ordered_float::OrderedFloat;
use partiql_value::datum::{DatumCategory, DatumCategoryRef, RefTupleView};
use partiql_value::Value;
use std::borrow::Cow;

use crate::engine::error::{EngineError, Result};
use crate::engine::row::Arena;
use partiql_value::BindingsName;

pub type ValueOwned = Value;

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
        match value {
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

pub fn value_ref_from_value<'a>(value: &'a Value) -> ValueRef<'a> {
    ValueRef::from_owned(value)
}

pub fn value_ref_from_value_in_arena<'a>(value: &Value, arena: &'a Arena) -> ValueRef<'a> {
    let owned = arena.alloc(value.clone());
    ValueRef::from_owned(owned)
}

pub fn value_owned_from_ref(value: ValueRef<'_>) -> ValueOwned {
    match value {
        ValueRef::Missing => Value::Missing,
        ValueRef::Null => Value::Null,
        ValueRef::Bool(v) => Value::Boolean(v),
        ValueRef::I64(v) => Value::Integer(v),
        ValueRef::F64(v) => Value::Real(OrderedFloat(v)),
        ValueRef::Str(v) => Value::String(Box::new(v.to_string())),
        ValueRef::Bytes(v) => Value::Blob(Box::new(v.to_vec())),
        ValueRef::Owned(v) => v.clone(),
    }
}

pub fn value_get_field<'a>(value: ValueRef<'a>, key: &str) -> ValueRef<'a> {
    match value {
        ValueRef::Owned(Value::Tuple(tuple)) => {
            let name = BindingsName::CaseInsensitive(key.into());
            tuple.get(&name).map(ValueRef::from_owned).unwrap_or(ValueRef::Missing)
        }
        _ => ValueRef::Missing,
    }
}

pub fn value_get_field_ref<'a>(
    value: ValueRef<'a>,
    key: &str,
    arena: &'a Arena,
) -> ValueRef<'a> {
    match value {
        ValueRef::Owned(Value::Tuple(tuple)) => {
            let name = BindingsName::CaseInsensitive(key.into());
            tuple
                .get(&name)
                .map(ValueRef::from_owned)
                .unwrap_or(ValueRef::Missing)
        }
        ValueRef::Owned(Value::Variant(variant)) => match variant.category() {
            DatumCategoryRef::Tuple(tuple_ref) => {
                let name = BindingsName::CaseInsensitive(key.into());
                match tuple_ref.get_val(&name) {
                    Some(Cow::Borrowed(v)) => ValueRef::from_owned(v),
                    Some(Cow::Owned(v)) => ValueRef::from_owned(arena.alloc(v)),
                    None => ValueRef::Missing,
                }
            }
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
        match self {
            ValueView::Missing => Value::Missing,
            ValueView::Null => Value::Null,
            ValueView::Bool(v) => Value::Boolean(v),
            ValueView::I64(v) => Value::Integer(v),
            ValueView::F64(v) => Value::Real(OrderedFloat(v)),
            ValueView::Str(v) => Value::String(Box::new(v.to_string())),
            ValueView::Bytes(v) => Value::Blob(Box::new(v.to_vec())),
            ValueView::Owned(v) => v.clone(),
        }
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
