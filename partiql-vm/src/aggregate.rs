//! Aggregate functions, shared by SQL aggregates (GROUP BY's `AggStep` / `AggFinal`) and the
//! `COLL_*` functions, as in the legacy evaluator:
//! - NULL and MISSING inputs are skipped; COUNT counts the others.
//! - SUM and AVG add numbers, widening an int to the other operand's type and a float with a
//!   decimal to a decimal. AVG of ints is a decimal.
//! - MIN and MAX compare with PartiQL's total order of values, across types.
//! - ANY (SOME) and EVERY combine booleans.
//! - An input the aggregate can't take (a string to SUM, a number to ANY) makes the result
//!   MISSING, or in strict mode is an error.
//! - With no input, COUNT is 0 and the others are NULL.

use crate::error::{EngineError, Result};
use crate::expr::AggFunc;
use crate::sorter;
use crate::value::ValueRef;
use rust_decimal::prelude::FromPrimitive;
use rust_decimal::Decimal;
use std::cmp::Ordering;

// An accumulator is MISSING before its first input, and NULL once it has taken an input it
// can't use: no input of its own is NULL, as NULL inputs are skipped.
const EMPTY: ValueRef<'static> = ValueRef::Missing;
const INVALID: ValueRef<'static> = ValueRef::Null;

/// Fold `input` into `accum`.
pub(crate) fn step<'a>(
    func: AggFunc,
    accum: ValueRef<'a>,
    input: ValueRef<'a>,
    strict: bool,
) -> Result<ValueRef<'a>> {
    if matches!(input, ValueRef::Null | ValueRef::Missing) {
        return Ok(accum);
    }
    if let AggFunc::Count = func {
        return Ok(match accum {
            ValueRef::I64(n) => ValueRef::I64(n.checked_add(1).ok_or_else(overflow)?),
            _ => ValueRef::I64(1),
        });
    }
    if matches!(accum, ValueRef::Null) {
        return Ok(INVALID);
    }
    let taken = match func {
        AggFunc::Sum | AggFunc::Avg => match (accum, is_number(input)) {
            (_, false) => None,
            (ValueRef::Missing, true) => Some(input),
            (accum, true) => add(accum, input)?,
        },
        AggFunc::Min | AggFunc::Max => Some(match accum {
            ValueRef::Missing => input,
            accum => {
                let keep = match func {
                    AggFunc::Min => Ordering::Less,
                    _ => Ordering::Greater,
                };
                match sorter::total_order(&input, &accum) == keep {
                    true => input,
                    false => accum,
                }
            }
        }),
        AggFunc::Any | AggFunc::Every => match (accum, input) {
            (ValueRef::Missing, ValueRef::Bool(b)) => Some(ValueRef::Bool(b)),
            (ValueRef::Bool(a), ValueRef::Bool(b)) => Some(ValueRef::Bool(match func {
                AggFunc::Any => a || b,
                _ => a && b,
            })),
            _ => None,
        },
        AggFunc::Count => unreachable!("counted above"),
    };
    match taken {
        Some(value) => Ok(value),
        None if strict => Err(EngineError::TypeError(format!(
            "{func:?} cannot aggregate {input:?}"
        ))),
        None => Ok(INVALID),
    }
}

/// The result of an aggregate from its accumulator. AVG's also needs its count.
pub(crate) fn finish<'a>(
    func: AggFunc,
    accum: ValueRef<'a>,
    count: ValueRef<'a>,
) -> Result<ValueRef<'a>> {
    Ok(match (func, accum) {
        (AggFunc::Count, ValueRef::Missing) => ValueRef::I64(0),
        (AggFunc::Count, accum) => accum,
        (_, ValueRef::Missing) => ValueRef::Null,
        (_, ValueRef::Null) => ValueRef::Missing,
        (AggFunc::Avg, sum) => {
            let ValueRef::I64(count) = count else {
                return Err(EngineError::IllegalState(format!("AVG count {count:?}")));
            };
            match sum {
                ValueRef::I64(n) => ValueRef::Decimal(
                    Decimal::from(n)
                        .checked_div(Decimal::from(count))
                        .ok_or_else(overflow)?,
                ),
                ValueRef::Decimal(d) => {
                    ValueRef::Decimal(d.checked_div(Decimal::from(count)).ok_or_else(overflow)?)
                }
                ValueRef::F64(f) => ValueRef::F64(f / count as f64),
                other => other,
            }
        }
        (_, accum) => accum,
    })
}

/// A `COLL_*` function over `collection`: NULL for NULL, MISSING for MISSING, and MISSING (or
/// in strict mode an error) for a value that is no collection.
pub(crate) fn over_collection<'a>(
    func: AggFunc,
    distinct: bool,
    collection: ValueRef<'a>,
    strict: bool,
) -> Result<ValueRef<'a>> {
    let items = match collection {
        ValueRef::List(items) | ValueRef::Bag(items) => items,
        ValueRef::Null => return Ok(ValueRef::Null),
        ValueRef::Missing => return Ok(ValueRef::Missing),
        other if strict => {
            return Err(EngineError::TypeError(format!(
                "{func:?} of a value that is no collection: {other:?}"
            )))
        }
        _ => return Ok(ValueRef::Missing),
    };
    let distinct_items;
    let items = match distinct {
        true => {
            distinct_items = sorter::distinct(items);
            &distinct_items[..]
        }
        false => items,
    };
    let (mut accum, mut count) = (EMPTY, EMPTY);
    for &item in items {
        accum = step(func, accum, item, strict)?;
        count = step(AggFunc::Count, count, item, strict)?;
    }
    finish(func, accum, count)
}

fn is_number(v: ValueRef<'_>) -> bool {
    matches!(
        v,
        ValueRef::I64(_) | ValueRef::Decimal(_) | ValueRef::F64(_)
    )
}

/// `a + b` for two numbers; `None` when a float with no decimal value (NaN, infinity) meets a
/// decimal.
fn add<'a>(a: ValueRef<'a>, b: ValueRef<'a>) -> Result<Option<ValueRef<'a>>> {
    let decimal = |f: f64| Decimal::from_f64(f).filter(|_| f.is_finite());
    Ok(match (a, b) {
        (ValueRef::I64(a), ValueRef::I64(b)) => {
            Some(ValueRef::I64(a.checked_add(b).ok_or_else(overflow)?))
        }
        (ValueRef::F64(a), ValueRef::F64(b)) => Some(ValueRef::F64(a + b)),
        (ValueRef::I64(i), ValueRef::F64(f)) | (ValueRef::F64(f), ValueRef::I64(i)) => {
            Some(ValueRef::F64(i as f64 + f))
        }
        (ValueRef::I64(i), ValueRef::Decimal(d)) | (ValueRef::Decimal(d), ValueRef::I64(i)) => {
            Some(ValueRef::Decimal(
                d.checked_add(Decimal::from(i)).ok_or_else(overflow)?,
            ))
        }
        (ValueRef::Decimal(a), ValueRef::Decimal(b)) => {
            Some(ValueRef::Decimal(a.checked_add(b).ok_or_else(overflow)?))
        }
        (ValueRef::F64(f), ValueRef::Decimal(d)) | (ValueRef::Decimal(d), ValueRef::F64(f)) => {
            match decimal(f) {
                Some(f) => Some(ValueRef::Decimal(d.checked_add(f).ok_or_else(overflow)?)),
                None => None,
            }
        }
        _ => None,
    })
}

fn overflow() -> EngineError {
    EngineError::Arithmetic("overflow".to_string())
}

#[cfg(test)]
mod tests {
    use super::*;

    fn bag(items: &[ValueRef<'static>]) -> ValueRef<'static> {
        ValueRef::Bag(Box::leak(items.to_vec().into_boxed_slice()))
    }

    fn coll(func: AggFunc, items: &[ValueRef<'static>]) -> String {
        format!(
            "{:?}",
            over_collection(func, false, bag(items), false).expect("permissive")
        )
    }

    #[test]
    fn sum_and_avg_widen_numbers() {
        let (i, d) = (ValueRef::I64, |s: &str| {
            ValueRef::Decimal(s.parse().unwrap())
        });
        assert_eq!(coll(AggFunc::Sum, &[i(1), d("2.5"), i(3)]), "Decimal(6.5)");
        assert_eq!(coll(AggFunc::Sum, &[i(1), ValueRef::F64(0.5)]), "F64(1.5)");
        // Decimals compare by value, whatever their scale
        let avg = |items: &[ValueRef<'static>]| match over_collection(
            AggFunc::Avg,
            false,
            bag(items),
            false,
        )
        .unwrap()
        {
            ValueRef::Decimal(d) => d,
            other => panic!("{other:?}"),
        };
        assert_eq!(avg(&[i(1), i(2)]), Decimal::new(15, 1));
        assert_eq!(avg(&[d("1.0"), d("2.0"), d("3.0")]), Decimal::from(2));
    }

    #[test]
    fn min_and_max_order_across_types() {
        let s = ValueRef::Str;
        assert_eq!(coll(AggFunc::Min, &[s("b"), s("a"), s("c")]), "Str(\"a\")");
        assert_eq!(
            coll(AggFunc::Max, &[ValueRef::I64(1), s("x")]),
            "Str(\"x\")"
        );
        assert_eq!(
            coll(AggFunc::Max, &[ValueRef::I64(1), ValueRef::F64(1.5)]),
            "F64(1.5)"
        );
    }

    #[test]
    fn absent_inputs_are_skipped_and_mistyped_ones_are_missing() {
        let (n, m) = (ValueRef::Null, ValueRef::Missing);
        assert_eq!(coll(AggFunc::Sum, &[n, m]), "Null");
        assert_eq!(
            coll(
                AggFunc::Count,
                &[n, ValueRef::I64(1), m, ValueRef::Str("a")]
            ),
            "I64(2)"
        );
        assert_eq!(coll(AggFunc::Count, &[]), "I64(0)");
        assert_eq!(
            coll(AggFunc::Sum, &[ValueRef::I64(1), ValueRef::Str("a")]),
            "Missing"
        );
        assert_eq!(
            coll(AggFunc::Any, &[ValueRef::Bool(true), ValueRef::I64(5)]),
            "Missing"
        );
        let strict = over_collection(AggFunc::Sum, false, bag(&[ValueRef::Str("a")]), true);
        assert!(strict.is_err());
    }

    #[test]
    fn collection_argument() {
        let f =
            |v, strict| over_collection(AggFunc::Count, false, v, strict).map(|v| format!("{v:?}"));
        assert_eq!(f(ValueRef::Null, false).unwrap(), "Null");
        assert_eq!(f(ValueRef::Missing, false).unwrap(), "Missing");
        assert_eq!(f(ValueRef::I64(1), false).unwrap(), "Missing");
        assert!(f(ValueRef::I64(1), true).is_err());
    }

    #[test]
    fn distinct() {
        let i = ValueRef::I64;
        let items = bag(&[i(1), i(1), i(3), i(1)]);
        let avg = over_collection(AggFunc::Avg, true, items, false).unwrap();
        assert_eq!(format!("{avg:?}"), "Decimal(2)");
    }
}
