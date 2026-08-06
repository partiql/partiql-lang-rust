//! DB open + schema migration ladder. Callers (CLI, tests) go through
//! `open_and_bootstrap`; direct construction is disallowed.

use std::path::Path;
use std::sync::Arc;

use crate::common::parse_statements;
use crate::session::debug::DebugFlags;
use crate::session::exec;
use crate::storage::{HeedDB, StorageError};

/// The schema version this binary bootstraps to and requires.
pub(super) const CURRENT_SCHEMA_VERSION: u32 = 1;

/// Open the DB and bring it to CURRENT_SCHEMA_VERSION. Idempotent + crash-safe.
pub(super) fn open_and_bootstrap(path: &Path) -> Result<Arc<HeedDB>, Box<dyn std::error::Error>> {
    let db = Arc::new(HeedDB::open(path)?);
    let version = db.read_schema_version()?;
    if version > CURRENT_SCHEMA_VERSION {
        return Err(format!(
            "database schema version {version} is newer than this binary \
             supports ({CURRENT_SCHEMA_VERSION}); upgrade pqlite"
        )
        .into());
    }
    // A stamped current version must satisfy its postcondition (one extra
    // catalog lookup at startup — a deliberate integrity check).
    if version == CURRENT_SCHEMA_VERSION && !db.tables_has_self_entry()? {
        return Err("database reports schema version 1 but the _tables catalog \
                    is missing or incomplete"
            .into());
    }
    // Migration ladder: each future migration is its own `version < N` block
    // that stamps N, so an older DB runs every step in order. Bumping
    // CURRENT_SCHEMA_VERSION alone is not enough — add a block.
    if version < 1 {
        bootstrap_v1(&db)?;
        let stamped = db.set_schema_version(1)?;
        // Guards a concurrent writer that stamped a newer version between our
        // read above and this stamp; monotonic set returns that higher version.
        if stamped > CURRENT_SCHEMA_VERSION {
            return Err(format!(
                "database schema version {stamped} is newer than this binary \
                 supports ({CURRENT_SCHEMA_VERSION}); upgrade pqlite"
            )
            .into());
        }
    }
    Ok(db)
}

/// Run the v1 bootstrap script. The `;`-separated script is parsed as a unit
/// and each statement executed in order. Idempotent: a crash-recovery DB whose
/// _tables already exists yields a typed StorageError::TableExists, reconciled
/// against the self-entry; any other error propagates.
fn bootstrap_v1(db: &Arc<HeedDB>) -> Result<(), Box<dyn std::error::Error>> {
    // include_str! resolves relative to this file's directory (src/session/),
    // which is the same depth as the old bin (src/bin/); the path is unchanged.
    let script = include_str!("../bootstrap/v1.pql");
    let debug = DebugFlags::from_args(&[]);
    let parsed = parse_statements(script).map_err(|e| format!("Parse error: {:?}", e))?;
    for stmt in &parsed.statements {
        // Bootstrap statements run muted: no timing footer, no captured debug.
        match exec::execute_statement_silent(stmt, &debug, Arc::clone(db)) {
            Ok(()) => {}
            Err(e) => {
                let is_self_tables_exists = e
                    .downcast_ref::<StorageError>()
                    .map(|se| matches!(se, StorageError::TableExists(n) if n == "_tables"))
                    .unwrap_or(false);
                // Crash-recovery: _tables already created on a prior run. Skip and
                // continue; any other error is real and aborts bootstrap.
                if is_self_tables_exists && db.tables_has_self_entry()? {
                    continue;
                }
                return Err(e);
            }
        }
    }
    Ok(())
}
