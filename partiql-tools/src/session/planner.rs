//! Compilation-catalog assembly, plan compilation, execution-context setup,
//! and source draining for the write paths.

use std::sync::{Arc, Mutex};
use std::time::Instant;

use partiql_common::catalog::CatalogId;
use partiql_eval::plan::EvaluationMode;
use partiql_eval::{CompilationContext, ExecutionCatalog, ExecutionContext, PlanCompiler};

use crate::catalog::{HeedCompilationCatalog, HeedExecutionCatalog};
use crate::common;
use crate::session::debug::DebugFlags;
use crate::session::outcome::DebugCapture;
use crate::storage::{HeedDB, StorageError};

/// Bundles `HeedCompilationCatalog` with the existing `TableFnCompilationCatalog`
/// under `PlanCompiler`'s single `"default"` catalog name. Also records the
/// first unresolved bare table name so `build_compiled` can fail fast with
/// "Table not found" instead of letting Permissive mode yield a MISSING row.
struct CombinedCatalog {
    table_fns: common::TableFnCompilationCatalog,
    heed: Option<HeedCompilationCatalog>,
    unresolved_table_name: Arc<Mutex<Option<String>>>,
}

impl partiql_eval::CompilationCatalog for CombinedCatalog {
    fn get_table(
        &self,
        path: &[partiql_value::BindingsName<'_>],
    ) -> Option<partiql_eval::source::DataSourceHandle> {
        if let Some(handle) = self.heed.as_ref().and_then(|h| h.get_table(path)) {
            return Some(handle);
        }
        // Only record bare single-element bindings — schema-qualified paths
        // never match anything here and aren't useful to surface as table names.
        // The compiler may probe a name multiple times; keep the first.
        if path.len() == 1 {
            if let Ok(mut guard) = self.unresolved_table_name.lock() {
                if guard.is_none() {
                    let name = match &path[0] {
                        partiql_value::BindingsName::CaseSensitive(s) => s.to_string(),
                        partiql_value::BindingsName::CaseInsensitive(s) => s.to_string(),
                    };
                    *guard = Some(name);
                }
            }
        }
        None
    }

    fn get_table_function(&self, name: &str) -> Option<partiql_eval::source::TableFunctionHandle> {
        self.table_fns.get_table_function(name)
    }
}

/// CatalogId is reused by the matching ExecutionCatalog.
pub(super) fn build_compiled(
    logical: &partiql_logical::LogicalPlan<partiql_logical::BindingsOp>,
    debug: &DebugFlags,
    db: Option<Arc<HeedDB>>,
    capture: &mut DebugCapture,
) -> Result<(partiql_eval::CompiledPlan, CatalogId), Box<dyn std::error::Error>> {
    if debug.plan {
        capture.plan = Some(format!("[Plan] {:?}", logical));
    }

    let mut context = CompilationContext::new();

    let column_names = vec!["a".to_string(), "b".to_string()];
    let unresolved_table_name: Arc<Mutex<Option<String>>> = Arc::new(Mutex::new(None));
    let combined = CombinedCatalog {
        table_fns: common::TableFnCompilationCatalog::new(column_names),
        heed: db.map(HeedCompilationCatalog::new),
        unresolved_table_name: Arc::clone(&unresolved_table_name),
    };
    let catalog: Arc<dyn partiql_eval::CompilationCatalog> = Arc::new(combined);
    let catalog_id = context.add_catalog("default", catalog);

    let mut compiler = PlanCompiler::new(&context, EvaluationMode::Permissive);
    let compiled = compiler
        .compile(logical)
        .map_err(|e| format!("Compile error: {:?}", e))?;

    // Recover from a poisoned lock so a panic in one probe doesn't downgrade a
    // hard "Table not found" error into a Permissive-mode MISSING row.
    let first_unresolved = unresolved_table_name
        .lock()
        .unwrap_or_else(|e| e.into_inner())
        .take();
    if let Some(name) = first_unresolved {
        return Err(format!("Error: Table '{}' not found", name).into());
    }

    if debug.program {
        capture.program = Some(format!("[Program]\n{}", compiled));
    }

    Ok((compiled, catalog_id))
}

/// The Heed catalog is `prepare()`-d from `compiled` before being boxed so
/// `ScanId → table-name` mappings are populated before the VM calls `create()`.
pub(super) fn build_exec_context(
    db: Option<Arc<HeedDB>>,
    catalog_id: CatalogId,
    compiled: &partiql_eval::CompiledPlan,
) -> ExecutionContext {
    let mut exec_context = ExecutionContext::new();
    exec_context.register_table_function("mem", Arc::new(common::MemTableFunction));
    exec_context.register_table_function("read", Arc::new(common::JsonIonTableFunction::read()));
    exec_context.register_table_function("stdin", Arc::new(common::JsonIonTableFunction::stdin()));
    exec_context.register_table_function("exec", Arc::new(common::JsonIonTableFunction::exec()));
    exec_context.register_table_function("curl", Arc::new(common::JsonIonTableFunction::curl()));

    if let Some(db) = db {
        let mut heed_exec = HeedExecutionCatalog::new(db);
        let scans = compiled.scans_for_catalog(catalog_id);
        heed_exec.prepare(&scans);
        exec_context.add_catalog(catalog_id, Box::new(heed_exec));
    }

    exec_context
}

/// Buffered source rows plus the timing a write-path caller needs to finish its
/// own exec timer: `(encoded rows, compile time, exec start instant)`.
pub(super) type DrainedSource = (Vec<Vec<u8>>, std::time::Duration, Instant);

/// Compile the source plan, run it, and buffer every result row into owned
/// bytes. The source is fully drained here — before any caller opens a write
/// txn — because a streaming source holds a read txn open across iteration, and
/// LMDB rejects opening a table handle in a write txn while one is live
/// (MDB_BAD_DBI). Returns the buffered rows, the compile time, and the exec
/// start instant so the caller can finish timing after its own write + commit.
pub(super) fn drain_source_rows(
    query: &partiql_logical::LogicalPlan<partiql_logical::BindingsOp>,
    debug: &DebugFlags,
    db: &Arc<HeedDB>,
    capture: &mut DebugCapture,
) -> Result<DrainedSource, Box<dyn std::error::Error>> {
    let compile_start = Instant::now();
    let (compiled, catalog_id) = build_compiled(query, debug, Some(Arc::clone(db)), capture)?;
    let compile_time = compile_start.elapsed();

    let exec_start = Instant::now();
    let exec_context = build_exec_context(Some(Arc::clone(db)), catalog_id, &compiled);
    let mut vm = partiql_eval::PartiQLVM::new(compiled, &exec_context)
        .map_err(|e| format!("Execution setup error: {:?}", e))?;

    // Snapshot RowShape before vm.execute() borrows the VM mutably.
    let row_shape = vm.shape().row_shape().clone();

    let mut rows: Vec<Vec<u8>> = Vec::new();
    // Reused warm across rows (serialize_row clears on entry); each retained row
    // is a tight clone (cap == len), so no per-row 4 KiB is held.
    let mut scratch: Vec<u8> = Vec::with_capacity(4096);
    match vm.execute() {
        Ok(partiql_eval::ExecutionResult::Query(iter)) => {
            // SAFETY: QueryIterator::next lifetime-extends its RegisterReader;
            // aliasing across next() is UB. Consume `row` before the next pull.
            for r in iter {
                let row = r.map_err(|e| {
                    format!("Error: {}", StorageError::Execution(format!("{:?}", e)))
                })?;
                crate::row_codec::serialize_row(&row, &row_shape, &mut scratch)
                    .map_err(|e| format!("Error: {}", StorageError::Codec(format!("{e}"))))?;
                rows.push(scratch.clone());
            }
        }
        Err(e) => return Err(format!("Execution setup error: {:?}", e).into()),
    }
    Ok((rows, compile_time, exec_start))
}
