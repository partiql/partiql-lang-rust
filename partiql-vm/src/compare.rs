//! PartiQL's comparisons of values:
//! - ORDER BY's total order (spec section 12.2), also used by MIN and MAX;
//! - `eqg`, the equivalence that tells values apart for GROUP BY, DISTINCT and the set
//!   operators (spec section 7.1.1), as a total order and with a matching hash.

use crate::arena::Arena;
use crate::error::Result;
use crate::plan::SortOrder;
use crate::value::{variant, TupleField, TupleRef, ValueRef};
use rust_decimal::Decimal;
use rustc_hash::FxHashMap;
use std::cmp::Ordering;
use std::hash::{BuildHasher, Hash, Hasher, RandomState};

impl SortOrder {
    /// Compare two sort keys as ORDER BY does, matching `partiql_value`'s
    /// `NullSortedValue`: NULL and MISSING are equal, and sort first or last (also when
    /// nested in a collection); other values follow PartiQL's total order (spec section 12.2).
    pub(crate) fn compare(&self, a: &ValueRef<'_>, b: &ValueRef<'_>) -> Ordering {
        // A descending sort reverses the comparison, NULLs included
        let nulls_low = self.desc != self.nulls_first;
        let cmp = order_cmp(a, b, nulls_low);
        if self.desc {
            cmp.reverse()
        } else {
            cmp
        }
    }
}

/// PartiQL's total order of values (NULL and MISSING first), as for MIN and MAX.
pub(crate) fn total_order(a: &ValueRef<'_>, b: &ValueRef<'_>) -> Ordering {
    order_cmp(a, b, true)
}

/// PartiQL's total order of values, with NULL and MISSING below every other value when
/// `nulls_low`, else above.
fn order_cmp(a: &ValueRef<'_>, b: &ValueRef<'_>, nulls_low: bool) -> Ordering {
    let nulls = |cmp: Ordering| if nulls_low { cmp } else { cmp.reverse() };
    match (a, b) {
        (ValueRef::Missing | ValueRef::Null, ValueRef::Missing | ValueRef::Null) => Ordering::Equal,
        (ValueRef::Missing | ValueRef::Null, _) => nulls(Ordering::Less),
        (_, ValueRef::Missing | ValueRef::Null) => nulls(Ordering::Greater),

        (ValueRef::Bool(a), ValueRef::Bool(b)) => a.cmp(b),
        (ValueRef::Str(a), ValueRef::Str(b)) => a.cmp(b),
        (ValueRef::Bytes(a), ValueRef::Bytes(b)) => a.cmp(b),
        // Times and timestamps without a time zone first, then those with, in UTC
        (ValueRef::Date(a), ValueRef::Date(b)) => a.cmp(b),
        (ValueRef::Time(a), ValueRef::Time(b)) => a.key().cmp(&b.key()),
        (ValueRef::Timestamp(a), ValueRef::Timestamp(b)) => a.key().cmp(&b.key()),
        (ValueRef::List(_) | ValueRef::Tuple(_) | ValueRef::Bag(_), _) if same_type(a, b) => {
            // Nested values sort with the same NULL order; bags and tuples are put in a
            // canonical order first, which puts NULLs low
            cmp_collections(a, b, &|x, y| order_cmp(x, y, nulls_low), &|x, y| {
                order_cmp(x, y, true)
            })
        }
        _ => match (Number::of(a), Number::of(b)) {
            (Some(a), Some(b)) => a.cmp(b),
            _ => type_rank(a).cmp(&type_rank(b)),
        },
    }
}

/// A value as `eqg` compares it: its key. Values are `eqg` exactly when their keys compare
/// equal under `eqg_cmp`, which reads keys element by element. A key has:
/// - its Ion variants decoded, annotations dropped; one the VM has no type for (a timestamp)
///   as the canonical text of its Ion value, so that `2000-01-01T00:00Z` and
///   ` 2000-01-01T00:00+00:00` are one key;
/// - each bag's elements sorted, and each tuple's attributes sorted by name then value, leaving
///   out those whose value is MISSING (which are no attributes).
///
/// What differs from `value` is allocated in `arena`.
pub(crate) fn eqg_key<'a>(value: ValueRef<'a>, arena: &'a Arena) -> Result<ValueRef<'a>> {
    let keys = |items: &[ValueRef<'a>]| -> Result<Vec<ValueRef<'a>>> {
        items.iter().map(|item| eqg_key(*item, arena)).collect()
    };
    Ok(match variant::lower(value, arena)? {
        ValueRef::Variant(bytes, type_name) => {
            ValueRef::Variant(variant::canonical(bytes, arena)?, type_name)
        }
        ValueRef::List(items) if items.iter().all(is_key_scalar) => ValueRef::List(items),
        ValueRef::List(items) => ValueRef::List(arena.alloc_slice(&keys(items)?)),
        ValueRef::Bag(items) => {
            let mut items = keys(items)?;
            items.sort_by(eqg_cmp);
            ValueRef::Bag(arena.alloc_slice(&items))
        }
        ValueRef::Tuple(tuple) => {
            let mut fields = Vec::with_capacity(tuple.fields.len());
            for field in tuple.fields {
                if !matches!(field.value, ValueRef::Missing) {
                    fields.push(TupleField {
                        name: field.name,
                        value: eqg_key(field.value, arena)?,
                    });
                }
            }
            fields.sort_by(|x, y| x.name.cmp(y.name).then_with(|| eqg_cmp(&x.value, &y.value)));
            ValueRef::Tuple(arena.alloc_tuple_ref(TupleRef {
                fields: arena.alloc_slice(&fields),
            }))
        }
        value => value,
    })
}

/// A value that is its own key: neither a collection nor a variant.
fn is_key_scalar(value: &ValueRef<'_>) -> bool {
    !matches!(
        value,
        ValueRef::List(_) | ValueRef::Bag(_) | ValueRef::Tuple(_) | ValueRef::Variant(..)
    )
}

/// PartiQL's `=` (spec section 7.1): MISSING if either side is MISSING, else NULL if either is
/// NULL; numbers are equal by value whatever their type (exactly: `0.1e0` is not `0.1`), and NaN
/// equals nothing; lists, bags and tuples are equal when their contents are `eqg` (so a NULL in
/// one equals a NULL in the other, NaN equals NaN, and the result is never NULL); values of
/// different types are not equal. Ion variants are compared as the values they hold, keys made
/// in `scratch`.
pub(crate) fn eq<'a>(a: ValueRef<'a>, b: ValueRef<'a>, scratch: &'a Arena) -> Result<ValueRef<'a>> {
    EqOperand::new(a, scratch)?.eq(b, scratch)
}

/// The left side of `=`, decoded and (for a collection or tuple) keyed once, to compare with
/// many values (`IN`).
pub(crate) struct EqOperand<'a> {
    value: ValueRef<'a>,
    /// The `eqg` key of a list, bag, tuple or undecoded variant.
    key: Option<ValueRef<'a>>,
}

impl<'a> EqOperand<'a> {
    pub(crate) fn new(value: ValueRef<'a>, scratch: &'a Arena) -> Result<Self> {
        let value = variant::lower(value, scratch)?;
        let key = match has_key(&value) {
            true => Some(eqg_key(value, scratch)?),
            false => None,
        };
        Ok(EqOperand { value, key })
    }

    /// `self = other`.
    pub(crate) fn eq(&self, other: ValueRef<'a>, scratch: &'a Arena) -> Result<ValueRef<'a>> {
        let other = variant::lower(other, scratch)?;
        Ok(match (self.value, other) {
            (ValueRef::Missing, _) | (_, ValueRef::Missing) => ValueRef::Missing,
            (ValueRef::Null, _) | (_, ValueRef::Null) => ValueRef::Null,
            (ValueRef::Bool(a), ValueRef::Bool(b)) => ValueRef::Bool(a == b),
            (ValueRef::Str(a), ValueRef::Str(b)) => ValueRef::Bool(a == b),
            (ValueRef::Bytes(a), ValueRef::Bytes(b)) => ValueRef::Bool(a == b),
            // A time or timestamp with a time zone equals one without none
            (ValueRef::Date(a), ValueRef::Date(b)) => ValueRef::Bool(a == b),
            (ValueRef::Time(a), ValueRef::Time(b)) => {
                ValueRef::Bool(a.compare(b) == Some(Ordering::Equal))
            }
            (ValueRef::Timestamp(a), ValueRef::Timestamp(b)) => {
                ValueRef::Bool(a.compare(b) == Some(Ordering::Equal))
            }
            // A list's or bag's key has its elements, so lengths that differ settle it
            (ValueRef::List(a), ValueRef::List(b)) | (ValueRef::Bag(a), ValueRef::Bag(b))
                if a.len() != b.len() =>
            {
                ValueRef::Bool(false)
            }
            (ValueRef::List(_), ValueRef::List(_))
            | (ValueRef::Bag(_), ValueRef::Bag(_))
            | (ValueRef::Tuple(_), ValueRef::Tuple(_))
            | (ValueRef::Variant(..), ValueRef::Variant(..)) => {
                let key = self.key.expect("a collection's key");
                ValueRef::Bool(eqg_cmp(&key, &eqg_key(other, scratch)?).is_eq())
            }
            (a, b) => match number_cmp(&a, &b) {
                Some(order) => ValueRef::Bool(order == Some(Ordering::Equal)),
                None => ValueRef::Bool(false),
            },
        })
    }
}

/// The name of a value's type, for errors.
pub(crate) fn type_name(value: &ValueRef<'_>) -> &'static str {
    match value {
        ValueRef::Missing => "MISSING",
        ValueRef::Null => "NULL",
        ValueRef::Bool(_) => "bool",
        ValueRef::I64(_) => "int",
        ValueRef::F64(_) => "float",
        ValueRef::Decimal(_) => "decimal",
        ValueRef::Str(_) => "string",
        ValueRef::Bytes(_) => "blob",
        ValueRef::Date(_) => "date",
        ValueRef::Time(_) => "time",
        ValueRef::Timestamp(_) => "timestamp",
        ValueRef::List(_) => "list",
        ValueRef::Bag(_) => "bag",
        ValueRef::Tuple(_) => "tuple",
        ValueRef::Variant(..) => "Ion value",
    }
}

/// Whether `=` compares `value` by its `eqg` key.
fn has_key(value: &ValueRef<'_>) -> bool {
    matches!(
        value,
        ValueRef::List(_) | ValueRef::Bag(_) | ValueRef::Tuple(_) | ValueRef::Variant(..)
    )
}

/// How `<`, `<=`, `>` and `>=` see two values (spec section 7.1).
pub(crate) enum Order {
    /// Either is MISSING.
    Missing,
    /// Either is NULL (and neither MISSING).
    Null,
    /// Values that are ordered: numbers by value exactly whatever their type, strings by code
    /// point, booleans (false first), blobs by byte, dates, times and timestamps (with a time
    /// zone, as instants: in UTC) of the same kind, and two lists, bags or tuples by their `eqg`
    /// keys, so that `<=` is `<` or `=` however nested (NULL and MISSING apart, NaN equal to
    /// NaN and before every other number).
    Ordered(Ordering),
    /// A NaN: no order holds.
    Unordered,
    /// Values of types that don't compare (`1 < 'a'`): MISSING, or an error in strict mode.
    /// So are a date, a time and a timestamp of different kinds (or one with a time zone and
    /// one without), and an Ion value the VM has no type for, or a collection holding one.
    Incomparable,
}

/// The order of `a` and `b`, as `<` and the like see it. Ion variants are compared as the
/// values they hold, decoded in `scratch`.
pub(crate) fn order<'a>(a: ValueRef<'a>, b: ValueRef<'a>, scratch: &'a Arena) -> Result<Order> {
    let (a, b) = (variant::lower(a, scratch)?, variant::lower(b, scratch)?);
    Ok(match (a, b) {
        (ValueRef::Missing, _) | (_, ValueRef::Missing) => Order::Missing,
        (ValueRef::Null, _) | (_, ValueRef::Null) => Order::Null,
        (ValueRef::Bool(a), ValueRef::Bool(b)) => Order::Ordered(a.cmp(&b)),
        (ValueRef::Str(a), ValueRef::Str(b)) => Order::Ordered(a.cmp(b)),
        (ValueRef::Bytes(a), ValueRef::Bytes(b)) => Order::Ordered(a.cmp(b)),
        // A time or timestamp with a time zone and one without don't compare: there is no
        // session time zone to place the one without in
        (ValueRef::Date(a), ValueRef::Date(b)) => Order::Ordered(a.cmp(&b)),
        (ValueRef::Time(a), ValueRef::Time(b)) => {
            a.compare(b).map_or(Order::Incomparable, Order::Ordered)
        }
        (ValueRef::Timestamp(a), ValueRef::Timestamp(b)) => {
            a.compare(b).map_or(Order::Incomparable, Order::Ordered)
        }
        (ValueRef::List(_), ValueRef::List(_))
        | (ValueRef::Bag(_), ValueRef::Bag(_))
        | (ValueRef::Tuple(_), ValueRef::Tuple(_)) => {
            let (a, b) = (eqg_key(a, scratch)?, eqg_key(b, scratch)?);
            match variant::holds_variant(&a) || variant::holds_variant(&b) {
                true => Order::Incomparable,
                false => Order::Ordered(eqg_cmp(&a, &b)),
            }
        }
        (a, b) => match number_cmp(&a, &b) {
            Some(Some(order)) => Order::Ordered(order),
            Some(None) => Order::Unordered,
            None => Order::Incomparable,
        },
    })
}

/// Two numbers compared by value, exactly, whatever their types: `None` unless both are
/// numbers, and `Some(None)` when either is NaN, which is unordered (not equal, less or
/// greater than anything) as `=`, `<` and `>` see it.
pub(crate) fn number_cmp(a: &ValueRef<'_>, b: &ValueRef<'_>) -> Option<Option<Ordering>> {
    let (a, b) = (Number::of(a)?, Number::of(b)?);
    let is_nan = |n: Number| matches!(n, Number::Float(f) if f.is_nan());
    Some(match is_nan(a) || is_nan(b) {
        true => None,
        false => Some(a.cmp(b)),
    })
}

/// `eqg` as a total order of keys (`eqg_key`): the order GROUP BY sorts its keys in, equal
/// exactly when the values are `eqg` (a time or timestamp with a time zone is equal to one at
/// the same instant, whatever their offsets, and to none without one). Unlike `=`, NULL equals NULL and MISSING equals MISSING
/// (but not each other); numbers are equal by value whatever their type (`1`, `1.0` and
/// `1e0`), and NaN equals NaN; a tuple's attributes and a bag's elements compare in any order.
pub(crate) fn eqg_cmp(a: &ValueRef<'_>, b: &ValueRef<'_>) -> Ordering {
    match (a, b) {
        (ValueRef::Missing, ValueRef::Missing) | (ValueRef::Null, ValueRef::Null) => {
            Ordering::Equal
        }
        (ValueRef::Bool(a), ValueRef::Bool(b)) => a.cmp(b),
        (ValueRef::Str(a), ValueRef::Str(b)) => a.cmp(b),
        (ValueRef::Bytes(a), ValueRef::Bytes(b)) => a.cmp(b),
        (ValueRef::List(a), ValueRef::List(b)) | (ValueRef::Bag(a), ValueRef::Bag(b)) => {
            cmp_seq(a, b, &eqg_cmp)
        }
        (ValueRef::Tuple(a), ValueRef::Tuple(b)) => a
            .fields
            .iter()
            .zip(b.fields)
            .map(|(x, y)| x.name.cmp(y.name).then_with(|| eqg_cmp(&x.value, &y.value)))
            .find(|ord| ord.is_ne())
            .unwrap_or_else(|| a.fields.len().cmp(&b.fields.len())),
        (ValueRef::Date(a), ValueRef::Date(b)) => a.cmp(b),
        (ValueRef::Time(a), ValueRef::Time(b)) => a.key().cmp(&b.key()),
        (ValueRef::Timestamp(a), ValueRef::Timestamp(b)) => a.key().cmp(&b.key()),
        (ValueRef::Variant(a, a_type), ValueRef::Variant(b, b_type)) => {
            (a_type, a).cmp(&(b_type, b))
        }
        _ => match (Number::of(a), Number::of(b)) {
            (Some(a), Some(b)) => a.cmp(b),
            _ => eqg_rank(a).cmp(&eqg_rank(b)),
        },
    }
}

/// Whether two rows of keys are `eqg`, key by key.
pub(crate) fn eqg_rows(a: &[ValueRef<'_>], b: &[ValueRef<'_>]) -> bool {
    a.len() == b.len() && a.iter().zip(b).all(|(x, y)| eqg_cmp(x, y).is_eq())
}

/// `items` without duplicates under `eqg`, each first occurrence in order (for
/// `COLL_*(DISTINCT ...)`). The keys are made in `arena`.
pub(crate) fn distinct<'a>(items: &[ValueRef<'a>], arena: &'a Arena) -> Result<Vec<ValueRef<'a>>> {
    let hasher = EqgHasher::new();
    let mut seen: FxHashMap<u64, Vec<usize>> = FxHashMap::default();
    let (mut kept, mut kept_keys) = (Vec::new(), Vec::new());
    for item in items {
        let key = eqg_key(*item, arena)?;
        let indexes = seen.entry(hasher.hash(&key)).or_default();
        if !indexes
            .iter()
            .any(|&i| eqg_cmp(&kept_keys[i], &key).is_eq())
        {
            indexes.push(kept.len());
            kept.push(*item);
            kept_keys.push(key);
        }
    }
    Ok(kept)
}

/// A hash of keys (`eqg_key`) consistent with `eqg`: `eqg` values hash alike. It is keyed
/// afresh for each hasher, so a query's data can't be chosen to make its values collide.
pub(crate) struct EqgHasher {
    keys: RandomState,
}

impl EqgHasher {
    pub(crate) fn new() -> Self {
        EqgHasher {
            keys: RandomState::new(),
        }
    }

    pub(crate) fn hash(&self, key: &ValueRef<'_>) -> u64 {
        let mut state = self.keys.build_hasher();
        write_key(key, &mut state);
        state.finish()
    }

    /// The hash of a row of keys, in order.
    pub(crate) fn hash_row(&self, row: &[ValueRef<'_>]) -> u64 {
        let mut state = self.keys.build_hasher();
        for key in row {
            write_key(key, &mut state);
        }
        state.finish()
    }
}

fn write_key(key: &ValueRef<'_>, state: &mut impl Hasher) {
    match key {
        ValueRef::Missing => state.write_u8(0),
        ValueRef::Null => state.write_u8(1),
        ValueRef::Bool(b) => {
            state.write_u8(2);
            b.hash(state);
        }
        ValueRef::I64(_) | ValueRef::F64(_) | ValueRef::Decimal(_) => {
            state.write_u8(3);
            let number = Number::of(key).expect("a number");
            number.canonical().hash(state);
        }
        ValueRef::Str(s) => {
            state.write_u8(4);
            s.hash(state);
        }
        ValueRef::Bytes(b) => {
            state.write_u8(5);
            b.hash(state);
        }
        // A key's bags and tuples are sorted, so they hash in order like lists
        ValueRef::List(items) | ValueRef::Bag(items) => {
            state.write_u8(if matches!(key, ValueRef::List(_)) {
                6
            } else {
                7
            });
            state.write_usize(items.len());
            for item in *items {
                write_key(item, state);
            }
        }
        ValueRef::Tuple(tuple) => {
            state.write_u8(8);
            state.write_usize(tuple.fields.len());
            for field in tuple.fields {
                field.name.hash(state);
                write_key(&field.value, state);
            }
        }
        ValueRef::Date(d) => {
            state.write_u8(10);
            d.hash(state);
        }
        ValueRef::Time(t) => {
            state.write_u8(11);
            t.key().hash(state);
        }
        ValueRef::Timestamp(t) => {
            state.write_u8(12);
            t.key().hash(state);
        }
        ValueRef::Variant(bytes, type_name) => {
            state.write_u8(9);
            type_name.hash(state);
            bytes.hash(state);
        }
    }
}

/// Compares two values.
type Cmp<'c> = &'c dyn Fn(&ValueRef<'_>, &ValueRef<'_>) -> Ordering;

fn same_type(a: &ValueRef<'_>, b: &ValueRef<'_>) -> bool {
    std::mem::discriminant(a) == std::mem::discriminant(b)
}

/// Two lists, tuples or bags of the same type, their elements compared with `cmp`:
/// - lists element by element, a prefix first;
/// - bags as lists of their elements sorted by `canon`;
/// - tuples field by field, each tuple's fields sorted by name then value (by `canon`):
///   names first, then values.
fn cmp_collections(a: &ValueRef<'_>, b: &ValueRef<'_>, cmp: Cmp<'_>, canon: Cmp<'_>) -> Ordering {
    match (a, b) {
        (ValueRef::List(a), ValueRef::List(b)) => cmp_seq(a, b, cmp),
        (ValueRef::Bag(a), ValueRef::Bag(b)) => {
            cmp_seq(&sorted_bag(a, canon), &sorted_bag(b, canon), cmp)
        }
        (ValueRef::Tuple(a), ValueRef::Tuple(b)) => {
            let (a, b) = (sorted_fields(a, canon), sorted_fields(b, canon));
            a.iter()
                .zip(&b)
                .map(|((xn, xv), (yn, yv))| xn.cmp(yn).then_with(|| cmp(xv, yv)))
                .find(|ord| ord.is_ne())
                .unwrap_or_else(|| a.len().cmp(&b.len()))
        }
        _ => type_rank(a).cmp(&type_rank(b)),
    }
}

fn cmp_seq(a: &[ValueRef<'_>], b: &[ValueRef<'_>], cmp: Cmp<'_>) -> Ordering {
    a.iter()
        .zip(b)
        .map(|(x, y)| cmp(x, y))
        .find(|ord| ord.is_ne())
        .unwrap_or_else(|| a.len().cmp(&b.len()))
}

fn sorted_bag<'a>(bag: &[ValueRef<'a>], canon: Cmp<'_>) -> Vec<ValueRef<'a>> {
    let mut elems = bag.to_vec();
    elems.sort_by(|x, y| canon(x, y));
    elems
}

fn sorted_fields<'a>(tuple: &TupleRef<'a>, canon: Cmp<'_>) -> Vec<(&'a str, ValueRef<'a>)> {
    let mut fields: Vec<_> = tuple.fields.iter().map(|f| (f.name, f.value)).collect();
    fields.sort_by(|(xn, xv), (yn, yv)| xn.cmp(yn).then_with(|| canon(xv, yv)));
    fields
}

/// The rank of a non-NULL value's type in PartiQL's order. Numbers share a rank.
fn type_rank(v: &ValueRef<'_>) -> u8 {
    match v {
        ValueRef::Missing | ValueRef::Null => 0,
        ValueRef::Bool(_) => 1,
        ValueRef::I64(_) | ValueRef::F64(_) | ValueRef::Decimal(_) => 2,
        // Datetimes after numbers, as in the legacy evaluator
        ValueRef::Date(_) => 3,
        ValueRef::Time(_) => 4,
        ValueRef::Timestamp(_) => 5,
        ValueRef::Str(_) => 6,
        ValueRef::Bytes(_) => 7,
        ValueRef::List(_) => 8,
        ValueRef::Tuple(_) => 9,
        ValueRef::Bag(_) => 10,
        // Not decoded: an Ion variant orders after every other type
        ValueRef::Variant(_, _) => 11,
    }
}

/// The rank of a value's type under `eqg`: as in `type_rank`, but MISSING and NULL apart.
fn eqg_rank(v: &ValueRef<'_>) -> u8 {
    match v {
        ValueRef::Missing => 0,
        other => type_rank(other) + 1,
    }
}

/// A number, compared by value across types.
#[derive(Clone, Copy)]
enum Number {
    Int(i64),
    Float(f64),
    Decimal(Decimal),
}

/// A number written one way for each value, so that equal numbers hash alike: an integer
/// that fits an `i64` as one, else a decimal that holds it exactly, else a float.
#[derive(Hash)]
enum Canonical {
    Int(i64),
    /// A normalized decimal's mantissa and scale.
    Decimal(i128, u32),
    /// A float that is neither an integer nor a decimal: an infinity, NaN (one for all of
    /// them), or one whose exact value has more digits than a decimal holds.
    Float(u64),
}

impl Number {
    fn of(v: &ValueRef<'_>) -> Option<Self> {
        match v {
            ValueRef::I64(n) => Some(Number::Int(*n)),
            ValueRef::F64(n) => Some(Number::Float(*n)),
            ValueRef::Decimal(n) => Some(Number::Decimal(*n)),
            _ => None,
        }
    }

    /// Exact, so a total order: NaN sorts before every other number (then -inf, the finite
    /// numbers, +inf).
    fn cmp(self, other: Self) -> Ordering {
        match (self, other) {
            (Number::Int(a), Number::Int(b)) => a.cmp(&b),
            (Number::Decimal(a), Number::Decimal(b)) => a.cmp(&b),
            (Number::Float(a), Number::Float(b)) => cmp_floats(a, b),
            (Number::Int(a), Number::Decimal(b)) => Decimal::from(a).cmp(&b),
            (Number::Decimal(a), Number::Int(b)) => a.cmp(&Decimal::from(b)),
            // Every i64 is a decimal: `as f64` would round it
            (Number::Int(a), Number::Float(b)) => cmp_float_decimal(b, Decimal::from(a)).reverse(),
            (Number::Float(a), Number::Int(b)) => cmp_float_decimal(a, Decimal::from(b)),
            (Number::Float(a), Number::Decimal(b)) => cmp_float_decimal(a, b),
            (Number::Decimal(a), Number::Float(b)) => cmp_float_decimal(b, a).reverse(),
        }
    }

    /// Numbers `cmp` finds equal have the same canonical form.
    fn canonical(self) -> Canonical {
        match self {
            Number::Int(n) => Canonical::Int(n),
            Number::Decimal(d) => canonical_decimal(d),
            Number::Float(f) if f.is_nan() => Canonical::Float(f64::NAN.to_bits()),
            Number::Float(f) if f.is_infinite() => Canonical::Float(f.to_bits()),
            Number::Float(f) => match exact_decimal(f) {
                Some(d) => canonical_decimal(d),
                None => Canonical::Float(f.to_bits()),
            },
        }
    }
}

fn canonical_decimal(d: Decimal) -> Canonical {
    let d = d.normalize();
    match (d.scale(), i64::try_from(d.mantissa())) {
        (0, Ok(n)) => Canonical::Int(n),
        _ => Canonical::Decimal(d.mantissa(), d.scale()),
    }
}

/// The decimal equal to a finite float, if one is: a float is `m / 2^k` (`m` odd), whose
/// decimal digits are `m * 5^k / 10^k`, so it is one when `k` is at most a decimal's 28
/// digits of scale and `m * 5^k` fits its 96 bits.
fn exact_decimal(f: f64) -> Option<Decimal> {
    if f == 0.0 {
        return Some(Decimal::ZERO);
    }
    let bits = f.to_bits();
    let biased_exp = ((bits >> 52) & 0x7ff) as i32;
    let fraction = bits & ((1 << 52) - 1);
    let (mut mant, mut exp) = match biased_exp {
        0 => (fraction as u128, -1074),
        _ => ((fraction | 1 << 52) as u128, biased_exp - 1075),
    };
    while mant % 2 == 0 && exp < 0 {
        mant /= 2;
        exp += 1;
    }
    const DECIMAL_MAX: u128 = (1 << 96) - 1;
    let (digits, scale) = if exp >= 0 {
        // `mant < 2^53`, so it fits 96 bits shifted by at most 43
        if exp > 43 {
            return None;
        }
        (mant << exp, 0)
    } else {
        let scale = u32::try_from(-exp).ok().filter(|&s| s <= 28)?;
        // `5^28 < 2^66`, so this fits 128 bits
        (mant * 5u128.pow(scale), scale)
    };
    if digits > DECIMAL_MAX {
        return None;
    }
    let signed = i128::try_from(digits).ok()?;
    let signed = if f < 0.0 { -signed } else { signed };
    Decimal::try_from_i128_with_scale(signed, scale).ok()
}

/// NaN first; otherwise IEEE order (-0 equals 0).
fn cmp_floats(a: f64, b: f64) -> Ordering {
    match (a.is_nan(), b.is_nan()) {
        (true, true) => Ordering::Equal,
        (true, false) => Ordering::Less,
        (false, true) => Ordering::Greater,
        (false, false) => a.partial_cmp(&b).unwrap_or(Ordering::Equal),
    }
}

/// Compare a float with a decimal exactly. Converting either to the other's type rounds,
/// which breaks transitivity (and makes `sort_by` panic).
fn cmp_float_decimal(a: f64, b: Decimal) -> Ordering {
    if a.is_nan() || a == f64::NEG_INFINITY {
        return Ordering::Less;
    }
    if a == f64::INFINITY {
        return Ordering::Greater;
    }
    let sign = |negative: bool, zero: bool| match (zero, negative) {
        (true, _) => 0,
        (false, true) => -1,
        (false, false) => 1,
    };
    let (sign_a, sign_b) = (
        sign(a < 0.0, a == 0.0),
        sign(b.is_sign_negative(), b.is_zero()),
    );
    if sign_a != sign_b {
        return sign_a.cmp(&sign_b);
    }
    let magnitude = cmp_magnitudes(a.abs(), b.mantissa().unsigned_abs(), b.scale());
    if sign_a < 0 {
        magnitude.reverse()
    } else {
        magnitude
    }
}

/// Compare a finite `x >= 0` with `m / 10^scale` (a decimal's magnitude: `m < 2^96`,
/// `scale <= 28`) exactly, as `mant * 2^exp * 10^scale` against `m`.
fn cmp_magnitudes(x: f64, m: u128, scale: u32) -> Ordering {
    let bits = x.to_bits();
    let biased_exp = ((bits >> 52) & 0x7ff) as i32;
    let fraction = bits & ((1 << 52) - 1);
    let (mant, exp) = match biased_exp {
        0 => (fraction as u128, -1074),
        _ => ((fraction | 1 << 52) as u128, biased_exp - 1075),
    };
    let pow10 = 10u128.pow(scale);
    let bit_len = |n: u128| 128 - n.leading_zeros() as i32;
    if exp >= 0 {
        // `mant * 2^exp` needs `bit_len(mant) + exp` bits; `m` fewer than 97
        if bit_len(mant) + exp > 127 {
            return Ordering::Greater;
        }
        match (mant << exp).checked_mul(pow10) {
            Some(lhs) => lhs.cmp(&m),
            None => Ordering::Greater,
        }
    } else {
        // `mant * 10^scale` against `m * 2^-exp`, in 256 bits
        let k = -exp;
        if m == 0 {
            return mant.cmp(&0);
        }
        if bit_len(m) + k > 256 {
            return Ordering::Less;
        }
        mul_wide(mant, pow10).cmp(&shl_wide(m, k as u32))
    }
}

/// `a * b` as (high, low) 128-bit words, for `a < 2^64`.
fn mul_wide(a: u128, b: u128) -> (u128, u128) {
    let (b_hi, b_lo) = (b >> 64, b & u64::MAX as u128);
    let (lo, hi) = (a * b_lo, a * b_hi);
    let (low, carry) = lo.overflowing_add(hi << 64);
    ((hi >> 64) + carry as u128, low)
}

/// `m << k` as (high, low) 128-bit words, for a result under `2^256`.
fn shl_wide(m: u128, k: u32) -> (u128, u128) {
    match k {
        0 => (0, m),
        1..=127 => (m >> (128 - k), m << k),
        _ => (m << (k - 128), 0),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::arena::Arena;
    use std::str::FromStr;

    fn dec(s: &str) -> ValueRef<'static> {
        ValueRef::Decimal(Decimal::from_str(s).unwrap())
    }

    /// Numbers near where conversions round: 2^53 (f64's integer precision), decimals
    /// that floats approximate, and the ends of each type's range.
    fn numbers() -> Vec<ValueRef<'static>> {
        let mut v = vec![
            ValueRef::I64(9_007_199_254_740_992),
            ValueRef::I64(9_007_199_254_740_993),
            ValueRef::F64(9_007_199_254_740_992.0),
            dec("9007199254740992.5"),
            ValueRef::I64(i64::MAX),
            ValueRef::I64(i64::MIN),
            ValueRef::F64(i64::MAX as f64),
            ValueRef::F64(i64::MIN as f64),
            dec("9223372036854775808"),
            dec("-9223372036854775808"),
            ValueRef::F64(0.1),
            dec("0.1"),
            dec("0.1000000000000000055511151231"),
            ValueRef::F64(0.5),
            dec("0.5"),
            dec("0.50"),
            ValueRef::F64(1e-300),
            ValueRef::F64(-1e-300),
            dec("0.0000000000000000000000000001"),
            ValueRef::F64(1e20),
            dec("100000000000000000000"),
            ValueRef::F64(1e300),
            ValueRef::F64(-1e300),
            ValueRef::Decimal(Decimal::MAX),
            ValueRef::Decimal(Decimal::MIN),
            ValueRef::F64(0.0),
            ValueRef::F64(-0.0),
            ValueRef::I64(0),
            dec("0"),
            dec("-0.0"),
            dec("1.0"),
            ValueRef::F64(1.0),
            dec("-1.5"),
            ValueRef::F64(-1.5),
            ValueRef::F64(f64::NAN),
            ValueRef::F64(-f64::NAN),
            ValueRef::F64(f64::INFINITY),
            ValueRef::F64(f64::NEG_INFINITY),
        ];
        v.extend((-3..=3).map(ValueRef::I64));
        v
    }

    /// Values of every type, numbers among them, nested in collections and tuples.
    fn values(arena: &Arena) -> Vec<ValueRef<'_>> {
        use crate::value::TupleField;
        let tuple = |fields: &[(&'static str, ValueRef<'static>)]| {
            let fields: Vec<_> = fields
                .iter()
                .map(|&(name, value)| TupleField { name, value })
                .collect();
            ValueRef::Tuple(arena.alloc_tuple_ref(TupleRef {
                fields: arena.alloc_slice(&fields),
            }))
        };
        let one = [ValueRef::I64(1), ValueRef::F64(1.0), dec("1.00")];
        let mut v = numbers();
        v.extend([
            ValueRef::Missing,
            ValueRef::Null,
            ValueRef::Bool(true),
            ValueRef::Bool(false),
            ValueRef::Str("a"),
            ValueRef::Str("b"),
            ValueRef::Bytes(b"a"),
            ValueRef::Variant(b"2000T", "Ion"),
            ValueRef::Variant(b"2000-01-01T", "Ion"),
        ]);
        for n in one {
            v.push(ValueRef::List(arena.alloc_slice(&[n, ValueRef::Null])));
            v.push(ValueRef::List(arena.alloc_slice(&[ValueRef::Null, n])));
            v.push(ValueRef::Bag(arena.alloc_slice(&[n, ValueRef::Null])));
            v.push(ValueRef::Bag(arena.alloc_slice(&[ValueRef::Null, n])));
            v.push(ValueRef::Bag(arena.alloc_slice(&[n, ValueRef::Missing])));
            v.push(tuple(&[("a", n), ("b", ValueRef::Null)]));
            v.push(tuple(&[("b", ValueRef::Null), ("a", n)]));
            v.push(tuple(&[("a", n), ("b", ValueRef::Missing)]));
            v.push(tuple(&[("a", n)]));
            v.push(tuple(&[("a", n), ("a", ValueRef::Str("x"))]));
            v.push(tuple(&[("a", ValueRef::Str("x")), ("a", n)]));
        }
        v.push(ValueRef::List(&[]));
        v.push(ValueRef::Bag(&[]));
        v.push(tuple(&[]));
        v
    }

    fn cmp(a: &ValueRef<'_>, b: &ValueRef<'_>) -> Ordering {
        order_cmp(a, b, true)
    }

    fn keys<'a>(values: &[ValueRef<'a>], arena: &'a Arena) -> Vec<ValueRef<'a>> {
        values
            .iter()
            .map(|v| eqg_key(*v, arena).expect("valid Ion"))
            .collect()
    }

    fn eqg(a: &ValueRef<'_>, b: &ValueRef<'_>) -> bool {
        let arena = Arena::default();
        let (a, b) = (eqg_key(*a, &arena).unwrap(), eqg_key(*b, &arena).unwrap());
        eqg_cmp(&a, &b).is_eq()
    }

    #[test]
    fn number_order_is_total() {
        let v = numbers();
        for a in &v {
            for b in &v {
                assert_eq!(cmp(a, b), cmp(b, a).reverse(), "{a:?} vs {b:?}");
                for c in &v {
                    if cmp(a, b).is_le() && cmp(b, c).is_le() {
                        assert!(cmp(a, c).is_le(), "{a:?} <= {b:?} <= {c:?}");
                    }
                }
            }
        }
    }

    #[test]
    fn numbers_compare_exactly() {
        let lt = |a: ValueRef<'_>, b: ValueRef<'_>| assert_eq!(cmp(&a, &b), Ordering::Less);
        let eq = |a: ValueRef<'_>, b: ValueRef<'_>| assert_eq!(cmp(&a, &b), Ordering::Equal);
        lt(
            ValueRef::F64(9_007_199_254_740_992.0),
            ValueRef::I64(9_007_199_254_740_993),
        );
        eq(
            ValueRef::F64(9_007_199_254_740_992.0),
            ValueRef::I64(9_007_199_254_740_992),
        );
        // 0.1 as a float is 0.1000000000000000055511151231257827…
        lt(dec("0.1"), ValueRef::F64(0.1));
        lt(ValueRef::F64(0.1), dec("0.1000000000000000055511151232"));
        lt(ValueRef::F64(0.0), ValueRef::F64(1e-300));
        lt(ValueRef::F64(1e-300), dec("0.0000000000000000000000000001"));
        lt(ValueRef::Decimal(Decimal::MAX), ValueRef::F64(1e300));
        lt(ValueRef::F64(f64::NAN), ValueRef::F64(f64::NEG_INFINITY));
        lt(ValueRef::F64(f64::NEG_INFINITY), ValueRef::I64(i64::MIN));
        eq(ValueRef::F64(-0.0), dec("0"));
        eq(ValueRef::F64(-1.5), dec("-1.5"));
    }

    #[test]
    fn eqg_order_is_total() {
        let arena = Arena::default();
        let v = keys(&values(&arena), &arena);
        for a in &v {
            for b in &v {
                assert_eq!(eqg_cmp(a, b), eqg_cmp(b, a).reverse(), "{a:?} vs {b:?}");
                for c in &v {
                    if eqg_cmp(a, b).is_le() && eqg_cmp(b, c).is_le() {
                        assert!(eqg_cmp(a, c).is_le(), "{a:?} <= {b:?} <= {c:?}");
                    }
                }
            }
        }
    }

    #[test]
    fn eqg_values_hash_alike() {
        let arena = Arena::default();
        let v = keys(&values(&arena), &arena);
        let hasher = EqgHasher::new();
        for a in &v {
            for b in &v {
                if eqg_cmp(a, b).is_eq() {
                    assert_eq!(hasher.hash(a), hasher.hash(b), "{a:?} eqg {b:?}");
                }
            }
        }
    }

    #[test]
    fn eqg_tells_values_apart_as_the_spec_does() {
        let arena = Arena::default();
        let same = |a: ValueRef<'_>, b: ValueRef<'_>| assert!(eqg(&a, &b), "{a:?} eqg {b:?}");
        let apart = |a: ValueRef<'_>, b: ValueRef<'_>| assert!(!eqg(&a, &b), "{a:?} vs {b:?}");
        same(ValueRef::I64(1), ValueRef::F64(1.0));
        same(ValueRef::I64(1), dec("1.00"));
        same(ValueRef::F64(0.5), dec("0.50"));
        same(ValueRef::F64(1e20), dec("100000000000000000000"));
        same(ValueRef::F64(f64::NAN), ValueRef::F64(-f64::NAN));
        same(ValueRef::F64(-0.0), ValueRef::I64(0));
        same(ValueRef::Null, ValueRef::Null);
        same(ValueRef::Missing, ValueRef::Missing);
        apart(ValueRef::Null, ValueRef::Missing);
        apart(ValueRef::F64(0.1), dec("0.1"));
        apart(ValueRef::I64(1), ValueRef::Str("1"));
        apart(ValueRef::Bool(true), ValueRef::I64(1));
        let bag = |items: &[ValueRef<'static>]| ValueRef::Bag(arena.alloc_slice(items));
        let list = |items: &[ValueRef<'static>]| ValueRef::List(arena.alloc_slice(items));
        same(
            bag(&[ValueRef::I64(1), ValueRef::I64(2)]),
            bag(&[ValueRef::F64(2.0), ValueRef::I64(1)]),
        );
        apart(
            list(&[ValueRef::I64(1), ValueRef::I64(2)]),
            list(&[ValueRef::I64(2), ValueRef::I64(1)]),
        );
        apart(list(&[ValueRef::I64(1)]), bag(&[ValueRef::I64(1)]));
        apart(list(&[ValueRef::Null]), list(&[ValueRef::Missing]));
        // An Ion value is the value it holds, its annotations dropped; one the VM has no type
        // for is its Ion value, however written
        let ion = |text: &'static str| ValueRef::Variant(text.as_bytes(), "Ion");
        same(ion("a::1"), ValueRef::I64(1));
        same(
            ion("[a::1, 2e0]"),
            list(&[ValueRef::I64(1), ValueRef::I64(2)]),
        );
        same(ion("2000-01-01T00:00Z"), ion(" 2000-01-01T00:00+00:00"));
        same(ion("2000-01-01T00:00Z"), ion("a::2000-01-01T00:00Z"));
        same(
            list(&[ion("2000-01-01T00:00Z")]),
            ion("[2000-01-01T00:00Z]"),
        );
        apart(ion("2000-01-01T00:00Z"), ion("2000-01-01T00:01Z"));
    }

    /// `=` of collections and tuples agrees with `eqg` of them, over every pair of values; at
    /// the top NULL and MISSING propagate, and NaN equals nothing.
    #[test]
    fn eq_of_collections_is_eqg() {
        let arena = Arena::default();
        let v = values(&arena);
        for a in &v {
            for b in &v {
                let result = eq(*a, *b, &arena).unwrap();
                let expected = match (a, b) {
                    (ValueRef::Missing, _) | (_, ValueRef::Missing) => ValueRef::Missing,
                    (ValueRef::Null, _) | (_, ValueRef::Null) => ValueRef::Null,
                    _ if [a, b]
                        .iter()
                        .any(|v| matches!(v, ValueRef::F64(f) if f.is_nan())) =>
                    {
                        ValueRef::Bool(false)
                    }
                    _ => ValueRef::Bool(
                        super::eqg_cmp(
                            &eqg_key(*a, &arena).unwrap(),
                            &eqg_key(*b, &arena).unwrap(),
                        )
                        .is_eq(),
                    ),
                };
                assert_eq!(
                    format!("{result:?}"),
                    format!("{expected:?}"),
                    "{a:?} = {b:?}"
                );
            }
        }
    }

    #[test]
    fn distinct_keeps_first_occurrences_in_order() {
        let items = [
            ValueRef::I64(2),
            ValueRef::F64(1.0),
            ValueRef::Null,
            ValueRef::I64(1),
            dec("2.0"),
            ValueRef::Missing,
            ValueRef::Null,
            ValueRef::Str("a"),
        ];
        let arena = Arena::default();
        let kept = distinct(&items, &arena).unwrap();
        assert_eq!(
            format!("{kept:?}"),
            r#"[I64(2), F64(1.0), Null, Missing, Str("a")]"#
        );
    }
}
