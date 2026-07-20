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
use smallvec::SmallVec;

use crate::row_codec::{deserialize_row_into, DeserializeError};
use crate::storage::{is_system_table, HeedDB, StorageError};

type EvalResult<T> = std::result::Result<T, EngineError>;

/// Rows fetched per read transaction.
const CHUNK_ROWS: usize = 64;

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

        Ok(Box::new(HeedTableSource {
            db: Arc::clone(&self.db),
            table: name.clone(),
            bytes: Vec::new(),
            offsets: Vec::new(),
            pos: 0,
            resume_after: None,
            exhausted: false,
            rows_emitted: 0,
        }))
    }
}

#[inline]
fn hash_to_entry_id(name: &str) -> EntryId {
    let mut hasher = std::collections::hash_map::DefaultHasher::new();
    std::hash::Hash::hash(name, &mut hasher);
    EntryId::from(std::hash::Hasher::finish(&hasher))
}

/// Streams a table one CHUNK_ROWS-sized buffer at a time. `resume_after` is a
/// keyset cursor (the real last key, not a counter) so chunk boundaries stay
/// correct across the gaps a future DELETE would leave.
pub(crate) struct HeedTableSource {
    db: Arc<HeedDB>,
    table: String,
    bytes: Vec<u8>,                          // current chunk's packed rows
    offsets: Vec<std::ops::Range<u32>>,      // per-row slices into `bytes`
    pos: usize,                              // next row within the chunk
    resume_after: Option<SmallVec<[u8; 8]>>, // opaque keyset cursor; heap-free for u64 keys
    exhausted: bool,                         // last chunk was short: no more rows
    rows_emitted: u64,                       // total so far, for decode-error messages
}

impl DataSource for HeedTableSource {
    fn open(&mut self) -> EvalResult<()> {
        // Reset for a fresh scan (VM lifecycle: open once, next_row until false, close once).
        self.bytes.clear();
        self.offsets.clear();
        self.pos = 0;
        self.resume_after = None;
        self.exhausted = false;
        self.rows_emitted = 0;
        Ok(())
    }

    fn next_row(&mut self, writer: &mut RegisterWriter<'_, '_>) -> EvalResult<bool> {
        if self.pos >= self.offsets.len() {
            if self.exhausted {
                return Ok(false);
            }
            self.load_next_chunk()
                .map_err(|e| EngineError::ReaderError(format!("chunk scan failed: {e}")))?;
            if self.offsets.is_empty() {
                return Ok(false);
            }
        }
        let r = self.offsets[self.pos].clone();
        let row = &self.bytes[r.start as usize..r.end as usize];
        // Safety: `row`'s string/bytes leaves are laundered into the arena, so
        // they must outlive the engine's use of this row. `BufferStability::
        // UntilNext` guarantees the engine won't hold a row past the next
        // `next_row` call (later consumers deep-copy), and `self.bytes` is only
        // refilled in load_next_chunk on a subsequent `next_row` — so no live
        // borrow aliases the buffer at refill. See deserialize_row_into's
        // # Safety block.
        unsafe { deserialize_row_into(row, writer, 0) }.map_err(|e: DeserializeError| {
            EngineError::ReaderError(format!("row {} decode failed: {e}", self.rows_emitted))
        })?;
        self.pos += 1;
        self.rows_emitted += 1;
        Ok(true)
    }

    fn close(&mut self) -> EvalResult<()> {
        Ok(())
    }
}

impl HeedTableSource {
    /// Fetch up to CHUNK_ROWS rows via one read txn and a sequential range scan
    /// (leaf-page walk, not point-gets), packing them into the reused buffer.
    /// Resumes strictly after the last key read (keyset pagination) so gaps from
    /// future DELETEs never cause skipped or double-read rows.
    fn load_next_chunk(&mut self) -> Result<(), StorageError> {
        let rtxn = self.db.read_txn()?;
        // Per-chunk _tables membership re-open is interim cost; a cached catalog
        // handle is deferred to the streaming/handle work.
        let tbl = self.db.open_table(&rtxn, &self.table)?;

        self.bytes.clear();
        self.offsets.clear();
        self.pos = 0;

        // Range lower bound: strictly after the last key of the previous chunk,
        // or unbounded for the first chunk.
        let range: (std::ops::Bound<&[u8]>, std::ops::Bound<&[u8]>) = match &self.resume_after {
            Some(k) => (
                std::ops::Bound::Excluded(&k[..]),
                std::ops::Bound::Unbounded,
            ),
            None => (std::ops::Bound::Unbounded, std::ops::Bound::Unbounded),
        };

        // Loop-invariant (table name is fixed for the whole scan), hoisted out.
        let key_len_checked = !is_system_table(&self.table);
        let mut count = 0usize;
        let mut prev_end: u32 = 0;
        let mut last_key: SmallVec<[u8; 8]> = SmallVec::new(); // reused; inline for u64 keys
        for result in tbl.range(&rtxn, &range).map_err(StorageError::Heed)? {
            let (k, v) = result.map_err(StorageError::Heed)?;
            // Corruption canary: user tables MUST use 8-byte BE-u64 keys. System
            // tables (_tables) are string-keyed by design, so they opt out.
            if key_len_checked && k.len() != 8 {
                return Err(StorageError::Codec(format!(
                    "table {}: row key is {} bytes, expected 8 (BE u64)",
                    self.table,
                    k.len()
                )));
            }
            self.bytes.extend_from_slice(v);
            let end = u32::try_from(self.bytes.len()).map_err(|_| StorageError::ScanTooLarge {
                table: self.table.clone(),
                observed_bytes: self.bytes.len(),
            })?;
            self.offsets.push(prev_end..end);
            prev_end = end;
            last_key.clear();
            last_key.extend_from_slice(k);
            count += 1;
            if count == CHUNK_ROWS {
                break;
            }
        }
        // txn drops at end of scope; the packed buffer is owned and outlives it.
        drop(rtxn);

        if count > 0 {
            self.resume_after = Some(last_key);
        }
        // A short chunk means the range is drained — no further chunks.
        if count < CHUNK_ROWS {
            self.exhausted = true;
        }
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn tables_value(name: &str) -> Vec<u8> {
        let mut v = Vec::new();
        crate::row_codec::serialize_name_row(&[name], &mut v).unwrap();
        v
    }

    /// Open a DB and bootstrap `_tables` so user-table creates can proceed
    /// (`create_table` now requires the catalog to exist first).
    fn open_bootstrapped(path: &std::path::Path) -> Arc<HeedDB> {
        let db = Arc::new(HeedDB::open(path).unwrap());
        db.create_table("_tables", &tables_value("_tables"))
            .unwrap()
            .commit()
            .unwrap();
        db
    }

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

    /// Build a `HeedTableSource` bound to `db`/`table` in the pre-scan state
    /// (as `create()` produces it), so tests can drive `load_next_chunk`
    /// directly without a `RegisterWriter`.
    fn source_for(db: &Arc<HeedDB>, table: &str) -> HeedTableSource {
        HeedTableSource {
            db: Arc::clone(db),
            table: table.to_string(),
            bytes: Vec::new(),
            offsets: Vec::new(),
            pos: 0,
            resume_after: None,
            exhausted: false,
            rows_emitted: 0,
        }
    }

    #[test]
    fn streaming_single_chunk_reads_all_rows_when_under_chunk_size() {
        let dir = tempfile::tempdir().unwrap();
        let path = dir.path().join("single.pqlite");
        let db = open_bootstrapped(&path);

        let mut w = db.create_table("t", &tables_value("t")).unwrap();
        w.push_row(&[0xAA, 0xBB]).unwrap();
        w.push_row(&[0xCC]).unwrap();
        w.push_row(&[0xDD, 0xEE, 0xFF]).unwrap();
        w.commit().unwrap();

        let mut src = source_for(&db, "t");
        src.load_next_chunk().unwrap();

        assert_eq!(src.offsets.len(), 3);
        assert!(src.exhausted);
        assert_eq!(src.offsets, vec![0u32..2, 2..3, 3..6]);
        assert_eq!(src.bytes, vec![0xAA, 0xBB, 0xCC, 0xDD, 0xEE, 0xFF]);
    }

    #[test]
    fn streaming_spans_multiple_chunks() {
        let dir = tempfile::tempdir().unwrap();
        let path = dir.path().join("multi.pqlite");
        let db = open_bootstrapped(&path);

        // Distinct byte per row so the asserts can catch any overlap or gap.
        let total = CHUNK_ROWS + 5;
        let mut w = db.create_table("t", &tables_value("t")).unwrap();
        for i in 0..total {
            w.push_row(&[i as u8]).unwrap();
        }
        w.commit().unwrap();

        let mut src = source_for(&db, "t");

        src.load_next_chunk().unwrap();
        assert_eq!(src.offsets.len(), CHUNK_ROWS);
        assert!(!src.exhausted);
        assert_eq!(
            src.resume_after.as_deref(),
            Some(&(CHUNK_ROWS as u64 - 1).to_be_bytes()[..])
        );
        let chunk1: Vec<u8> = src.bytes.clone();
        assert_eq!(chunk1, (0..CHUNK_ROWS as u8).collect::<Vec<u8>>());

        src.load_next_chunk().unwrap();
        assert_eq!(src.offsets.len(), 5);
        assert!(src.exhausted);
        let chunk2: Vec<u8> = src.bytes.clone();
        assert_eq!(
            chunk2[0], CHUNK_ROWS as u8,
            "chunk 2 resumes at row 64, no re-read"
        );
        assert_eq!(chunk2, (CHUNK_ROWS as u8..total as u8).collect::<Vec<u8>>());

        let mut all = chunk1;
        all.extend_from_slice(&chunk2);
        assert_eq!(all, (0..total as u8).collect::<Vec<u8>>());
    }

    #[test]
    fn streaming_empty_table() {
        let dir = tempfile::tempdir().unwrap();
        let path = dir.path().join("empty.pqlite");
        let db = open_bootstrapped(&path);
        db.create_table("t", &tables_value("t"))
            .unwrap()
            .commit()
            .unwrap();

        let mut src = source_for(&db, "t");
        src.load_next_chunk().unwrap();

        assert!(src.offsets.is_empty());
        assert!(src.bytes.is_empty());
        assert!(src.exhausted);
    }

    #[test]
    fn streaming_reads_sparse_keys_within_one_chunk() {
        let dir = tempfile::tempdir().unwrap();
        let path = dir.path().join("sparse.pqlite");
        let db = open_bootstrapped(&path);
        db.create_table("t", &tables_value("t"))
            .unwrap()
            .commit()
            .unwrap();

        // inject bypasses the encoder to make the gapped keys a DELETE would leave.
        let keys = [0u64, 1, 2, 100, 101, 200];
        for (i, &k) in keys.iter().enumerate() {
            db.inject_row_for_tests("t", k, &[i as u8]);
        }

        let mut src = source_for(&db, "t");
        src.load_next_chunk().unwrap();

        assert_eq!(src.offsets.len(), 6);
        assert!(src.exhausted);
        assert_eq!(src.bytes, (0u8..6).collect::<Vec<u8>>());
        assert_eq!(src.resume_after.as_deref(), Some(&200u64.to_be_bytes()[..]));
    }

    #[test]
    fn streaming_keyset_resume_survives_gaps() {
        let dir = tempfile::tempdir().unwrap();
        let path = dir.path().join("gaps.pqlite");
        let db = open_bootstrapped(&path);
        db.create_table("t", &tables_value("t"))
            .unwrap()
            .commit()
            .unwrap();

        // Dense keys 0..64 (payload = low byte, so key 63 = 0x3F), a gap, then
        // keys 1000.. (payload 0xF0..) — distinct so chunk 2's first row is identifiable.
        for k in 0..CHUNK_ROWS as u64 {
            db.inject_row_for_tests("t", k, &[k as u8]);
        }
        for (i, k) in (1000u64..1006).enumerate() {
            db.inject_row_for_tests("t", k, &[0xF0 | i as u8]);
        }

        let mut src = source_for(&db, "t");

        src.load_next_chunk().unwrap();
        assert_eq!(src.offsets.len(), CHUNK_ROWS);
        assert!(!src.exhausted);
        assert_eq!(
            src.resume_after.as_deref(),
            Some(&(CHUNK_ROWS as u64 - 1).to_be_bytes()[..])
        );

        src.load_next_chunk().unwrap();
        assert_eq!(src.offsets.len(), 6);
        assert!(src.exhausted);
        let first = &src.bytes[src.offsets[0].start as usize..src.offsets[0].end as usize];
        assert_eq!(
            first,
            &[0xF0],
            "chunk 2 resumes at key 1000, not a re-read of 63"
        );
        assert_eq!(
            src.resume_after.as_deref(),
            Some(&1005u64.to_be_bytes()[..])
        );
    }

    #[test]
    fn streaming_tolerates_string_keys_for_system_table() {
        let dir = tempfile::tempdir().unwrap();
        let db = Arc::new(HeedDB::open(&dir.path().join("systbl.pqlite")).unwrap());
        db.create_table("_tables", &tables_value("_tables"))
            .unwrap()
            .commit()
            .unwrap();
        let mut src = source_for(&db, "_tables");
        src.load_next_chunk().unwrap(); // must NOT fire the 8-byte canary
        assert_eq!(src.offsets.len(), 1);
        assert_eq!(src.resume_after.as_deref(), Some(&b"_tables"[..]));
        assert!(src.exhausted);
    }

    #[test]
    fn streaming_string_keys_resume_across_chunk_boundary() {
        let dir = tempfile::tempdir().unwrap();
        let db = Arc::new(HeedDB::open(&dir.path().join("bigcat.pqlite")).unwrap());
        db.create_table("_tables", &tables_value("_tables"))
            .unwrap()
            .commit()
            .unwrap();
        for i in 0..70u32 {
            let name = format!("tbl_{i:03}");
            db.create_table(&name, &tables_value(&name))
                .unwrap()
                .commit()
                .unwrap();
        }
        let mut src = source_for(&db, "_tables");
        src.load_next_chunk().unwrap();
        assert_eq!(src.offsets.len(), CHUNK_ROWS, "first chunk full");
        assert!(!src.exhausted);
        let first_resume = src.resume_after.clone();
        src.load_next_chunk().unwrap();
        assert!(src.exhausted, "second chunk drains the rest");
        assert_ne!(
            src.resume_after, first_resume,
            "cursor advanced past chunk 1"
        );
        assert_eq!(src.offsets.len(), 7); // 71 total (_tables + tbl_000..069) = 64 + 7
    }

    #[test]
    fn streaming_still_rejects_non_8_byte_keys_for_user_tables() {
        let dir = tempfile::tempdir().unwrap();
        let db = open_bootstrapped(&dir.path().join("baduser.pqlite"));
        db.create_table("t", &tables_value("t"))
            .unwrap()
            .commit()
            .unwrap();
        {
            let mut wtxn = db.env().write_txn().unwrap();
            let tbl: heed::Database<heed::types::Bytes, heed::types::Bytes> =
                db.env().open_database(&wtxn, Some("t")).unwrap().unwrap();
            tbl.put(&mut wtxn, b"abc", &[0x00]).unwrap(); // 3-byte key in a USER table
            wtxn.commit().unwrap();
        }
        let mut src = source_for(&db, "t");
        let err = src.load_next_chunk().unwrap_err();
        assert!(
            matches!(err, StorageError::Codec(ref m) if m.contains("expected 8")),
            "user-table non-8-byte key must fail the canary; got {err:?}"
        );
    }

    #[test]
    fn compilation_catalog_returns_handle_for_existing_table() {
        let dir = tempfile::tempdir().unwrap();
        let path = dir.path().join("cat.pqlite");
        let db = open_bootstrapped(&path);
        let mut w = db
            .create_table("widgets", &tables_value("widgets"))
            .unwrap();
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
