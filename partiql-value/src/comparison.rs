use crate::util;
use crate::Value;

pub trait Comparable {
    fn is_comparable_to(&self, rhs: &Self) -> bool;
}

impl Comparable for Value {
    /// Returns true if and only if `self` is comparable to `rhs`
    fn is_comparable_to(&self, rhs: &Self) -> bool {
        match (self, rhs) {
            // Null/Missing compare to anything
            (Value::Missing | Value::Null, _)
            | (_, Value::Missing | Value::Null)
            // Everything compares to its own type
            | (Value::Boolean(_), Value::Boolean(_))
            | (Value::String(_), Value::String(_))
            | (Value::Blob(_), Value::Blob(_))
            | (Value::List(_), Value::List(_))
            | (Value::Bag(_), Value::Bag(_))
            | (Value::Tuple(_), Value::Tuple(_))
            // Numerics compare to each other
            | (
                Value::Integer(_) | Value::Real(_) | Value::Decimal(_),
                Value::Integer(_) | Value::Real(_) | Value::Decimal(_),
            )=> true,
            (_, _) => false,
        }
    }
}

// `Value` `eq` and `neq` with Missing and Null propagation
pub trait NullableEq {
    type Output;
    fn eq(&self, rhs: &Self) -> Self::Output;
    fn neq(&self, rhs: &Self) -> Self::Output;
}

/// A wrapper on [`T`] that specifies if missing and null values should be equal.
#[derive(Eq, PartialEq)]
pub struct EqualityValue<'a, const NULLS_EQUAL: bool, T>(pub &'a T);

impl<'a, const GROUP_NULLS: bool> NullableEq for EqualityValue<'a, GROUP_NULLS, Value> {
    type Output = Value;

    fn eq(&self, rhs: &Self) -> Self::Output {
        if GROUP_NULLS {
            if let (Value::Missing | Value::Null, Value::Missing | Value::Null) = (self.0, rhs.0) {
                return Value::Boolean(true);
            }
        } else if matches!(self.0, Value::Missing) || matches!(rhs.0, Value::Missing) {
            return Value::Missing;
        } else if matches!(self.0, Value::Null) || matches!(rhs.0, Value::Null) {
            return Value::Null;
        }

        match (self.0, rhs.0) {
            (Value::Integer(_), Value::Real(_)) => {
                Value::from(&util::coerce_int_to_real(self.0) == rhs.0)
            }
            (Value::Integer(_), Value::Decimal(_)) => {
                Value::from(&util::coerce_int_or_real_to_decimal(self.0) == rhs.0)
            }
            (Value::Real(_), Value::Decimal(_)) => {
                Value::from(&util::coerce_int_or_real_to_decimal(self.0) == rhs.0)
            }
            (Value::Real(_), Value::Integer(_)) => {
                Value::from(self.0 == &util::coerce_int_to_real(rhs.0))
            }
            (Value::Decimal(_), Value::Integer(_)) => {
                Value::from(self.0 == &util::coerce_int_or_real_to_decimal(rhs.0))
            }
            (Value::Decimal(_), Value::Real(_)) => {
                Value::from(self.0 == &util::coerce_int_or_real_to_decimal(rhs.0))
            }
            (_, _) => Value::from(self.0 == rhs.0),
        }
    }

    fn neq(&self, rhs: &Self) -> Self::Output {
        let eq_result = NullableEq::eq(self, rhs);
        match eq_result {
            Value::Boolean(_) | Value::Null => !eq_result,
            _ => Value::Missing,
        }
    }
}

// `Value` comparison with Missing and Null propagation
pub trait NullableOrd {
    type Output;

    fn lt(&self, rhs: &Self) -> Self::Output;
    fn gt(&self, rhs: &Self) -> Self::Output;
    fn lteq(&self, rhs: &Self) -> Self::Output;
    fn gteq(&self, rhs: &Self) -> Self::Output;
}

impl NullableOrd for Value {
    type Output = Self;

    fn lt(&self, rhs: &Self) -> Self::Output {
        match (self, rhs) {
            (Value::Missing, _) => Value::Missing,
            (_, Value::Missing) => Value::Missing,
            (Value::Null, _) => Value::Null,
            (_, Value::Null) => Value::Null,
            (_, _) => {
                if self.is_comparable_to(rhs) {
                    Value::from(self < rhs)
                } else {
                    Value::Missing
                }
            }
        }
    }

    fn gt(&self, rhs: &Self) -> Self::Output {
        match (self, rhs) {
            (Value::Missing, _) => Value::Missing,
            (_, Value::Missing) => Value::Missing,
            (Value::Null, _) => Value::Null,
            (_, Value::Null) => Value::Null,
            (_, _) => {
                if self.is_comparable_to(rhs) {
                    Value::from(self > rhs)
                } else {
                    Value::Missing
                }
            }
        }
    }

    fn lteq(&self, rhs: &Self) -> Self::Output {
        match (self, rhs) {
            (Value::Missing, _) => Value::Missing,
            (_, Value::Missing) => Value::Missing,
            (Value::Null, _) => Value::Null,
            (_, Value::Null) => Value::Null,
            (_, _) => {
                if self.is_comparable_to(rhs) {
                    Value::from(self <= rhs)
                } else {
                    Value::Missing
                }
            }
        }
    }

    fn gteq(&self, rhs: &Self) -> Self::Output {
        match (self, rhs) {
            (Value::Missing, _) => Value::Missing,
            (_, Value::Missing) => Value::Missing,
            (Value::Null, _) => Value::Null,
            (_, Value::Null) => Value::Null,
            (_, _) => {
                if self.is_comparable_to(rhs) {
                    Value::from(self >= rhs)
                } else {
                    Value::Missing
                }
            }
        }
    }
}
