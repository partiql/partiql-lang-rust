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

/// Errors from opening or initializing the storage environment.
#[derive(Debug)]
pub enum StorageError {
    /// Filesystem-level error surfaced while working with the database file.
    Io(std::io::Error),
    /// Error from the heed/LMDB layer.
    Heed(heed::Error),
}

impl std::fmt::Display for StorageError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            StorageError::Io(e) => write!(f, "storage I/O error: {e}"),
            StorageError::Heed(e) => write!(f, "storage engine error: {e}"),
        }
    }
}

impl std::error::Error for StorageError {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        match self {
            StorageError::Io(e) => Some(e),
            StorageError::Heed(e) => Some(e),
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
    fn storage_error_displays_and_converts_from_io() {
        let io = std::io::Error::new(std::io::ErrorKind::NotFound, "boom");
        let err: StorageError = io.into();
        assert!(matches!(err, StorageError::Io(_)));
        let msg = err.to_string();
        assert!(msg.contains("storage I/O error"), "got: {msg}");
        assert!(msg.contains("boom"), "got: {msg}");
    }

    #[test]
    fn open_creates_empty_tables_catalog() {
        let dir = tempfile::tempdir().unwrap();
        let path = dir.path().join("cat.pal");
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
        let path = dir.path().join("reopen.pal");
        {
            let _db = HeedDB::open(&path).unwrap();
        } // first handle dropped, env closed
        let reopened = HeedDB::open(&path);
        assert!(reopened.is_ok(), "reopening an existing db must succeed");
    }
}
