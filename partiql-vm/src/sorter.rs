use crate::compare::eqg_cmp;
use crate::plan::SortKeys;
use crate::value::ValueRef;
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

    /// How many of a record's leading fields are its keys.
    pub fn key_fields(&self) -> usize {
        match &self.keys {
            SortKeys::Group(keys) => *keys,
            SortKeys::Order(orders) => orders.len(),
        }
    }

    /// How many of a record's leading fields are read as values (keys, aggregate inputs),
    /// so that an Ion variant in them is decoded on insert: all of GROUP BY's, ORDER BY's
    /// keys but not its payload, which is copied back to the rows unchanged.
    pub fn decoded_fields(&self) -> usize {
        match &self.keys {
            SortKeys::Group(_) => usize::MAX,
            SortKeys::Order(orders) => orders.len(),
        }
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
                        .map(|i| eqg_cmp(&a.fields[i], &b.fields[i]))
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

    /// Every record's fields, in insertion order unless sorted since.
    pub fn records(&self) -> impl Iterator<Item = &[ValueRef<'static>]> {
        self.records.iter().map(|r| r.fields.as_slice())
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
