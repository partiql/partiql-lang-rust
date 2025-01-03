use crate::Value;
use crate::{util, Bag, List, Tuple};

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
            | (Value::DateTime(_), Value::DateTime(_))
            | (Value::List(_), Value::List(_))
            | (Value::Bag(_), Value::Bag(_))
            | (Value::Tuple(_), Value::Tuple(_))
            // Numerics compare to each other
            | (
                Value::Integer(_) | Value::Real(_) | Value::Decimal(_),
                Value::Integer(_) | Value::Real(_) | Value::Decimal(_),
            )=> true,
            (Value::Variant(lhs), Value::Variant(rhs)) => {
                lhs.is_comparable_to(rhs)
            }
            (_, _) => false,
        }
    }
}

// `Value` `eq` and `neq` with Missing and Null propagation
pub trait NullableEq {
    fn eq(&self, rhs: &Self) -> Value;

    fn neq(&self, rhs: &Self) -> Value {
        let eq_result = NullableEq::eq(self, rhs);
        match eq_result {
            Value::Boolean(_) | Value::Null => !eq_result,
            _ => Value::Missing,
        }
    }

    /// `PartiQL's `eqg` is used to compare the internals of Lists, Bags, and Tuples.
    ///
    /// > The eqg, unlike the =, returns true when a NULL is compared to a NULL or a MISSING
    /// > to a MISSING
    fn eqg(&self, rhs: &Self) -> Value;

    fn neqg(&self, rhs: &Self) -> Value {
        let eqg_result = NullableEq::eqg(self, rhs);
        match eqg_result {
            Value::Boolean(_) | Value::Null => !eqg_result,
            _ => Value::Missing,
        }
    }
}

/// A wrapper on [`T`] that specifies equality outcome for missing and null, and `NaN` values.
#[derive(Eq, PartialEq, Debug)]
pub struct EqualityValue<'a, const NULLS_EQUAL: bool, const NAN_EQUAL: bool, T>(pub &'a T);

impl<const GROUP_NULLS: bool, const NAN_EQUAL: bool> NullableEq
    for EqualityValue<'_, GROUP_NULLS, NAN_EQUAL, Value>
{
    #[inline(always)]
    fn eq(&self, rhs: &Self) -> Value {
        let wrap_list = EqualityValue::<'_, { GROUP_NULLS }, { NAN_EQUAL }, List>;
        let wrap_bag = EqualityValue::<'_, { GROUP_NULLS }, { NAN_EQUAL }, Bag>;
        let wrap_tuple = EqualityValue::<'_, { GROUP_NULLS }, { NAN_EQUAL }, Tuple>;
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
            (Value::Real(l), Value::Real(r)) => {
                if NAN_EQUAL && l.is_nan() && r.is_nan() {
                    return Value::Boolean(true);
                }
                Value::from(l == r)
            }
            (Value::List(l), Value::List(r)) => NullableEq::eq(&wrap_list(l), &wrap_list(r)),
            (Value::Bag(l), Value::Bag(r)) => NullableEq::eq(&wrap_bag(l), &wrap_bag(r)),
            (Value::Tuple(l), Value::Tuple(r)) => NullableEq::eq(&wrap_tuple(l), &wrap_tuple(r)),
            (_, _) => Value::from(self.0 == rhs.0),
        }
    }

    #[inline(always)]
    fn eqg(&self, rhs: &Self) -> Value {
        let wrap = EqualityValue::<'_, true, { NAN_EQUAL }, _>;
        NullableEq::eq(&wrap(self.0), &wrap(rhs.0))
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
