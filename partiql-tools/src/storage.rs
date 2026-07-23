//! LMDB-backed storage for pqlite.

use std::path::{Path, PathBuf};

/// Virtual address space reserved for the LMDB map; sparse on disk.
const DEFAULT_MAP_SIZE: usize = 1024 * 1024 * 1024; // 1 GiB

/// One catalog plus headroom for one named db per user table.
const MAX_DBS: u32 = 128;

pub(crate) const TABLES_DB: &str = "_tables";

/// NUL-prefixed so a user table (names can't contain NUL) can't collide.
const SCHEMA_VERSION_KEY: &str = "\0schema_version";

/// True for exact system-table names (currently just `_tables`). `pub` because
/// the binary is a separate crate target and needs it for the INSERT guard —
/// the single definition of "system name" that every guard checks against.
pub fn is_system_table(name: &str) -> bool {
    name == TABLES_DB
}

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
    SystemTableReadOnly(String),
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
            StorageError::SystemTableReadOnly(name) => {
                write!(f, "cannot modify system table '{name}'")
            }
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
            | StorageError::SystemTableReadOnly(_)
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

/// The named-database handles are valid only while the env is alive, so they
/// live together with it.
#[derive(Debug)]
pub struct HeedDB {
    env: heed::Env,
    /// Unnamed LMDB db holding raw schema-version bytes (NUL-prefixed key).
    default_db: heed::Database<heed::types::Str, heed::types::Bytes>,
    path: PathBuf,
}

impl HeedDB {
    /// Open (or create) the LMDB environment at `path` as a single file. The
    /// `_tables` system catalog is NOT created here — it is materialized by the
    /// startup bootstrap running `CREATE TABLE _tables` through the query
    /// pipeline, and opened on demand thereafter.
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
        let default_db = env.create_database(&mut wtxn, None)?;
        wtxn.commit()?;

        Ok(HeedDB {
            env,
            default_db,
            path: path.to_path_buf(),
        })
    }

    /// Open `_tables` for reading if it exists yet. `None` on a fresh DB whose
    /// catalog the bootstrap has not created.
    fn tables_ro(&self, txn: &heed::RoTxn<'_>) -> Result<Option<TablesDb>, StorageError> {
        Ok(self.env.open_database(txn, Some(TABLES_DB))?)
    }

    /// Reject names the storage layer refuses: interior NUL (would panic heed's
    /// key encoding).
    fn guard_name(name: &str) -> Result<(), StorageError> {
        if name.contains('\0') {
            return Err(StorageError::InvalidName(name.to_string()));
        }
        Ok(())
    }

    /// Open a write handle for a new table. `name` must already be
    /// canonicalized by the caller (lowercase for bare identifiers, verbatim
    /// for quoted). `tables_value` is the caller-encoded catalog row stored in
    /// `_tables` under `name` — storage stays codec-free and writes opaque bytes.
    pub fn create_table(
        &self,
        name: &str,
        tables_value: &[u8],
    ) -> Result<TableWriter<'_>, StorageError> {
        Self::guard_name(name)?;
        let mut wtxn = self.env.write_txn()?;
        // Create the target DB FIRST. For the bootstrap call `name == "_tables"`
        // this brings the catalog into existence (self-referential, no special-
        // casing). When txn ownership moves to the app layer for streaming, this
        // registration relocates to the binary.
        let table: RowDb = self.env.create_database(&mut wtxn, Some(name))?;
        let tables = self
            .env
            .open_database::<heed::types::Str, heed::types::Bytes>(&wtxn, Some(TABLES_DB))?
            .ok_or_else(|| StorageError::TableMissing(TABLES_DB.to_string()))?;
        if tables.get(&wtxn, name)?.is_some() {
            return Err(StorageError::TableExists(name.to_string()));
        }
        tables.put(&mut wtxn, name, tables_value)?;
        Ok(TableWriter {
            wtxn,
            table,
            row_id: 0,
            key_buf: [0u8; 8],
        })
    }

    /// Open an existing table for appending. Errors if the table is absent, and
    /// seeds the writer's `row_id` at the current high-water key + 1 so appended
    /// rows get fresh keys after the max.
    pub fn open_table_for_append(&self, name: &str) -> Result<TableWriter<'_>, StorageError> {
        Self::guard_name(name)?;
        if is_system_table(name) {
            return Err(StorageError::SystemTableReadOnly(name.to_string()));
        }
        let wtxn = self.env.write_txn()?;
        // Catalog membership defines existence for reads (see `open_table`), so
        // the append path checks it too: a table absent from `_tables` is not
        // appendable, matching what a subsequent SELECT would see. `_tables`
        // must exist to append to any table — a fresh, un-bootstrapped DB has
        // no appendable tables.
        let tables = self
            .env
            .open_database::<heed::types::Str, heed::types::Bytes>(&wtxn, Some(TABLES_DB))?
            .ok_or_else(|| StorageError::TableMissing(TABLES_DB.to_string()))?;
        if tables.get(&wtxn, name)?.is_none() {
            return Err(StorageError::TableMissing(name.to_string()));
        }
        // Open (not create): fail loudly if the named db is absent, rather than
        // silently re-creating an empty table and appending at row_id 0 on a
        // catalog/data drift. (`&wtxn` coerces to `&RoTxn` via Deref.)
        let table: RowDb = self
            .env
            .open_database(&wtxn, Some(name))?
            .ok_or_else(|| StorageError::TableMissing(name.to_string()))?;
        // Keys are 8-byte BE u64, so byte order equals numeric order and the last
        // key is the highest row_id. A non-8-byte key means the row-id scheme
        // changed; fail loudly rather than truncate.
        let next_row_id = match table.last(&wtxn)? {
            Some((k, _)) => {
                let arr = <[u8; 8]>::try_from(k).map_err(|_| {
                    StorageError::Codec(format!(
                        "table {}: row key is {} bytes, expected 8 (BE u64)",
                        name,
                        k.len()
                    ))
                })?;
                u64::from_be_bytes(arr) + 1
            }
            None => 0,
        };
        Ok(TableWriter {
            wtxn,
            table,
            row_id: next_row_id,
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
        match self.tables_ro(txn)? {
            Some(tables) if tables.get(txn, name)?.is_some() => {}
            _ => return Err(StorageError::TableMissing(name.to_string())),
        }
        self.env
            .open_database(txn, Some(name))?
            .ok_or_else(|| StorageError::TableMissing(name.to_string()))
    }

    /// Enumerate every table name in `_tables`, in lexicographic key order.
    /// A fresh DB whose catalog the bootstrap has not created yields an empty
    /// list.
    pub fn list_table_names(&self, txn: &heed::RoTxn<'_>) -> Result<Vec<String>, StorageError> {
        let Some(tables) = self.tables_ro(txn)? else {
            return Ok(Vec::new());
        };
        let mut out = Vec::with_capacity(tables.len(txn)? as usize);
        for result in tables.iter(txn)? {
            let (k, _v) = result?;
            out.push(k.to_string());
        }
        Ok(out)
    }

    pub fn read_schema_version(&self) -> Result<u32, StorageError> {
        let rtxn = self.env.read_txn()?;
        match self.default_db.get(&rtxn, SCHEMA_VERSION_KEY)? {
            Some(b) => Ok(u32::from_le_bytes(<[u8; 4]>::try_from(b).map_err(
                |_| {
                    StorageError::Codec(format!(
                        "schema_version is {} bytes, expected 4 (u32 LE)",
                        b.len()
                    ))
                },
            )?)),
            None => Ok(0),
        }
    }

    /// Stamp the version (raw u32 LE). Monotonic: never decreases. Returns the
    /// version now on disk (>= requested) so the caller can detect a concurrent
    /// newer stamp and reject the DB.
    pub fn set_schema_version(&self, version: u32) -> Result<u32, StorageError> {
        let mut wtxn = self.env.write_txn()?;
        let current = match self.default_db.get(&wtxn, SCHEMA_VERSION_KEY)? {
            Some(b) => u32::from_le_bytes(<[u8; 4]>::try_from(b).map_err(|_| {
                StorageError::Codec(format!(
                    "schema_version is {} bytes, expected 4 (u32 LE)",
                    b.len()
                ))
            })?),
            None => 0,
        };
        if current >= version {
            return Ok(current);
        }
        self.default_db
            .put(&mut wtxn, SCHEMA_VERSION_KEY, &version.to_le_bytes())?;
        wtxn.commit()?;
        Ok(version)
    }

    pub fn tables_has_self_entry(&self) -> Result<bool, StorageError> {
        let rtxn = self.env.read_txn()?;
        match self.tables_ro(&rtxn)? {
            Some(tables) => Ok(tables.get(&rtxn, TABLES_DB)?.is_some()),
            None => Ok(false),
        }
    }

    #[cfg(test)]
    pub(crate) fn env(&self) -> &heed::Env {
        &self.env
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

    fn tables_value(name: &str) -> Vec<u8> {
        let mut v = Vec::new();
        crate::row_codec::serialize_name_row(&[name], &mut v).unwrap();
        v
    }

    /// Open a DB and bootstrap `_tables` so user-table creates can proceed.
    fn open_bootstrapped(path: &std::path::Path) -> HeedDB {
        let db = HeedDB::open(path).unwrap();
        db.create_table("_tables", &tables_value("_tables"))
            .unwrap()
            .commit()
            .unwrap();
        db
    }

    /// Open the `_tables` catalog for reading; panics if it does not exist.
    fn tables_of(
        db: &HeedDB,
        rtxn: &heed::RoTxn<'_>,
    ) -> heed::Database<heed::types::Str, heed::types::Bytes> {
        db.env()
            .open_database(rtxn, Some("_tables"))
            .unwrap()
            .unwrap()
    }

    #[test]
    fn open_does_not_create_tables_catalog_eagerly() {
        let dir = tempfile::tempdir().unwrap();
        let db = HeedDB::open(&dir.path().join("lazy.pqlite")).unwrap();
        let rtxn = db.env().read_txn().unwrap();
        let named = db
            .env()
            .open_database::<heed::types::Str, heed::types::Bytes>(&rtxn, Some("_tables"))
            .unwrap();
        assert!(named.is_none(), "_tables must not be created eagerly");
    }

    #[test]
    fn create_user_table_before_catalog_errors() {
        let dir = tempfile::tempdir().unwrap();
        let db = HeedDB::open(&dir.path().join("nocat.pqlite")).unwrap();
        let res = db.create_table("foo", &tables_value("foo"));
        assert!(matches!(res, Err(StorageError::TableMissing(ref n)) if n == "_tables"));
    }

    #[test]
    fn self_referential_create_tables_bootstraps_catalog() {
        let dir = tempfile::tempdir().unwrap();
        let db = HeedDB::open(&dir.path().join("selfref.pqlite")).unwrap();
        db.create_table("_tables", &tables_value("_tables"))
            .unwrap()
            .commit()
            .unwrap();
        assert!(db.tables_has_self_entry().unwrap());
        db.create_table("foo", &tables_value("foo"))
            .unwrap()
            .commit()
            .unwrap();
    }

    #[test]
    fn create_table_stores_caller_supplied_tables_value() {
        let dir = tempfile::tempdir().unwrap();
        let db = open_bootstrapped(&dir.path().join("val.pqlite"));
        let value = tables_value("widgets");
        db.create_table("widgets", &value)
            .unwrap()
            .commit()
            .unwrap();
        let rtxn = db.env().read_txn().unwrap();
        let tables = tables_of(&db, &rtxn);
        assert_eq!(tables.get(&rtxn, "widgets").unwrap().unwrap(), &value[..]);
    }

    #[test]
    fn schema_version_absent_reads_zero() {
        let dir = tempfile::tempdir().unwrap();
        let db = HeedDB::open(&dir.path().join("v.pqlite")).unwrap();
        assert_eq!(db.read_schema_version().unwrap(), 0);
    }

    #[test]
    fn schema_version_round_trips_raw_le_bytes() {
        let dir = tempfile::tempdir().unwrap();
        let db = HeedDB::open(&dir.path().join("v.pqlite")).unwrap();
        assert_eq!(db.set_schema_version(1).unwrap(), 1);
        assert_eq!(db.read_schema_version().unwrap(), 1);
        let rtxn = db.env().read_txn().unwrap();
        let d: heed::Database<heed::types::Str, heed::types::Bytes> =
            db.env().open_database(&rtxn, None).unwrap().unwrap();
        assert_eq!(
            d.get(&rtxn, "\0schema_version").unwrap().unwrap(),
            &1u32.to_le_bytes()
        );
    }

    #[test]
    fn set_schema_version_is_monotonic_and_returns_observed() {
        let dir = tempfile::tempdir().unwrap();
        let db = HeedDB::open(&dir.path().join("v.pqlite")).unwrap();
        assert_eq!(db.set_schema_version(2).unwrap(), 2);
        assert_eq!(
            db.set_schema_version(1).unwrap(),
            2,
            "downgrade no-ops, returns observed"
        );
        assert_eq!(db.read_schema_version().unwrap(), 2);
    }

    #[test]
    fn malformed_version_bytes_error_not_panic() {
        let dir = tempfile::tempdir().unwrap();
        let db = HeedDB::open(&dir.path().join("v.pqlite")).unwrap();
        {
            let mut wtxn = db.env().write_txn().unwrap();
            let d: heed::Database<heed::types::Str, heed::types::Bytes> =
                db.env().create_database(&mut wtxn, None).unwrap();
            d.put(&mut wtxn, "\0schema_version", &[0x01, 0x02, 0x03])
                .unwrap();
            wtxn.commit().unwrap();
        }
        assert!(matches!(
            db.read_schema_version(),
            Err(StorageError::Codec(_))
        ));
    }

    #[test]
    fn tables_has_self_entry_reflects_catalog() {
        let dir = tempfile::tempdir().unwrap();
        let db = HeedDB::open(&dir.path().join("v.pqlite")).unwrap();
        // No catalog exists until _tables is created as a table.
        assert!(!db.tables_has_self_entry().unwrap());
        db.create_table("_tables", &tables_value("_tables"))
            .unwrap()
            .commit()
            .unwrap();
        assert!(db.tables_has_self_entry().unwrap());
    }

    #[test]
    fn table_writer_round_trip_via_push_and_commit() {
        let dir = tempfile::tempdir().unwrap();
        let path = dir.path().join("writer.pqlite");
        let db = open_bootstrapped(&path);

        let mut w = db
            .create_table("widgets", &tables_value("widgets"))
            .expect("create_table");
        w.push_row(&[0xDE, 0xAD]).unwrap();
        w.push_row(&[0xBE, 0xEF, 0x01]).unwrap();
        let n = w.commit().unwrap();
        assert_eq!(n, 2);

        let rtxn = db.env().read_txn().unwrap();
        assert!(tables_of(&db, &rtxn)
            .get(&rtxn, "widgets")
            .unwrap()
            .is_some());
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
        let db = open_bootstrapped(&path);

        let payloads: [&[u8]; 3] = [
            &[0xDE, 0xAD, 0xBE, 0xEF],
            &[0x01],
            &[0xFF, 0x00, 0xFF, 0x00, 0xFF],
        ];
        let mut w = db
            .create_table("widgets", &tables_value("widgets"))
            .unwrap();
        for p in payloads.iter() {
            w.push_row(p).unwrap();
        }
        let n = w.commit().unwrap();
        assert_eq!(n, 3);

        let rtxn = db.env().read_txn().unwrap();
        assert!(
            tables_of(&db, &rtxn)
                .get(&rtxn, "widgets")
                .unwrap()
                .is_some(),
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
        let db = open_bootstrapped(&path);

        let n = db
            .create_table("blank", &tables_value("blank"))
            .unwrap()
            .commit()
            .unwrap();
        assert_eq!(n, 0);

        let rtxn = db.env().read_txn().unwrap();
        assert!(tables_of(&db, &rtxn).get(&rtxn, "blank").unwrap().is_some());
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
        let db = open_bootstrapped(&path);

        let mut w1 = db.create_table("t", &tables_value("t")).unwrap();
        w1.push_row(&[0x01]).unwrap();
        w1.commit().unwrap();
        let again = db.create_table("t", &tables_value("t"));
        assert!(
            matches!(again, Err(StorageError::TableExists(ref n)) if n == "t"),
            "expected TableExists; got {:?}",
            again
        );

        let rtxn = db.env().read_txn().unwrap();
        // _tables (self-entry) plus the one user table t.
        assert_eq!(tables_of(&db, &rtxn).len(&rtxn).unwrap(), 2);
        // Release the read txn before read_row_for_tests opens its own; LMDB
        // rejects a second concurrent read txn on the same thread (BadRslot).
        drop(rtxn);

        // Rejected re-create must not clear the original row: create_table now
        // creates the target DB before the duplicate check, so a future
        // clear-then-recreate regression would silently drop existing data.
        assert_eq!(db.read_row_for_tests("t", 0), vec![0x01]);
    }

    #[test]
    fn create_table_allows_underscore_prefixed_names_and_persists() {
        let dir = tempfile::tempdir().unwrap();
        let db = open_bootstrapped(&dir.path().join("underscore.pqlite"));
        for name in ["_x", "_temp", "_internal"] {
            db.create_table(name, &tables_value(name))
                .unwrap()
                .commit()
                .unwrap(); // commit → prove persistence
        }
        let rtxn = db.env().read_txn().unwrap();
        let names = db.list_table_names(&rtxn).unwrap();
        for name in ["_x", "_temp", "_internal"] {
            assert!(names.contains(&name.to_string()), "{name} should persist");
        }
    }

    #[test]
    fn create_table_rejects_interior_nul_names() {
        let dir = tempfile::tempdir().unwrap();
        let path = dir.path().join("nul.pqlite");
        let db = open_bootstrapped(&path);

        // Interior NUL would otherwise panic heed's CString::new.
        let res = db.create_table("a\0b", &tables_value("x"));
        assert!(
            matches!(res, Err(StorageError::InvalidName(_))),
            "expected InvalidName; got {:?}",
            res
        );

        let rtxn = db.env().read_txn().unwrap();
        // Only the bootstrapped _tables self-entry; the rejected name added nothing.
        assert_eq!(tables_of(&db, &rtxn).len(&rtxn).unwrap(), 1);
    }

    #[test]
    fn dropping_writer_rolls_back_uncommitted_rows() {
        let dir = tempfile::tempdir().unwrap();
        let path = dir.path().join("rb.pqlite");
        let db = open_bootstrapped(&path);
        {
            let mut w = db
                .create_table("partial", &tables_value("partial"))
                .unwrap();
            w.push_row(&[0x42]).unwrap();
            // Drop without commit — wtxn rolls back.
        }
        let rtxn = db.env().read_txn().unwrap();
        assert!(
            tables_of(&db, &rtxn)
                .get(&rtxn, "partial")
                .unwrap()
                .is_none(),
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

        let sr = StorageError::SystemTableReadOnly("_tables".to_string());
        let s = sr.to_string();
        assert!(s.contains("system table"), "got: {s}");
        assert!(s.contains("_tables"), "got: {s}");

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
        let db = open_bootstrapped(&path);

        let mut w = db.create_table("t", &tables_value("t")).unwrap();
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
    fn append_to_existing_table_continues_row_ids() {
        let dir = tempfile::tempdir().unwrap();
        let db = open_bootstrapped(&dir.path().join("t.pqlite"));
        let mut w = db.create_table("t", &tables_value("t")).unwrap();
        w.push_row(&[0xAA]).unwrap();
        w.push_row(&[0xBB]).unwrap();
        w.push_row(&[0xCC]).unwrap();
        w.commit().unwrap();
        let mut a = db.open_table_for_append("t").unwrap();
        a.push_row(&[0xDD]).unwrap();
        a.push_row(&[0xEE]).unwrap();
        let final_row_id = a.commit().unwrap();
        assert_eq!(final_row_id, 5, "row_id counter should be 5 after 3+2 rows");
        assert_eq!(db.read_row_for_tests("t", 3), vec![0xDD]);
        assert_eq!(db.read_row_for_tests("t", 4), vec![0xEE]);
        assert_eq!(db.read_row_for_tests("t", 0), vec![0xAA]);
    }

    #[test]
    fn append_to_empty_table_starts_at_zero() {
        let dir = tempfile::tempdir().unwrap();
        let db = open_bootstrapped(&dir.path().join("t.pqlite"));
        db.create_table("t", &tables_value("t"))
            .unwrap()
            .commit()
            .unwrap(); // empty table, no rows
        let mut a = db.open_table_for_append("t").unwrap();
        a.push_row(&[0x01]).unwrap();
        assert_eq!(a.commit().unwrap(), 1);
        assert_eq!(db.read_row_for_tests("t", 0), vec![0x01]);
    }

    #[test]
    fn append_to_missing_table_errors() {
        let dir = tempfile::tempdir().unwrap();
        let db = open_bootstrapped(&dir.path().join("t.pqlite"));
        let r = db.open_table_for_append("ghost");
        assert!(matches!(r, Err(StorageError::TableMissing(ref n)) if n == "ghost"));
    }

    #[test]
    fn append_rejects_nul_names() {
        let dir = tempfile::tempdir().unwrap();
        let db = HeedDB::open(&dir.path().join("t.pqlite")).unwrap();
        assert!(matches!(
            db.open_table_for_append("a\0b"),
            Err(StorageError::InvalidName(_))
        ));
    }

    #[test]
    fn append_rejects_exact_system_table_name() {
        let dir = tempfile::tempdir().unwrap();
        let db = HeedDB::open(&dir.path().join("t.pqlite")).unwrap();
        assert!(
            matches!(db.open_table_for_append("_tables"), Err(StorageError::SystemTableReadOnly(ref n)) if n == "_tables"),
            "INSERT into _tables must be rejected to protect the catalog"
        );
    }

    #[test]
    fn is_system_table_matches_only_tables_db() {
        assert!(is_system_table("_tables"));
        assert!(!is_system_table("_foo"));
        assert!(!is_system_table("tables"));
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
