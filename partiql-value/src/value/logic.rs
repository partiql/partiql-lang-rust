use crate::Value;
use std::ops;

impl ops::Not for &Value {
    type Output = Value;

    fn not(self) -> Self::Output {
        match self {
            Value::Boolean(b) => Value::from(!b),
            Value::Null | Value::Missing => Value::Null,
            _ => Value::Missing, // data type mismatch => Missing
        }
    }
}

impl ops::Not for Value {
    type Output = Self;

    fn not(self) -> Self::Output {
        match self {
            Value::Boolean(b) => Value::from(!b),
            Value::Null | Value::Missing => Value::Null,
            _ => Value::Missing, // data type mismatch => Missing
        }
    }
}

pub trait BinaryAnd {
    type Output;

    fn and(&self, rhs: &Self) -> Self::Output;
}

impl BinaryAnd for Value {
    type Output = Self;
    fn and(&self, rhs: &Self) -> Self::Output {
        match (self, rhs) {
            (Value::Boolean(l), Value::Boolean(r)) => Value::from(*l && *r),
            (Value::Null | Value::Missing, Value::Boolean(false))
            | (Value::Boolean(false), Value::Null | Value::Missing) => Value::from(false),
            _ => {
                if matches!(self, Value::Missing | Value::Null | Value::Boolean(true))
                    && matches!(rhs, Value::Missing | Value::Null | Value::Boolean(true))
                {
                    Value::Null
                } else {
                    Value::Missing
                }
            }
        }
    }
}

pub trait BinaryOr {
    type Output;

    fn or(&self, rhs: &Self) -> Self::Output;
}

impl BinaryOr for Value {
    type Output = Self;
    fn or(&self, rhs: &Self) -> Self::Output {
        match (self, rhs) {
            (Value::Boolean(l), Value::Boolean(r)) => Value::from(*l || *r),
            (Value::Null | Value::Missing, Value::Boolean(true))
            | (Value::Boolean(true), Value::Null | Value::Missing) => Value::from(true),
            _ => {
                if matches!(self, Value::Missing | Value::Null | Value::Boolean(false))
                    && matches!(rhs, Value::Missing | Value::Null | Value::Boolean(false))
                {
                    Value::Null
                } else {
                    Value::Missing
                }
            }
        }
    }
}
