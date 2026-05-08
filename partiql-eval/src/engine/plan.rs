use std::collections::HashMap;
use std::fmt;
use std::sync::Arc;

use crate::engine::arena::Arena;
use crate::engine::catalog::ExecutionContext;
use crate::engine::error::{EngineError, Result};
use crate::engine::expr::{Inst, Program};
use crate::engine::source::RegisterWriter;
use crate::engine::source::{DataSourceImpl, ScanLayout};
use crate::engine::value::{RegisterReader, Shape, ValueRef};

/// Unique identifier for a scan operation within a compiled plan.
///
/// Each table scan gets a unique ScanId, even when scanning the same table
/// multiple times (e.g., self-joins). This enables external data sources to
/// provide different implementations for different scans of the same table.
#[derive(Clone, Copy, Debug, Hash, Eq, PartialEq)]
pub struct ScanId(u64);

impl ScanId {
    pub(crate) fn new(id: u64) -> Self {
        ScanId(id)
    }

    pub fn as_u64(&self) -> u64 {
        self.0
    }
}

// Re-export ObjectId from partiql_common for convenience
pub use partiql_common::catalog::ObjectId;

/// Metadata for a scan operation, known at compile time.
///
/// Associates a scan with its layout and the table being scanned.
#[derive(Clone, Debug)]
pub struct ScanMetadata {
    pub layout: ScanLayout,
    pub object_id: ObjectId,
}

/// Metadata for a cursor in bytecode mode.
///
/// Each cursor corresponds to a data source that can be opened/closed/iterated.
/// The cursor_id in bytecode instructions indexes into the VM's cursor array,
/// and this maps it to the scan_id for data source resolution.
#[derive(Clone, Debug)]
pub struct CursorInfo {
    pub scan_id: ScanId,
}

/// Compiled query execution plan — pure bytecode.
///
/// Contains a single flat `Program` (instruction stream) that encodes the entire
/// query: scan, filter, project, limit — all as bytecode instructions. The VM
/// dispatches these instructions directly in a tight loop.
///
/// # Execution Model
/// - `OpenCursor` / `NextRow` / `CloseCursor` manage data source iteration
/// - Scalar instructions (Add, Eq, GetField, MakeTuple, etc.) compute expressions
/// - `JumpIfNotTrue` implements WHERE clause filtering
/// - `EmitRow` yields a result row (coroutine-style pause)
/// - `DecrOrJump` implements LIMIT
/// - `Halt` terminates execution
///
/// # Thread Safety
/// CompiledPlan is Send + Sync. Multiple VMs can be created from the same plan
/// for concurrent execution — each VM has its own registers, arena, and cursors.
pub struct CompiledPlan {
    /// The bytecode program encoding the full query.
    pub(crate) program: Program,
    /// Cursor metadata. Index = cursor_id used in OpenCursor/NextRow/CloseCursor.
    pub(crate) cursors: Vec<CursorInfo>,
    /// Shape describing the structure of query results.
    pub(crate) shape: Shape,
    /// Number of output slots (first N registers are the result row).
    pub(crate) slot_count: usize,
    /// Scan metadata keyed by ScanId for catalog resolution.
    pub(crate) scan_metadata: HashMap<ScanId, ScanMetadata>,
}

// Safety: Program is Send+Sync (verified by its own unsafe impl).
// All other fields are trivially Send+Sync.
unsafe impl Send for CompiledPlan {}
unsafe impl Sync for CompiledPlan {}

impl Clone for CompiledPlan {
    fn clone(&self) -> Self {
        Self {
            program: self.program.clone(),
            cursors: self.cursors.clone(),
            shape: self.shape.clone(),
            slot_count: self.slot_count,
            scan_metadata: self.scan_metadata.clone(),
        }
    }
}

impl fmt::Display for CompiledPlan {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        writeln!(f, "CompiledPlan {{")?;
        writeln!(f, "  cursors: {}", self.cursors.len())?;
        writeln!(f, "  slot_count: {}", self.slot_count)?;
        writeln!(f, "  registers: {}", self.program.reg_count)?;
        writeln!(f, "  instructions: {}", self.program.insts.len())?;
        for (i, inst) in self.program.insts.iter().enumerate() {
            writeln!(f, "    {:4}: {:?}", i, inst)?;
        }
        write!(f, "}}")
    }
}

impl Default for CompiledPlan {
    fn default() -> Self {
        CompiledPlan {
            program: Program::empty(),
            cursors: Vec::new(),
            shape: Shape::default(),
            slot_count: 0,
            scan_metadata: HashMap::new(),
        }
    }
}

impl CompiledPlan {
    /// Get the shape describing the structure of query results.
    pub fn result_shape(&self) -> &Shape {
        &self.shape
    }

    /// Get metadata for a specific scan operation.
    ///
    /// Returns `None` if the scan_id is not found in this plan.
    pub fn get_scan(&self, scan_id: ScanId) -> Option<&ScanMetadata> {
        self.scan_metadata.get(&scan_id)
    }

    /// Iterate over all scans in this plan.
    ///
    /// This allows external catalogs to inspect the plan and prepare
    /// their internal mappings during setup.
    pub fn scans(&self) -> impl Iterator<Item = (ScanId, &ScanMetadata)> + '_ {
        self.scan_metadata.iter().map(|(id, meta)| (*id, meta))
    }

    /// Extract scans for a specific catalog.
    ///
    /// Returns a `CatalogScans` containing only the scans that belong to the
    /// specified catalog, ready to pass to `ExecutionCatalog::prepare()`.
    pub fn scans_for_catalog(
        &self,
        catalog_id: partiql_common::catalog::CatalogId,
    ) -> crate::engine::catalog::CatalogScans {
        let mut scans = crate::engine::catalog::CatalogScans::new();
        for (scan_id, scan_meta) in self.scan_metadata.iter() {
            if scan_meta.object_id.catalog_id() == catalog_id {
                scans.add(
                    *scan_id,
                    scan_meta.object_id.entry_id(),
                    scan_meta.layout.clone(),
                );
            }
        }
        scans
    }
}

/// Single-threaded bytecode virtual machine for executing a compiled PartiQL plan.
///
/// The VM owns all execution state:
/// - Instruction pointer (IP) for the bytecode dispatch loop
/// - Data source cursors (one per `cursor_id` in the program)
/// - Memory arena for per-row computed values
/// - Unified register array (slots + temporaries)
///
/// # Execution Model
/// The VM implements a coroutine-style iterator:
/// - `execute()` returns a `QueryIterator`
/// - Each call to `next()` runs the bytecode dispatch loop until `EmitRow` or `Halt`
/// - `EmitRow` yields a row and saves IP; next call resumes from IP
/// - `Halt` terminates the iteration
///
/// The VM is fully reusable — call `set_context()` to rebind data sources.
/// Multiple VMs can be created from the same `CompiledPlan` for concurrent execution.
pub struct PartiQLVM {
    compiled: Arc<CompiledPlan>,
    /// Data source cursors. Index = cursor_id.
    cursors: Vec<Option<DataSourceImpl>>,
    /// Per-row memory arena for computed values.
    /// Reset at each `NextRow` instruction.
    arena: Arena,
    /// Unified register array: [0..slot_count] are output slots, rest are temporaries.
    registers: Vec<ValueRef<'static>>,
    /// Instruction pointer — saved across `EmitRow` yield points.
    ip: usize,
    /// Whether execution has completed (Halt reached).
    halted: bool,
    /// Number of output slots.
    slot_count: usize,
}

impl PartiQLVM {
    /// Create a new VM instance from a compiled plan.
    ///
    /// # Arguments
    /// * `compiled` - The compiled query plan (bytecode) to execute
    /// * `exec_context` - ExecutionContext for resolving catalog-based data sources
    pub fn new(compiled: CompiledPlan, exec_context: &ExecutionContext) -> Result<Self> {
        let compiled = Arc::new(compiled);
        let slot_count = compiled.slot_count;

        // Allocate cursor slots (None until OpenCursor is executed)
        let mut cursors: Vec<Option<DataSourceImpl>> = Vec::with_capacity(compiled.cursors.len());
        for _ in 0..compiled.cursors.len() {
            cursors.push(None);
        }

        // Allocate unified register array
        let total_regs = compiled.program.reg_count as usize;
        let registers = vec![ValueRef::Missing; total_regs];

        let mut vm = PartiQLVM {
            compiled,
            cursors,
            arena: Arena::new(16384),
            registers,
            ip: 0,
            halted: false,
            slot_count,
        };

        // Pre-create data sources from catalog
        vm.bind_cursors(exec_context)?;

        Ok(vm)
    }

    /// Bind all cursors to data sources from the execution context.
    fn bind_cursors(&mut self, exec_context: &ExecutionContext) -> Result<()> {
        for (cursor_id, cursor_info) in self.compiled.cursors.iter().enumerate() {
            let scan_meta = self.compiled.get_scan(cursor_info.scan_id).ok_or_else(|| {
                EngineError::IllegalState(format!("Unknown scan_id: {:?}", cursor_info.scan_id))
            })?;
            let catalog = exec_context
                .get_catalog(scan_meta.object_id.catalog_id())
                .ok_or_else(|| {
                    EngineError::IllegalState(format!(
                        "Catalog {:?} not found",
                        scan_meta.object_id.catalog_id()
                    ))
                })?;
            let data_source = catalog.create(cursor_info.scan_id)?;
            self.cursors[cursor_id] = Some(DataSourceImpl::Catalog(data_source));
        }
        Ok(())
    }

    /// Get the result shape for this VM's query.
    pub fn shape(&self) -> &Shape {
        self.compiled.result_shape()
    }

    /// Execute the plan and return streaming results.
    ///
    /// Returns a `QueryIterator` that yields rows one at a time via the
    /// bytecode dispatch loop. Each call to `next()` runs until `EmitRow`
    /// or `Halt`.
    ///
    /// # Example
    /// ```ignore
    /// let mut vm = PartiQLVM::new(compiled, &exec_context)?;
    /// match vm.execute()? {
    ///     ExecutionResult::Query(iter) => {
    ///         for row in iter {
    ///             let row = row?;
    ///             // Process row — valid until next iteration
    ///         }
    ///     }
    /// }
    /// ```
    pub fn execute(&mut self) -> Result<ExecutionResult<'_>> {
        // Reset VM state for a new execution
        self.ip = 0;
        self.halted = false;
        self.arena.reset();
        // Reset registers
        for reg in self.registers.iter_mut() {
            *reg = ValueRef::Missing;
        }
        Ok(ExecutionResult::Query(QueryIterator::new(self)))
    }

    /// Update the ExecutionContext for this VM.
    ///
    /// Rebinds all cursors to new data sources. The VM must not have active iterators.
    pub fn set_context(&mut self, exec_context: &ExecutionContext) -> Result<()> {
        // Close any open cursors
        for cursor in self.cursors.iter_mut() {
            if let Some(ds) = cursor.as_mut() {
                let _ = ds.close();
            }
            *cursor = None;
        }
        self.bind_cursors(exec_context)?;
        self.ip = 0;
        self.halted = false;
        self.arena.reset();
        Ok(())
    }
}

/// Result of query execution.
pub enum ExecutionResult<'vm> {
    /// Query results — streaming iterator over rows.
    Query(QueryIterator<'vm>),
}

/// Iterator over query result rows, driven by the bytecode dispatch loop.
///
/// Each call to `next()` runs the VM from the current IP until:
/// - `EmitRow` → yields a row, saves IP for next resumption
/// - `Halt` → returns None (query complete)
/// - Error → returns Some(Err(...))
///
/// Implements RAII: cursors are closed on drop.
///
/// **Note**: Each row is only valid until the next call to `next()`.
pub struct QueryIterator<'vm> {
    vm: &'vm mut PartiQLVM,
}

impl<'vm> QueryIterator<'vm> {
    fn new(vm: &'vm mut PartiQLVM) -> Self {
        QueryIterator { vm }
    }

    /// Run the bytecode dispatch loop until EmitRow or Halt.
    ///
    /// This is the hot path — a tight loop dispatching relational and scalar
    /// instructions. Relational instructions (OpenCursor, NextRow, etc.) are
    /// handled inline. Scalar instructions delegate to `Program::eval_inst()`.
    fn next_row_internal(&mut self) -> Option<Result<RegisterReader<'_>>> {
        if self.vm.halted {
            return None;
        }

        // Transmute register lifetimes to match arena lifetime.
        // Safety: VM owns both arena and registers; arena is reset per-row,
        // and callers don't hold row references across next() calls.
        let regs = unsafe {
            std::mem::transmute::<&mut [ValueRef<'static>], &mut [ValueRef<'_>]>(
                self.vm.registers.as_mut_slice(),
            )
        };

        let program = &self.vm.compiled.program;
        let insts = &program.insts;

        loop {
            if self.vm.ip >= insts.len() {
                self.vm.halted = true;
                return None;
            }

            let inst = &insts[self.vm.ip];
            self.vm.ip += 1;

            match inst {
                Inst::OpenCursor { cursor_id } => {
                    let cursor = match self.vm.cursors.get_mut(*cursor_id as usize) {
                        Some(Some(c)) => c,
                        _ => {
                            return Some(Err(EngineError::IllegalState(format!(
                                "cursor {} not bound",
                                cursor_id
                            ))))
                        }
                    };
                    if let Err(e) = cursor.open() {
                        return Some(Err(e));
                    }
                }

                Inst::NextRow {
                    cursor_id,
                    eof_target,
                } => {
                    // Reset arena for the new row
                    self.vm.arena.reset();

                    let cursor = match self.vm.cursors.get_mut(*cursor_id as usize) {
                        Some(Some(c)) => c,
                        _ => {
                            return Some(Err(EngineError::IllegalState(format!(
                                "cursor {} not bound",
                                cursor_id
                            ))))
                        }
                    };

                    // Write next row into registers via RegisterWriter
                    let has_row = {
                        let mut writer = RegisterWriter::new(regs, &self.vm.arena);
                        match cursor.next_row(&mut writer) {
                            Ok(has) => has,
                            Err(e) => return Some(Err(e)),
                        }
                    };

                    if !has_row {
                        self.vm.ip = *eof_target as usize;
                    }
                }

                Inst::CloseCursor { cursor_id } => {
                    let cursor = match self.vm.cursors.get_mut(*cursor_id as usize) {
                        Some(Some(c)) => c,
                        _ => {
                            return Some(Err(EngineError::IllegalState(format!(
                                "cursor {} not bound",
                                cursor_id
                            ))))
                        }
                    };
                    if let Err(e) = cursor.close() {
                        return Some(Err(e));
                    }
                }

                Inst::Jump { target } => {
                    self.vm.ip = *target as usize;
                }

                Inst::JumpIfNotTrue { src, target } => {
                    let val = regs[*src as usize];
                    if !matches!(val, ValueRef::Bool(true)) {
                        self.vm.ip = *target as usize;
                    }
                }

                Inst::EmitRow => {
                    // Yield the current register state as a result row.
                    // IP is already advanced past EmitRow, so next call resumes correctly.
                    let slots = &regs[0..self.vm.slot_count];
                    return Some(Ok(RegisterReader::new(slots)));
                }

                Inst::Halt => {
                    self.vm.halted = true;
                    return None;
                }

                Inst::DecrOrJump {
                    counter_reg,
                    target,
                } => {
                    let counter = &mut regs[*counter_reg as usize];
                    match counter {
                        ValueRef::I64(n) if *n > 0 => {
                            *n -= 1;
                        }
                        _ => {
                            // Counter is 0 or not an integer — jump (limit reached)
                            self.vm.ip = *target as usize;
                        }
                    }
                }

                // All scalar instructions delegate to eval_inst
                other => {
                    if let Err(e) = program.eval_inst(other, &self.vm.arena, regs, None) {
                        return Some(Err(e));
                    }
                }
            }
        }
    }
}

impl<'vm> Iterator for QueryIterator<'vm> {
    type Item = Result<RegisterReader<'vm>>;

    fn next(&mut self) -> Option<Self::Item> {
        // Safety: We extend the lifetime to 'vm. This is safe because:
        // 1. The VM owns the arena and registers
        // 2. The iterator has exclusive access to the VM via &mut
        // 3. Callers must not hold references across next() calls (standard iterator contract)
        match self.next_row_internal() {
            Some(Ok(row)) => {
                let row =
                    unsafe { std::mem::transmute::<RegisterReader<'_>, RegisterReader<'vm>>(row) };
                Some(Ok(row))
            }
            Some(Err(e)) => Some(Err(e)),
            None => None,
        }
    }
}

impl Drop for QueryIterator<'_> {
    fn drop(&mut self) {
        // Best-effort close all open cursors
        for cursor in self.vm.cursors.iter_mut() {
            if let Some(ds) = cursor.as_mut() {
                let _ = ds.close();
            }
        }
        self.vm.arena.reset();
        self.vm.halted = true;
    }
}
