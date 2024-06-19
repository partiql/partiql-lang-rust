use crate::eval::evaluable::SetQuantifier;

use crate::eval::expr::{BindError, BindEvalExpr, EvalExpr};

use itertools::{Itertools, Unique};

use partiql_types::{
    ArrayType, BagType, PartiqlShape, StaticTypeVariant, TYPE_BOOL, TYPE_NUMERIC_TYPES,
};
use partiql_value::Value::{Missing, Null};
use partiql_value::{BinaryAnd, BinaryOr, Value, ValueIter};

use std::fmt::Debug;
use std::hash::Hash;

use crate::eval::eval_expr_wrapper::UnaryValueExpr;
use std::ops::ControlFlow;

#[derive(Debug, Clone, Copy, Eq, PartialEq)]
pub(crate) enum EvalCollFn {
    /// Represents the `COLL_COUNT` function, e.g. `COLL_COUNT(DISTINCT [1, 2, 2, 3])`.
    Count(SetQuantifier),
    /// Represents the `COLL_AVG` function, e.g. `COLL_AVG(DISTINCT [1, 2, 2, 3])`.
    Avg(SetQuantifier),
    /// Represents the `COLL_MAX` function, e.g. `COLL_MAX(DISTINCT [1, 2, 2, 3])`.
    Max(SetQuantifier),
    /// Represents the `COLL_MIN` function, e.g. `COLL_MIN(DISTINCT [1, 2, 2, 3])`.
    Min(SetQuantifier),
    /// Represents the `COLL_SUM` function, e.g. `COLL_SUM(DISTINCT [1, 2, 2, 3])`.
    Sum(SetQuantifier),
    /// Represents the `COLL_ANY`/`COLL_SOME` function, e.g. `COLL_ANY(DISTINCT [true, true, false])`.
    Any(SetQuantifier),
    /// Represents the `COLL_EVERY` function, e.g. `COLL_EVERY(DISTINCT [true, true, false])`.
    Every(SetQuantifier),
}

impl BindEvalExpr for EvalCollFn {
    fn bind<const STRICT: bool>(
        &self,
        args: Vec<Box<dyn EvalExpr>>,
    ) -> Result<Box<dyn EvalExpr>, BindError> {
        fn create<const STRICT: bool, F>(
            types: [PartiqlShape; 1],
            args: Vec<Box<dyn EvalExpr>>,
            f: F,
        ) -> Result<Box<dyn EvalExpr>, BindError>
        where
            F: Fn(ValueIter<'_>) -> Value + 'static,
        {
            UnaryValueExpr::create_typed::<{ STRICT }, _>(types, args, move |value| {
                value.sequence_iter().map_or(Missing, &f)
            })
        }
        let boolean_elems = [PartiqlShape::any_of([
            PartiqlShape::new(StaticTypeVariant::Array(ArrayType::new(Box::new(
                TYPE_BOOL,
            )))),
            PartiqlShape::new(StaticTypeVariant::Bag(BagType::new(Box::new(TYPE_BOOL)))),
        ])];
        let numeric_elems = [PartiqlShape::any_of([
            PartiqlShape::new(StaticTypeVariant::Array(ArrayType::new(Box::new(
                PartiqlShape::any_of(TYPE_NUMERIC_TYPES),
            )))),
            PartiqlShape::new(StaticTypeVariant::Bag(BagType::new(Box::new(
                PartiqlShape::any_of(TYPE_NUMERIC_TYPES),
            )))),
        ])];
        let any_elems = [PartiqlShape::any_of([
            PartiqlShape::new(StaticTypeVariant::Array(ArrayType::new_any())),
            PartiqlShape::new(StaticTypeVariant::Bag(BagType::new_any())),
        ])];

        match *self {
            EvalCollFn::Count(setq) => {
                create::<{ STRICT }, _>(any_elems, args, move |it| it.coll_count(setq))
            }
            EvalCollFn::Avg(setq) => {
                create::<{ STRICT }, _>(numeric_elems, args, move |it| it.coll_avg(setq))
            }
            EvalCollFn::Max(setq) => {
                create::<{ STRICT }, _>(any_elems, args, move |it| it.coll_max(setq))
            }
            EvalCollFn::Min(setq) => {
                create::<{ STRICT }, _>(any_elems, args, move |it| it.coll_min(setq))
            }
            EvalCollFn::Sum(setq) => {
                create::<{ STRICT }, _>(numeric_elems, args, move |it| it.coll_sum(setq))
            }
            EvalCollFn::Any(setq) => {
                create::<{ STRICT }, _>(boolean_elems, args, move |it| it.coll_any(setq))
            }
            EvalCollFn::Every(setq) => {
                create::<{ STRICT }, _>(boolean_elems, args, move |it| it.coll_every(setq))
            }
        }
    }
}

/// An [`Iterator`] over either `ALL` or `DISTINCT` items
enum SetQuantified<V, I>
where
    V: Clone + Eq + Hash,
    I: Iterator<Item = V>,
{
    All(I),
    Distinct(Unique<I>),
}

impl<V, I> Iterator for SetQuantified<V, I>
where
    V: Clone + Eq + Hash,
    I: Iterator<Item = V>,
{
    type Item = V;

    #[inline]
    fn next(&mut self) -> Option<Self::Item> {
        match self {
            SetQuantified::All(i) => i.next(),
            SetQuantified::Distinct(i) => i.next(),
        }
    }

    #[inline]
    fn size_hint(&self) -> (usize, Option<usize>) {
        match self {
            SetQuantified::All(i) => i.size_hint(),
            SetQuantified::Distinct(i) => i.size_hint(),
        }
    }
}

/// An [`Iterator`] over a 'set' of values
trait SetIterator: Iterator {
    #[inline]
    fn set_quantified(self, setq: SetQuantifier) -> SetQuantified<Self::Item, Self>
    where
        Self: Sized,
        Self::Item: Clone + Eq + Hash,
    {
        match setq {
            SetQuantifier::All => SetQuantified::All(self),
            SetQuantifier::Distinct => SetQuantified::Distinct(self.unique()),
        }
    }
}

impl<T: ?Sized> SetIterator for T where T: Iterator {}

/// [`Iterator`] methods for performing `COLL_*` operations
trait CollIterator<'a>: Iterator<Item = &'a Value> {
    #[inline]
    fn coll_sum(self, setq: SetQuantifier) -> Value
    where
        Self: Sized,
    {
        self.filter(|e| e.is_present())
            .set_quantified(setq)
            .coll_reduce_or(Null, |prev, x| {
                if x.is_number() {
                    ControlFlow::Continue(&prev + x)
                } else {
                    ControlFlow::Break(Missing)
                }
            })
    }

    #[inline]
    fn coll_count(self, setq: SetQuantifier) -> Value
    where
        Self: Sized,
    {
        self.filter(|e| e.is_present())
            .set_quantified(setq)
            .count()
            .into()
    }

    #[inline]
    fn coll_min(self, setq: SetQuantifier) -> Value
    where
        Self: Sized,
    {
        self.filter(|e| e.is_present())
            .set_quantified(setq)
            .coll_reduce_or(Null, |prev, x| {
                ControlFlow::Continue(if &prev <= x { prev } else { x.clone() })
            })
    }

    #[inline]
    fn coll_max(self, setq: SetQuantifier) -> Value
    where
        Self: Sized,
    {
        self.filter(|e| e.is_present())
            .set_quantified(setq)
            .coll_reduce_or(Null, |prev, x| {
                ControlFlow::Continue(if &prev > x { prev } else { x.clone() })
            })
    }

    #[inline]
    fn coll_avg(self, setq: SetQuantifier) -> Value
    where
        Self: Sized,
    {
        let mut enumerated = self
            .filter(|e| e.is_present())
            .set_quantified(setq)
            .enumerate();
        if let Some((n, v)) = enumerated.next() {
            let folded = enumerated.try_fold((n, v.clone()), |(_, sum), (n, v)| {
                if v.is_number() {
                    ControlFlow::Continue((n, (&sum + v)))
                } else {
                    ControlFlow::Break(Missing)
                }
            });
            match folded {
                ControlFlow::Continue((enumeration, sum)) => {
                    let count = enumeration + 1;
                    &sum / &Value::Decimal(Box::new(rust_decimal::Decimal::from(count)))
                }
                ControlFlow::Break(v) => v,
            }
        } else {
            Null
        }
    }

    #[inline]
    fn coll_any(self, setq: SetQuantifier) -> Value
    where
        Self: Sized,
    {
        self.filter(|e| e.is_present())
            .set_quantified(setq)
            .coll_reduce_or(Null, |prev, x| {
                if let Value::Boolean(_) = x {
                    ControlFlow::Continue(prev.or(x))
                } else {
                    ControlFlow::Break(Missing)
                }
            })
    }

    #[inline]
    fn coll_every(self, setq: SetQuantifier) -> Value
    where
        Self: Sized,
    {
        self.filter(|e| e.is_present())
            .set_quantified(setq)
            .coll_reduce_or(Null, |prev, x| {
                if let Value::Boolean(_) = x {
                    ControlFlow::Continue(prev.and(x))
                } else {
                    ControlFlow::Break(Missing)
                }
            })
    }
}

/// [`Iterator`] helper methods for `COLL_*` operators for reducing values to a single value while
/// allowing the reducing closure to signal an early return with [`ControlFlow::Break`]
trait ShortCircuitReduceIterator<'a, R: 'a>: Iterator<Item = &'a R>
where
    R: Clone,
{
    #[inline]
    fn coll_reduce_or<F>(self, default: R, f: F) -> R
    where
        Self: Sized,
        Self::Item: Clone,
        F: FnMut(R, &'a R) -> ControlFlow<R, R>,
    {
        self.coll_reduce_or_else(|| default, f)
    }

    #[inline]
    fn coll_reduce_or_else<D, F>(mut self, default: D, f: F) -> R
    where
        Self: Sized,
        Self::Item: Clone,
        D: FnOnce() -> R,
        F: FnMut(R, &'a R) -> ControlFlow<R, R>,
    {
        if let Some(init) = self.next() {
            let init: R = init.clone();
            match self.try_fold(init, f) {
                ControlFlow::Continue(v) => v,
                ControlFlow::Break(v) => v,
            }
        } else {
            default()
        }
    }
}

impl<'a, T: ?Sized> CollIterator<'a> for T where T: Iterator<Item = &'a Value> {}
impl<'a, R: 'a, T: ?Sized> ShortCircuitReduceIterator<'a, R> for T
where
    R: Clone,
    T: Iterator<Item = &'a R>,
{
}
