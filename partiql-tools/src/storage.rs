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

/// Row-store key codec: an 8-byte big-endian `u64` row id passed as raw bytes.
/// We use `heed::types::Bytes` (not `heed::types::U64<BigEndian>`) because
/// `U64`'s `BytesEncode` impl heap-allocates an 8-byte `Vec` on every `put`,
/// while `Bytes` returns `Cow::Borrowed(&[u8])` — zero allocations per row.
/// Big-endian is preserved at the call site (the caller fills a stack-local
/// `[u8; 8]` via `row_id.to_be_bytes()`). LMDB's B+tree keys are
/// byte-comparison-ordered, so BE keeps lexicographic key order matching
/// numeric order; LE would reverse iteration order.
type RowKey = heed::types::Bytes;

/// `RowDb` is the per-table B+tree keyed by big-endian u64 row ID. Bytes
/// pushed via `TableWriter::push_row` are stored verbatim — storage adds
/// no framing.
type RowDb = heed::Database<RowKey, heed::types::Bytes>;

/// Write-side handle for a single CTAS operation.
///
/// Holds the open `wtxn` and the table's `RowDb` between `create_table` and
/// `commit`. Drop without `commit` triggers `wtxn`'s rollback — no catalog
/// entry, no row bytes.
///
/// The `row_id` counter encodes as big-endian into a stack-local `[u8; 8]`
/// key buffer reused across every `push_row` call. Combined with `RowKey =
/// heed::types::Bytes` (Cow::Borrowed), this gives zero heap allocations per
/// row for the key side of the put.
pub struct TableWriter<'env> {
    wtxn: heed::RwTxn<'env>,
    table: RowDb,
    row_id: u64,
    key_buf: [u8; 8],
}

// `heed::RwTxn` does not impl `Debug`, so derive can't be used. Hand-rolled
// impl skips the txn field and surfaces the row counter — enough to read in
// test diagnostics (`{:?}` on `Result<TableWriter, _>` in the name-rejection
// tests) without leaking heed internals.
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
    /// A row-codec rejection (e.g. unsupported type/shape). The carried string
    /// already reads as a complete error sentence (e.g. "unsupported: column
    /// 'b': Bool — tag reserved..."); Display surfaces it verbatim so stderr
    /// stays single-level ("Error: unsupported: ..."), matching PartiQL's style.
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

/// Normalize a database path so heed can open a not-yet-existing single-file
/// (`NO_SUB_DIR`) database given as a bare filename.
///
/// For a file that doesn't exist yet, heed recovers by canonicalizing
/// `path.parent()` and re-joining the file name. But `parent()` of a bare name
/// like `foo.pqlite` is the empty path `""`, and canonicalizing `""` is ENOENT —
/// so `pqlite --db foo.pqlite` would spuriously fail even though the current
/// directory exists and every other UNIX tool (`sqlite3 foo.db`, `touch`, ...)
/// would just create the file there. Joining a bare name onto `.` gives heed a
/// real parent (`.`) to canonicalize.
///
/// This neither invents a default path nor creates any directory: a path that
/// already has a directory component is returned unchanged, so a genuinely
/// missing parent (`missing_dir/foo.pqlite`) still errors — the behavior the
/// `--db` contract wants.
fn normalize_db_path(path: &Path) -> PathBuf {
    if path.parent() == Some(Path::new("")) {
        Path::new(".").join(path)
    } else {
        path.to_path_buf()
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

        // Normalize a bare filename to an explicit `./name` before handing it
        // to heed (see `normalize_db_path` for the why).
        let normalized = normalize_db_path(path);

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
                .open(&normalized)?
        };

        // `create_database` is idempotent — reopening an existing db is fine.
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
    /// check, format-version write) and opens a write transaction. The
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
        // Catalog value is the format-version byte. Zero-row tables have no
        // rows to read it from, and a `[0x00]` sentinel would collide with
        // TAG_INTEGER if a tool ran the catalog cell through the row parser.
        self.tables
            .put(&mut wtxn, name, &[crate::row_codec::FORMAT_VERSION])?;
        Ok(TableWriter {
            wtxn,
            table,
            row_id: 0,
            key_buf: [0u8; 8],
        })
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
