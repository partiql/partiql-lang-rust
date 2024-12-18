use crate::{Bag, List, Tuple, Value};
use std::cmp::Ordering;

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
        let wrap_list = NullSortedValue::<'_, { NULLS_FIRST }, List>;
        let wrap_tuple = NullSortedValue::<'_, { NULLS_FIRST }, Tuple>;
        let wrap_bag = NullSortedValue::<'_, { NULLS_FIRST }, Bag>;
        let wrap_value = NullSortedValue::<'_, { NULLS_FIRST }, Value>;
        let wrap_var = NullSortedValue::<'_, { NULLS_FIRST }, Variant>;
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
