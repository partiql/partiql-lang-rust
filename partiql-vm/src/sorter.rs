use crate::compare::{eqg_cmp, eqg_rows};
use crate::plan::SortKeys;
use crate::value::ValueRef;
use std::cmp::Ordering;

/// A record stored in the sorter. Fields are ValueRefs pointing into the sorter's bank.
pub(crate) struct SorterRecord {
    pub fields: Vec<ValueRef<'static>>,
    /// GROUP BY: the `eqg` keys of the leading `Group(n)` fields, which the records are sorted
    /// and grouped by, while the fields keep the values as written. Empty for ORDER BY.
    pub group_keys: Vec<ValueRef<'static>>,
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

    /// How many of a record's leading fields are GROUP BY keys, compared by their `eqg` keys.
    pub fn group_key_fields(&self) -> usize {
        match &self.keys {
            SortKeys::Group(keys) => *keys,
            SortKeys::Order(_) => 0,
        }
    }

    /// How many of a record's leading fields are ORDER BY keys, compared as the values they
    /// hold however nested, so that their Ion variants are decoded all the way down.
    pub fn order_key_fields(&self) -> usize {
        match &self.keys {
            SortKeys::Group(_) => 0,
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
            SortKeys::Group(_) => {
                self.records.sort_by(|a, b| {
                    a.group_keys
                        .iter()
                        .zip(&b.group_keys)
                        .map(|(x, y)| eqg_cmp(x, y))
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

    /// Whether the current record starts a group: it is the first, or its GROUP BY keys are
    /// not `eqg` to the previous record's.
    pub fn new_group(&self) -> bool {
        match self.position.checked_sub(1) {
            None => true,
            Some(previous) => !eqg_rows(
                &self.records[previous].group_keys,
                &self.records[self.position].group_keys,
            ),
        }
    }

    /// Field `field` of each record of the group that just ended, in input order: the run of
    /// records with equal GROUP BY keys that ends before the current record (or at the end,
    /// once the sorter is exhausted).
    pub fn group_field(&self, field: usize) -> Vec<ValueRef<'static>> {
        let end = self.position.min(self.records.len());
        let Some(last) = end.checked_sub(1) else {
            return vec![];
        };
        let keys = &self.records[last].group_keys;
        let mut start = last;
        while start > 0 && eqg_rows(&self.records[start - 1].group_keys, keys) {
            start -= 1;
        }
        self.records[start..end]
            .iter()
            .map(|record| record.fields[field])
            .collect()
    }

    /// Advance to the next record. Returns true if there is a next record.
    pub fn advance(&mut self) -> bool {
        self.position += 1;
        self.position < self.records.len()
    }
}
