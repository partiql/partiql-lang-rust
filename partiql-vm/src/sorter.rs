use crate::plan::{SortKeys, SortOrder};
use crate::value::{TupleRef, ValueRef};
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

    /// Drop the records of a previous run, so that an operator can run again. The caller
    /// resets the sorter's bank with it.
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
        // A total order (NaN equals NaN), as `sort_by` requires
        (ValueRef::F64(a), ValueRef::F64(b)) => cmp_floats(*a, *b),
        (ValueRef::Decimal(a), ValueRef::Decimal(b)) => a.cmp(b),
        (ValueRef::Str(a), ValueRef::Str(b)) => a.cmp(b),
        (ValueRef::Bytes(a), ValueRef::Bytes(b)) => a.cmp(b),
        (ValueRef::List(_) | ValueRef::Tuple(_) | ValueRef::Bag(_), _) if same_type(a, b) => {
            cmp_collections(a, b, &compare_value_ref, &compare_value_ref)
        }

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

#[cfg(test)]
mod tests {
    use super::*;
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
            ValueRef::F64(0.1),
            dec("0.1"),
            dec("0.1000000000000000055511151231"),
            ValueRef::F64(1e-300),
            ValueRef::F64(-1e-300),
            dec("0.0000000000000000000000000001"),
            ValueRef::F64(1e300),
            ValueRef::F64(-1e300),
            ValueRef::Decimal(Decimal::MAX),
            ValueRef::Decimal(Decimal::MIN),
            ValueRef::F64(0.0),
            ValueRef::F64(-0.0),
            ValueRef::I64(0),
            dec("0"),
            dec("-1.5"),
            ValueRef::F64(-1.5),
            ValueRef::F64(f64::NAN),
            ValueRef::F64(f64::INFINITY),
            ValueRef::F64(f64::NEG_INFINITY),
        ];
        v.extend((-3..=3).map(ValueRef::I64));
        v
    }

    fn cmp(a: &ValueRef<'_>, b: &ValueRef<'_>) -> Ordering {
        order_cmp(a, b, true)
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
    fn group_order_is_total_for_nan() {
        let v = [f64::NAN, 1.0, f64::NAN, 2.0, f64::NEG_INFINITY].map(ValueRef::F64);
        for a in &v {
            for b in &v {
                assert_eq!(
                    compare_value_ref(a, b),
                    compare_value_ref(b, a).reverse(),
                    "{a:?} vs {b:?}"
                );
            }
        }
        assert!(value_ref_eq(
            &ValueRef::F64(f64::NAN),
            &ValueRef::F64(f64::NAN)
        ));
        assert!(!value_ref_eq(&ValueRef::F64(f64::NAN), &ValueRef::F64(1.0)));
    }
}
