use crate::engine::value::ValueRef;

/// A record stored in the sorter. Fields are ValueRefs pointing into the sorter's bank.
pub(crate) struct SorterRecord {
    pub fields: Vec<ValueRef<'static>>,
}

/// Sort-based grouping structure. Accumulates records during the insert phase,
/// sorts them by key fields, then iterates in sorted order for group boundary detection.
pub(crate) struct Sorter {
    /// Which bank in the VM this sorter's data lives in.
    pub bank_id: usize,
    /// All inserted records (unsorted until `sort()` is called).
    records: Vec<SorterRecord>,
    /// Number of leading fields that form the sort key.
    key_count: usize,
    /// Whether `sort()` has been called.
    sorted: bool,
    /// Current position during iteration (after sort).
    position: usize,
}

impl Sorter {
    pub fn new(bank_id: usize, key_count: usize) -> Self {
        Sorter {
            bank_id,
            records: Vec::new(),
            key_count,
            sorted: false,
            position: 0,
        }
    }

    /// Insert a record. The fields must already point into this sorter's bank.
    pub fn insert(&mut self, record: SorterRecord) {
        self.records.push(record);
    }

    /// Sort all records by the first `key_count` fields.
    /// Returns false if the sorter is empty.
    pub fn sort(&mut self) -> bool {
        if self.records.is_empty() {
            return false;
        }
        let key_count = self.key_count;
        self.records.sort_by(|a, b| {
            for i in 0..key_count {
                let cmp = compare_value_ref(&a.fields[i], &b.fields[i]);
                if cmp != std::cmp::Ordering::Equal {
                    return cmp;
                }
            }
            std::cmp::Ordering::Equal
        });
        self.sorted = true;
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

/// Compare two ValueRefs for ordering. Used by the sorter for sort-based grouping.
/// MISSING and NULL are treated as equal for grouping purposes (SQL semantics).
fn compare_value_ref(a: &ValueRef<'_>, b: &ValueRef<'_>) -> std::cmp::Ordering {
    use std::cmp::Ordering;
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
