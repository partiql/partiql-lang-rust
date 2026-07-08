//! Heed-backed CompilationCatalog, ExecutionCatalog, and DataSource for pqlite.

use std::collections::HashMap;
use std::sync::Arc;

use partiql_common::catalog::EntryId;
use partiql_eval::source::{
    BufferStability, CatalogScans, DataSource, DataSourceHandle, DataSourceMetadata,
    RegisterWriter, ScanId, ScanSource,
};
use partiql_eval::{CompilationCatalog, EngineError, ExecutionCatalog};
use partiql_value::BindingsName;

use crate::row_codec::{deserialize_row_into, DeserializeError};
use crate::storage::{HeedDB, StorageError};

type EvalResult<T> = std::result::Result<T, EngineError>;

/// Table identifiers are hashed to a `u64` EntryId at `get_table` time; the
/// execution-side catalog rebuilds the reverse map by re-hashing every name
/// in `_tables`. Hash collision on a `u64` is structurally possible but
/// astronomically unlikely at realistic table counts. On collision,
/// `HeedExecutionCatalog::prepare` returns `StorageError::HashCollision` and
/// the per-query `prepare_error` stash surfaces it through `create()` — the
/// colliding table is NOT silently hidden.
pub struct HeedCompilationCatalog {
    db: Arc<HeedDB>,
}

impl HeedCompilationCatalog {
    pub fn new(db: Arc<HeedDB>) -> Self {
        Self { db }
    }
}

impl CompilationCatalog for HeedCompilationCatalog {
    fn get_table(&self, path: &[BindingsName<'_>]) -> Option<DataSourceHandle> {
        // pqlite has no schemas or databases — only bare table names.
        if path.len() != 1 {
            return None;
        }
        // Bare identifiers (case-insensitive) fold to ASCII lowercase; quoted
        // identifiers (case-sensitive) are taken verbatim. This is the standard
        // SQL identifier canonicalization rule.
        let canonical = match &path[0] {
            BindingsName::CaseInsensitive(s) => s.to_lowercase(),
            BindingsName::CaseSensitive(s) => s.to_string(),
        };
        let entry_id = hash_to_entry_id(&canonical);

        // Scope the read txn so it drops before the handle is returned.
        {
            let rtxn = self.db.read_txn().ok()?;
            if self.db.open_table(&rtxn, &canonical).is_err() {
                return None;
            }
        }

        let metadata: Arc<dyn DataSourceMetadata> = Arc::new(HeedTableMetadata);
        Some(DataSourceHandle::new(entry_id, metadata))
    }
}

/// `resolve` returns `None` for every field, forcing the compiler down the
/// `WholeValue` branch — the row is decoded as a tuple by `deserialize_row_into`.
struct HeedTableMetadata;

impl DataSourceMetadata for HeedTableMetadata {
    fn buffer_stability(&self) -> BufferStability {
        BufferStability::UntilNext
    }

    fn resolve(&self, _field_name: &str) -> Option<ScanSource> {
        None
    }
}

pub struct HeedExecutionCatalog {
    db: Arc<HeedDB>,
    scans: HashMap<ScanId, EntryId>,
    entry_to_name: HashMap<EntryId, String>,
    /// `prepare` returns `()`, so storage errors during index rebuild must be
    /// stashed and surfaced from the next `create()` call.
    prepare_error: Option<String>,
}

impl HeedExecutionCatalog {
    pub fn new(db: Arc<HeedDB>) -> Self {
        Self {
            db,
            scans: HashMap::new(),
            entry_to_name: HashMap::new(),
            prepare_error: None,
        }
    }

    fn rebuild_entry_index(&mut self) -> Result<(), StorageError> {
        self.entry_to_name.clear();
        let rtxn = self.db.read_txn()?;
        let tables = self.db.list_table_names(&rtxn)?;
        for name in tables {
            let entry_id = hash_to_entry_id(&name);
            if let Some(existing) = self.entry_to_name.insert(entry_id, name.clone()) {
                return Err(StorageError::HashCollision {
                    existing,
                    new: name,
                });
            }
        }
        Ok(())
    }
}

impl ExecutionCatalog for HeedExecutionCatalog {
    fn prepare(&mut self, scans: &CatalogScans) {
        self.scans.clear();
        self.prepare_error = None;
        if let Err(e) = self.rebuild_entry_index() {
            self.prepare_error = Some(format!("catalog index rebuild failed: {e}"));
            return;
        }
        for (scan_id, entry_id, _layout) in scans.iter() {
            if self.entry_to_name.contains_key(&entry_id) {
                self.scans.insert(scan_id, entry_id);
            }
        }
    }

    fn create(&self, scan_id: ScanId) -> EvalResult<Box<dyn DataSource>> {
        if let Some(msg) = &self.prepare_error {
            return Err(EngineError::ReaderError(msg.clone()));
        }
        let entry_id = *self.scans.get(&scan_id).ok_or_else(|| {
            EngineError::IllegalState(format!(
                "scan_id {:?} not registered; prepare() must run first",
                scan_id
            ))
        })?;
        let name = self.entry_to_name.get(&entry_id).ok_or_else(|| {
            EngineError::IllegalState(format!(
                "entry_id {:?} missing from index; prepare() must run first",
                entry_id
            ))
        })?;

        let (bytes, rows) = read_all_rows(&self.db, name)
            .map_err(|e| EngineError::ReaderError(format!("table scan failed: {e}")))?;

        Ok(Box::new(HeedTableSource {
            bytes,
            rows,
            cursor: 0,
        }))
    }
}

#[inline]
fn hash_to_entry_id(name: &str) -> EntryId {
    let mut hasher = std::collections::hash_map::DefaultHasher::new();
    std::hash::Hash::hash(name, &mut hasher);
    EntryId::from(std::hash::Hasher::finish(&hasher))
}

// Eagerly materialized because Box<dyn DataSource + 'static> cannot hold a borrowed RoTxn.
/// Pack the entire scan into one byte buffer with per-row `u32` offsets.
/// One `Vec<u8>` (geometric growth) replaces one allocation per row.
/// Fails with `ScanTooLarge` if the packed size exceeds `u32::MAX`.
fn read_all_rows(
    db: &HeedDB,
    table_name: &str,
) -> Result<(Vec<u8>, Vec<std::ops::Range<u32>>), StorageError> {
    let rtxn = db.read_txn()?;
    let tbl = db.open_table(&rtxn, table_name)?;
    let row_count = tbl.len(&rtxn).map_err(StorageError::Heed)? as usize;
    let mut bytes: Vec<u8> = Vec::new();
    let mut rows: Vec<std::ops::Range<u32>> = Vec::with_capacity(row_count);
    let mut prev_end: u32 = 0;
    for result in tbl.iter(&rtxn).map_err(StorageError::Heed)? {
        let (_k, v) = result.map_err(StorageError::Heed)?;
        bytes.extend_from_slice(v);
        // Only the cumulative end is checked; the start equals the prior
        // iteration's end (or 0), which already fit in u32 by induction.
        let end = u32::try_from(bytes.len()).map_err(|_| StorageError::ScanTooLarge {
            table: table_name.to_string(),
            observed_bytes: bytes.len(),
        })?;
        rows.push(prev_end..end);
        prev_end = end;
    }
    Ok((bytes, rows))
}

pub(crate) struct HeedTableSource {
    bytes: Vec<u8>,
    rows: Vec<std::ops::Range<u32>>,
    cursor: usize,
}

impl DataSource for HeedTableSource {
    fn open(&mut self) -> EvalResult<()> {
        self.cursor = 0;
        Ok(())
    }

    fn next_row(&mut self, writer: &mut RegisterWriter<'_, '_>) -> EvalResult<bool> {
        if self.cursor >= self.rows.len() {
            return Ok(false);
        }
        let r = &self.rows[self.cursor];
        let row = &self.bytes[r.start as usize..r.end as usize];
        // Safety: HeedTableSource owns self.bytes for the entire scan; the
        // borrowed slice outlives every per-row arena reset the writer performs.
        unsafe { deserialize_row_into(row, writer, 0) }.map_err(|e: DeserializeError| {
            EngineError::ReaderError(format!("row {} decode failed: {e}", self.cursor))
        })?;
        self.cursor += 1;
        Ok(true)
    }

    fn close(&mut self) -> EvalResult<()> {
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn metadata_resolve_returns_none() {
        let m = HeedTableMetadata;
        assert!(m.resolve("anything").is_none());
        assert!(matches!(m.buffer_stability(), BufferStability::UntilNext));
    }

    #[test]
    fn compilation_catalog_returns_none_for_missing_table() {
        let dir = tempfile::tempdir().unwrap();
        let path = dir.path().join("cat.pqlite");
        let db = Arc::new(HeedDB::open(&path).unwrap());
        let cat = HeedCompilationCatalog::new(db);
        let bindings = [BindingsName::CaseInsensitive(std::borrow::Cow::Borrowed(
            "nope",
        ))];
        assert!(cat.get_table(&bindings).is_none());
    }

    #[test]
    fn read_all_rows_packs_rows_into_flat_buffer() {
        let dir = tempfile::tempdir().unwrap();
        let path = dir.path().join("flat.pqlite");
        let db = HeedDB::open(&path).unwrap();

        let mut w = db.create_table("t").unwrap();
        w.push_row(&[0xAA, 0xBB]).unwrap();
        w.push_row(&[0xCC]).unwrap();
        w.push_row(&[0xDD, 0xEE, 0xFF]).unwrap();
        w.commit().unwrap();

        let (bytes, rows) = read_all_rows(&db, "t").unwrap();
        assert_eq!(bytes, vec![0xAA, 0xBB, 0xCC, 0xDD, 0xEE, 0xFF]);
        assert_eq!(rows, vec![0u32..2, 2..3, 3..6]);
    }

    #[test]
    fn read_all_rows_handles_empty_table() {
        let dir = tempfile::tempdir().unwrap();
        let path = dir.path().join("empty_flat.pqlite");
        let db = HeedDB::open(&path).unwrap();
        db.create_table("t").unwrap().commit().unwrap();

        let (bytes, rows) = read_all_rows(&db, "t").unwrap();
        assert!(bytes.is_empty());
        assert!(rows.is_empty());
    }

    #[test]
    fn compilation_catalog_returns_handle_for_existing_table() {
        let dir = tempfile::tempdir().unwrap();
        let path = dir.path().join("cat.pqlite");
        let db = Arc::new(HeedDB::open(&path).unwrap());
        let mut w = db.create_table("widgets").unwrap();
        // TAG_NULL: a complete single-byte NULL row (NULL has no payload);
        // only used to confirm catalog plumbing.
        w.push_row(&[crate::row_codec::TAG_NULL]).unwrap();
        w.commit().unwrap();

        let cat = HeedCompilationCatalog::new(Arc::clone(&db));
        let bindings = [BindingsName::CaseInsensitive(std::borrow::Cow::Borrowed(
            "widgets",
        ))];
        assert!(
            cat.get_table(&bindings).is_some(),
            "expected a handle for 'widgets'"
        );
    }
}
