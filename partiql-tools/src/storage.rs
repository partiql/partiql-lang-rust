//! LMDB-backed storage for pqlite.

use std::path::{Path, PathBuf};

/// Virtual address space reserved for the LMDB map. Sparse on disk.
const DEFAULT_MAP_SIZE: usize = 1024 * 1024 * 1024; // 1 GiB

/// One catalog plus headroom for one named db per user table.
const MAX_DBS: u32 = 128;

const TABLES_DB: &str = "_tables";

type TablesDb = heed::Database<heed::types::Str, heed::types::Bytes>;

/// Row-store keys are an 8-byte big-endian `u64` row id passed as raw bytes.
/// `Bytes` (not `U64<BigEndian>`) so the caller can pass a stack-local
/// `[u8; 8]` — zero allocations per put. BE keeps lexicographic key order
/// matching numeric order.
type RowKey = heed::types::Bytes;

type RowDb = heed::Database<RowKey, heed::types::Bytes>;

/// Write-side handle for a single CTAS operation. Drop without `commit`
/// triggers `wtxn`'s rollback. `key_buf` is reused across pushes for a
/// zero-alloc key path.
pub struct TableWriter<'env> {
    wtxn: heed::RwTxn<'env>,
    table: RowDb,
    row_id: u64,
    key_buf: [u8; 8],
}

// Hand-rolled `Debug` because `heed::RwTxn` does not implement it.
impl<'env> std::fmt::Debug for TableWriter<'env> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("TableWriter")
            .field("row_id", &self.row_id)
            .finish_non_exhaustive()
    }
}

impl<'env> TableWriter<'env> {
    /// Encode the next row. `bytes` is the caller-encoded row payload —
    /// storage adds no framing.
    pub fn push_row(&mut self, bytes: &[u8]) -> Result<(), StorageError> {
        self.key_buf = self.row_id.to_be_bytes();
        self.table.put(&mut self.wtxn, &self.key_buf[..], bytes)?;
        self.row_id += 1;
        Ok(())
    }

    /// Commit the write txn and return the number of rows persisted.
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
    /// Name contains an interior NUL byte (heed would panic).
    InvalidName(String),
    Execution(String),
    Codec(String),
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
            StorageError::Execution(msg) => write!(f, "execution error: {msg}"),
            StorageError::Codec(msg) => write!(f, "{msg}"),
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
            | StorageError::Execution(_)
            | StorageError::Codec(_) => None,
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
/// Without this, `--db foo.pqlite` fails ENOENT because `parent()` of a
/// bare name is the empty path. Paths with a real directory component are
/// returned unchanged, so a genuinely missing parent still errors.
fn normalize_db_path(path: &Path) -> PathBuf {
    if path.parent() == Some(Path::new("")) {
        Path::new(".").join(path)
    } else {
        path.to_path_buf()
    }
}

/// Owns the LMDB environment and the `_tables` catalog handle. The catalog
/// handle is valid only while the env is alive, so both live together.
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

        // `create_database` is idempotent.
        let mut wtxn = env.write_txn()?;
        let tables: TablesDb = env.create_database(&mut wtxn, Some(TABLES_DB))?;
        wtxn.commit()?;

        Ok(HeedDB {
            env,
            tables,
            path: path.to_path_buf(),
        })
    }

    /// Open a write handle for a new table named `name`.
    ///
    /// Runs all up-front catalog work (reserved-name guard, NUL guard, dup
    /// check, catalog registration) and opens a write transaction. The
    /// returned `TableWriter` lives until `commit` (success) or drop
    /// (rollback). `name` must already be canonicalized by the caller —
    /// bare identifiers folded to lowercase, quoted identifiers verbatim.
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
        // Catalog value is empty: the cell records existence only. Table-
        // level metadata (schema, etc.) lands in a future PR.
        self.tables.put(&mut wtxn, name, &[])?;
        Ok(TableWriter {
            wtxn,
            table,
            row_id: 0,
            key_buf: [0u8; 8],
        })
    }

    /// The live LMDB environment. Test-only; heed types do not leak across
    /// the module boundary.
    #[cfg(test)]
    pub(crate) fn env(&self) -> &heed::Env {
        &self.env
    }

    /// The `_tables` system catalog handle. Test-only.
    #[cfg(test)]
    pub(crate) fn tables(&self) -> &TablesDb {
        &self.tables
    }

    pub fn path(&self) -> &Path {
        &self.path
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
        // Bit-perfect roundtrip: storage's contract is "persist whatever bytes
        // the caller pushes." A concrete sentinel sequence proves it.
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

        // Guard runs before any transaction, so the catalog is untouched.
        let rtxn = db.env().read_txn().unwrap();
        assert_eq!(db.tables().len(&rtxn).unwrap(), 0);
    }

    #[test]
    fn create_table_rejects_interior_nul_names() {
        let dir = tempfile::tempdir().unwrap();
        let path = dir.path().join("nul.pqlite");
        let db = HeedDB::open(&path).unwrap();

        // An interior NUL would otherwise panic heed's CString::new; the guard
        // turns it into a clean error and never opens a transaction.
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
            // Drop w without commit — wtxn rolls back.
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
    }

    #[test]
    fn open_creates_empty_tables_catalog() {
        let dir = tempfile::tempdir().unwrap();
        let path = dir.path().join("cat.pqlite");
        let db = HeedDB::open(&path).unwrap();

        // NO_SUB_DIR mode stores the environment as a single file, not a dir.
        assert!(path.is_file(), "db should be a single file (NO_SUB_DIR)");

        let rtxn = db.env().read_txn().unwrap();

        // Assert the named `_tables` database specifically exists: look it up by
        // the literal on-disk name (not the TABLES_DB constant, so renaming the
        // constant can't make this pass spuriously). `open_database` returns
        // `None` when no database of that name exists.
        let named = db
            .env()
            .open_database::<heed::types::Str, heed::types::Bytes>(&rtxn, Some("_tables"))
            .unwrap();
        assert!(
            named.is_some(),
            "the `_tables` catalog should exist by name"
        );

        // And it starts empty.
        assert_eq!(db.tables().len(&rtxn).unwrap(), 0);
    }

    #[test]
    fn open_is_idempotent_on_reopen() {
        let dir = tempfile::tempdir().unwrap();
        let path = dir.path().join("reopen.pqlite");
        {
            let _db = HeedDB::open(&path).unwrap();
        } // first handle dropped, env closed
        let reopened = HeedDB::open(&path);
        assert!(reopened.is_ok(), "reopening an existing db must succeed");
    }

    #[test]
    fn normalize_db_path_prefixes_bare_filename_with_dot() {
        // A bare filename (empty parent) gets an explicit `.` parent so heed can
        // canonicalize it; anything with a real directory component is untouched.
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
