//! LMDB-backed storage for pqlite.
//!
//! All heed/LMDB contact lives here so the binary stays thin and this logic is
//! unit-testable in isolation. PR 1 opens the environment and creates the
//! `_tables` system catalog; it writes nothing into the catalog.

use std::path::{Path, PathBuf};

/// Virtual address space reserved for the LMDB map. Sparse (not pre-allocated
/// on disk); LMDB grows actual usage up to this ceiling.
const DEFAULT_MAP_SIZE: usize = 1024 * 1024 * 1024; // 1 GiB

/// Upper bound on named databases in the environment. One catalog db now, with
/// headroom for one named db per user table in later PRs.
const MAX_DBS: u32 = 128;

/// Name of the system catalog database inside the environment.
const TABLES_DB: &str = "_tables";

/// The `_tables` catalog database: table-name keys to opaque byte values. The
/// value format is deliberately left opaque until the table-registration PR.
type TablesDb = heed::Database<heed::types::Str, heed::types::Bytes>;

/// Row-store key codec: a `u64` row id, big-endian so LMDB's lexicographic key
/// order matches numeric order.
type RowKey = heed::types::U64<heed::byteorder::BigEndian>;

/// A user table's row store: `u64` row-id keys to opaque byte values. PR 2
/// writes a constant `&[0x00]` value per row; Ion-encoded values land in PR 3.
type RowDb = heed::Database<RowKey, heed::types::Bytes>;

/// Errors from opening or initializing the storage environment, or from
/// executing a write path against it.
#[derive(Debug)]
pub enum StorageError {
    /// Filesystem-level error surfaced while working with the database file.
    Io(std::io::Error),
    /// Error from the heed/LMDB layer.
    Heed(heed::Error),
    /// A `CREATE TABLE AS` named a table already registered in the catalog.
    TableExists(String),
    /// A `CREATE TABLE AS` used a reserved name (`_`-prefixed system namespace).
    ReservedName(String),
    /// A table name heed cannot use as a database name (it contains an interior
    /// NUL byte, which would panic heed's internal `CString::new`).
    InvalidName(String),
    /// A VM execution error, pre-stringified by the caller so this layer does
    /// not depend on `partiql-eval`'s error type.
    Execution(String),
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
            | StorageError::Execution(_) => None,
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

/// Owns the LMDB environment and the `_tables` system catalog handle.
///
/// The `Env` and the `Database` handle share a lifecycle: the catalog handle is
/// valid only while the `Env` is alive, so both live on this struct together.
/// Later PRs reuse `env()` to begin transactions and `tables()` across them
/// without re-resolving the catalog by name.
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
        // We do not create the parent directory on the user's behalf. In
        // single-file (`NO_SUB_DIR`) mode LMDB opens the file directly, so if
        // the parent directory is missing the filesystem error bubbles up
        // naturally — matching how standard UNIX tools behave.

        // Open the environment. `flags` and `open` are both unsafe in heed
        // 0.20, so the whole builder chain is in one unsafe block.
        // SAFETY: NO_SUB_DIR only changes the on-disk layout (single file vs.
        // directory) and carries none of the corruption-prone semantics of the
        // dangerous flags (NO_SYNC / NO_META_SYNC / NO_LOCK). open() memory-maps
        // the file; inter-process coordination is handled by LMDB's sibling
        // lock file, and pqlite opens each env path at most once per process.
        let env = unsafe {
            heed::EnvOpenOptions::new()
                .map_size(DEFAULT_MAP_SIZE)
                .max_dbs(MAX_DBS)
                .flags(heed::EnvFlags::NO_SUB_DIR)
                .open(path)?
        };

        // Open/create the `_tables` catalog in a write transaction.
        // `create_database` borrows the RwTxn mutably and opens the db if it
        // already exists, so reopening is not an error.
        let mut wtxn = env.write_txn()?;
        let tables: TablesDb = env.create_database(&mut wtxn, Some(TABLES_DB))?;
        wtxn.commit()?;

        Ok(HeedDB {
            env,
            tables,
            path: path.to_path_buf(),
        })
    }

    /// Create a new table from a stream of rows, in a single write transaction.
    ///
    /// PR 2 writes a constant `&[0x00]` value per row; Ion serialization lands
    /// in PR 3. Only the transaction boundary, table creation, catalog
    /// registration, and row-iteration loop are exercised here.
    ///
    /// `name` must already be canonicalized by the caller (bare identifiers
    /// folded to lowercase, quoted identifiers verbatim). Returns the number
    /// of rows written. Any error drops the `RwTxn` before commit, so LMDB
    /// rolls the whole operation back and nothing partial persists.
    pub fn create_table_from_rows<I>(&self, name: &str, rows: I) -> Result<u64, StorageError>
    where
        I: IntoIterator<Item = Result<(), StorageError>>,
    {
        // Reserved-name guard runs FIRST, before any transaction. heed performs
        // no runtime codec check when reopening a named db, so creating a table
        // called `_tables` would reopen the catalog's own dbi with integer keys
        // and corrupt it. Reject the whole `_` namespace up front.
        if name.starts_with('_') {
            return Err(StorageError::ReservedName(name.to_string()));
        }

        // A name with an interior NUL would panic heed's internal
        // `CString::new(name).unwrap()` in `create_database`. Reject it here as a
        // clean error rather than letting a user-supplied quoted identifier
        // (e.g. `CREATE TABLE "a\0b" AS ...`) reach that unwrap.
        if name.contains('\0') {
            return Err(StorageError::InvalidName(name.to_string()));
        }

        let mut wtxn = self.env.write_txn()?;

        // Duplicate check against the catalog, in the same txn. `RwTxn` derefs
        // to `RoTxn`, so the existence check and the creation are atomic.
        if self.tables.get(&wtxn, name)?.is_some() {
            return Err(StorageError::TableExists(name.to_string()));
        }

        // Create the per-table row store and register the table in the catalog.
        let table: RowDb = self.env.create_database(&mut wtxn, Some(name))?;
        self.tables.put(&mut wtxn, name, &[0x00])?;

        // Dummy write loop: sequential u64 key, constant byte value.
        let mut row_id: u64 = 0;
        for row in rows {
            row?; // a VM error (Execution) bubbles out; dropping wtxn rolls back.
            table.put(&mut wtxn, &row_id, &[0x00])?;
            row_id += 1;
        }

        wtxn.commit()?;
        Ok(row_id)
    }

    // TODO: these accessors expose heed types (`Env`, `Database`) directly,
    // which leaks the storage engine across the module boundary. If a later
    // milestone abstracts storage away from heed (e.g. custom index
    // structures), wrap these behind an engine-agnostic interface instead.

    /// The live environment. Used by later PRs to begin transactions.
    pub fn env(&self) -> &heed::Env {
        &self.env
    }

    /// The `_tables` system catalog handle (opaque byte values for now).
    pub fn tables(&self) -> &TablesDb {
        &self.tables
    }

    /// The resolved on-disk path this environment was opened at.
    pub fn path(&self) -> &Path {
        &self.path
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn create_table_from_rows_writes_all_rows_and_registers_catalog() {
        let dir = tempfile::tempdir().unwrap();
        let path = dir.path().join("ctas.pqlite");
        let db = HeedDB::open(&path).unwrap();

        let rows: Vec<Result<(), StorageError>> = vec![Ok(()), Ok(()), Ok(())];
        let n = db.create_table_from_rows("widgets", rows).unwrap();
        assert_eq!(n, 3);

        let rtxn = db.env().read_txn().unwrap();

        // The table is registered in the catalog.
        assert!(
            db.tables().get(&rtxn, "widgets").unwrap().is_some(),
            "widgets should be registered in _tables"
        );

        // The row store has exactly keys 0, 1, 2.
        let rowdb: RowDb = db
            .env()
            .open_database::<RowKey, heed::types::Bytes>(&rtxn, Some("widgets"))
            .unwrap()
            .expect("widgets row db should exist");
        assert_eq!(rowdb.len(&rtxn).unwrap(), 3);
        for i in 0u64..3 {
            assert!(rowdb.get(&rtxn, &i).unwrap().is_some(), "row {i} missing");
        }
    }

    #[test]
    fn create_table_from_rows_with_no_rows_creates_empty_table() {
        let dir = tempfile::tempdir().unwrap();
        let path = dir.path().join("empty.pqlite");
        let db = HeedDB::open(&path).unwrap();

        let rows: Vec<Result<(), StorageError>> = Vec::new();
        let n = db.create_table_from_rows("blank", rows).unwrap();
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
    fn create_table_from_rows_rejects_duplicate_name() {
        let dir = tempfile::tempdir().unwrap();
        let path = dir.path().join("dup.pqlite");
        let db = HeedDB::open(&path).unwrap();

        db.create_table_from_rows("t", vec![Ok(()), Ok(())])
            .unwrap();

        let again = db.create_table_from_rows("t", vec![Ok(())]);
        assert!(
            matches!(again, Err(StorageError::TableExists(ref n)) if n == "t"),
            "second create should be TableExists, got: {again:?}"
        );

        // The original table is untouched: catalog still has exactly one entry.
        let rtxn = db.env().read_txn().unwrap();
        assert_eq!(db.tables().len(&rtxn).unwrap(), 1);
    }

    #[test]
    fn create_table_from_rows_rejects_reserved_names() {
        let dir = tempfile::tempdir().unwrap();
        let path = dir.path().join("reserved.pqlite");
        let db = HeedDB::open(&path).unwrap();

        for name in ["_tables", "_indexes", "_x"] {
            let res = db.create_table_from_rows(name, vec![Ok(())]);
            assert!(
                matches!(res, Err(StorageError::ReservedName(_))),
                "{name} should be reserved, got: {res:?}"
            );
        }

        // The guard runs before any transaction, so the catalog is untouched.
        let rtxn = db.env().read_txn().unwrap();
        assert_eq!(db.tables().len(&rtxn).unwrap(), 0);
    }

    #[test]
    fn create_table_from_rows_rejects_interior_nul_names() {
        let dir = tempfile::tempdir().unwrap();
        let path = dir.path().join("nul.pqlite");
        let db = HeedDB::open(&path).unwrap();

        // An interior NUL would otherwise panic heed's CString::new; the guard
        // turns it into a clean error and never opens a transaction.
        let res = db.create_table_from_rows("a\0b", vec![Ok(())]);
        assert!(
            matches!(res, Err(StorageError::InvalidName(_))),
            "interior-NUL name should be InvalidName, got: {res:?}"
        );

        let rtxn = db.env().read_txn().unwrap();
        assert_eq!(db.tables().len(&rtxn).unwrap(), 0);
    }

    #[test]
    fn create_table_from_rows_rolls_back_on_mid_stream_error() {
        let dir = tempfile::tempdir().unwrap();
        let path = dir.path().join("rollback.pqlite");
        let db = HeedDB::open(&path).unwrap();

        let rows: Vec<Result<(), StorageError>> = vec![
            Ok(()),
            Ok(()),
            Err(StorageError::Execution("boom".to_string())),
        ];
        let res = db.create_table_from_rows("partial", rows);
        assert!(
            matches!(res, Err(StorageError::Execution(_))),
            "expected the mid-stream Execution error to surface, got: {res:?}"
        );

        // Rollback: the txn was never committed, so the catalog has no entry.
        let rtxn = db.env().read_txn().unwrap();
        assert_eq!(
            db.tables().len(&rtxn).unwrap(),
            0,
            "the failed CTAS must not register the table"
        );

        // The orphan row-store db must also be absent — rollback dropped it.
        assert!(
            db.env()
                .open_database::<RowKey, heed::types::Bytes>(&rtxn, Some("partial"))
                .unwrap()
                .is_none(),
            "the failed CTAS must not leave a row-store db behind"
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
}
