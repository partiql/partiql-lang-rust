use itertools::Itertools;
use ordered_float::OrderedFloat;
use std::cmp::Ordering;

use std::collections::HashMap;
use std::fmt::{Debug, Formatter};
use std::hash::{Hash, Hasher};
use std::{ops, vec};

use rust_decimal::prelude::FromPrimitive;
use rust_decimal::{Decimal as RustDecimal, Decimal};

#[derive(Debug)]
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

fn coerce_int_or_real_to_decimal(value: &Value) -> Value {
    match value {
        Value::Integer(int_value) => Value::Decimal(rust_decimal::Decimal::from(*int_value)),
        Value::Real(real_value) => {
            if !real_value.is_finite() {
                Value::Missing
            } else {
                match Decimal::from_f64(real_value.0) {
                    Some(d_from_r) => Value::Decimal(rust_decimal::Decimal::from(d_from_r)),
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
            let fresh_key = "_1".to_string(); // TODO don't hard-code 'fresh' keys
            Tuple(HashMap::from([(fresh_key, self)]))
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
            Value::Tuple(t) => t.fmt(f),
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
                        Some(l_d) => l_d.cmp(&r),
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
            (_, Value::Integer(_)) => Ordering::Greater,
            (_, Value::Real(_)) => Ordering::Greater,
            (_, Value::Decimal(_)) => Ordering::Greater,

            (Value::String(l), Value::String(r)) => l.cmp(r),
            (_, Value::String(_)) => Ordering::Greater,

            (Value::Blob(l), Value::Blob(r)) => l.cmp(r),
            (_, Value::Blob(_)) => Ordering::Greater,

            (Value::List(l), Value::List(r)) => l.cmp(r),
            (_, Value::List(_)) => Ordering::Greater,

            (Value::Tuple(l), Value::Tuple(r)) => l.cmp(r),
            (_, Value::Tuple(_)) => Ordering::Greater,

            (Value::Bag(l), Value::Bag(r)) => l.cmp(r),
            (_, Value::Bag(_)) => Ordering::Greater,
        }
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

// TODO replace HashMap to support multi key
#[derive(Default, Eq, Clone)]
pub struct Tuple(pub HashMap<String, Value>);

impl<const N: usize, T> From<[(&str, T); N]> for Tuple
where
    T: Into<Value>,
{
    #[inline]
    fn from(arr: [(&str, T); N]) -> Self {
        Tuple(HashMap::from_iter(
            arr.into_iter().map(|(k, v)| (k.to_string(), v.into())),
        ))
    }
}

#[macro_export]
macro_rules! partiql_tuple {
    () => (
         Tuple::from(vec![])
    );
    ($(($x:expr, $y:expr)),+ $(,)?) => (
        Tuple::from([$(($x, Value::from($y))),+])
    );
}

impl Debug for Tuple {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        let mut dbg = f.debug_struct("");
        for (k, v) in &self.0 {
            dbg.field(k, v);
        }
        dbg.finish()
    }
}

impl PartialOrd for Tuple {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}

impl Ord for Tuple {
    fn cmp(&self, other: &Self) -> Ordering {
        let mut l = self.0.keys().sorted();
        let mut r = other.0.keys().sorted();

        loop {
            match (l.next(), r.next()) {
                (None, None) => return Ordering::Equal,
                (Some(_), None) => return Ordering::Greater,
                (None, Some(_)) => return Ordering::Less,
                (Some(lv), Some(rv)) => match lv.cmp(rv) {
                    Ordering::Less => return Ordering::Less,
                    Ordering::Greater => return Ordering::Greater,
                    Ordering::Equal => match self.0[lv].cmp(&other.0[rv]) {
                        Ordering::Less => return Ordering::Less,
                        Ordering::Greater => return Ordering::Greater,
                        Ordering::Equal => continue,
                    },
                },
            }
        }
    }
}

impl PartialEq for Tuple {
    fn eq(&self, other: &Self) -> bool {
        self.0 == other.0
    }
}

impl Hash for Tuple {
    fn hash<H: Hasher>(&self, state: &mut H) {
        let keys = self.0.keys().into_iter().sorted();
        for k in keys {
            k.hash(state);
            self.0[k].hash(state);
        }
    }
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
    fn partiql_value_arithmetic() {
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
}
