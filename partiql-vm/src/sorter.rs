use crate::plan::{SortKeys, SortOrder};
use crate::value::{TupleRef, ValueRef};
use rust_decimal::prelude::{FromPrimitive, ToPrimitive};
use rust_decimal::Decimal;
use std::cmp::Ordering;

/// A record stored in the sorter. Fields are ValueRefs pointing into the sorter's bank.
pub(crate) struct SorterRecord {
    pub fields: Vec<ValueRef<'static>>,
}

/// Sort-based grouping and ordering structure. Accumulates records during the insert
/// phase, sorts them by their key fields, then iterates in sorted order.
pub(crate) struct Sorter {
    /// Which bank in the VM this sorter's data lives in.
    pub bank_id: usize,
    /// All inserted records (unsorted until `sort()` is called).
    records: Vec<SorterRecord>,
    /// How the leading fields order the records.
    keys: SortKeys,
    /// Current position during iteration (after sort).
    position: usize,
}

impl Sorter {
    pub fn new(bank_id: usize, keys: SortKeys) -> Self {
        Sorter {
            bank_id,
            records: Vec::new(),
            keys,
            position: 0,
        }
    }

    /// Drop the records of a previous run, so that an operator can run again.
    ///
    /// TODO: the bank is not reset (registers may still point into it), so each run's
    /// values stay allocated until the VM is dropped.
    pub fn reset(&mut self) {
        self.records.clear();
        self.position = 0;
    }

    /// Insert a record. The fields must already point into this sorter's bank.
    pub fn insert(&mut self, record: SorterRecord) {
        self.records.push(record);
    }

    /// Sort all records by their key fields. The sort is stable: records with equal keys
    /// keep their insertion order. Returns false if the sorter is empty.
    pub fn sort(&mut self) -> bool {
        if self.records.is_empty() {
            return false;
        }
        match &self.keys {
            SortKeys::Group(key_count) => {
                let key_count = *key_count;
                self.records.sort_by(|a, b| {
                    (0..key_count)
                        .map(|i| compare_value_ref(&a.fields[i], &b.fields[i]))
                        .find(|cmp| cmp.is_ne())
                        .unwrap_or(Ordering::Equal)
                });
            }
            SortKeys::Order(orders) => {
                self.records.sort_by(|a, b| {
                    orders
                        .iter()
                        .enumerate()
                        .map(|(i, order)| order.compare(&a.fields[i], &b.fields[i]))
                        .find(|cmp| cmp.is_ne())
                        .unwrap_or(Ordering::Equal)
                });
            }
        }
        self.position = 0;
        true
    }

    /// Get the current record's fields. Returns None if positioned past end.
    pub fn current(&self) -> Option<&[ValueRef<'static>]> {
        if self.position < self.records.len() {
            Some(&self.records[self.position].fields)
        } else {
            None
        }
    }

    /// Advance to the next record. Returns true if there is a next record.
    pub fn advance(&mut self) -> bool {
        self.position += 1;
        self.position < self.records.len()
    }
}

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
        (ValueRef::List(a), ValueRef::List(b)) => cmp_seq(a, b, nulls_low),
        (ValueRef::Tuple(a), ValueRef::Tuple(b)) => cmp_tuples(a, b, nulls_low),
        (ValueRef::Bag(a), ValueRef::Bag(b)) => {
            // Bags compare as lists of their sorted elements
            cmp_seq(&sorted_bag(a), &sorted_bag(b), nulls_low)
        }
        _ => match (Number::of(a), Number::of(b)) {
            (Some(a), Some(b)) => a.cmp(b),
            _ => type_rank(a).cmp(&type_rank(b)),
        },
    }
}

/// Element by element; a prefix sorts first.
fn cmp_seq(a: &[ValueRef<'_>], b: &[ValueRef<'_>], nulls_low: bool) -> Ordering {
    a.iter()
        .zip(b)
        .map(|(x, y)| order_cmp(x, y, nulls_low))
        .find(|cmp| cmp.is_ne())
        .unwrap_or_else(|| a.len().cmp(&b.len()))
}

/// Field by field, each tuple's fields sorted by name then value: names first, then values.
fn cmp_tuples(a: &TupleRef<'_>, b: &TupleRef<'_>, nulls_low: bool) -> Ordering {
    let (a, b) = (sorted_fields(a), sorted_fields(b));
    a.iter()
        .zip(&b)
        .map(|((xn, xv), (yn, yv))| xn.cmp(yn).then_with(|| order_cmp(xv, yv, nulls_low)))
        .find(|cmp| cmp.is_ne())
        .unwrap_or_else(|| a.len().cmp(&b.len()))
}

fn sorted_bag<'a>(bag: &[ValueRef<'a>]) -> Vec<ValueRef<'a>> {
    let mut elems = bag.to_vec();
    elems.sort_by(|x, y| order_cmp(x, y, true));
    elems
}

fn sorted_fields<'a>(tuple: &TupleRef<'a>) -> Vec<(&'a str, ValueRef<'a>)> {
    let mut fields: Vec<_> = tuple.fields.iter().map(|f| (f.name, f.value)).collect();
    fields.sort_by(|(xn, xv), (yn, yv)| xn.cmp(yn).then_with(|| order_cmp(xv, yv, true)));
    fields
}

/// The rank of a non-NULL value's type in PartiQL's order. Numbers share a rank.
fn type_rank(v: &ValueRef<'_>) -> u8 {
    match v {
        ValueRef::Missing | ValueRef::Null => 0,
        ValueRef::Bool(_) => 1,
        ValueRef::I64(_) | ValueRef::F64(_) | ValueRef::Decimal(_) => 2,
        ValueRef::Str(_) => 3,
        ValueRef::Bytes(_) => 4,
        ValueRef::List(_) => 5,
        ValueRef::Tuple(_) => 6,
        ValueRef::Bag(_) => 7,
    }
}

/// A number, compared by value across types.
#[derive(Clone, Copy)]
enum Number {
    Int(i64),
    Float(f64),
    Decimal(Decimal),
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

    /// NaN sorts before every other number (then -inf, the finite numbers, +inf).
    fn cmp(self, other: Self) -> Ordering {
        match (self, other) {
            (Number::Int(a), Number::Int(b)) => a.cmp(&b),
            (Number::Decimal(a), Number::Decimal(b)) => a.cmp(&b),
            (Number::Float(a), Number::Float(b)) => cmp_floats(a, b),
            (Number::Int(a), Number::Float(b)) => cmp_floats(a as f64, b),
            (Number::Float(a), Number::Int(b)) => cmp_floats(a, b as f64),
            (Number::Int(a), Number::Decimal(b)) => Decimal::from(a).cmp(&b),
            (Number::Decimal(a), Number::Int(b)) => a.cmp(&Decimal::from(b)),
            (Number::Float(a), Number::Decimal(b)) => cmp_float_decimal(a, b),
            (Number::Decimal(a), Number::Float(b)) => cmp_float_decimal(b, a).reverse(),
        }
    }
}

fn cmp_floats(a: f64, b: f64) -> Ordering {
    match (a.is_nan(), b.is_nan()) {
        (true, true) => Ordering::Equal,
        (true, false) => Ordering::Less,
        (false, true) => Ordering::Greater,
        (false, false) => a.partial_cmp(&b).unwrap_or(Ordering::Equal),
    }
}

fn cmp_float_decimal(a: f64, b: Decimal) -> Ordering {
    if a.is_nan() || a == f64::NEG_INFINITY {
        Ordering::Less
    } else if a == f64::INFINITY {
        Ordering::Greater
    } else {
        match Decimal::from_f64(a) {
            Some(a) => a.cmp(&b),
            // Out of the decimal range: compare as floats
            None => cmp_floats(a, b.to_f64().unwrap_or(f64::NAN)),
        }
    }
}

/// Compare two ValueRefs for ordering. Used by the sorter for sort-based grouping.
/// MISSING and NULL are treated as equal for grouping purposes (SQL semantics).
fn compare_value_ref(a: &ValueRef<'_>, b: &ValueRef<'_>) -> std::cmp::Ordering {
    match (a, b) {
        // MISSING and NULL are equivalent for GROUP BY
        (ValueRef::Missing | ValueRef::Null, ValueRef::Missing | ValueRef::Null) => Ordering::Equal,
        (ValueRef::Missing | ValueRef::Null, _) => Ordering::Less,
        (_, ValueRef::Missing | ValueRef::Null) => Ordering::Greater,

        (ValueRef::Bool(a), ValueRef::Bool(b)) => a.cmp(b),
        (ValueRef::I64(a), ValueRef::I64(b)) => a.cmp(b),
        (ValueRef::F64(a), ValueRef::F64(b)) => a.partial_cmp(b).unwrap_or(Ordering::Equal),
        (ValueRef::Decimal(a), ValueRef::Decimal(b)) => a.cmp(b),
        (ValueRef::Str(a), ValueRef::Str(b)) => a.cmp(b),
        (ValueRef::Bytes(a), ValueRef::Bytes(b)) => a.cmp(b),

        // Cross-type: order by type discriminant
        _ => type_ordinal(a).cmp(&type_ordinal(b)),
    }
}

/// Assign a numeric ordinal to each type for cross-type comparison ordering.
fn type_ordinal(v: &ValueRef<'_>) -> u8 {
    match v {
        ValueRef::Missing => 0,
        ValueRef::Null => 1,
        ValueRef::Bool(_) => 2,
        ValueRef::I64(_) => 3,
        ValueRef::F64(_) => 4,
        ValueRef::Decimal(_) => 5,
        ValueRef::Str(_) => 6,
        ValueRef::Bytes(_) => 7,
        ValueRef::Tuple(_) => 8,
        ValueRef::List(_) => 9,
        ValueRef::Bag(_) => 10,
    }
}

/// Check if two ValueRefs are equal (for group boundary detection).
pub(crate) fn value_ref_eq(a: &ValueRef<'_>, b: &ValueRef<'_>) -> bool {
    compare_value_ref(a, b) == std::cmp::Ordering::Equal
}
