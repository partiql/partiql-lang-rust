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
    /// Filesystem error (e.g. the parent directory could not be created).
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

/// Create the parent directory of `path` if needed.
///
/// In single-file (`NO_SUB_DIR`) mode LMDB does not create the containing
/// directory, so we must. A bare filename like `local.pal` has a parent of
/// `Some("")`; `create_dir_all("")` errors with `NotFound`, so we skip
/// creation when the parent is absent or empty.
fn ensure_parent_dir(path: &Path) -> std::io::Result<()> {
    if let Some(parent) = path.parent() {
        if !parent.as_os_str().is_empty() {
            std::fs::create_dir_all(parent)?;
        }
    }
    Ok(())
}

/// The default database path: `~/.pqlite/default.pal`.
///
/// Returns `None` if the home directory cannot be resolved; the binary treats
/// that as a startup error and asks the user to pass `--db`.
pub fn default_db_path() -> Option<PathBuf> {
    // `std::env::home_dir` was un-deprecated in Rust 1.85 and is correct on all
    // supported platforms, matching how the pqlite binary resolves home.
    #[allow(deprecated)]
    let mut home = std::env::home_dir()?;
    home.push(".pqlite");
    home.push("default.pal");
    Some(home)
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
        // 1. Single-file mode does not create the containing directory.
        ensure_parent_dir(path)?;

        // 2. Open the environment. `flags` and `open` are both unsafe in heed
        //    0.20, so the whole builder chain is in one unsafe block.
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

        // 3. Open/create the `_tables` catalog in a write transaction.
        //    `create_database` borrows the RwTxn mutably and opens the db if it
        //    already exists, so reopening is not an error.
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
    fn ensure_parent_dir_skips_bare_filename() {
        // Precondition: a bare filename's parent is Some(""), the case the
        // guard exists for. Assert it explicitly so this test fails loudly
        // (rather than passing vacuously) if `Path::parent`'s contract shifts.
        assert_eq!(
            Path::new("local.pal")
                .parent()
                .map(|p| p.as_os_str().is_empty()),
            Some(true),
            "test precondition: bare-filename parent must be Some(\"\")"
        );
        // `create_dir_all("")` would error with NotFound, so the guard must
        // skip it and return Ok without touching the filesystem.
        let result = ensure_parent_dir(Path::new("local.pal"));
        assert!(result.is_ok(), "bare filename should not error: {result:?}");
    }

    #[test]
    fn ensure_parent_dir_skips_root_path() {
        // `Path::new("/").parent()` is None; the guard must not error.
        assert!(ensure_parent_dir(Path::new("/")).is_ok());
    }

    #[test]
    fn ensure_parent_dir_creates_missing_parents() {
        let dir = tempfile::tempdir().unwrap();
        let nested = dir.path().join("a").join("b").join("c.pal");
        ensure_parent_dir(&nested).unwrap();
        assert!(
            nested.parent().unwrap().is_dir(),
            "parent chain should exist"
        );
    }

    #[test]
    fn default_db_path_ends_with_pqlite_default_pal() {
        // If home isn't resolvable (e.g. a hermetic CI sandbox with no HOME),
        // skip rather than fail — the binary handles that case at startup.
        if let Some(path) = default_db_path() {
            // Build the expected suffix with the OS-native separator so the
            // component-wise `ends_with` matches on every platform (a literal
            // ".pqlite/default.pal" string is a single component on Windows).
            let suffix = Path::new(".pqlite").join("default.pal");
            assert!(
                path.ends_with(&suffix),
                "unexpected default path: {}",
                path.display()
            );
        }
    }

    #[test]
    fn open_creates_file_and_missing_parent() {
        let dir = tempfile::tempdir().unwrap();
        // Parent chain `nested/sub` does not exist yet.
        let path = dir.path().join("nested").join("sub").join("test.pal");
        let db = HeedDB::open(&path).expect("open should succeed");
        assert!(path.is_file(), "db should be a single file (NO_SUB_DIR)");
        assert_eq!(db.path(), path.as_path());
    }

    #[test]
    fn open_creates_empty_tables_catalog() {
        let dir = tempfile::tempdir().unwrap();
        let path = dir.path().join("cat.pal");
        let db = HeedDB::open(&path).unwrap();
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

    #[test]
    fn open_errors_when_parent_cannot_be_created() {
        let dir = tempfile::tempdir().unwrap();
        // Create a regular FILE, then try to open a db "underneath" it, so the
        // parent-dir creation must fail.
        let blocker = dir.path().join("afile");
        std::fs::write(&blocker, b"x").unwrap();
        let bad = blocker.join("test.pal");
        let res = HeedDB::open(&bad);
        assert!(matches!(res, Err(StorageError::Io(_))), "got: {res:?}");
    }
}
