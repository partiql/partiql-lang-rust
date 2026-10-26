use crate::compare::{eqg_rows, EqgHasher};
use crate::value::ValueRef;
use rustc_hash::FxHashMap;

/// A key's count in a hash table.
struct Entry {
    /// Values in the table's bank.
    key: Vec<ValueRef<'static>>,
    count: u64,
}

/// A multiset of keys (rows of `eqg` keys, as `compare::eqg_key` makes them), told apart by
/// `eqg`: the duplicate elimination and counting of DISTINCT, DISTINCT aggregates and the set
/// operators. A key is copied into the table's bank the first time it is added, and lives
/// until the table is reset.
pub(crate) struct HashTable {
    /// Which bank in the VM the keys live in.
    pub bank_id: usize,
    hasher: EqgHasher,
    /// The entries with each hash of their key: keys that collide share a hash.
    entries: FxHashMap<u64, Vec<Entry>>,
}

impl HashTable {
    pub fn new(bank_id: usize) -> Self {
        HashTable {
            bank_id,
            hasher: EqgHasher::new(),
            entries: FxHashMap::default(),
        }
    }

    /// Drop every key, so that an operator can run again. The caller resets the table's
    /// bank with it.
    pub fn reset(&mut self) {
        self.entries.clear();
    }

    /// Drop every key and the memory that held them, once the query is done.
    pub fn free(&mut self) {
        self.entries = FxHashMap::default();
    }

    /// Count `key` once more, adding it (as `copy` makes it, in the table's bank) when it is
    /// new. Returns whether it was.
    pub fn add(
        &mut self,
        key: &[ValueRef<'_>],
        copy: impl FnOnce() -> Vec<ValueRef<'static>>,
    ) -> bool {
        let entries = self.entries.entry(self.hasher.hash_row(key)).or_default();
        match entries.iter_mut().find(|e| eqg_rows(&e.key, key)) {
            Some(entry) => {
                entry.count += 1;
                false
            }
            None => {
                entries.push(Entry {
                    key: copy(),
                    count: 1,
                });
                true
            }
        }
    }

    /// Add `key` unless the table holds it. Returns whether it was new.
    pub fn insert(
        &mut self,
        key: &[ValueRef<'_>],
        copy: impl FnOnce() -> Vec<ValueRef<'static>>,
    ) -> bool {
        let entries = self.entries.entry(self.hasher.hash_row(key)).or_default();
        if entries.iter().any(|e| eqg_rows(&e.key, key)) {
            return false;
        }
        entries.push(Entry {
            key: copy(),
            count: 1,
        });
        true
    }

    /// Count `key` once less, if the table holds it. Returns whether it did.
    pub fn take(&mut self, key: &[ValueRef<'_>]) -> bool {
        let Some(entries) = self.entries.get_mut(&self.hasher.hash_row(key)) else {
            return false;
        };
        match entries
            .iter_mut()
            .find(|e| e.count > 0 && eqg_rows(&e.key, key))
        {
            Some(entry) => {
                entry.count -= 1;
                true
            }
            None => false,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn row(values: &[ValueRef<'static>]) -> Vec<ValueRef<'static>> {
        values.to_vec()
    }

    #[test]
    fn insert_tells_eqg_keys_apart() {
        let mut table = HashTable::new(0);
        let one = [ValueRef::I64(1), ValueRef::Null];
        assert!(table.insert(&one, || row(&one)));
        assert!(!table.insert(&[ValueRef::F64(1.0), ValueRef::Null], || unreachable!()));
        let other = [ValueRef::I64(1), ValueRef::Missing];
        assert!(table.insert(&other, || row(&other)));
        assert!(table.insert(&one[..1], || row(&one[..1])));
        table.reset();
        assert!(table.insert(&one, || row(&one)));
    }

    #[test]
    fn add_counts_and_take_uses_up_the_count() {
        let mut table = HashTable::new(0);
        let key = [ValueRef::Str("a")];
        assert!(table.add(&key, || row(&key)));
        assert!(!table.add(&key, || unreachable!()));
        assert!(table.take(&key));
        assert!(table.take(&key));
        assert!(!table.take(&key));
        assert!(!table.take(&[ValueRef::Str("b")]));
    }
}
