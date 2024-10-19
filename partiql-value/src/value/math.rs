use crate::util;
use crate::Value;
use std::ops;

impl ops::Add for &Value {
    type Output = Value;

    fn add(self, rhs: Self) -> Self::Output {
        match (&self, &rhs) {
            // TODO: edge cases dealing with overflow
            (Value::Missing, _) => Value::Missing,
            (_, Value::Missing) => Value::Missing,
            (Value::Null, _) => Value::Null,
            (_, Value::Null) => Value::Null,
            (Value::Integer(l), Value::Integer(r)) => Value::Integer(l + r),
            (Value::Real(l), Value::Real(r)) => Value::Real(*l + *r),
            (Value::Decimal(l), Value::Decimal(r)) => {
                Value::Decimal(Box::new(l.as_ref() + r.as_ref()))
            }
            (Value::Integer(_), Value::Real(_)) => &util::coerce_int_to_real(self) + rhs,
            (Value::Integer(_), Value::Decimal(_)) => {
                &util::coerce_int_or_real_to_decimal(self) + rhs
            }
            (Value::Real(_), Value::Decimal(_)) => &util::coerce_int_or_real_to_decimal(self) + rhs,
            (Value::Real(_), Value::Integer(_)) => self + &util::coerce_int_to_real(rhs),
            (Value::Decimal(_), Value::Integer(_)) => {
                self + &util::coerce_int_or_real_to_decimal(rhs)
            }
            (Value::Decimal(_), Value::Real(_)) => self + &util::coerce_int_or_real_to_decimal(rhs),
            _ => Value::Missing, // data type mismatch => Missing
        }
    }
}

impl ops::AddAssign<&Value> for Value {
    fn add_assign(&mut self, rhs: &Value) {
        match (self, &rhs) {
            // TODO: edge cases dealing with overflow
            (Value::Missing, _) => {}
            (this, Value::Missing) => *this = Value::Missing,
            (Value::Null, _) => {}
            (this, Value::Null) => *this = Value::Null,

            (Value::Integer(l), Value::Integer(r)) => l.add_assign(r),

            (Value::Real(l), Value::Real(r)) => l.add_assign(r),
            (Value::Real(l), Value::Integer(i)) => l.add_assign(*i as f64),

            (Value::Decimal(l), Value::Decimal(r)) => l.add_assign(r.as_ref()),
            (Value::Decimal(l), Value::Integer(i)) => l.add_assign(rust_decimal::Decimal::from(*i)),
            (Value::Decimal(l), Value::Real(r)) => match util::coerce_f64_to_decimal(r) {
                Some(d) => l.add_assign(d),
                None => todo!(),
            },

            (this, Value::Real(r)) => {
                *this = match &this {
                    Value::Integer(l) => Value::from((*l as f64) + r.0),
                    _ => Value::Missing,
                };
            }
            (this, Value::Decimal(r)) => {
                *this = match &this {
                    Value::Integer(l) => {
                        Value::Decimal(Box::new(rust_decimal::Decimal::from(*l) + r.as_ref()))
                    }
                    Value::Real(l) => match util::coerce_f64_to_decimal(&l.0) {
                        None => Value::Missing,
                        Some(d) => Value::Decimal(Box::new(d + r.as_ref())),
                    },
                    _ => Value::Missing,
                };
            }
            (this, _) => *this = Value::Missing, // data type mismatch => Missing
        }
    }
}

pub trait UnaryPlus {
    type Output;

    fn positive(self) -> Self::Output;
}

impl UnaryPlus for Value {
    type Output = Self;
    fn positive(self) -> Self::Output {
        match self {
            Value::Null => Value::Null,
            Value::Missing => Value::Missing,
            Value::Integer(_) | Value::Real(_) | Value::Decimal(_) => self,
            _ => Value::Missing, // data type mismatch => Missing
        }
    }
}

impl ops::Sub for &Value {
    type Output = Value;

    fn sub(self, rhs: Self) -> Self::Output {
        match (&self, &rhs) {
            // TODO: edge cases dealing with overflow
            (Value::Missing, _) => Value::Missing,
            (_, Value::Missing) => Value::Missing,
            (Value::Null, _) => Value::Null,
            (_, Value::Null) => Value::Null,
            (Value::Integer(l), Value::Integer(r)) => Value::Integer(l - r),
            (Value::Real(l), Value::Real(r)) => Value::Real(*l - *r),
            (Value::Decimal(l), Value::Decimal(r)) => {
                Value::Decimal(Box::new(l.as_ref() - r.as_ref()))
            }
            (Value::Integer(_), Value::Real(_)) => &util::coerce_int_to_real(self) - rhs,
            (Value::Integer(_), Value::Decimal(_)) => {
                &util::coerce_int_or_real_to_decimal(self) - rhs
            }
            (Value::Real(_), Value::Decimal(_)) => &util::coerce_int_or_real_to_decimal(self) - rhs,
            (Value::Real(_), Value::Integer(_)) => self - &util::coerce_int_to_real(rhs),
            (Value::Decimal(_), Value::Integer(_)) => {
                self - &util::coerce_int_or_real_to_decimal(rhs)
            }
            (Value::Decimal(_), Value::Real(_)) => self - &util::coerce_int_or_real_to_decimal(rhs),
            _ => Value::Missing, // data type mismatch => Missing
        }
    }
}

impl ops::Mul for &Value {
    type Output = Value;

    fn mul(self, rhs: Self) -> Self::Output {
        match (&self, &rhs) {
            // TODO: edge cases dealing with overflow
            (Value::Missing, _) => Value::Missing,
            (_, Value::Missing) => Value::Missing,
            (Value::Null, _) => Value::Null,
            (_, Value::Null) => Value::Null,
            (Value::Integer(l), Value::Integer(r)) => Value::Integer(l * r),
            (Value::Real(l), Value::Real(r)) => Value::Real(*l * *r),
            (Value::Decimal(l), Value::Decimal(r)) => {
                Value::Decimal(Box::new(l.as_ref() * r.as_ref()))
            }
            (Value::Integer(_), Value::Real(_)) => &util::coerce_int_to_real(self) * rhs,
            (Value::Integer(_), Value::Decimal(_)) => {
                &util::coerce_int_or_real_to_decimal(self) * rhs
            }
            (Value::Real(_), Value::Decimal(_)) => &util::coerce_int_or_real_to_decimal(self) * rhs,
            (Value::Real(_), Value::Integer(_)) => self * &util::coerce_int_to_real(rhs),
            (Value::Decimal(_), Value::Integer(_)) => {
                self * &util::coerce_int_or_real_to_decimal(rhs)
            }
            (Value::Decimal(_), Value::Real(_)) => self * &util::coerce_int_or_real_to_decimal(rhs),
            _ => Value::Missing, // data type mismatch => Missing
        }
    }
}

impl ops::Div for &Value {
    type Output = Value;

    fn div(self, rhs: Self) -> Self::Output {
        match (&self, &rhs) {
            // TODO: edge cases dealing with division by 0
            (Value::Missing, _) => Value::Missing,
            (_, Value::Missing) => Value::Missing,
            (Value::Null, _) => Value::Null,
            (_, Value::Null) => Value::Null,
            (Value::Integer(l), Value::Integer(r)) => Value::Integer(l / r),
            (Value::Real(l), Value::Real(r)) => Value::Real(*l / *r),
            (Value::Decimal(l), Value::Decimal(r)) => {
                Value::Decimal(Box::new(l.as_ref() / r.as_ref()))
            }
            (Value::Integer(_), Value::Real(_)) => &util::coerce_int_to_real(self) / rhs,
            (Value::Integer(_), Value::Decimal(_)) => {
                &util::coerce_int_or_real_to_decimal(self) / rhs
            }
            (Value::Real(_), Value::Decimal(_)) => &util::coerce_int_or_real_to_decimal(self) / rhs,
            (Value::Real(_), Value::Integer(_)) => self / &util::coerce_int_to_real(rhs),
            (Value::Decimal(_), Value::Integer(_)) => {
                self / &util::coerce_int_or_real_to_decimal(rhs)
            }
            (Value::Decimal(_), Value::Real(_)) => self / &util::coerce_int_or_real_to_decimal(rhs),
            _ => Value::Missing, // data type mismatch => Missing
        }
    }
}

impl ops::Rem for &Value {
    type Output = Value;

    fn rem(self, rhs: Self) -> Self::Output {
        match (&self, &rhs) {
            // TODO: edge cases dealing with division by 0
            (Value::Missing, _) => Value::Missing,
            (_, Value::Missing) => Value::Missing,
            (Value::Null, _) => Value::Null,
            (_, Value::Null) => Value::Null,
            (Value::Integer(l), Value::Integer(r)) => Value::Integer(l % r),
            (Value::Real(l), Value::Real(r)) => Value::Real(*l % *r),
            (Value::Decimal(l), Value::Decimal(r)) => {
                Value::Decimal(Box::new(l.as_ref() % r.as_ref()))
            }
            (Value::Integer(_), Value::Real(_)) => &util::coerce_int_to_real(self) % rhs,
            (Value::Integer(_), Value::Decimal(_)) => {
                &util::coerce_int_or_real_to_decimal(self) % rhs
            }
            (Value::Real(_), Value::Decimal(_)) => &util::coerce_int_or_real_to_decimal(self) % rhs,
            (Value::Real(_), Value::Integer(_)) => self % &util::coerce_int_to_real(rhs),
            (Value::Decimal(_), Value::Integer(_)) => {
                self % &util::coerce_int_or_real_to_decimal(rhs)
            }
            (Value::Decimal(_), Value::Real(_)) => self % &util::coerce_int_or_real_to_decimal(rhs),
            _ => Value::Missing, // data type mismatch => Missing
        }
    }
}

impl ops::Neg for &Value {
    type Output = Value;

    fn neg(self) -> Self::Output {
        match self {
            // TODO: handle overflow for negation
            Value::Null => Value::Null,
            Value::Missing => Value::Missing,
            Value::Integer(i) => Value::from(-i),
            Value::Real(f) => Value::Real(-f),
            Value::Decimal(d) => Value::from(-d.as_ref()),
            _ => Value::Missing, // data type mismatch => Missing
        }
    }
}

impl ops::Neg for Value {
    type Output = Value;

    fn neg(self) -> Self::Output {
        match self {
            // TODO: handle overflow for negation
            Value::Null => self,
            Value::Missing => self,
            Value::Integer(i) => Value::from(-i),
            Value::Real(f) => Value::Real(-f),
            Value::Decimal(d) => Value::from(-d.as_ref()),
            _ => Value::Missing, // data type mismatch => Missing
        }
    }
}
