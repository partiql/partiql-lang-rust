use itertools::Itertools;
use ordered_float::OrderedFloat;
use std::cmp::Ordering;

use std::collections::HashSet;
use std::fmt::{Debug, Formatter};
use std::hash::{Hash, Hasher};
use std::iter::zip;
use std::{ops, vec};

use rust_decimal::prelude::FromPrimitive;
use rust_decimal::{Decimal as RustDecimal, Decimal};

#[derive(Clone, Hash, Debug)]
pub enum BindingsName {
    CaseSensitive(String),
    CaseInsensitive(String),
}

// TODO these are all quite simplified for PoC/demonstration
// TODO have an optional-like wrapper for null/missing instead of inlined here?
#[derive(Hash, PartialEq, Eq, Clone)]
#[allow(dead_code)] // TODO remove once out of PoC
pub enum Value {
    Null,
    Missing,
    Boolean(bool),
    Integer(i64),
    Real(OrderedFloat<f64>),
    Decimal(RustDecimal),
    String(Box<String>),
    Blob(Box<Vec<u8>>),
    List(Box<List>),
    Bag(Box<Bag>),
    Tuple(Box<Tuple>),
    // TODO: add other supported PartiQL values -- timestamp, date, time, sexp
}

impl ops::Add for Value {
    type Output = Self;

    fn add(self, rhs: Self) -> Self::Output {
        match (&self, &rhs) {
            // TODO: edge cases dealing with overflow
            (Value::Missing, _) => Value::Missing,
            (_, Value::Missing) => Value::Missing,
            (Value::Null, _) => Value::Null,
            (_, Value::Null) => Value::Null,
            (Value::Integer(l), Value::Integer(r)) => Value::Integer(l + r),
            (Value::Real(l), Value::Real(r)) => Value::Real(*l + *r),
            (Value::Decimal(l), Value::Decimal(r)) => Value::Decimal(l + r),
            (Value::Integer(_), Value::Real(_)) => coerce_int_to_real(&self) + rhs,
            (Value::Integer(_), Value::Decimal(_)) => coerce_int_or_real_to_decimal(&self) + rhs,
            (Value::Real(_), Value::Decimal(_)) => coerce_int_or_real_to_decimal(&self) + rhs,
            (Value::Real(_), Value::Integer(_)) => self + coerce_int_to_real(&rhs),
            (Value::Decimal(_), Value::Integer(_)) => self + coerce_int_or_real_to_decimal(&rhs),
            (Value::Decimal(_), Value::Real(_)) => self + coerce_int_or_real_to_decimal(&rhs),
            _ => Value::Missing, // data type mismatch => Missing
        }
    }
}

impl ops::Sub for Value {
    type Output = Self;

    fn sub(self, rhs: Self) -> Self::Output {
        match (&self, &rhs) {
            // TODO: edge cases dealing with overflow
            (Value::Missing, _) => Value::Missing,
            (_, Value::Missing) => Value::Missing,
            (Value::Null, _) => Value::Null,
            (_, Value::Null) => Value::Null,
            (Value::Integer(l), Value::Integer(r)) => Value::Integer(l - r),
            (Value::Real(l), Value::Real(r)) => Value::Real(*l - *r),
            (Value::Decimal(l), Value::Decimal(r)) => Value::Decimal(l - r),
            (Value::Integer(_), Value::Real(_)) => coerce_int_to_real(&self) - rhs,
            (Value::Integer(_), Value::Decimal(_)) => coerce_int_or_real_to_decimal(&self) - rhs,
            (Value::Real(_), Value::Decimal(_)) => coerce_int_or_real_to_decimal(&self) - rhs,
            (Value::Real(_), Value::Integer(_)) => self - coerce_int_to_real(&rhs),
            (Value::Decimal(_), Value::Integer(_)) => self - coerce_int_or_real_to_decimal(&rhs),
            (Value::Decimal(_), Value::Real(_)) => self - coerce_int_or_real_to_decimal(&rhs),
            _ => Value::Missing, // data type mismatch => Missing
        }
    }
}

impl ops::Mul for Value {
    type Output = Self;

    fn mul(self, rhs: Self) -> Self::Output {
        match (&self, &rhs) {
            // TODO: edge cases dealing with overflow
            (Value::Missing, _) => Value::Missing,
            (_, Value::Missing) => Value::Missing,
            (Value::Null, _) => Value::Null,
            (_, Value::Null) => Value::Null,
            (Value::Integer(l), Value::Integer(r)) => Value::Integer(l * r),
            (Value::Real(l), Value::Real(r)) => Value::Real(*l * *r),
            (Value::Decimal(l), Value::Decimal(r)) => Value::Decimal(l * r),
            (Value::Integer(_), Value::Real(_)) => coerce_int_to_real(&self) * rhs,
            (Value::Integer(_), Value::Decimal(_)) => coerce_int_or_real_to_decimal(&self) * rhs,
            (Value::Real(_), Value::Decimal(_)) => coerce_int_or_real_to_decimal(&self) * rhs,
            (Value::Real(_), Value::Integer(_)) => self * coerce_int_to_real(&rhs),
            (Value::Decimal(_), Value::Integer(_)) => self * coerce_int_or_real_to_decimal(&rhs),
            (Value::Decimal(_), Value::Real(_)) => self * coerce_int_or_real_to_decimal(&rhs),
            _ => Value::Missing, // data type mismatch => Missing
        }
    }
}

impl ops::Div for Value {
    type Output = Self;

    fn div(self, rhs: Self) -> Self::Output {
        match (&self, &rhs) {
            // TODO: edge cases dealing with division by 0
            (Value::Missing, _) => Value::Missing,
            (_, Value::Missing) => Value::Missing,
            (Value::Null, _) => Value::Null,
            (_, Value::Null) => Value::Null,
            (Value::Integer(l), Value::Integer(r)) => Value::Integer(l / r),
            (Value::Real(l), Value::Real(r)) => Value::Real(*l / *r),
            (Value::Decimal(l), Value::Decimal(r)) => Value::Decimal(l / r),
            (Value::Integer(_), Value::Real(_)) => coerce_int_to_real(&self) / rhs,
            (Value::Integer(_), Value::Decimal(_)) => coerce_int_or_real_to_decimal(&self) / rhs,
            (Value::Real(_), Value::Decimal(_)) => coerce_int_or_real_to_decimal(&self) / rhs,
            (Value::Real(_), Value::Integer(_)) => self / coerce_int_to_real(&rhs),
            (Value::Decimal(_), Value::Integer(_)) => self / coerce_int_or_real_to_decimal(&rhs),
            (Value::Decimal(_), Value::Real(_)) => self / coerce_int_or_real_to_decimal(&rhs),
            _ => Value::Missing, // data type mismatch => Missing
        }
    }
}

impl ops::Rem for Value {
    type Output = Self;

    fn rem(self, rhs: Self) -> Self::Output {
        match (&self, &rhs) {
            // TODO: edge cases dealing with division by 0
            (Value::Missing, _) => Value::Missing,
            (_, Value::Missing) => Value::Missing,
            (Value::Null, _) => Value::Null,
            (_, Value::Null) => Value::Null,
            (Value::Integer(l), Value::Integer(r)) => Value::Integer(l % r),
            (Value::Real(l), Value::Real(r)) => Value::Real(*l % *r),
            (Value::Decimal(l), Value::Decimal(r)) => Value::Decimal(l % r),
            (Value::Integer(_), Value::Real(_)) => coerce_int_to_real(&self) % rhs,
            (Value::Integer(_), Value::Decimal(_)) => coerce_int_or_real_to_decimal(&self) % rhs,
            (Value::Real(_), Value::Decimal(_)) => coerce_int_or_real_to_decimal(&self) % rhs,
            (Value::Real(_), Value::Integer(_)) => self % coerce_int_to_real(&rhs),
            (Value::Decimal(_), Value::Integer(_)) => self % coerce_int_or_real_to_decimal(&rhs),
            (Value::Decimal(_), Value::Real(_)) => self % coerce_int_or_real_to_decimal(&rhs),
            _ => Value::Missing, // data type mismatch => Missing
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

impl ops::Neg for Value {
    type Output = Self;

    fn neg(self) -> Self::Output {
        match &self {
            // TODO: handle overflow for negation
            Value::Null => Value::Null,
            Value::Missing => Value::Missing,
            Value::Integer(i) => Value::from(-i),
            Value::Real(f) => Value::Real(-f),
            Value::Decimal(d) => Value::from(-d),
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
            (Value::Null, Value::Boolean(false))
            | (Value::Boolean(false), Value::Null)
            | (Value::Missing, Value::Boolean(false))
            | (Value::Boolean(false), Value::Missing) => Value::from(false),
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
            (Value::Null, Value::Boolean(true))
            | (Value::Boolean(true), Value::Null)
            | (Value::Missing, Value::Boolean(true))
            | (Value::Boolean(true), Value::Missing) => Value::from(true),
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

impl ops::Not for Value {
    type Output = Self;

    fn not(self) -> Self::Output {
        match &self {
            Value::Boolean(b) => Value::from(!b),
            Value::Null | Value::Missing => Value::Null,
            _ => Value::Missing, // data type mismatch => Missing
        }
    }
}

pub trait Comparable {
    type Output;

    fn is_comparable_to(&self, rhs: &Self) -> bool;
}

impl Comparable for Value {
    type Output = Self;

    /// Returns true if and only if `self` is comparable to `rhs`
    fn is_comparable_to(&self, rhs: &Self) -> bool {
        match (self, rhs) {
            (Value::Missing, _) |
            (_, Value::Missing) |
            (Value::Null, _) |
            (_, Value::Null) |
            (Value::Boolean(_), Value::Boolean(_)) |
            (Value::Integer(_), Value::Integer(_)) |
            (Value::Real(_), Value::Real(_)) |
            (Value::Decimal(_), Value::Decimal(_)) |
            (Value::String(_), Value::String(_)) |
            (Value::Blob(_), Value::Blob(_)) |
            (Value::List(_), Value::List(_)) |
            (Value::Bag(_), Value::Bag(_)) |
            (Value::Tuple(_), Value::Tuple(_)) |
            // Numeric types are comparable to one another
            (Value::Integer(_), Value::Real(_)) |
            (Value::Integer(_), Value::Decimal(_)) |
            (Value::Real(_), Value::Integer(_)) |
            (Value::Real(_), Value::Decimal(_)) |
            (Value::Decimal(_), Value::Integer(_)) |
            (Value::Decimal(_), Value::Real(_)) => true,
            (_, _) => false
        }
    }
}

// `Value` `eq` and `neq` with Missing and Null propagation
pub trait NullableEq {
    type Output;

    fn eq(&self, rhs: &Self) -> Self::Output;
    fn neq(&self, rhs: &Self) -> Self::Output;
}

// `Value` comparison with Missing and Null propagation
pub trait NullableOrd {
    type Output;

    fn lt(&self, rhs: &Self) -> Self::Output;
    fn gt(&self, rhs: &Self) -> Self::Output;
    fn lteq(&self, rhs: &Self) -> Self::Output;
    fn gteq(&self, rhs: &Self) -> Self::Output;
}

impl NullableEq for Value {
    type Output = Self;

    fn eq(&self, rhs: &Self) -> Self::Output {
        match (self, rhs) {
            (Value::Missing, _) => Value::Missing,
            (_, Value::Missing) => Value::Missing,
            (Value::Null, _) => Value::Null,
            (_, Value::Null) => Value::Null,
            (_, _) => Value::from(self == rhs),
        }
    }

    fn neq(&self, rhs: &Self) -> Self::Output {
        match (self, rhs) {
            (Value::Missing, _) => Value::Missing,
            (_, Value::Missing) => Value::Missing,
            (Value::Null, _) => Value::Null,
            (_, Value::Null) => Value::Null,
            (_, _) => Value::from(self != rhs),
        }
    }
}

impl NullableOrd for Value {
    type Output = Self;

    // TODO: comparison is not right for data type mismatches. Equality permits mistyped arguments
    //  while comparison ops should return Missing
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

fn coerce_int_or_real_to_decimal(value: &Value) -> Value {
    match value {
        Value::Integer(int_value) => Value::Decimal(rust_decimal::Decimal::from(*int_value)),
        Value::Real(real_value) => {
            if !real_value.is_finite() {
                Value::Missing
            } else {
                match Decimal::from_f64(real_value.0) {
                    Some(d_from_r) => Value::Decimal(d_from_r),
                    None => Value::Missing, // TODO: decide on behavior when float cannot be coerced to Decimal
                }
            }
        }
        _ => todo!("Unsupported coercion to Decimal"),
    }
}

fn coerce_int_to_real(value: &Value) -> Value {
    match value {
        Value::Integer(int_value) => Value::Real(OrderedFloat(*int_value as f64)),
        _ => todo!("Unsupported coercion to Real"),
    }
}

impl Default for Value {
    fn default() -> Self {
        Value::Missing
    }
}

impl Value {
    #[inline]
    pub fn is_list(&self) -> bool {
        matches!(self, Value::List(_))
    }

    #[inline]
    pub fn is_bag(&self) -> bool {
        matches!(self, Value::Bag(_))
    }

    #[inline]
    pub fn is_ordered(&self) -> bool {
        self.is_list()
    }

    #[inline]
    pub fn coerce_to_tuple(self) -> Tuple {
        if let Value::Tuple(t) = self {
            *t
        } else {
            let fresh_key = "_1"; // TODO don't hard-code 'fresh' keys
            Tuple::from([(fresh_key, self)])
        }
    }

    #[inline]
    pub fn coerce_to_bag(self) -> Bag {
        if let Value::Bag(b) = self {
            *b
        } else {
            Bag(vec![self])
        }
    }
}

impl IntoIterator for Value {
    type Item = Value;
    type IntoIter = ValueIntoIterator;

    fn into_iter(self) -> ValueIntoIterator {
        match self {
            Value::List(list) => ValueIntoIterator::List(list.into_iter()),
            Value::Bag(bag) => ValueIntoIterator::Bag(bag.into_iter()),
            other => ValueIntoIterator::Single(Some(other)),
        }
    }
}

pub enum ValueIntoIterator {
    List(ListIntoIterator),
    Bag(BagIntoIterator),
    Single(Option<Value>),
}

impl Iterator for ValueIntoIterator {
    type Item = Value;

    fn next(&mut self) -> Option<Self::Item> {
        match self {
            ValueIntoIterator::List(list) => list.next(),
            ValueIntoIterator::Bag(bag) => bag.next(),
            ValueIntoIterator::Single(v) => v.take(),
        }
    }
}

// TODO make debug emit proper PartiQL notation
// TODO perhaps this should be display as well?
impl Debug for Value {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        match self {
            Value::Null => write!(f, "NULL"),
            Value::Missing => write!(f, "MISSING"),
            Value::Boolean(b) => write!(f, "{}", b),
            Value::Integer(i) => write!(f, "{}", i),
            Value::Real(r) => write!(f, "{}", r.0),
            Value::Decimal(d) => write!(f, "{}", d),
            Value::String(s) => write!(f, "'{}'", s),
            Value::Blob(s) => write!(f, "'{:?}'", s),
            Value::List(l) => l.fmt(f),
            Value::Bag(b) => b.fmt(f),
            Value::Tuple(t) => write!(f, "{:?}", t),
        }
    }
}

impl PartialOrd for Value {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}

/// Implementation of Spec's `order-by less-than`
impl Ord for Value {
    fn cmp(&self, other: &Self) -> Ordering {
        match (self, other) {
            (Value::Null, Value::Null) => Ordering::Equal,
            (Value::Missing, Value::Null) => Ordering::Equal,

            (Value::Null, Value::Missing) => Ordering::Equal,
            (Value::Null, _) => Ordering::Less,
            (_, Value::Null) => Ordering::Greater,

            (Value::Missing, Value::Missing) => Ordering::Equal,
            (Value::Missing, _) => Ordering::Less,
            (_, Value::Missing) => Ordering::Greater,

            (Value::Boolean(l), Value::Boolean(r)) => match (l, r) {
                (false, true) => Ordering::Less,
                (true, false) => Ordering::Greater,
                (_, _) => Ordering::Equal,
            },
            (Value::Boolean(_), _) => Ordering::Less,
            (_, Value::Boolean(_)) => Ordering::Greater,

            // TODO: `OrderedFloat`'s implementation of `Ord` slightly differs from what we want in
            //  the PartiQL spec. See https://partiql.org/assets/PartiQL-Specification.pdf#subsection.12.2
            //  point 3. In PartiQL, `nan`, comes before `-inf` which comes before all numeric
            //  values, which are followed by `+inf`. `OrderedFloat` places `NaN` as greater than
            //  all other `OrderedFloat` values. We could consider creating our own float type
            //  to get around this annoyance.
            (Value::Real(l), Value::Real(r)) => {
                if l.is_nan() {
                    if r.is_nan() {
                        Ordering::Equal
                    } else {
                        Ordering::Less
                    }
                } else if r.is_nan() {
                    Ordering::Greater
                } else {
                    l.cmp(r)
                }
            }
            (Value::Integer(l), Value::Integer(r)) => l.cmp(r),
            (Value::Decimal(l), Value::Decimal(r)) => l.cmp(r),
            (Value::Integer(l), Value::Real(_)) => {
                Value::Real(ordered_float::OrderedFloat(*l as f64)).cmp(other)
            }
            (Value::Real(_), Value::Integer(r)) => {
                self.cmp(&Value::Real(ordered_float::OrderedFloat(*r as f64)))
            }
            (Value::Integer(l), Value::Decimal(r)) => RustDecimal::from(*l).cmp(r),
            (Value::Decimal(l), Value::Integer(r)) => l.cmp(&RustDecimal::from(*r)),
            (Value::Real(l), Value::Decimal(r)) => {
                if l.is_nan() || l.0 == f64::NEG_INFINITY {
                    Ordering::Less
                } else if l.0 == f64::INFINITY {
                    Ordering::Greater
                } else {
                    match RustDecimal::from_f64(l.0) {
                        Some(l_d) => l_d.cmp(r),
                        None => todo!(
                            "Decide default behavior when f64 can't be converted to RustDecimal"
                        ),
                    }
                }
            }
            (Value::Decimal(l), Value::Real(r)) => {
                if r.is_nan() || r.0 == f64::NEG_INFINITY {
                    Ordering::Greater
                } else if r.0 == f64::INFINITY {
                    Ordering::Less
                } else {
                    match RustDecimal::from_f64(r.0) {
                        Some(r_d) => l.cmp(&r_d),
                        None => todo!(
                            "Decide default behavior when f64 can't be converted to RustDecimal"
                        ),
                    }
                }
            }
            (Value::Integer(_), _) => Ordering::Less,
            (Value::Real(_), _) => Ordering::Less,
            (Value::Decimal(_), _) => Ordering::Less,
            (_, Value::Integer(_)) => Ordering::Greater,
            (_, Value::Real(_)) => Ordering::Greater,
            (_, Value::Decimal(_)) => Ordering::Greater,

            (Value::String(l), Value::String(r)) => l.cmp(r),
            (Value::String(_), _) => Ordering::Less,
            (_, Value::String(_)) => Ordering::Greater,

            (Value::Blob(l), Value::Blob(r)) => l.cmp(r),
            (Value::Blob(_), _) => Ordering::Less,
            (_, Value::Blob(_)) => Ordering::Greater,

            (Value::List(l), Value::List(r)) => l.cmp(r),
            (Value::List(_), _) => Ordering::Less,
            (_, Value::List(_)) => Ordering::Greater,

            (Value::Tuple(l), Value::Tuple(r)) => l.cmp(r),
            (Value::Tuple(_), _) => Ordering::Less,
            (_, Value::Tuple(_)) => Ordering::Greater,

            (Value::Bag(l), Value::Bag(r)) => l.cmp(r),
        }
    }
}

impl From<bool> for Value {
    #[inline]
    fn from(b: bool) -> Self {
        Value::Boolean(b)
    }
}

impl From<String> for Value {
    #[inline]
    fn from(s: String) -> Self {
        Value::String(Box::new(s))
    }
}

impl From<&str> for Value {
    #[inline]
    fn from(s: &str) -> Self {
        Value::String(Box::new(s.to_string()))
    }
}

impl From<i64> for Value {
    #[inline]
    fn from(n: i64) -> Self {
        Value::Integer(n)
    }
}

impl From<f64> for Value {
    #[inline]
    fn from(f: f64) -> Self {
        Value::Real(OrderedFloat(f))
    }
}

impl From<RustDecimal> for Value {
    #[inline]
    fn from(d: RustDecimal) -> Self {
        Value::Decimal(d)
    }
}

impl From<List> for Value {
    #[inline]
    fn from(v: List) -> Self {
        Value::List(Box::new(v))
    }
}

impl From<Tuple> for Value {
    #[inline]
    fn from(v: Tuple) -> Self {
        Value::Tuple(Box::new(v))
    }
}

impl From<Bag> for Value {
    #[inline]
    fn from(v: Bag) -> Self {
        Value::Bag(Box::new(v))
    }
}

#[derive(Default, Hash, PartialEq, Eq, Clone)]
/// Represents a PartiQL List value, e.g. [1, 2, 'one']
pub struct List(Vec<Value>);

impl List {
    #[inline]
    pub fn push(&mut self, value: Value) {
        self.0.push(value);
    }

    #[inline]
    pub fn len(&self) -> usize {
        self.0.len()
    }

    #[inline]
    pub fn is_empty(&self) -> bool {
        self.0.is_empty()
    }

    #[inline]
    pub fn get(&self, idx: i64) -> Option<&Value> {
        self.0.get(idx as usize)
    }

    #[inline]
    pub fn get_mut(&mut self, idx: i64) -> Option<&mut Value> {
        self.0.get_mut(idx as usize)
    }
}

impl From<Vec<Value>> for List {
    #[inline]
    fn from(values: Vec<Value>) -> Self {
        List(values)
    }
}

impl From<Bag> for List {
    #[inline]
    fn from(bag: Bag) -> Self {
        List(bag.0)
    }
}

#[macro_export]
macro_rules! partiql_list {
    () => (
         List::from(vec![])
    );
    ($elem:expr; $n:expr) => (
        List::from(vec![Value::from($elem); $n])
    );
    ($($x:expr),+ $(,)?) => (
        List::from(vec![$(Value::from($x)),+])
    );
}

impl IntoIterator for List {
    type Item = Value;
    type IntoIter = ListIntoIterator;

    fn into_iter(self) -> ListIntoIterator {
        ListIntoIterator(self.0.into_iter())
    }
}

pub struct ListIntoIterator(vec::IntoIter<Value>);

impl Iterator for ListIntoIterator {
    type Item = Value;

    fn next(&mut self) -> Option<Self::Item> {
        self.0.next()
    }
}

impl Debug for List {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        f.debug_list().entries(&self.0).finish()
    }
}

impl PartialOrd for List {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        let mut l = self.0.iter();
        let mut r = other.0.iter();

        loop {
            match (l.next(), r.next()) {
                (None, None) => return Some(Ordering::Equal),
                (Some(_), None) => return Some(Ordering::Greater),
                (None, Some(_)) => return Some(Ordering::Less),
                (Some(lv), Some(rv)) => match lv.partial_cmp(rv) {
                    None => return None,
                    Some(Ordering::Less) => return Some(Ordering::Less),
                    Some(Ordering::Greater) => return Some(Ordering::Greater),
                    Some(Ordering::Equal) => continue,
                },
            }
        }
    }
}

impl Ord for List {
    fn cmp(&self, other: &Self) -> Ordering {
        let mut l = self.0.iter();
        let mut r = other.0.iter();

        loop {
            match (l.next(), r.next()) {
                (None, None) => return Ordering::Equal,
                (Some(_), None) => return Ordering::Greater,
                (None, Some(_)) => return Ordering::Less,
                (Some(lv), Some(rv)) => match lv.cmp(rv) {
                    Ordering::Less => return Ordering::Less,
                    Ordering::Greater => return Ordering::Greater,
                    Ordering::Equal => continue,
                },
            }
        }
    }
}

#[derive(Default, Eq, Clone)]
/// Represents a PartiQL BAG value, e.g.: <<1, 'two', 4>>
pub struct Bag(Vec<Value>);

impl Bag {
    #[inline]
    pub fn push(&mut self, value: Value) {
        self.0.push(value);
    }

    #[inline]
    pub fn len(&self) -> usize {
        self.0.len()
    }

    #[inline]
    pub fn is_empty(&self) -> bool {
        self.0.is_empty()
    }
}

impl From<Vec<Value>> for Bag {
    #[inline]
    fn from(values: Vec<Value>) -> Self {
        Bag(values)
    }
}

impl From<HashSet<Value>> for Bag {
    #[inline]
    fn from(values: HashSet<Value>) -> Self {
        Bag(values.into_iter().collect())
    }
}

impl From<List> for Bag {
    #[inline]
    fn from(list: List) -> Self {
        Bag(list.0)
    }
}

#[macro_export]
macro_rules! partiql_bag {
    () => (
         Bag::from(vec![])
    );
    ($elem:expr; $n:expr) => (
        Bag::from(vec![Value::from($elem); $n])
    );
    ($($x:expr),+ $(,)?) => (
        Bag::from(vec![$(Value::from($x)),+])
    );
}

impl IntoIterator for Bag {
    type Item = Value;
    type IntoIter = BagIntoIterator;

    fn into_iter(self) -> BagIntoIterator {
        BagIntoIterator(self.0.into_iter())
    }
}

pub struct BagIntoIterator(vec::IntoIter<Value>);

impl Iterator for BagIntoIterator {
    type Item = Value;

    fn next(&mut self) -> Option<Self::Item> {
        self.0.next()
    }
}

impl Debug for Bag {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f, "<<")?;
        f.debug_list().entries(&self.0).finish()?; // TODO currently outputs <<[ ... ]>>
        write!(f, ">>")
    }
}

impl PartialEq for Bag {
    fn eq(&self, other: &Self) -> bool {
        if self.len() != other.len() {
            return false;
        }

        let lhs = self.0.iter().sorted();
        let rhs = other.0.iter().sorted();
        for (l, r) in lhs.zip(rhs) {
            if l != r {
                return false;
            }
        }
        true
    }
}

impl PartialOrd for Bag {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}

impl Ord for Bag {
    fn cmp(&self, other: &Self) -> Ordering {
        let mut l = self.0.clone();
        l.sort();
        let mut r = other.0.clone();
        r.sort();
        List(l).cmp(&List(r))
    }
}

impl Hash for Bag {
    fn hash<H: Hasher>(&self, state: &mut H) {
        for v in self.0.iter().sorted() {
            v.hash(state);
        }
    }
}

#[derive(Default, Eq, Clone)]
pub struct Tuple {
    attrs: Vec<String>,
    vals: Vec<Value>,
}

impl Tuple {
    pub fn new() -> Self {
        Tuple {
            attrs: vec![],
            vals: vec![],
        }
    }

    #[inline]
    pub fn insert(&mut self, attr: &str, val: Value) {
        self.attrs.push(attr.to_string());
        self.vals.push(val);
    }

    #[inline]
    pub fn get(&self, attr: &str) -> Option<&Value> {
        match self.attrs.iter().position(|a| a.as_str() == attr) {
            Some(i) => Some(&self.vals[i]),
            _ => None,
        }
    }

    #[inline]
    pub fn remove(&mut self, attr: &str) -> Option<Value> {
        match self.attrs.iter().position(|a| a.as_str() == attr) {
            Some(i) => {
                self.attrs.remove(i);
                Some(self.vals.remove(i))
            }
            _ => None,
        }
    }

    #[inline]
    pub fn pairs(&self) -> Vec<(&str, &Value)> {
        zip(&self.attrs, &self.vals)
            .map(|(k, v)| (k.as_str(), v))
            .collect()
    }
}

impl<const N: usize, T> From<[(&str, T); N]> for Tuple
where
    T: Into<Value>,
{
    #[inline]
    fn from(arr: [(&str, T); N]) -> Self {
        arr.into_iter()
            .fold(Tuple::new(), |mut acc: Tuple, (attr, val)| {
                acc.insert(attr, val.into());
                acc
            })
    }
}

impl Iterator for Tuple {
    type Item = (String, Value);

    fn next(&mut self) -> Option<Self::Item> {
        match (self.attrs.pop(), self.vals.pop()) {
            (Some(attr), Some(val)) => Some((attr, val)),
            _ => None,
        }
    }
}

impl PartialEq for Tuple {
    fn eq(&self, other: &Self) -> bool {
        let s1: HashSet<(&str, &Value)> = self.pairs().into_iter().collect();
        let s2: HashSet<(&str, &Value)> = other.pairs().into_iter().collect();
        s1.eq(&s2)
    }
}

impl PartialOrd for Tuple {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}

impl Hash for Tuple {
    fn hash<H: Hasher>(&self, state: &mut H) {
        for (k, v) in self.pairs() {
            k.hash(state);
            v.hash(state);
        }
    }
}

impl Debug for Tuple {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        let pairs = self.pairs();
        let mut fmt = f.debug_struct("Tuple");
        pairs.into_iter().for_each(|(k, v)| {
            fmt.field(k, v);
        });

        fmt.finish()
    }
}

impl Ord for Tuple {
    fn cmp(&self, other: &Self) -> Ordering {
        let self_pairs = self.pairs();
        let other_pairs = other.pairs();
        let mut p1 = self_pairs.iter().sorted();
        let mut p2 = other_pairs.iter().sorted();

        loop {
            return match (p1.next(), p2.next()) {
                (None, None) => Ordering::Equal,
                (Some(_), None) => Ordering::Greater,
                (None, Some(_)) => Ordering::Less,
                (Some(lv), Some(rv)) => match lv.cmp(rv) {
                    Ordering::Less => Ordering::Less,
                    Ordering::Greater => Ordering::Greater,
                    Ordering::Equal => continue,
                },
            };
        }
    }
}

#[macro_export]
macro_rules! partiql_tuple {
    () => (
         Tuple::new()
    );
    ($(($x:expr, $y:expr)),+ $(,)?) => (
        Tuple::from([$(($x, Value::from($y))),+])
    );
}

#[cfg(test)]
mod tests {
    use super::*;
    use rust_decimal_macros::dec;
    use std::borrow::Cow;
    use std::cell::RefCell;
    use std::mem;
    use std::rc::Rc;

    #[test]
    fn value_size() {
        println!("bool size: {}", mem::size_of::<bool>());
        println!("i64 size: {}", mem::size_of::<i64>());
        println!(
            "OrderedFloat<f64> size: {}",
            mem::size_of::<OrderedFloat<f64>>()
        );
        println!("Decimal size: {}", mem::size_of::<RustDecimal>());
        println!("String size: {}", mem::size_of::<String>());
        println!("Bag size: {}", mem::size_of::<Bag>());
        println!("List size: {}", mem::size_of::<List>());
        println!("Tuple size: {}", mem::size_of::<Tuple>());
        println!("Box<Tuple> size: {}", mem::size_of::<Box<Tuple>>());
        println!("Rc<Tuple> size: {}", mem::size_of::<Rc<Tuple>>());
        println!(
            "Rc<RefCell<Tuple>> size: {}",
            mem::size_of::<Rc<RefCell<Tuple>>>()
        );
        println!("Cow<&Tuple> size: {}", mem::size_of::<Cow<&Tuple>>());
        println!("Value size: {}", mem::size_of::<Value>());
        println!("Option<Value> size: {}", mem::size_of::<Option<Value>>());
        println!("Cow<Value> size: {}", mem::size_of::<Cow<Value>>());
        println!("Cow<&Value> size: {}", mem::size_of::<Cow<&Value>>());
    }

    #[test]
    fn macro_rules_tests() {
        println!("partiql_list:{:?}", partiql_list!());
        println!("partiql_list:{:?}", partiql_list![10, 10]);
        println!("partiql_list:{:?}", partiql_list!(5; 3));
        println!("partiql_bag:{:?}", partiql_bag!());
        println!("partiql_bag:{:?}", partiql_bag![10, 10]);
        println!("partiql_bag:{:?}", partiql_bag!(5; 3));
        println!("partiql_tuple:{:?}", partiql_tuple![("a", 1), ("b", 2)]);
    }

    #[test]
    fn partiql_value_ordering() {
        // TODO: some additional checking can be included in the ordering testing
        //  - add timestamp, date, time once added to `Value`
        //  - equality checking between equivalent ordered values (e.g. missing and null, same numeric values)
        let mut vals = vec![
            Value::Missing,
            Value::from(false),
            Value::from(true),
            Value::from(f64::NAN),
            Value::from(f64::NEG_INFINITY),
            Value::from(-123.456),
            Value::Decimal(dec!(1.23456)),
            Value::from(123456),
            Value::from(f64::INFINITY),
            Value::from(""),
            Value::from("abc"),
            Value::Blob(Box::new(vec![])),
            Value::Blob(Box::new(vec![1, 2, 3])),
            Value::from(partiql_list!()),
            Value::from(partiql_list!(1, 2, 3)),
            Value::from(partiql_list!(1, 2, 3, 4, 5)),
            Value::from(partiql_tuple!()),
            Value::from(partiql_tuple![("a", 1), ("b", 2)]),
            Value::from(partiql_tuple![("a", 1), ("b", 3)]),
            Value::from(partiql_tuple![("a", 1), ("c", 2)]),
            Value::from(partiql_bag!()),
            Value::from(partiql_bag!(1, 2, 3)),
            Value::from(partiql_bag!(3, 3, 3)),
        ];
        let expected_vals = vals.clone();
        vals.reverse();
        vals.sort();
        assert_eq!(expected_vals, vals);
    }

    #[test]
    fn partiql_value_arithmetic() {
        // Unary plus
        assert_eq!(Value::Missing, Value::Missing.positive());
        assert_eq!(Value::Null, Value::Null.positive());
        assert_eq!(Value::Integer(123), Value::Integer(123).positive());
        assert_eq!(Value::Decimal(dec!(3)), Value::Decimal(dec!(3)).positive());
        assert_eq!(Value::from(4.0), Value::from(4.0).positive());
        assert_eq!(Value::Missing, Value::from("foo").positive());

        // Negation
        assert_eq!(Value::Missing, -Value::Missing);
        assert_eq!(Value::Null, -Value::Null);
        assert_eq!(Value::Integer(-123), -Value::Integer(123));
        assert_eq!(Value::Decimal(dec!(-3)), -Value::Decimal(dec!(3)));
        assert_eq!(Value::from(-4.0), -Value::from(4.0));
        assert_eq!(Value::Missing, -Value::from("foo"));

        // Add
        assert_eq!(Value::Missing, Value::Missing + Value::Missing);
        assert_eq!(Value::Missing, Value::Missing + Value::Null);
        assert_eq!(Value::Missing, Value::Null + Value::Missing);
        assert_eq!(Value::Null, Value::Null + Value::Null);
        assert_eq!(Value::Missing, Value::Integer(1) + Value::from("a"));
        assert_eq!(Value::Integer(3), Value::Integer(1) + Value::Integer(2));
        assert_eq!(Value::from(4.0), Value::from(1.5) + Value::from(2.5));
        assert_eq!(
            Value::Decimal(dec!(3)),
            Value::Decimal(dec!(1)) + Value::Decimal(dec!(2))
        );
        assert_eq!(Value::from(3.5), Value::Integer(1) + Value::from(2.5));
        assert_eq!(Value::from(3.), Value::from(1.) + Value::from(2.));
        assert_eq!(
            Value::Decimal(dec!(3)),
            Value::Integer(1) + Value::Decimal(dec!(2))
        );
        assert_eq!(
            Value::Decimal(dec!(3)),
            Value::Decimal(dec!(1)) + Value::Integer(2)
        );
        assert_eq!(
            Value::Decimal(dec!(3)),
            Value::from(1.) + Value::Decimal(dec!(2))
        );
        assert_eq!(
            Value::Decimal(dec!(3)),
            Value::Decimal(dec!(1)) + Value::from(2.)
        );

        // Sub
        assert_eq!(Value::Missing, Value::Missing - Value::Missing);
        assert_eq!(Value::Missing, Value::Missing - Value::Null);
        assert_eq!(Value::Missing, Value::Null - Value::Missing);
        assert_eq!(Value::Null, Value::Null - Value::Null);
        assert_eq!(Value::Missing, Value::Integer(1) - Value::from("a"));
        assert_eq!(Value::Integer(-1), Value::Integer(1) - Value::Integer(2));
        assert_eq!(Value::from(-1.0), Value::from(1.5) - Value::from(2.5));
        assert_eq!(
            Value::Decimal(dec!(-1)),
            Value::Decimal(dec!(1)) - Value::Decimal(dec!(2))
        );
        assert_eq!(Value::from(-1.5), Value::Integer(1) - Value::from(2.5));
        assert_eq!(Value::from(-1.), Value::from(1.) - Value::from(2.));
        assert_eq!(
            Value::Decimal(dec!(-1)),
            Value::Integer(1) - Value::Decimal(dec!(2))
        );
        assert_eq!(
            Value::Decimal(dec!(-1)),
            Value::Decimal(dec!(1)) - Value::Integer(2)
        );
        assert_eq!(
            Value::Decimal(dec!(-1)),
            Value::from(1.) - Value::Decimal(dec!(2))
        );
        assert_eq!(
            Value::Decimal(dec!(-1)),
            Value::Decimal(dec!(1)) - Value::from(2.)
        );

        // Mul
        assert_eq!(Value::Missing, Value::Missing * Value::Missing);
        assert_eq!(Value::Missing, Value::Missing * Value::Null);
        assert_eq!(Value::Missing, Value::Null * Value::Missing);
        assert_eq!(Value::Null, Value::Null * Value::Null);
        assert_eq!(Value::Missing, Value::Integer(1) * Value::from("a"));
        assert_eq!(Value::Integer(2), Value::Integer(1) * Value::Integer(2));
        assert_eq!(Value::from(3.75), Value::from(1.5) * Value::from(2.5));
        assert_eq!(
            Value::Decimal(Decimal::new(2, 0)),
            Value::Decimal(dec!(1)) * Value::Decimal(dec!(2))
        );
        assert_eq!(Value::from(2.5), Value::Integer(1) * Value::from(2.5));
        assert_eq!(Value::from(2.), Value::from(1.) * Value::from(2.));
        assert_eq!(
            Value::Decimal(Decimal::new(2, 0)),
            Value::Integer(1) * Value::Decimal(dec!(2))
        );
        assert_eq!(
            Value::Decimal(Decimal::new(2, 0)),
            Value::Decimal(dec!(1)) * Value::Integer(2)
        );
        assert_eq!(
            Value::Decimal(Decimal::new(2, 0)),
            Value::from(1.) * Value::Decimal(dec!(2))
        );
        assert_eq!(
            Value::Decimal(Decimal::new(2, 0)),
            Value::Decimal(dec!(1)) * Value::from(2.)
        );

        // Div
        assert_eq!(Value::Missing, Value::Missing / Value::Missing);
        assert_eq!(Value::Missing, Value::Missing / Value::Null);
        assert_eq!(Value::Missing, Value::Null / Value::Missing);
        assert_eq!(Value::Null, Value::Null / Value::Null);
        assert_eq!(Value::Missing, Value::Integer(1) / Value::from("a"));
        assert_eq!(Value::Integer(0), Value::Integer(1) / Value::Integer(2));
        assert_eq!(Value::from(0.6), Value::from(1.5) / Value::from(2.5));
        assert_eq!(
            Value::Decimal(dec!(0.5)),
            Value::Decimal(dec!(1)) / Value::Decimal(dec!(2))
        );
        assert_eq!(Value::from(0.4), Value::Integer(1) / Value::from(2.5));
        assert_eq!(Value::from(0.5), Value::from(1.) / Value::from(2.));
        assert_eq!(
            Value::Decimal(dec!(0.5)),
            Value::Integer(1) / Value::Decimal(dec!(2))
        );
        assert_eq!(
            Value::Decimal(dec!(0.5)),
            Value::Decimal(dec!(1)) / Value::Integer(2)
        );
        assert_eq!(
            Value::Decimal(dec!(0.5)),
            Value::from(1.) / Value::Decimal(dec!(2))
        );
        assert_eq!(
            Value::Decimal(dec!(0.5)),
            Value::Decimal(dec!(1)) / Value::from(2.)
        );

        // Mod
        assert_eq!(Value::Missing, Value::Missing % Value::Missing);
        assert_eq!(Value::Missing, Value::Missing % Value::Null);
        assert_eq!(Value::Missing, Value::Null % Value::Missing);
        assert_eq!(Value::Null, Value::Null % Value::Null);
        assert_eq!(Value::Missing, Value::Integer(1) % Value::from("a"));
        assert_eq!(Value::Integer(1), Value::Integer(1) % Value::Integer(2));
        assert_eq!(Value::from(1.5), Value::from(1.5) % Value::from(2.5));
        assert_eq!(
            Value::Decimal(dec!(1)),
            Value::Decimal(dec!(1)) % Value::Decimal(dec!(2))
        );
        assert_eq!(Value::from(1.), Value::Integer(1) % Value::from(2.5));
        assert_eq!(Value::from(1.), Value::from(1.) % Value::from(2.));
        assert_eq!(
            Value::Decimal(dec!(1)),
            Value::Integer(1) % Value::Decimal(dec!(2))
        );
        assert_eq!(
            Value::Decimal(dec!(1)),
            Value::Decimal(dec!(1)) % Value::Integer(2)
        );
        assert_eq!(
            Value::Decimal(dec!(1)),
            Value::from(1.) % Value::Decimal(dec!(2))
        );
        assert_eq!(
            Value::Decimal(dec!(1)),
            Value::Decimal(dec!(1)) % Value::from(2.)
        );
    }

    #[test]
    fn partiql_value_logical() {
        // Unary NOT
        assert_eq!(Value::Null, !Value::Missing);
        assert_eq!(Value::Null, !Value::Null);
        assert_eq!(Value::from(true), !Value::from(false));
        assert_eq!(Value::from(false), !Value::from(true));
        assert_eq!(Value::Missing, !Value::from("foo"));

        // AND
        assert_eq!(
            Value::from(false),
            Value::from(false).and(&Value::from(true))
        );
        assert_eq!(
            Value::from(false),
            Value::from(true).and(&Value::from(false))
        );
        assert_eq!(Value::from(true), Value::from(true).and(&Value::from(true)));
        assert_eq!(
            Value::from(false),
            Value::from(false).and(&Value::from(false))
        );

        // false with null or missing => false
        assert_eq!(Value::from(false), Value::Null.and(&Value::from(false)));
        assert_eq!(Value::from(false), Value::from(false).and(&Value::Null));
        assert_eq!(Value::from(false), Value::Missing.and(&Value::from(false)));
        assert_eq!(Value::from(false), Value::from(false).and(&Value::Missing));

        // Null propagation => Null
        assert_eq!(Value::Null, Value::Null.and(&Value::Null));
        assert_eq!(Value::Null, Value::Missing.and(&Value::Missing));
        assert_eq!(Value::Null, Value::Null.and(&Value::Missing));
        assert_eq!(Value::Null, Value::Missing.and(&Value::Null));
        assert_eq!(Value::Null, Value::Null.and(&Value::from(true)));
        assert_eq!(Value::Null, Value::Missing.and(&Value::from(true)));
        assert_eq!(Value::Null, Value::from(true).and(&Value::Null));
        assert_eq!(Value::Null, Value::from(true).and(&Value::Missing));

        // Data type mismatch cases => Missing
        assert_eq!(Value::Missing, Value::from(123).and(&Value::from(false)));
        assert_eq!(Value::Missing, Value::from(false).and(&Value::from(123)));
        assert_eq!(Value::Missing, Value::from(123).and(&Value::from(true)));
        assert_eq!(Value::Missing, Value::from(true).and(&Value::from(123)));

        // OR
        assert_eq!(Value::from(true), Value::from(false).or(&Value::from(true)));
        assert_eq!(Value::from(true), Value::from(true).or(&Value::from(false)));
        assert_eq!(Value::from(true), Value::from(true).or(&Value::from(true)));
        assert_eq!(
            Value::from(false),
            Value::from(false).or(&Value::from(false))
        );

        // true with null or missing => true
        assert_eq!(Value::from(true), Value::Null.or(&Value::from(true)));
        assert_eq!(Value::from(true), Value::from(true).or(&Value::Null));
        assert_eq!(Value::from(true), Value::Missing.or(&Value::from(true)));
        assert_eq!(Value::from(true), Value::from(true).or(&Value::Missing));

        // Null propagation => Null
        assert_eq!(Value::Null, Value::Null.or(&Value::Null));
        assert_eq!(Value::Null, Value::Missing.or(&Value::Missing));
        assert_eq!(Value::Null, Value::Null.or(&Value::Missing));
        assert_eq!(Value::Null, Value::Missing.or(&Value::Null));
        assert_eq!(Value::Null, Value::Null.or(&Value::from(false)));
        assert_eq!(Value::Null, Value::Missing.or(&Value::from(false)));
        assert_eq!(Value::Null, Value::from(false).or(&Value::Null));
        assert_eq!(Value::Null, Value::from(false).or(&Value::Missing));

        // Data type mismatch cases => Missing
        assert_eq!(Value::Missing, Value::from(123).or(&Value::from(false)));
        assert_eq!(Value::Missing, Value::from(false).or(&Value::from(123)));
        assert_eq!(Value::Missing, Value::from(123).or(&Value::from(true)));
        assert_eq!(Value::Missing, Value::from(true).or(&Value::from(123)));
    }

    #[test]
    fn partiql_value_equality() {
        // TODO: many equality tests missing. Can use conformance tests to fill the gap or some other
        //  tests
        // Eq
        assert_eq!(
            Value::from(true),
            NullableEq::eq(&Value::from(true), &Value::from(true))
        );
        assert_eq!(
            Value::from(false),
            NullableEq::eq(&Value::from(true), &Value::from(false))
        );
        assert_eq!(
            Value::Null,
            NullableEq::eq(&Value::from(true), &Value::Null)
        );
        assert_eq!(
            Value::Null,
            NullableEq::eq(&Value::Null, &Value::from(true))
        );
        assert_eq!(
            Value::Missing,
            NullableEq::eq(&Value::from(true), &Value::Missing)
        );
        assert_eq!(
            Value::Missing,
            NullableEq::eq(&Value::Missing, &Value::from(true))
        );

        // different types result in boolean
        assert_eq!(
            Value::from(false),
            NullableEq::eq(&Value::from(true), &Value::from("abc"))
        );
        assert_eq!(
            Value::from(false),
            NullableEq::eq(&Value::from("abc"), &Value::from(true))
        );

        // Neq
        assert_eq!(
            Value::from(false),
            Value::from(true).neq(&Value::from(true))
        );
        assert_eq!(
            Value::from(true),
            Value::from(true).neq(&Value::from(false))
        );
        assert_eq!(Value::Null, Value::from(true).neq(&Value::Null));
        assert_eq!(Value::Null, Value::Null.neq(&Value::from(true)));
        assert_eq!(Value::Missing, Value::from(true).neq(&Value::Missing));
        assert_eq!(Value::Missing, Value::Missing.neq(&Value::from(true)));

        // different types result in boolean
        assert_eq!(
            Value::from(true),
            Value::from(true).neq(&Value::from("abc"))
        );
        assert_eq!(
            Value::from(true),
            Value::from("abc").neq(&Value::from(true))
        );
    }

    #[test]
    fn partiql_value_comparison() {
        // LT
        assert_eq!(
            Value::from(true),
            NullableOrd::lt(&Value::from(1), &Value::from(2))
        );
        assert_eq!(
            Value::from(false),
            NullableOrd::lt(&Value::from(1), &Value::from(0))
        );
        assert_eq!(
            Value::from(false),
            NullableOrd::lt(&Value::from(1), &Value::from(1))
        );

        // GT
        assert_eq!(
            Value::from(false),
            NullableOrd::gt(&Value::from(1), &Value::from(2))
        );
        assert_eq!(
            Value::from(true),
            NullableOrd::gt(&Value::from(1), &Value::from(0))
        );
        assert_eq!(
            Value::from(false),
            NullableOrd::gt(&Value::from(1), &Value::from(1))
        );

        // LTEQ
        assert_eq!(
            Value::from(true),
            NullableOrd::lteq(&Value::from(1), &Value::from(2))
        );
        assert_eq!(
            Value::from(false),
            NullableOrd::lteq(&Value::from(1), &Value::from(0))
        );
        assert_eq!(
            Value::from(true),
            NullableOrd::lteq(&Value::from(1), &Value::from(1))
        );

        // GTEQ
        assert_eq!(
            Value::from(false),
            NullableOrd::gteq(&Value::from(1), &Value::from(2))
        );
        assert_eq!(
            Value::from(true),
            NullableOrd::gteq(&Value::from(1), &Value::from(0))
        );
        assert_eq!(
            Value::from(true),
            NullableOrd::gteq(&Value::from(1), &Value::from(1))
        );

        // Missing propagation
        assert_eq!(
            Value::Missing,
            NullableOrd::lt(&Value::Missing, &Value::from(2))
        );
        assert_eq!(
            Value::Missing,
            NullableOrd::lt(&Value::from(1), &Value::Missing)
        );
        assert_eq!(
            Value::Missing,
            NullableOrd::lt(&Value::Null, &Value::Missing)
        );
        assert_eq!(
            Value::Missing,
            NullableOrd::lt(&Value::Missing, &Value::Null)
        );
        assert_eq!(
            Value::Missing,
            NullableOrd::gt(&Value::Missing, &Value::from(2))
        );
        assert_eq!(
            Value::Missing,
            NullableOrd::gt(&Value::from(1), &Value::Missing)
        );
        assert_eq!(
            Value::Missing,
            NullableOrd::gt(&Value::Null, &Value::Missing)
        );
        assert_eq!(
            Value::Missing,
            NullableOrd::gt(&Value::Missing, &Value::Null)
        );
        assert_eq!(
            Value::Missing,
            NullableOrd::lteq(&Value::Missing, &Value::from(2))
        );
        assert_eq!(
            Value::Missing,
            NullableOrd::lteq(&Value::from(1), &Value::Missing)
        );
        assert_eq!(
            Value::Missing,
            NullableOrd::lteq(&Value::Null, &Value::Missing)
        );
        assert_eq!(
            Value::Missing,
            NullableOrd::lteq(&Value::Missing, &Value::Null)
        );
        assert_eq!(
            Value::Missing,
            NullableOrd::gteq(&Value::Missing, &Value::from(2))
        );
        assert_eq!(
            Value::Missing,
            NullableOrd::gteq(&Value::from(1), &Value::Missing)
        );
        assert_eq!(
            Value::Missing,
            NullableOrd::gteq(&Value::Null, &Value::Missing)
        );
        assert_eq!(
            Value::Missing,
            NullableOrd::gteq(&Value::Missing, &Value::Null)
        );

        // Null propagation
        assert_eq!(Value::Null, NullableOrd::lt(&Value::Null, &Value::from(2)));
        assert_eq!(Value::Null, NullableOrd::lt(&Value::from(1), &Value::Null));
        assert_eq!(Value::Null, NullableOrd::gt(&Value::Null, &Value::from(2)));
        assert_eq!(Value::Null, NullableOrd::gt(&Value::from(1), &Value::Null));
        assert_eq!(
            Value::Null,
            NullableOrd::lteq(&Value::Null, &Value::from(2))
        );
        assert_eq!(
            Value::Null,
            NullableOrd::lteq(&Value::from(1), &Value::Null)
        );
        assert_eq!(
            Value::Null,
            NullableOrd::gteq(&Value::Null, &Value::from(2))
        );
        assert_eq!(
            Value::Null,
            NullableOrd::gteq(&Value::from(1), &Value::Null)
        );

        // Data type mismatch
        assert_eq!(
            Value::Missing,
            NullableOrd::lt(&Value::from(1), &Value::from("abc"))
        );
        assert_eq!(
            Value::Missing,
            NullableOrd::lt(&Value::from("abc"), &Value::from(1))
        );
        assert_eq!(
            Value::Missing,
            NullableOrd::gt(&Value::from(1), &Value::from("abc"))
        );
        assert_eq!(
            Value::Missing,
            NullableOrd::gt(&Value::from("abc"), &Value::from(1))
        );
        assert_eq!(
            Value::Missing,
            NullableOrd::lteq(&Value::from(1), &Value::from("abc"))
        );
        assert_eq!(
            Value::Missing,
            NullableOrd::lteq(&Value::from("abc"), &Value::from(1))
        );
        assert_eq!(
            Value::Missing,
            NullableOrd::gteq(&Value::from(1), &Value::from("abc"))
        );
        assert_eq!(
            Value::Missing,
            NullableOrd::gteq(&Value::from("abc"), &Value::from(1))
        );

        // Numeric type comparison
        // LT
        assert_eq!(
            Value::from(true),
            NullableOrd::lt(&Value::from(1), &Value::from(2.0))
        );
        assert_eq!(
            Value::from(true),
            NullableOrd::lt(&Value::from(1), &Value::Decimal(dec!(2.0)))
        );
        assert_eq!(
            Value::from(true),
            NullableOrd::lt(&Value::from(1.0), &Value::from(2))
        );
        assert_eq!(
            Value::from(true),
            NullableOrd::lt(&Value::from(1.0), &Value::Decimal(dec!(2.0)))
        );
        assert_eq!(
            Value::from(true),
            NullableOrd::lt(&Value::Decimal(dec!(1.0)), &Value::from(2))
        );
        assert_eq!(
            Value::from(true),
            NullableOrd::lt(&Value::Decimal(dec!(1.0)), &Value::from(2.))
        );

        // GT
        assert_eq!(
            Value::from(false),
            NullableOrd::gt(&Value::from(1), &Value::from(2.0))
        );
        assert_eq!(
            Value::from(false),
            NullableOrd::gt(&Value::from(1), &Value::Decimal(dec!(2.0)))
        );
        assert_eq!(
            Value::from(false),
            NullableOrd::gt(&Value::from(1.0), &Value::from(2))
        );
        assert_eq!(
            Value::from(false),
            NullableOrd::gt(&Value::from(1.0), &Value::Decimal(dec!(2.0)))
        );
        assert_eq!(
            Value::from(false),
            NullableOrd::gt(&Value::Decimal(dec!(1.0)), &Value::from(2))
        );
        assert_eq!(
            Value::from(false),
            NullableOrd::gt(&Value::Decimal(dec!(1.0)), &Value::from(2.))
        );

        // LTEQ
        assert_eq!(
            Value::from(true),
            NullableOrd::lteq(&Value::from(1), &Value::from(2.0))
        );
        assert_eq!(
            Value::from(true),
            NullableOrd::lteq(&Value::from(1), &Value::Decimal(dec!(2.0)))
        );
        assert_eq!(
            Value::from(true),
            NullableOrd::lteq(&Value::from(1.0), &Value::from(2))
        );
        assert_eq!(
            Value::from(true),
            NullableOrd::lteq(&Value::from(1.0), &Value::Decimal(dec!(2.0)))
        );
        assert_eq!(
            Value::from(true),
            NullableOrd::lteq(&Value::Decimal(dec!(1.0)), &Value::from(2))
        );
        assert_eq!(
            Value::from(true),
            NullableOrd::lteq(&Value::Decimal(dec!(1.0)), &Value::from(2.))
        );

        // GTEQ
        assert_eq!(
            Value::from(false),
            NullableOrd::gteq(&Value::from(1), &Value::from(2.0))
        );
        assert_eq!(
            Value::from(false),
            NullableOrd::gteq(&Value::from(1), &Value::Decimal(dec!(2.0)))
        );
        assert_eq!(
            Value::from(false),
            NullableOrd::gteq(&Value::from(1.0), &Value::from(2))
        );
        assert_eq!(
            Value::from(false),
            NullableOrd::gteq(&Value::from(1.0), &Value::Decimal(dec!(2.0)))
        );
        assert_eq!(
            Value::from(false),
            NullableOrd::gteq(&Value::Decimal(dec!(1.0)), &Value::from(2))
        );
        assert_eq!(
            Value::from(false),
            NullableOrd::gteq(&Value::Decimal(dec!(1.0)), &Value::from(2.))
        );
    }
}
