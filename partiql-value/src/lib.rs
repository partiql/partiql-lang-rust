#![deny(rust_2018_idioms)]
#![deny(clippy::all)]

mod bag;
mod bindings;
pub mod boxed_variant;
pub mod comparison;
mod datetime;
pub mod datum;
mod list;
mod pretty;
mod sort;
mod tuple;
mod util;
mod value;
mod variant;

pub use bag::*;
pub use bindings::*;
pub use comparison::*;
pub use datetime::*;
pub use list::*;
pub use pretty::*;
pub use sort::*;
pub use tuple::*;
pub use value::*;
pub use variant::*;

#[cfg(feature = "serde")]
use serde::{Deserialize, Serialize};

#[cfg(test)]
mod tests {
    use super::*;
    use crate::comparison::{EqualityValue, NullableEq, NullableOrd};
    use crate::sort::NullSortedValue;
    use ordered_float::OrderedFloat;
    use rust_decimal::Decimal as RustDecimal;
    use rust_decimal_macros::dec;
    use std::borrow::Cow;
    use std::cell::RefCell;
    use std::cmp::Ordering;
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
        let bag: Bag = [1, 10, 3, 4].into_iter().collect();
        assert_eq!(bag.len(), 4);
        let max = bag
            .iter()
            .fold(Value::Integer(0), |x, y| if y > &x { y.clone() } else { x });
        assert_eq!(max, Value::Integer(10));
        let _bref = Value::from(bag).as_bag_ref();

        let list: List = [1, 2, 3, -4].into_iter().collect();
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
            let wrap = EqualityValue::<false, false, Value>;
            let lhs = wrap(&lhs);
            let rhs = wrap(&rhs);
            NullableEq::eq(&lhs, &rhs)
        }

        fn nullable_neq(lhs: Value, rhs: Value) -> Value {
            let wrap = EqualityValue::<false, false, Value>;
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
