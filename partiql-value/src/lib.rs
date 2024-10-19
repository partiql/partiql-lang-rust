#![deny(rust_2018_idioms)]
#![deny(clippy::all)]

use ordered_float::OrderedFloat;
use std::cmp::Ordering;

use std::borrow::Cow;

use std::fmt::{Debug, Display, Formatter};
use std::hash::Hash;

use std::iter::Once;

use rust_decimal::prelude::FromPrimitive;
use rust_decimal::Decimal as RustDecimal;

mod bag;
pub mod comparison;
mod datetime;
mod list;
mod pretty;
mod tuple;
mod util;
mod value;

pub use bag::*;
pub use comparison::*;
pub use datetime::*;
pub use list::*;
pub use pretty::*;
pub use tuple::*;
pub use value::*;

use partiql_common::pretty::ToPretty;
#[cfg(feature = "serde")]
use serde::{Deserialize, Serialize};

#[derive(Clone, Hash, Debug, Eq, PartialEq)]
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
pub enum BindingsName<'s> {
    CaseSensitive(Cow<'s, str>),
    CaseInsensitive(Cow<'s, str>),
}

#[derive(Debug, Clone)]
pub enum BindingIter<'a> {
    Tuple(PairsIter<'a>),
    Single(Once<&'a Value>),
    Empty,
}

impl<'a> Iterator for BindingIter<'a> {
    type Item = (Option<&'a String>, &'a Value);

    #[inline]
    fn next(&mut self) -> Option<Self::Item> {
        match self {
            BindingIter::Tuple(t) => t.next().map(|(k, v)| (Some(k), v)),
            BindingIter::Single(single) => single.next().map(|v| (None, v)),
            BindingIter::Empty => None,
        }
    }

    #[inline]
    fn size_hint(&self) -> (usize, Option<usize>) {
        match self {
            BindingIter::Tuple(t) => t.size_hint(),
            BindingIter::Single(_single) => (1, Some(1)),
            BindingIter::Empty => (0, Some(0)),
        }
    }
}

#[derive(Debug)]
pub enum BindingIntoIter {
    Tuple(PairsIntoIter),
    Single(Once<Value>),
    Empty,
}

impl Iterator for BindingIntoIter {
    type Item = (Option<String>, Value);

    #[inline]
    fn next(&mut self) -> Option<Self::Item> {
        match self {
            BindingIntoIter::Tuple(t) => t.next().map(|(k, v)| (Some(k), v)),
            BindingIntoIter::Single(single) => single.next().map(|v| (None, v)),
            BindingIntoIter::Empty => None,
        }
    }

    #[inline]
    fn size_hint(&self) -> (usize, Option<usize>) {
        match self {
            BindingIntoIter::Tuple(t) => t.size_hint(),
            BindingIntoIter::Single(_single) => (1, Some(1)),
            BindingIntoIter::Empty => (0, Some(0)),
        }
    }
}
impl Display for Value {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        match self.to_pretty_string(f.width().unwrap_or(80)) {
            Ok(pretty) => f.write_str(&pretty),
            Err(_) => f.write_str("<internal value error occurred>"),
        }
    }
}

impl Debug for Value {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        match self {
            Value::Null => write!(f, "NULL"),
            Value::Missing => write!(f, "MISSING"),
            Value::Boolean(b) => write!(f, "{b}"),
            Value::Integer(i) => write!(f, "{i}"),
            Value::Real(r) => write!(f, "{}", r.0),
            Value::Decimal(d) => write!(f, "{d}"),
            Value::String(s) => write!(f, "'{s}'"),
            Value::Blob(s) => write!(f, "'{s:?}'"),
            Value::DateTime(t) => t.fmt(f),
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

/// A wrapper on [`T`] that specifies if a null or missing value should be ordered before
/// ([`NULLS_FIRST`] is true) or after ([`NULLS_FIRST`] is false) other values.
#[derive(Eq, PartialEq)]
pub struct NullSortedValue<'a, const NULLS_FIRST: bool, T>(pub &'a T);

impl<'a, const NULLS_FIRST: bool, T> PartialOrd for NullSortedValue<'a, NULLS_FIRST, T>
where
    T: PartialOrd,
    NullSortedValue<'a, NULLS_FIRST, T>: Ord,
{
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}

impl<const NULLS_FIRST: bool> Ord for NullSortedValue<'_, NULLS_FIRST, Value> {
    fn cmp(&self, other: &Self) -> Ordering {
        let wrap_list = NullSortedValue::<{ NULLS_FIRST }, List>;
        let wrap_tuple = NullSortedValue::<{ NULLS_FIRST }, Tuple>;
        let wrap_bag = NullSortedValue::<{ NULLS_FIRST }, Bag>;
        let null_cond = |order: Ordering| {
            if NULLS_FIRST {
                order
            } else {
                order.reverse()
            }
        };

        match (self.0, other.0) {
            (Value::Null, Value::Null) => Ordering::Equal,
            (Value::Missing, Value::Null) => Ordering::Equal,

            (Value::Null, Value::Missing) => Ordering::Equal,
            (Value::Null, _) => null_cond(Ordering::Less),
            (_, Value::Null) => null_cond(Ordering::Greater),

            (Value::Missing, Value::Missing) => Ordering::Equal,
            (Value::Missing, _) => null_cond(Ordering::Less),
            (_, Value::Missing) => null_cond(Ordering::Greater),

            (Value::List(l), Value::List(r)) => wrap_list(l.as_ref()).cmp(&wrap_list(r.as_ref())),

            (Value::Tuple(l), Value::Tuple(r)) => {
                wrap_tuple(l.as_ref()).cmp(&wrap_tuple(r.as_ref()))
            }

            (Value::Bag(l), Value::Bag(r)) => wrap_bag(l.as_ref()).cmp(&wrap_bag(r.as_ref())),
            (l, r) => l.cmp(r),
        }
    }
}

/// Implementation of spec's `order-by less-than` assuming nulls first.
/// TODO: more tests for Ord on Value
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
            (Value::Decimal(l), Value::Integer(r)) => l.as_ref().cmp(&RustDecimal::from(*r)),
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
                        Some(r_d) => l.as_ref().cmp(&r_d),
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

            (Value::DateTime(l), Value::DateTime(r)) => l.cmp(r),
            (Value::DateTime(_), _) => Ordering::Less,
            (_, Value::DateTime(_)) => Ordering::Greater,

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

impl<T> From<&T> for Value
where
    T: Copy,
    Value: From<T>,
{
    #[inline]
    fn from(t: &T) -> Self {
        Value::from(*t)
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

impl From<i32> for Value {
    #[inline]
    fn from(n: i32) -> Self {
        i64::from(n).into()
    }
}

impl From<i16> for Value {
    #[inline]
    fn from(n: i16) -> Self {
        i64::from(n).into()
    }
}

impl From<i8> for Value {
    #[inline]
    fn from(n: i8) -> Self {
        i64::from(n).into()
    }
}

impl From<usize> for Value {
    #[inline]
    fn from(n: usize) -> Self {
        // TODO overflow to bigint/decimal
        Value::Integer(n as i64)
    }
}

impl From<u8> for Value {
    #[inline]
    fn from(n: u8) -> Self {
        (n as usize).into()
    }
}

impl From<u16> for Value {
    #[inline]
    fn from(n: u16) -> Self {
        (n as usize).into()
    }
}

impl From<u32> for Value {
    #[inline]
    fn from(n: u32) -> Self {
        (n as usize).into()
    }
}

impl From<u64> for Value {
    #[inline]
    fn from(n: u64) -> Self {
        (n as usize).into()
    }
}

impl From<u128> for Value {
    #[inline]
    fn from(n: u128) -> Self {
        (n as usize).into()
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
        Value::Decimal(Box::new(d))
    }
}

impl From<DateTime> for Value {
    #[inline]
    fn from(t: DateTime) -> Self {
        Value::DateTime(Box::new(t))
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

#[cfg(test)]
mod tests {
    use super::*;
    use crate::comparison::{EqualityValue, NullableEq, NullableOrd};
    use rust_decimal_macros::dec;
    use std::borrow::Cow;
    use std::cell::RefCell;
    use std::collections::HashSet;
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
        println!("Cow<&Tuple> size: {}", mem::size_of::<Cow<'_, &Tuple>>());
        println!("Value size: {}", mem::size_of::<Value>());
        println!("Option<Value> size: {}", mem::size_of::<Option<Value>>());
        println!(
            "Option<Option<Value>> size: {}",
            mem::size_of::<Option<Option<Value>>>()
        );
        println!("Cow<'_, Value> size: {}", mem::size_of::<Cow<'_, Value>>());
        println!("Cow<&Value> size: {}", mem::size_of::<Cow<'_, &Value>>());

        assert_eq!(mem::size_of::<Value>(), 16);
        assert_eq!(mem::size_of::<Option<Option<Value>>>(), 16);
    }

    #[test]
    fn macro_rules_tests() {
        println!("partiql_list:{:?}", list!());
        println!("partiql_list:{:?}", list![10, 10]);
        println!("partiql_list:{:?}", list!(5; 3));
        println!("partiql_bag:{:?}", bag!());
        println!("partiql_bag:{:?}", bag![10, 10]);
        println!("partiql_bag:{:?}", bag!(5; 3));
        println!("partiql_tuple:{:?}", tuple![]);
        println!("partiql_tuple:{:?}", tuple![("a", 1), ("b", 2)]);
    }

    #[test]
    fn iterators() {
        let bag: Bag = [1, 10, 3, 4].iter().collect();
        assert_eq!(bag.len(), 4);
        let max = bag
            .iter()
            .fold(Value::Integer(0), |x, y| if y > &x { y.clone() } else { x });
        assert_eq!(max, Value::Integer(10));
        let _bref = Value::from(bag).as_bag_ref();

        let list: List = [1, 2, 3, -4].iter().collect();
        assert_eq!(list.len(), 4);
        let max = list
            .iter()
            .fold(Value::Integer(0), |x, y| if y > &x { y.clone() } else { x });
        assert_eq!(max, Value::Integer(3));
        let _lref = Value::from(list).as_bag_ref();

        let bag: Bag = [Value::from(5), "text".into(), true.into()]
            .iter()
            .map(Clone::clone)
            .collect();
        assert_eq!(bag.len(), 3);
        let max = bag
            .iter()
            .fold(Value::Integer(0), |x, y| if y > &x { y.clone() } else { x });
        assert_eq!(max, Value::String(Box::new("text".to_string())));

        let list: List = [Value::from(5), Value::from(bag.clone()), true.into()]
            .iter()
            .map(Clone::clone)
            .collect();
        assert_eq!(list.len(), 3);
        let max = list
            .iter()
            .fold(Value::Integer(0), |x, y| if y > &x { y.clone() } else { x });
        assert_eq!(max, Value::from(bag.clone()));

        let tuple: Tuple = [
            ("list", Value::from(list.clone())),
            ("bag", Value::from(bag.clone())),
        ]
        .iter()
        .cloned()
        .collect();

        let mut pairs = tuple.pairs();
        let list_val = Value::from(list);
        assert_eq!(pairs.next(), Some((&"list".to_string(), &list_val)));
        let bag_val = Value::from(bag);
        assert_eq!(pairs.next(), Some((&"bag".to_string(), &bag_val)));
        assert_eq!(pairs.next(), None);
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
            Value::Decimal(Box::new(dec!(1.23456))),
            Value::from(138u8),
            Value::from(1348u16),
            Value::from(13849u32),
            Value::from(123_456),
            Value::from(1_384_449_u64),
            Value::from(138_444_339_u128),
            Value::from(f64::INFINITY),
            Value::from(""),
            Value::from("abc"),
            Value::Blob(Box::default()),
            Value::Blob(Box::new(vec![1, 2, 3])),
            Value::from(list!()),
            Value::from(list!(1, 2, 3)),
            Value::from(list!(1, 2, 3, 4, 5)),
            Value::from(tuple!()),
            Value::from(tuple![("a", 1), ("b", 2)]),
            Value::from(tuple![("a", 1), ("b", 3)]),
            Value::from(tuple![("a", 1), ("c", 2)]),
            Value::from(bag!()),
            Value::from(bag!(1, 2, 3)),
            Value::from(bag!(3, 3, 3)),
        ];
        let expected_vals = vals.clone();
        vals.reverse();
        vals.sort();
        assert_eq!(expected_vals, vals);
    }

    #[test]
    fn partiql_value_arithmetic() {
        // Unary plus
        assert_eq!(&Value::Missing, &Value::Missing.positive());
        assert_eq!(&Value::Null, &Value::Null.positive());
        assert_eq!(&Value::Integer(123), &Value::Integer(123).positive());
        assert_eq!(
            &Value::Decimal(Box::new(dec!(3))),
            &Value::Decimal(Box::new(dec!(3))).positive()
        );
        assert_eq!(&Value::from(4.0), &Value::from(4.0).positive());
        assert_eq!(&Value::Missing, &Value::from("foo").positive());

        // Negation
        assert_eq!(Value::Missing, -&Value::Missing);
        assert_eq!(Value::Null, -&Value::Null);
        assert_eq!(Value::Integer(-123), -&Value::Integer(123));
        assert_eq!(
            Value::Decimal(Box::new(dec!(-3))),
            -&Value::Decimal(Box::new(dec!(3)))
        );
        assert_eq!(Value::from(-4.0), -&Value::from(4.0));
        assert_eq!(Value::Missing, -&Value::from("foo"));

        // Add
        assert_eq!(Value::Missing, &Value::Missing + &Value::Missing);
        assert_eq!(Value::Missing, &Value::Missing + &Value::Null);
        assert_eq!(Value::Missing, &Value::Null + &Value::Missing);
        assert_eq!(Value::Null, &Value::Null + &Value::Null);
        assert_eq!(Value::Missing, &Value::Integer(1) + &Value::from("a"));
        assert_eq!(Value::Integer(3), &Value::Integer(1) + &Value::Integer(2));
        assert_eq!(Value::from(4.0), &Value::from(1.5) + &Value::from(2.5));
        assert_eq!(
            Value::Decimal(Box::new(dec!(3))),
            &Value::Decimal(Box::new(dec!(1))) + &Value::Decimal(Box::new(dec!(2)))
        );
        assert_eq!(Value::from(3.5), &Value::Integer(1) + &Value::from(2.5));
        assert_eq!(Value::from(3.), &Value::from(1.) + &Value::from(2.));
        assert_eq!(
            Value::Decimal(Box::new(dec!(3))),
            &Value::Integer(1) + &Value::Decimal(Box::new(dec!(2)))
        );
        assert_eq!(
            Value::Decimal(Box::new(dec!(3))),
            &Value::Decimal(Box::new(dec!(1))) + &Value::Integer(2)
        );
        assert_eq!(
            Value::Decimal(Box::new(dec!(3))),
            &Value::from(1.) + &Value::Decimal(Box::new(dec!(2)))
        );
        assert_eq!(
            Value::Decimal(Box::new(dec!(3))),
            &Value::Decimal(Box::new(dec!(1))) + &Value::from(2.)
        );

        // Sub
        assert_eq!(Value::Missing, &Value::Missing - &Value::Missing);
        assert_eq!(Value::Missing, &Value::Missing - &Value::Null);
        assert_eq!(Value::Missing, &Value::Null - &Value::Missing);
        assert_eq!(Value::Null, &Value::Null - &Value::Null);
        assert_eq!(Value::Missing, &Value::Integer(1) - &Value::from("a"));
        assert_eq!(Value::Integer(-1), &Value::Integer(1) - &Value::Integer(2));
        assert_eq!(Value::from(-1.0), &Value::from(1.5) - &Value::from(2.5));
        assert_eq!(
            Value::Decimal(Box::new(dec!(-1))),
            &Value::Decimal(Box::new(dec!(1))) - &Value::Decimal(Box::new(dec!(2)))
        );
        assert_eq!(Value::from(-1.5), &Value::Integer(1) - &Value::from(2.5));
        assert_eq!(Value::from(-1.), &Value::from(1.) - &Value::from(2.));
        assert_eq!(
            Value::Decimal(Box::new(dec!(-1))),
            &Value::Integer(1) - &Value::Decimal(Box::new(dec!(2)))
        );
        assert_eq!(
            Value::Decimal(Box::new(dec!(-1))),
            &Value::Decimal(Box::new(dec!(1))) - &Value::Integer(2)
        );
        assert_eq!(
            Value::Decimal(Box::new(dec!(-1))),
            &Value::from(1.) - &Value::Decimal(Box::new(dec!(2)))
        );
        assert_eq!(
            Value::Decimal(Box::new(dec!(-1))),
            &Value::Decimal(Box::new(dec!(1))) - &Value::from(2.)
        );

        // Mul
        assert_eq!(Value::Missing, &Value::Missing * &Value::Missing);
        assert_eq!(Value::Missing, &Value::Missing * &Value::Null);
        assert_eq!(Value::Missing, &Value::Null * &Value::Missing);
        assert_eq!(Value::Null, &Value::Null * &Value::Null);
        assert_eq!(Value::Missing, &Value::Integer(1) * &Value::from("a"));
        assert_eq!(Value::Integer(2), &Value::Integer(1) * &Value::Integer(2));
        assert_eq!(Value::from(3.75), &Value::from(1.5) * &Value::from(2.5));
        assert_eq!(
            Value::from(RustDecimal::new(2, 0)),
            &Value::Decimal(Box::new(dec!(1))) * &Value::from(dec!(2))
        );
        assert_eq!(Value::from(2.5), &Value::Integer(1) * &Value::from(2.5));
        assert_eq!(Value::from(2.), &Value::from(1.) * &Value::from(2.));
        assert_eq!(
            Value::from(RustDecimal::new(2, 0)),
            &Value::Integer(1) * &Value::Decimal(Box::new(dec!(2)))
        );
        assert_eq!(
            Value::from(RustDecimal::new(2, 0)),
            &Value::Decimal(Box::new(dec!(1))) * &Value::Integer(2)
        );
        assert_eq!(
            Value::from(RustDecimal::new(2, 0)),
            &Value::from(1.) * &Value::Decimal(Box::new(dec!(2)))
        );
        assert_eq!(
            Value::from(RustDecimal::new(2, 0)),
            &Value::Decimal(Box::new(dec!(1))) * &Value::from(2.)
        );

        // Div
        assert_eq!(Value::Missing, &Value::Missing / &Value::Missing);
        assert_eq!(Value::Missing, &Value::Missing / &Value::Null);
        assert_eq!(Value::Missing, &Value::Null / &Value::Missing);
        assert_eq!(Value::Null, &Value::Null / &Value::Null);
        assert_eq!(Value::Missing, &Value::Integer(1) / &Value::from("a"));
        assert_eq!(Value::Integer(0), &Value::Integer(1) / &Value::Integer(2));
        assert_eq!(Value::from(0.6), &Value::from(1.5) / &Value::from(2.5));
        assert_eq!(
            Value::Decimal(Box::new(dec!(0.5))),
            &Value::Decimal(Box::new(dec!(1))) / &Value::from(dec!(2))
        );
        assert_eq!(Value::from(0.4), &Value::Integer(1) / &Value::from(2.5));
        assert_eq!(Value::from(0.5), &Value::from(1.) / &Value::from(2.));
        assert_eq!(
            Value::Decimal(Box::new(dec!(0.5))),
            &Value::Integer(1) / &Value::Decimal(Box::new(dec!(2)))
        );
        assert_eq!(
            Value::Decimal(Box::new(dec!(0.5))),
            &Value::Decimal(Box::new(dec!(1))) / &Value::Integer(2)
        );
        assert_eq!(
            Value::Decimal(Box::new(dec!(0.5))),
            &Value::from(1.) / &Value::Decimal(Box::new(dec!(2)))
        );
        assert_eq!(
            Value::Decimal(Box::new(dec!(0.5))),
            &Value::Decimal(Box::new(dec!(1))) / &Value::from(2.)
        );

        // Mod
        assert_eq!(Value::Missing, &Value::Missing % &Value::Missing);
        assert_eq!(Value::Missing, &Value::Missing % &Value::Null);
        assert_eq!(Value::Missing, &Value::Null % &Value::Missing);
        assert_eq!(Value::Null, &Value::Null % &Value::Null);
        assert_eq!(Value::Missing, &Value::Integer(1) % &Value::from("a"));
        assert_eq!(Value::Integer(1), &Value::Integer(1) % &Value::Integer(2));
        assert_eq!(Value::from(1.5), &Value::from(1.5) % &Value::from(2.5));
        assert_eq!(
            Value::Decimal(Box::new(dec!(1))),
            &Value::Decimal(Box::new(dec!(1))) % &Value::from(dec!(2))
        );
        assert_eq!(Value::from(1.), &Value::Integer(1) % &Value::from(2.5));
        assert_eq!(Value::from(1.), &Value::from(1.) % &Value::from(2.));
        assert_eq!(
            Value::Decimal(Box::new(dec!(1))),
            &Value::Integer(1) % &Value::Decimal(Box::new(dec!(2)))
        );
        assert_eq!(
            Value::Decimal(Box::new(dec!(1))),
            &Value::Decimal(Box::new(dec!(1))) % &Value::Integer(2)
        );
        assert_eq!(
            Value::Decimal(Box::new(dec!(1))),
            &Value::from(1.) % &Value::Decimal(Box::new(dec!(2)))
        );
        assert_eq!(
            Value::Decimal(Box::new(dec!(1))),
            &Value::Decimal(Box::new(dec!(1))) % &Value::from(2.)
        );
    }

    #[test]
    fn partiql_value_logical() {
        // Unary NOT
        assert_eq!(Value::Null, !&Value::Missing);
        assert_eq!(Value::Null, !&Value::Null);
        assert_eq!(Value::from(true), !&Value::from(false));
        assert_eq!(Value::from(false), !&Value::from(true));
        assert_eq!(Value::Missing, !&Value::from("foo"));

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

        fn nullable_eq(lhs: Value, rhs: Value) -> Value {
            let wrap = EqualityValue::<false, Value>;
            let lhs = wrap(&lhs);
            let rhs = wrap(&rhs);
            NullableEq::eq(&lhs, &rhs)
        }

        fn nullable_neq(lhs: Value, rhs: Value) -> Value {
            let wrap = EqualityValue::<false, Value>;
            let lhs = wrap(&lhs);
            let rhs = wrap(&rhs);
            NullableEq::neq(&lhs, &rhs)
        }

        // Eq
        assert_eq!(
            Value::from(true),
            nullable_eq(Value::from(true), Value::from(true))
        );
        assert_eq!(
            Value::from(false),
            nullable_eq(Value::from(true), Value::from(false))
        );
        // Container examples from spec section 7.1.1 https://partiql.org/assets/PartiQL-Specification.pdf#subsubsection.7.1.1
        assert_eq!(
            Value::from(true),
            nullable_eq(Value::from(bag![3, 2, 4, 2]), Value::from(bag![2, 2, 3, 4]))
        );
        assert_eq!(
            Value::from(true),
            nullable_eq(
                Value::from(tuple![("a", 1), ("b", 2)]),
                Value::from(tuple![("a", 1), ("b", 2)])
            )
        );
        assert_eq!(
            Value::from(true),
            nullable_eq(
                Value::from(tuple![("a", list![0, 1]), ("b", 2)]),
                Value::from(tuple![("a", list![0, 1]), ("b", 2)])
            )
        );
        assert_eq!(
            Value::from(false),
            nullable_eq(Value::from(bag![3, 4, 2]), Value::from(bag![2, 2, 3, 4]))
        );
        assert_eq!(
            Value::from(true),
            nullable_eq(Value::from(list![1, 2]), Value::from(list![1e0, 2.0]))
        );
        assert_eq!(
            Value::from(false),
            nullable_eq(Value::from(list![1, 2]), Value::from(list![2.0, 1e0]))
        );
        assert_eq!(
            Value::from(true),
            nullable_eq(Value::from(bag![1, 2]), Value::from(bag![2.0, 1e0]))
        );
        assert_eq!(
            Value::from(false),
            nullable_eq(
                Value::from(tuple![("a", 1), ("b", 2)]),
                Value::from(tuple![("a", 1)])
            )
        );
        assert_eq!(
            Value::from(false),
            nullable_eq(
                Value::from(tuple![("a", list![0, 1]), ("b", 2)]),
                Value::from(tuple![("a", list![0, 1, 2]), ("b", 2)])
            )
        );
        assert_eq!(
            Value::from(false),
            nullable_eq(
                Value::from(tuple![("a", 1), ("b", 2)]),
                Value::from(tuple![("a", 1), ("b", Value::Null)])
            )
        );
        assert_eq!(
            Value::from(false),
            nullable_eq(
                Value::from(tuple![("a", list![0, 1]), ("b", 2)]),
                Value::from(tuple![("a", list![Value::Null, 1]), ("b", 2)])
            )
        );
        assert_eq!(Value::Null, nullable_eq(Value::from(true), Value::Null));
        assert_eq!(Value::Null, nullable_eq(Value::Null, Value::from(true)));
        assert_eq!(
            Value::Missing,
            nullable_eq(Value::from(true), Value::Missing)
        );
        assert_eq!(
            Value::Missing,
            nullable_eq(Value::Missing, Value::from(true))
        );

        // different, comparable types result in boolean true
        assert_eq!(
            Value::from(true),
            nullable_eq(Value::from(1), Value::from(1.0))
        );
        assert_eq!(
            Value::from(true),
            nullable_eq(Value::from(1.0), Value::from(1))
        );
        assert_eq!(
            Value::from(true),
            nullable_eq(Value::from(1), Value::from(dec!(1.0)))
        );
        assert_eq!(
            Value::from(true),
            nullable_eq(Value::from(dec!(1.0)), Value::from(1))
        );
        assert_eq!(
            Value::from(true),
            nullable_eq(Value::from(1.0), Value::from(dec!(1.0)))
        );
        assert_eq!(
            Value::from(true),
            nullable_eq(Value::from(dec!(1.0)), Value::from(1.0))
        );
        // different, comparable types result in boolean false
        assert_eq!(
            Value::from(false),
            nullable_eq(Value::from(1), Value::from(2.0))
        );
        assert_eq!(
            Value::from(false),
            nullable_eq(Value::from(1.0), Value::from(2))
        );
        assert_eq!(
            Value::from(false),
            nullable_eq(Value::from(1), Value::from(dec!(2.0)))
        );
        assert_eq!(
            Value::from(false),
            nullable_eq(Value::from(dec!(1.0)), Value::from(2))
        );
        assert_eq!(
            Value::from(false),
            nullable_eq(Value::from(1.0), Value::from(dec!(2.0)))
        );
        assert_eq!(
            Value::from(false),
            nullable_eq(Value::from(dec!(1.0)), Value::from(2.0))
        );
        assert_eq!(
            Value::from(false),
            nullable_eq(Value::from(1), Value::from(f64::NEG_INFINITY))
        );
        assert_eq!(
            Value::from(false),
            nullable_eq(Value::from(f64::NEG_INFINITY), Value::from(1))
        );
        // different, non-comparable types result in boolean true
        assert_eq!(
            Value::from(false),
            nullable_eq(Value::from(true), Value::from("abc"))
        );
        assert_eq!(
            Value::from(false),
            nullable_eq(Value::from("abc"), Value::from(true))
        );

        // Neq
        assert_eq!(
            Value::from(false),
            nullable_neq(Value::from(true), Value::from(true))
        );
        assert_eq!(
            Value::from(true),
            nullable_neq(Value::from(true), Value::from(false))
        );
        // Container examples from spec section 7.1.1 https://partiql.org/assets/PartiQL-Specification.pdf#subsubsection.7.1.1
        // (opposite result of eq cases)
        assert_eq!(
            Value::from(false),
            nullable_neq(Value::from(bag![3, 2, 4, 2]), Value::from(bag![2, 2, 3, 4]))
        );
        assert_eq!(
            Value::from(false),
            nullable_neq(
                Value::from(tuple![("a", 1), ("b", 2)]),
                Value::from(tuple![("a", 1), ("b", 2)])
            )
        );
        assert_eq!(
            Value::from(false),
            nullable_neq(
                Value::from(tuple![("a", list![0, 1]), ("b", 2)]),
                Value::from(tuple![("a", list![0, 1]), ("b", 2)])
            )
        );
        assert_eq!(
            Value::from(true),
            nullable_neq(Value::from(bag![3, 4, 2]), Value::from(bag![2, 2, 3, 4]))
        );
        assert_eq!(
            Value::from(true),
            nullable_neq(
                Value::from(tuple![("a", 1), ("b", 2)]),
                Value::from(tuple![("a", 1)])
            )
        );
        assert_eq!(
            Value::from(true),
            nullable_neq(
                Value::from(tuple![("a", list![0, 1]), ("b", 2)]),
                Value::from(tuple![("a", list![0, 1, 2]), ("b", 2)])
            )
        );
        assert_eq!(
            Value::from(true),
            nullable_neq(
                Value::from(tuple![("a", 1), ("b", 2)]),
                Value::from(tuple![("a", 1), ("b", Value::Null)])
            )
        );
        assert_eq!(
            Value::from(true),
            nullable_neq(
                Value::from(tuple![("a", list![0, 1]), ("b", 2)]),
                Value::from(tuple![("a", list![Value::Null, 1]), ("b", 2)])
            )
        );
        assert_eq!(Value::Null, nullable_neq(Value::from(true), Value::Null));
        assert_eq!(Value::Null, nullable_neq(Value::Null, Value::from(true)));
        assert_eq!(
            Value::Missing,
            nullable_neq(Value::from(true), Value::Missing)
        );
        assert_eq!(
            Value::Missing,
            nullable_neq(Value::Missing, Value::from(true))
        );

        // different, comparable types result in boolean true
        assert_eq!(
            Value::from(true),
            nullable_neq(Value::from(1), Value::from(2.0))
        );
        assert_eq!(
            Value::from(true),
            nullable_neq(Value::from(1.0), Value::from(2))
        );
        assert_eq!(
            Value::from(true),
            nullable_neq(Value::from(1), Value::from(dec!(2.0)))
        );
        assert_eq!(
            Value::from(true),
            nullable_neq(Value::from(dec!(1.0)), Value::from(2))
        );
        assert_eq!(
            Value::from(true),
            nullable_neq(Value::from(1.0), Value::from(dec!(2.0)))
        );
        assert_eq!(
            Value::from(true),
            nullable_neq(Value::from(dec!(1.0)), Value::from(2.0))
        );
        assert_eq!(
            Value::from(true),
            nullable_neq(Value::from(1), Value::from(f64::NEG_INFINITY))
        );
        assert_eq!(
            Value::from(true),
            nullable_neq(Value::from(f64::NEG_INFINITY), Value::from(1))
        );
        // different, comparable types result in boolean false
        assert_eq!(
            Value::from(false),
            nullable_neq(Value::from(1), Value::from(1.0))
        );
        assert_eq!(
            Value::from(false),
            nullable_neq(Value::from(1.0), Value::from(1))
        );
        assert_eq!(
            Value::from(false),
            nullable_neq(Value::from(1), Value::from(dec!(1.0)))
        );
        assert_eq!(
            Value::from(false),
            nullable_neq(Value::from(dec!(1.0)), Value::from(1))
        );
        assert_eq!(
            Value::from(false),
            nullable_neq(Value::from(1.0), Value::from(dec!(1.0)))
        );
        assert_eq!(
            Value::from(false),
            nullable_neq(Value::from(dec!(1.0)), Value::from(1.0))
        );
        // different, non-comparable types result in boolean true
        assert_eq!(
            Value::from(true),
            nullable_neq(Value::from(true), Value::from("abc"))
        );
        assert_eq!(
            Value::from(true),
            nullable_neq(Value::from("abc"), Value::from(true))
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
            NullableOrd::lt(&Value::from(1), &Value::Decimal(Box::new(dec!(2.0))))
        );
        assert_eq!(
            Value::from(true),
            NullableOrd::lt(&Value::from(1.0), &Value::from(2))
        );
        assert_eq!(
            Value::from(true),
            NullableOrd::lt(&Value::from(1.0), &Value::Decimal(Box::new(dec!(2.0))))
        );
        assert_eq!(
            Value::from(true),
            NullableOrd::lt(&Value::Decimal(Box::new(dec!(1.0))), &Value::from(2))
        );
        assert_eq!(
            Value::from(true),
            NullableOrd::lt(&Value::Decimal(Box::new(dec!(1.0))), &Value::from(2.))
        );

        // GT
        assert_eq!(
            Value::from(false),
            NullableOrd::gt(&Value::from(1), &Value::from(2.0))
        );
        assert_eq!(
            Value::from(false),
            NullableOrd::gt(&Value::from(1), &Value::Decimal(Box::new(dec!(2.0))))
        );
        assert_eq!(
            Value::from(false),
            NullableOrd::gt(&Value::from(1.0), &Value::from(2))
        );
        assert_eq!(
            Value::from(false),
            NullableOrd::gt(&Value::from(1.0), &Value::Decimal(Box::new(dec!(2.0))))
        );
        assert_eq!(
            Value::from(false),
            NullableOrd::gt(&Value::Decimal(Box::new(dec!(1.0))), &Value::from(2))
        );
        assert_eq!(
            Value::from(false),
            NullableOrd::gt(&Value::Decimal(Box::new(dec!(1.0))), &Value::from(2.))
        );

        // LTEQ
        assert_eq!(
            Value::from(true),
            NullableOrd::lteq(&Value::from(1), &Value::from(2.0))
        );
        assert_eq!(
            Value::from(true),
            NullableOrd::lteq(&Value::from(1), &Value::Decimal(Box::new(dec!(2.0))))
        );
        assert_eq!(
            Value::from(true),
            NullableOrd::lteq(&Value::from(1.0), &Value::from(2))
        );
        assert_eq!(
            Value::from(true),
            NullableOrd::lteq(&Value::from(1.0), &Value::Decimal(Box::new(dec!(2.0))))
        );
        assert_eq!(
            Value::from(true),
            NullableOrd::lteq(&Value::Decimal(Box::new(dec!(1.0))), &Value::from(2))
        );
        assert_eq!(
            Value::from(true),
            NullableOrd::lteq(&Value::Decimal(Box::new(dec!(1.0))), &Value::from(2.))
        );

        // GTEQ
        assert_eq!(
            Value::from(false),
            NullableOrd::gteq(&Value::from(1), &Value::from(2.0))
        );
        assert_eq!(
            Value::from(false),
            NullableOrd::gteq(&Value::from(1), &Value::Decimal(Box::new(dec!(2.0))))
        );
        assert_eq!(
            Value::from(false),
            NullableOrd::gteq(&Value::from(1.0), &Value::from(2))
        );
        assert_eq!(
            Value::from(false),
            NullableOrd::gteq(&Value::from(1.0), &Value::Decimal(Box::new(dec!(2.0))))
        );
        assert_eq!(
            Value::from(false),
            NullableOrd::gteq(&Value::Decimal(Box::new(dec!(1.0))), &Value::from(2))
        );
        assert_eq!(
            Value::from(false),
            NullableOrd::gteq(&Value::Decimal(Box::new(dec!(1.0))), &Value::from(2.))
        );
    }

    #[test]
    fn tuple_concat() {
        let lhs = Tuple::from([("a", 1), ("b", 2), ("c", 3), ("d", 44)]);
        let rhs = Tuple::from([("a", 11), ("b", 22), ("c", 33), ("e", 55)]);
        assert_eq!(
            Tuple::from([("a", 11), ("b", 22), ("c", 33), ("d", 44), ("e", 55)]),
            lhs.tuple_concat(&rhs)
        );
    }

    #[test]
    fn tuple_get() {
        let tuple = Tuple::from([("a", 1), ("A", 2), ("a", 3), ("A", 4)]);
        // case sensitive
        assert_eq!(
            Some(&Value::from(1)),
            tuple.get(&BindingsName::CaseSensitive(Cow::Owned("a".to_string())))
        );
        assert_eq!(
            Some(&Value::from(2)),
            tuple.get(&BindingsName::CaseSensitive(Cow::Owned("A".to_string())))
        );
        // case insensitive
        assert_eq!(
            Some(&Value::from(1)),
            tuple.get(&BindingsName::CaseInsensitive(Cow::Owned("a".to_string())))
        );
        assert_eq!(
            Some(&Value::from(1)),
            tuple.get(&BindingsName::CaseInsensitive(Cow::Owned("A".to_string())))
        );
    }

    #[test]
    fn tuple_remove() {
        let mut tuple = Tuple::from([("a", 1), ("A", 2), ("a", 3), ("A", 4)]);
        // case sensitive
        assert_eq!(
            Some(Value::from(2)),
            tuple.remove(&BindingsName::CaseSensitive(Cow::Owned("A".to_string())))
        );
        assert_eq!(
            Some(Value::from(1)),
            tuple.remove(&BindingsName::CaseSensitive(Cow::Owned("a".to_string())))
        );
        // case insensitive
        assert_eq!(
            Some(Value::from(3)),
            tuple.remove(&BindingsName::CaseInsensitive(Cow::Owned("A".to_string())))
        );
        assert_eq!(
            Some(Value::from(4)),
            tuple.remove(&BindingsName::CaseInsensitive(Cow::Owned("a".to_string())))
        );
    }

    #[test]
    fn bag_of_tuple_equality() {
        // Asserts the following PartiQL Values are equal
        // <<{
        //   'outer_elem_1': 1,
        //   'outer_elem_2': <<{
        //      'inner_elem_1': {'bar': 4},
        //      'inner_elem_2': {'foo': 3},
        //   }>>
        // }>>
        // with
        // <<{
        //   'outer_elem_2': <<{
        //      'inner_elem_2': {'foo': 3},
        //      'inner_elem_1': {'bar': 4},
        //   }>>
        //   'outer_elem_1': 1,
        // }>>
        let bag1 = bag!(tuple![
            ("outer_elem_1", 1),
            (
                "outer_elem_2",
                bag![tuple![
                    ("inner_elem_1", tuple![("bar", 3)]),
                    ("inner_elem_2", tuple![("foo", 4)])
                ]]
            )
        ]);
        let bag2 = bag!(tuple![
            (
                "outer_elem_2",
                bag![tuple![
                    ("inner_elem_2", tuple![("foo", 4)]),
                    ("inner_elem_1", tuple![("bar", 3)])
                ]]
            ),
            ("outer_elem_1", 1)
        ]);
        assert_eq!(bag1, bag2);
    }

    #[test]
    fn duplicate_tuple_elems() {
        let tuple1 = tuple![("a", 1), ("a", 1), ("b", 2)];
        let tuple2 = tuple![("a", 1), ("b", 2)];
        assert_ne!(tuple1, tuple2);
    }

    #[test]
    fn tuple_hashing() {
        let tuple1 = tuple![("a", 1), ("b", 2)];
        let mut s: HashSet<Tuple> = HashSet::from([tuple1]);
        assert_eq!(1, s.len());
        let tuple2 = tuple![("b", 2), ("a", 1)];
        s.insert(tuple2);
        assert_eq!(1, s.len());
    }

    #[test]
    fn null_sorted_value_ord_simple_nulls_first() {
        let v1 = Value::Missing;
        let v2 = Value::Boolean(true);
        let null_sorted_v1 = NullSortedValue::<true, Value>(&v1);
        let null_sorted_v2 = NullSortedValue::<true, Value>(&v2);
        assert_eq!(Ordering::Less, null_sorted_v1.cmp(&null_sorted_v2));

        let v3 = Value::Null;
        let null_sorted_v3 = NullSortedValue::<true, Value>(&v3);
        assert_eq!(Ordering::Less, null_sorted_v3.cmp(&null_sorted_v2));
        assert_eq!(Ordering::Equal, null_sorted_v1.cmp(&null_sorted_v3));
    }

    #[test]
    fn null_sorted_value_ord_simple_nulls_last() {
        let v1 = Value::Missing;
        let v2 = Value::Boolean(true);
        let null_sorted_v1 = NullSortedValue::<false, Value>(&v1);
        let null_sorted_v2 = NullSortedValue::<false, Value>(&v2);
        assert_eq!(Ordering::Greater, null_sorted_v1.cmp(&null_sorted_v2));

        let v3 = Value::Null;
        let null_sorted_v3 = NullSortedValue::<false, Value>(&v3);
        assert_eq!(Ordering::Greater, null_sorted_v3.cmp(&null_sorted_v2));
        assert_eq!(Ordering::Equal, null_sorted_v1.cmp(&null_sorted_v3));
    }

    #[test]
    fn null_sorted_value_ord_collection_nulls_first() {
        // list
        let v1 = list![Value::Missing].into();
        let v2 = list![Value::Boolean(true)].into();
        let null_sorted_v1 = NullSortedValue::<true, Value>(&v1);
        let null_sorted_v2 = NullSortedValue::<true, Value>(&v2);
        assert_eq!(Ordering::Less, null_sorted_v1.cmp(&null_sorted_v2));

        let v3 = list![Value::Null].into();
        let null_sorted_v3 = NullSortedValue::<true, Value>(&v3);
        assert_eq!(Ordering::Less, null_sorted_v3.cmp(&null_sorted_v2));
        assert_eq!(Ordering::Equal, null_sorted_v1.cmp(&null_sorted_v3));

        // bag
        let v1 = bag![Value::Missing].into();
        let v2 = bag![Value::Boolean(true)].into();
        let null_sorted_v1 = NullSortedValue::<true, Value>(&v1);
        let null_sorted_v2 = NullSortedValue::<true, Value>(&v2);
        assert_eq!(Ordering::Less, null_sorted_v1.cmp(&null_sorted_v2));

        let v3 = bag![Value::Null].into();
        let null_sorted_v3 = NullSortedValue::<true, Value>(&v3);
        assert_eq!(Ordering::Less, null_sorted_v3.cmp(&null_sorted_v2));
        assert_eq!(Ordering::Equal, null_sorted_v1.cmp(&null_sorted_v3));

        // tuple
        let v1 = tuple![("a", Value::Missing)].into();
        let v2 = tuple![("a", Value::Boolean(true))].into();
        let null_sorted_v1 = NullSortedValue::<true, Value>(&v1);
        let null_sorted_v2 = NullSortedValue::<true, Value>(&v2);
        assert_eq!(Ordering::Less, null_sorted_v1.cmp(&null_sorted_v2));

        let v3 = tuple![("a", Value::Null)].into();
        let null_sorted_v3 = NullSortedValue::<true, Value>(&v3);
        assert_eq!(Ordering::Less, null_sorted_v3.cmp(&null_sorted_v2));
        assert_eq!(Ordering::Equal, null_sorted_v1.cmp(&null_sorted_v3));
    }

    #[test]
    fn null_sorted_value_ord_collection_nulls_last() {
        // list
        let v1 = list![Value::Missing].into();
        let v2 = list![Value::Boolean(true)].into();
        let null_sorted_v1 = NullSortedValue::<false, Value>(&v1);
        let null_sorted_v2 = NullSortedValue::<false, Value>(&v2);
        assert_eq!(Ordering::Greater, null_sorted_v1.cmp(&null_sorted_v2));

        let v3 = list![Value::Null].into();
        let null_sorted_v3 = NullSortedValue::<false, Value>(&v3);
        assert_eq!(Ordering::Greater, null_sorted_v3.cmp(&null_sorted_v2));
        assert_eq!(Ordering::Equal, null_sorted_v1.cmp(&null_sorted_v3));

        // bag
        let v1 = bag![Value::Missing].into();
        let v2 = bag![Value::Boolean(true)].into();
        let null_sorted_v1 = NullSortedValue::<false, Value>(&v1);
        let null_sorted_v2 = NullSortedValue::<false, Value>(&v2);
        assert_eq!(Ordering::Greater, null_sorted_v1.cmp(&null_sorted_v2));

        let v3 = bag![Value::Null].into();
        let null_sorted_v3 = NullSortedValue::<false, Value>(&v3);
        assert_eq!(Ordering::Greater, null_sorted_v3.cmp(&null_sorted_v2));
        assert_eq!(Ordering::Equal, null_sorted_v1.cmp(&null_sorted_v3));

        // tuple
        let v1 = tuple![("a", Value::Missing)].into();
        let v2 = tuple![("a", Value::Boolean(true))].into();
        let null_sorted_v1 = NullSortedValue::<false, Value>(&v1);
        let null_sorted_v2 = NullSortedValue::<false, Value>(&v2);
        assert_eq!(Ordering::Greater, null_sorted_v1.cmp(&null_sorted_v2));

        let v3 = tuple![("a", Value::Null)].into();
        let null_sorted_v3 = NullSortedValue::<false, Value>(&v3);
        assert_eq!(Ordering::Greater, null_sorted_v3.cmp(&null_sorted_v2));
        assert_eq!(Ordering::Equal, null_sorted_v1.cmp(&null_sorted_v3));
    }
}
