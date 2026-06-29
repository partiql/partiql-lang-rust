//! LMDB-backed storage for pqlite.

use std::path::{Path, PathBuf};

/// Virtual address space reserved for the LMDB map; sparse on disk.
const DEFAULT_MAP_SIZE: usize = 1024 * 1024 * 1024; // 1 GiB

/// One catalog plus headroom for one named db per user table.
const MAX_DBS: u32 = 128;

const TABLES_DB: &str = "_tables";

type TablesDb = heed::Database<heed::types::Str, heed::types::Bytes>;

/// Row-store keys are 8-byte big-endian `u64` row ids passed as raw bytes.
/// `Bytes` (not `U64<BigEndian>`) lets the caller pass a stack-local `[u8; 8]`
/// for zero per-put allocations. BE keeps lexicographic key order matching
/// numeric order.
type RowKey = heed::types::Bytes;

type RowDb = heed::Database<RowKey, heed::types::Bytes>;

/// Drop without `commit` rolls back. `key_buf` is reused across pushes.
pub struct TableWriter<'env> {
    wtxn: heed::RwTxn<'env>,
    table: RowDb,
    row_id: u64,
    key_buf: [u8; 8],
}

// `heed::RwTxn` is not `Debug`, so derive can't be used.
impl<'env> std::fmt::Debug for TableWriter<'env> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("TableWriter")
            .field("row_id", &self.row_id)
            .finish_non_exhaustive()
    }
}

impl<'env> TableWriter<'env> {
    /// Append the next row. Storage adds no framing.
    pub fn push_row(&mut self, bytes: &[u8]) -> Result<(), StorageError> {
        self.key_buf = self.row_id.to_be_bytes();
        self.table.put(&mut self.wtxn, &self.key_buf[..], bytes)?;
        self.row_id += 1;
        Ok(())
    }

    /// Commit the write txn and return the row count.
    pub fn commit(self) -> Result<u64, StorageError> {
        self.wtxn.commit()?;
        Ok(self.row_id)
    }
}

#[derive(Debug)]
pub enum StorageError {
    Io(std::io::Error),
    Heed(heed::Error),
    TableExists(String),
    /// `_`-prefixed name; the namespace is reserved.
    ReservedName(String),
    /// Interior NUL byte (heed would otherwise panic).
    InvalidName(String),
    TableMissing(String),
    Execution(String),
    Codec(String),
    /// Table scan exceeded 4 GiB; row offsets do not fit in `u32`.
    /// Carries the offending table name and the byte count observed so far.
    ScanTooLarge {
        table: String,
        observed_bytes: usize,
    },
    /// Two table names hashed to the same EntryId. Astronomically unlikely
    /// at u64 widths but craftable against the fixed-seed DefaultHasher; this
    /// would silently make one of the tables unreachable, so we fail hard.
    HashCollision {
        existing: String,
        new: String,
    },
}

impl std::fmt::Display for StorageError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            StorageError::Io(e) => write!(f, "storage I/O error: {e}"),
            StorageError::Heed(e) => write!(f, "storage engine error: {e}"),
            StorageError::TableExists(name) => write!(f, "table already exists: {name}"),
            StorageError::ReservedName(name) => write!(
                f,
                "reserved table name '{name}' (names starting with '_' are for system use)"
            ),
            StorageError::InvalidName(name) => {
                write!(f, "invalid table name {name:?}: contains a NUL byte")
            }
            StorageError::TableMissing(name) => write!(f, "table not found: {name}"),
            StorageError::Execution(msg) => write!(f, "execution error: {msg}"),
            StorageError::Codec(msg) => write!(f, "{msg}"),
            StorageError::ScanTooLarge {
                table,
                observed_bytes,
            } => write!(
                f,
                "table scan too large for {table:?}: {observed_bytes} bytes exceeds 4 GiB u32 offset limit"
            ),
            StorageError::HashCollision { existing, new } => write!(
                f,
                "table name hash collision: '{existing}' and '{new}' share an EntryId; rename one to break the tie"
            ),
        }
    }
}

impl std::error::Error for StorageError {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        match self {
            StorageError::Io(e) => Some(e),
            StorageError::Heed(e) => Some(e),
            StorageError::TableExists(_)
            | StorageError::ReservedName(_)
            | StorageError::InvalidName(_)
            | StorageError::TableMissing(_)
            | StorageError::Execution(_)
            | StorageError::Codec(_)
            | StorageError::ScanTooLarge { .. }
            | StorageError::HashCollision { .. } => None,
        }
    }
}

impl From<std::io::Error> for StorageError {
    fn from(e: std::io::Error) -> Self {
        StorageError::Io(e)
    }
}

impl From<heed::Error> for StorageError {
    fn from(e: heed::Error) -> Self {
        StorageError::Heed(e)
    }
}

/// Prefix a bare filename with `./` so heed can canonicalize its parent.
/// Without this, `--db foo.pqlite` fails ENOENT because `parent()` of a bare
/// name is the empty path. Paths with a real directory component are returned
/// unchanged, so a genuinely missing parent still errors.
fn normalize_db_path(path: &Path) -> PathBuf {
    if path.parent() == Some(Path::new("")) {
        Path::new(".").join(path)
    } else {
        path.to_path_buf()
    }
}

/// The `_tables` catalog handle is valid only while the env is alive, so
/// both live together.
#[derive(Debug)]
pub struct HeedDB {
    env: heed::Env,
    tables: TablesDb,
    path: PathBuf,
}

impl HeedDB {
    /// Open (or create) the LMDB environment at `path` as a single file, and
    /// open/create the `_tables` system catalog inside it.
    pub fn open(path: &Path) -> Result<HeedDB, StorageError> {
        let normalized = normalize_db_path(path);

        // SAFETY: NO_SUB_DIR only changes the on-disk layout (single file vs.
        // directory). The dangerous flags (NO_SYNC / NO_META_SYNC / NO_LOCK)
        // are not set, and pqlite opens each env path at most once per process.
        let env = unsafe {
            heed::EnvOpenOptions::new()
                .map_size(DEFAULT_MAP_SIZE)
                .max_dbs(MAX_DBS)
                .flags(heed::EnvFlags::NO_SUB_DIR)
                .open(&normalized)?
        };

        let mut wtxn = env.write_txn()?;
        // `create_database` is idempotent.
        let tables: TablesDb = env.create_database(&mut wtxn, Some(TABLES_DB))?;
        wtxn.commit()?;

        Ok(HeedDB {
            env,
            tables,
            path: path.to_path_buf(),
        })
    }

    /// Open a write handle for a new table. `name` must already be
    /// canonicalized by the caller (lowercase for bare identifiers, verbatim
    /// for quoted).
    pub fn create_table(&self, name: &str) -> Result<TableWriter<'_>, StorageError> {
        if name.starts_with('_') {
            return Err(StorageError::ReservedName(name.to_string()));
        }
        if name.contains('\0') {
            return Err(StorageError::InvalidName(name.to_string()));
        }
        let mut wtxn = self.env.write_txn()?;
        if self.tables.get(&wtxn, name)?.is_some() {
            return Err(StorageError::TableExists(name.to_string()));
        }
        let table: RowDb = self.env.create_database(&mut wtxn, Some(name))?;
        // Catalog cell records existence only.
        self.tables.put(&mut wtxn, name, &[])?;
        Ok(TableWriter {
            wtxn,
            table,
            row_id: 0,
            key_buf: [0u8; 8],
        })
    }

    /// Drop the returned txn to release the read snapshot.
    pub fn read_txn(&self) -> Result<heed::RoTxn<'_>, StorageError> {
        Ok(self.env.read_txn()?)
    }

    /// Open the named table for read. Returns `TableMissing` if no named
    /// database called `name` exists in the env (i.e., the table was never
    /// created or was dropped).
    pub fn open_table(&self, txn: &heed::RoTxn<'_>, name: &str) -> Result<RowDb, StorageError> {
        self.env
            .open_database(txn, Some(name))?
            .ok_or_else(|| StorageError::TableMissing(name.to_string()))
    }

    /// Enumerate every table name in `_tables`, in lexicographic key order.
    pub fn list_table_names(&self, txn: &heed::RoTxn<'_>) -> Result<Vec<String>, StorageError> {
        let mut out =
            Vec::with_capacity(self.tables.len(txn).map_err(StorageError::Heed)? as usize);
        for result in self.tables.iter(txn).map_err(StorageError::Heed)? {
            let (k, _v) = result.map_err(StorageError::Heed)?;
            out.push(k.to_string());
        }
        Ok(out)
    }

    #[cfg(test)]
    pub(crate) fn env(&self) -> &heed::Env {
        &self.env
    }

    #[cfg(test)]
    pub(crate) fn tables(&self) -> &TablesDb {
        &self.tables
    }

    pub fn path(&self) -> &Path {
        &self.path
    }
}

#[cfg(any(test, feature = "test-support"))]
#[doc(hidden)]
impl HeedDB {
    /// Test-only: write `payload` at `row_id` bypassing the encoder, so tests
    /// can inject malformed rows.
    pub fn inject_row_for_tests(&self, table: &str, row_id: u64, payload: &[u8]) {
        let mut wtxn = self.env.write_txn().unwrap();
        let tbl: heed::Database<heed::types::Bytes, heed::types::Bytes> = self
            .env
            .open_database(&wtxn, Some(table))
            .unwrap()
            .expect("row db must exist");
        let key = row_id.to_be_bytes();
        tbl.put(&mut wtxn, &key[..], payload).unwrap();
        wtxn.commit().unwrap();
    }

    /// Test-only: read raw payload at `row_id`. Row keys are 8-byte BE `u64`.
    pub fn read_row_for_tests(&self, table: &str, row_id: u64) -> Vec<u8> {
        let rtxn = self.env.read_txn().unwrap();
        let tbl: heed::Database<heed::types::Bytes, heed::types::Bytes> = self
            .env
            .open_database(&rtxn, Some(table))
            .unwrap()
            .expect("row db must exist");
        let key = row_id.to_be_bytes();
        tbl.get(&rtxn, &key[..])
            .unwrap()
            .expect("row must exist")
            .to_vec()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn table_writer_round_trip_via_push_and_commit() {
        let dir = tempfile::tempdir().unwrap();
        let path = dir.path().join("writer.pqlite");
        let db = HeedDB::open(&path).unwrap();

        let mut w = db.create_table("widgets").expect("create_table");
        w.push_row(&[0xDE, 0xAD]).unwrap();
        w.push_row(&[0xBE, 0xEF, 0x01]).unwrap();
        let n = w.commit().unwrap();
        assert_eq!(n, 2);

        let rtxn = db.env().read_txn().unwrap();
        assert!(db.tables().get(&rtxn, "widgets").unwrap().is_some());
        let rowdb: RowDb = db
            .env()
            .open_database(&rtxn, Some("widgets"))
            .unwrap()
            .expect("rowdb must exist after commit");
        assert_eq!(rowdb.len(&rtxn).unwrap(), 2);
    }

    #[test]
    fn create_table_writes_bytes_verbatim_and_registers_catalog() {
        let dir = tempfile::tempdir().unwrap();
        let path = dir.path().join("ctas.pqlite");
        let db = HeedDB::open(&path).unwrap();

        let payloads: [&[u8]; 3] = [
            &[0xDE, 0xAD, 0xBE, 0xEF],
            &[0x01],
            &[0xFF, 0x00, 0xFF, 0x00, 0xFF],
        ];
        let mut w = db.create_table("widgets").unwrap();
        for p in payloads.iter() {
            w.push_row(p).unwrap();
        }
        let n = w.commit().unwrap();
        assert_eq!(n, 3);

        let rtxn = db.env().read_txn().unwrap();
        assert!(
            db.tables().get(&rtxn, "widgets").unwrap().is_some(),
            "widgets should be registered in _tables"
        );

        let rowdb: RowDb = db
            .env()
            .open_database::<RowKey, heed::types::Bytes>(&rtxn, Some("widgets"))
            .unwrap()
            .expect("widgets row db should exist");
        assert_eq!(rowdb.len(&rtxn).unwrap(), 3);
        for (i, expected) in payloads.iter().enumerate() {
            let key = (i as u64).to_be_bytes();
            let got = rowdb
                .get(&rtxn, &key[..])
                .unwrap()
                .expect("row should be present");
            assert_eq!(got, *expected, "row {i} bytes must round-trip verbatim");
        }
    }

    #[test]
    fn create_table_with_no_rows_creates_empty_table() {
        let dir = tempfile::tempdir().unwrap();
        let path = dir.path().join("empty.pqlite");
        let db = HeedDB::open(&path).unwrap();

        let n = db.create_table("blank").unwrap().commit().unwrap();
        assert_eq!(n, 0);

        let rtxn = db.env().read_txn().unwrap();
        assert!(db.tables().get(&rtxn, "blank").unwrap().is_some());
        let rowdb: RowDb = db
            .env()
            .open_database::<RowKey, heed::types::Bytes>(&rtxn, Some("blank"))
            .unwrap()
            .expect("blank row db should exist");
        assert_eq!(rowdb.len(&rtxn).unwrap(), 0);
    }

    #[test]
    fn create_table_rejects_duplicate_name() {
        let dir = tempfile::tempdir().unwrap();
        let path = dir.path().join("dup.pqlite");
        let db = HeedDB::open(&path).unwrap();

        let mut w1 = db.create_table("t").unwrap();
        w1.push_row(&[0x01]).unwrap();
        w1.commit().unwrap();
        let again = db.create_table("t");
        assert!(
            matches!(again, Err(StorageError::TableExists(ref n)) if n == "t"),
            "expected TableExists; got {:?}",
            again
        );

        let rtxn = db.env().read_txn().unwrap();
        assert_eq!(db.tables().len(&rtxn).unwrap(), 1);
    }

    #[test]
    fn create_table_rejects_reserved_names() {
        let dir = tempfile::tempdir().unwrap();
        let path = dir.path().join("reserved.pqlite");
        let db = HeedDB::open(&path).unwrap();

        for name in ["_tables", "_x", "_"] {
            let res = db.create_table(name);
            assert!(
                matches!(res, Err(StorageError::ReservedName(_))),
                "expected ReservedName for {name:?}; got {:?}",
                res
            );
        }

        let rtxn = db.env().read_txn().unwrap();
        assert_eq!(db.tables().len(&rtxn).unwrap(), 0);
    }

    #[test]
    fn create_table_rejects_interior_nul_names() {
        let dir = tempfile::tempdir().unwrap();
        let path = dir.path().join("nul.pqlite");
        let db = HeedDB::open(&path).unwrap();

        // Interior NUL would otherwise panic heed's CString::new.
        let res = db.create_table("a\0b");
        assert!(
            matches!(res, Err(StorageError::InvalidName(_))),
            "expected InvalidName; got {:?}",
            res
        );

        let rtxn = db.env().read_txn().unwrap();
        assert_eq!(db.tables().len(&rtxn).unwrap(), 0);
    }

    #[test]
    fn dropping_writer_rolls_back_uncommitted_rows() {
        let dir = tempfile::tempdir().unwrap();
        let path = dir.path().join("rb.pqlite");
        let db = HeedDB::open(&path).unwrap();
        {
            let mut w = db.create_table("partial").unwrap();
            w.push_row(&[0x42]).unwrap();
            // Drop without commit — wtxn rolls back.
        }
        let rtxn = db.env().read_txn().unwrap();
        assert!(
            db.tables().get(&rtxn, "partial").unwrap().is_none(),
            "rolled-back create must leave no catalog entry"
        );
    }

    #[test]
    fn storage_error_displays_and_converts_from_io() {
        let io = std::io::Error::new(std::io::ErrorKind::NotFound, "boom");
        let err: StorageError = io.into();
        assert!(matches!(err, StorageError::Io(_)));
        let msg = err.to_string();
        assert!(msg.contains("storage I/O error"), "got: {msg}");
        assert!(msg.contains("boom"), "got: {msg}");
    }

    #[test]
    fn new_storage_errors_display() {
        let te = StorageError::TableExists("foo".to_string());
        assert!(te.to_string().contains("already exists"), "got: {te}");
        assert!(te.to_string().contains("foo"), "got: {te}");

        let rn = StorageError::ReservedName("_x".to_string());
        let s = rn.to_string();
        assert!(s.contains("reserved"), "got: {s}");
        assert!(s.contains("_x"), "got: {s}");

        let inv = StorageError::InvalidName("a\0b".to_string());
        let s = inv.to_string();
        assert!(s.contains("invalid table name"), "got: {s}");
        assert!(s.contains("NUL"), "got: {s}");

        let ex = StorageError::Execution("boom".to_string());
        assert!(ex.to_string().contains("boom"), "got: {ex}");

        let tm = StorageError::TableMissing("ghost".to_string());
        let s = tm.to_string();
        assert!(s.contains("table not found"), "got: {s}");
        assert!(s.contains("ghost"), "got: {s}");

        let stl = StorageError::ScanTooLarge {
            table: "widgets".to_string(),
            observed_bytes: 5_000_000_000,
        };
        let s = stl.to_string();
        assert!(s.contains("4 GiB"), "got: {s}");
        assert!(s.contains("widgets"), "got: {s}");
        assert!(s.contains("5000000000"), "got: {s}");
    }

    #[test]
    fn open_creates_empty_tables_catalog() {
        let dir = tempfile::tempdir().unwrap();
        let path = dir.path().join("cat.pqlite");
        let db = HeedDB::open(&path).unwrap();

        // NO_SUB_DIR stores the environment as a single file.
        assert!(path.is_file(), "db should be a single file (NO_SUB_DIR)");

        let rtxn = db.env().read_txn().unwrap();

        // Look up by literal name so renaming the TABLES_DB constant can't pass spuriously.
        let named = db
            .env()
            .open_database::<heed::types::Str, heed::types::Bytes>(&rtxn, Some("_tables"))
            .unwrap();
        assert!(
            named.is_some(),
            "the `_tables` catalog should exist by name"
        );

        assert_eq!(db.tables().len(&rtxn).unwrap(), 0);
    }

    #[test]
    fn open_is_idempotent_on_reopen() {
        let dir = tempfile::tempdir().unwrap();
        let path = dir.path().join("reopen.pqlite");
        {
            let _db = HeedDB::open(&path).unwrap();
        }
        let reopened = HeedDB::open(&path);
        assert!(reopened.is_ok(), "reopening an existing db must succeed");
    }

    #[test]
    fn open_table_yields_pushed_rows_in_insertion_order() {
        let dir = tempfile::tempdir().unwrap();
        let path = dir.path().join("read.pqlite");
        let db = HeedDB::open(&path).unwrap();

        let mut w = db.create_table("t").unwrap();
        w.push_row(&[0x01, 0x02]).unwrap();
        w.push_row(&[0x03, 0x04, 0x05]).unwrap();
        w.commit().unwrap();

        let rtxn = db.read_txn().unwrap();
        let tbl = db.open_table(&rtxn, "t").unwrap();

        let rows: Vec<Vec<u8>> = tbl
            .iter(&rtxn)
            .unwrap()
            .map(|res| res.unwrap().1.to_vec())
            .collect();

        assert_eq!(rows, vec![vec![0x01, 0x02], vec![0x03, 0x04, 0x05]]);
    }

    #[test]
    fn open_table_returns_table_missing_for_unknown_table() {
        let dir = tempfile::tempdir().unwrap();
        let path = dir.path().join("missing.pqlite");
        let db = HeedDB::open(&path).unwrap();
        let rtxn = db.read_txn().unwrap();
        let res = db.open_table(&rtxn, "no_such_table");
        assert!(
            matches!(res, Err(StorageError::TableMissing(ref n)) if n == "no_such_table"),
            "expected TableMissing; got {:?}",
            res
        );
    }

    #[test]
    fn normalize_db_path_prefixes_bare_filename_with_dot() {
        assert_eq!(
            normalize_db_path(Path::new("foo.pqlite")),
            Path::new("./foo.pqlite")
        );
        assert_eq!(
            normalize_db_path(Path::new("./foo.pqlite")),
            Path::new("./foo.pqlite")
        );
        assert_eq!(
            normalize_db_path(Path::new("sub/foo.pqlite")),
            Path::new("sub/foo.pqlite")
        );
        assert_eq!(
            normalize_db_path(Path::new("/tmp/foo.pqlite")),
            Path::new("/tmp/foo.pqlite")
        );
    }
}
