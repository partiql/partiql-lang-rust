use partiql_value::datum::{DatumCategory, DatumCategoryRef, RefTupleView};
use partiql_value::Value;
use std::borrow::Cow;

use super::value_owned::ValueOwned;
use crate::engine::error::{EngineError, Result};
use crate::engine::row::Arena;
use partiql_value::BindingsName;

#[derive(Clone, Copy, Debug)]
pub(crate) enum ValueRef<'a> {
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
        match value.as_ref() {
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

pub(crate) fn value_get_field_ref<'a>(
    value: ValueRef<'a>,
    key: &str,
    arena: &'a Arena,
) -> ValueRef<'a> {
    match value {
        ValueRef::Owned(owned) => match owned.as_ref() {
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
