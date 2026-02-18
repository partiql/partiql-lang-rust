use super::value_owned::ValueOwned;
use crate::engine::arena::Arena;
use crate::engine::error::{EngineError, Result};
use rust_decimal::Decimal as RustDecimal;

/// Compact tuple representation stored in arena for zero-copy operations
#[derive(Clone, Copy, Debug)]
pub(crate) struct TupleRef<'a> {
    pub fields: &'a [TupleField<'a>],
}

/// A single field in a tuple - name and value stored contiguously
#[derive(Clone, Copy, Debug)]
pub(crate) struct TupleField<'a> {
    pub name: &'a str,
    pub value: ValueRef<'a>,
}

#[derive(Clone, Copy, Debug)]
pub(crate) enum ValueRef<'a> {
    Missing,
    Null,
    Bool(bool),
    I64(i64),
    F64(f64),
    Decimal(RustDecimal),
    Str(&'a str),
    Bytes(&'a [u8]),
    Tuple(&'a TupleRef<'a>),
    #[allow(dead_code)]
    List(&'a [ValueRef<'a>]),
    #[allow(dead_code)]
    Bag(&'a [ValueRef<'a>]),
}

impl<'a> ValueRef<'a> {
    pub fn from_owned(value: &'a ValueOwned, arena: &'a Arena) -> Self {
        match value {
            ValueOwned::Missing => ValueRef::Missing,
            ValueOwned::Null => ValueRef::Null,
            ValueOwned::Bool(v) => ValueRef::Bool(*v),
            ValueOwned::I64(v) => ValueRef::I64(*v),
            ValueOwned::F64(v) => ValueRef::F64(v.0),
            ValueOwned::Decimal(d) => ValueRef::Decimal(*d),
            ValueOwned::String(v) => ValueRef::Str(v.as_str()),
            ValueOwned::Bytes(v) => ValueRef::Bytes(v.as_slice()),
            ValueOwned::Tuple(t) => {
                // We still need arena for tuples because we need to convert
                // Vec<TupleFieldOwned> to &[TupleField] with different memory layout
                let fields: Vec<TupleField<'a>> = t
                    .fields
                    .iter()
                    .map(|f| TupleField {
                        name: f.name.as_str(),
                        value: ValueRef::from_owned(&f.value, arena),
                    })
                    .collect();

                let fields_slice = arena.alloc_slice(&fields);
                let tuple_ref = arena.alloc_tuple_ref(TupleRef {
                    fields: fields_slice,
                });
                ValueRef::Tuple(tuple_ref)
            }
            ValueOwned::List(items) => {
                // Convert Vec<ValueOwned> to &[ValueRef] by recursively converting each item
                let refs: Vec<ValueRef<'a>> = items
                    .iter()
                    .map(|item| ValueRef::from_owned(item, arena))
                    .collect();
                let refs_slice = arena.alloc_slice(&refs);
                ValueRef::List(refs_slice)
            }
            ValueOwned::Bag(items) => {
                // Convert Vec<ValueOwned> to &[ValueRef] by recursively converting each item
                let refs: Vec<ValueRef<'a>> = items
                    .iter()
                    .map(|item| ValueRef::from_owned(item, arena))
                    .collect();
                let refs_slice = arena.alloc_slice(&refs);
                ValueRef::Bag(refs_slice)
            }
        }
    }

    pub fn as_i64(&self) -> Result<i64> {
        match *self {
            ValueRef::I64(v) => Ok(v),
            _ => Err(EngineError::TypeError(format!(
                "expected i64, but received {:?}",
                *self
            ))),
        }
    }
    pub fn as_f64(&self) -> Result<f64> {
        match *self {
            ValueRef::F64(v) => Ok(v),
            _ => Err(EngineError::TypeError(format!(
                "expected f64, but received {:?}",
                *self
            ))),
        }
    }

    pub fn as_bool(&self) -> Result<bool> {
        match *self {
            ValueRef::Bool(v) => Ok(v),
            _ => Err(EngineError::TypeError("expected bool".to_string())),
        }
    }

    pub fn as_decimal(&self) -> Result<RustDecimal> {
        match *self {
            ValueRef::Decimal(d) => Ok(d),
            _ => Err(EngineError::TypeError(format!(
                "expected decimal, but received {:?}",
                *self
            ))),
        }
    }
}

pub(crate) fn value_get_field_ref<'a>(value: ValueRef<'a>, key: &str) -> ValueRef<'a> {
    match value {
        ValueRef::Tuple(tuple_ref) => {
            // Fast path: direct field lookup in arena-allocated tuple
            tuple_ref
                .fields
                .iter()
                .find(|field| field.name.eq_ignore_ascii_case(key))
                .map(|field| field.value)
                .unwrap_or(ValueRef::Missing)
        }
        _ => ValueRef::Missing,
    }
}
