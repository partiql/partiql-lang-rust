use std::collections::HashMap;
use std::fmt;
use std::sync::Arc;

use crate::engine::arena::Arena;
use crate::engine::builtins::BuiltinFunctions;
use crate::engine::catalog::ExecutionContext;
use crate::engine::error::{EngineError, Result};
use crate::engine::expr::{AggFunc, Inst, Program};
use crate::engine::source::RegisterWriter;
use crate::engine::source::{DataSourceImpl, InlineDataSource, ScanLayout};
use crate::engine::value::{value_ref_to_owned, RegisterReader, Shape, ValueOwned, ValueRef};

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

/// Metadata for a table function scan, known at compile time.
///
/// Associates a table function scan with its function name, output layout,
/// and the register slots where each argument has been evaluated.
#[derive(Clone, Debug)]
pub struct TableFnScanMetadata {
    pub func_name: String,
    pub layout: ScanLayout,
    pub arg_slots: Vec<crate::engine::arena::SlotId>,
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
    /// Inline data for scans backed by literal expressions (not catalog tables).
    pub(crate) inline_scans: HashMap<ScanId, Vec<ValueOwned>>,
    /// Scan IDs that are expression-based (materialized at runtime via MaterializeCursor).
    pub(crate) expr_scan_ids: Vec<ScanId>,
    /// Table function scans keyed by ScanId. These are bound at runtime by
    /// the `CreateTableFnCursor` instruction.
    pub(crate) table_fn_scans: HashMap<ScanId, TableFnScanMetadata>,
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
            inline_scans: self.inline_scans.clone(),
            expr_scan_ids: self.expr_scan_ids.clone(),
            table_fn_scans: self.table_fn_scans.clone(),
        }
    }
}

impl fmt::Display for CompiledPlan {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        writeln!(f, "CompiledPlan {{")?;
        writeln!(f, "  slot_count: {}", self.slot_count)?;
        writeln!(f, "  registers: {}", self.program.reg_count)?;
        writeln!(f, "  cursors: {}", self.cursors.len())?;
        for (i, cursor) in self.cursors.iter().enumerate() {
            if let Some(tf_meta) = self.table_fn_scans.get(&cursor.scan_id) {
                write!(
                    f,
                    "    {:4}: table_fn \"{}\" args={:?} → ",
                    i, tf_meta.func_name, tf_meta.arg_slots
                )?;
                for (j, proj) in tf_meta.layout.projections.iter().enumerate() {
                    if j > 0 {
                        write!(f, ", ")?;
                    }
                    match &proj.source.source_type {
                        crate::engine::source::ScanSourceType::WholeValue => {
                            write!(f, "slot {} ← WholeValue", proj.target_slot)?;
                        }
                        crate::engine::source::ScanSourceType::FieldPath(name) => {
                            write!(f, "slot {} ← Field(\"{}\")", proj.target_slot, name)?;
                        }
                        crate::engine::source::ScanSourceType::ColumnIndex(idx) => {
                            write!(f, "slot {} ← Column({})", proj.target_slot, idx)?;
                        }
                    }
                }
                writeln!(f)?;
            } else if let Some(meta) = self.scan_metadata.get(&cursor.scan_id) {
                write!(f, "    {:4}: ", i)?;
                for (j, proj) in meta.layout.projections.iter().enumerate() {
                    if j > 0 {
                        write!(f, ", ")?;
                    }
                    match &proj.source.source_type {
                        crate::engine::source::ScanSourceType::WholeValue => {
                            write!(f, "slot {} ← WholeValue", proj.target_slot)?;
                        }
                        crate::engine::source::ScanSourceType::FieldPath(name) => {
                            write!(f, "slot {} ← Field(\"{}\")", proj.target_slot, name)?;
                        }
                        crate::engine::source::ScanSourceType::ColumnIndex(idx) => {
                            write!(f, "slot {} ← Column({})", proj.target_slot, idx)?;
                        }
                    }
                }
                writeln!(f)?;
            } else {
                writeln!(f, "    {:4}: scan_id={:?}", i, cursor.scan_id)?;
            }
        }
        write!(f, "  output: ")?;
        match &self.shape {
            crate::engine::value::Shape::Bag(row) => {
                write!(f, "Bag(")?;
                fmt_row_shape(f, row)?;
                writeln!(f, ")")?;
            }
            crate::engine::value::Shape::List(row) => {
                write!(f, "List(")?;
                fmt_row_shape(f, row)?;
                writeln!(f, ")")?;
            }
            crate::engine::value::Shape::Single(row) => {
                write!(f, "Single(")?;
                fmt_row_shape(f, row)?;
                writeln!(f, ")")?;
            }
        }
        writeln!(f, "  constants: {}", self.program.consts.len())?;
        for (i, c) in self.program.consts.iter().enumerate() {
            writeln!(f, "    {:4}: {}", i, c)?;
        }
        writeln!(f, "  keys: {}", self.program.keys.len())?;
        for (i, k) in self.program.keys.iter().enumerate() {
            writeln!(f, "    {:4}: \"{}\"", i, k)?;
        }
        writeln!(f, "  instructions: {}", self.program.insts.len())?;
        for (i, inst) in self.program.insts.iter().enumerate() {
            writeln!(f, "    {:4}: {:?}", i, inst)?;
        }
        write!(f, "}}")
    }
}

fn fmt_row_shape(f: &mut fmt::Formatter<'_>, row: &crate::engine::value::RowShape) -> fmt::Result {
    use crate::engine::value::{FieldName, RowShape};
    match row {
        RowShape::Register(idx, _) => write!(f, "slot {idx}"),
        RowShape::Struct(fields) => {
            write!(f, "{{ ")?;
            for (i, field) in fields.iter().enumerate() {
                if i > 0 {
                    write!(f, ", ")?;
                }
                match &field.name {
                    FieldName::Static(name) => write!(f, "'{name}'")?,
                    FieldName::Register(reg) => write!(f, "slot {reg}")?,
                }
                write!(f, ": ")?;
                match &field.value {
                    RowShape::Register(idx, _) => write!(f, "slot {idx}")?,
                    RowShape::Struct(_) => write!(f, "{{...}}")?,
                }
            }
            write!(f, " }}")
        }
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
            inline_scans: HashMap::new(),
            expr_scan_ids: Vec::new(),
            table_fn_scans: HashMap::new(),
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
    /// Memory banks. Bank 0 = query-level (persistent across rows).
    /// Bank 1 = per-cursor row-level bank, reset at each NextRow.
    /// Bank 2+ = sorter banks (persistent until sorter closes).
    banks: Vec<Arena>,
    /// Unified register array: [0..slot_count] are output slots, rest are temporaries.
    registers: Vec<ValueRef<'static>>,
    /// Instruction pointer — saved across `EmitRow` yield points.
    ip: usize,
    /// Whether execution has completed (Halt reached).
    halted: bool,
    /// Number of output slots.
    slot_count: usize,
    /// Built-in function registry.
    builtins: BuiltinFunctions,
    /// Registered table functions, keyed by name.
    table_functions: HashMap<String, Arc<dyn crate::engine::source::TableFunction>>,
    /// Sorters for GROUP BY (indexed by sorter_id from bytecode).
    sorters: Vec<Option<crate::engine::sorter::Sorter>>,
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
            banks: vec![Arena::new(4096), Arena::new(16384)],
            registers,
            ip: 0,
            halted: false,
            slot_count,
            builtins: BuiltinFunctions::new(),
            table_functions: exec_context.table_functions().clone(),
            sorters: Vec::new(),
        };

        // Pre-create data sources from catalog
        vm.bind_cursors(exec_context)?;

        Ok(vm)
    }

    /// Bind all cursors to data sources from the execution context.
    fn bind_cursors(&mut self, exec_context: &ExecutionContext) -> Result<()> {
        for (cursor_id, cursor_info) in self.compiled.cursors.iter().enumerate() {
            // Check for inline scans first (literal collections in FROM)
            if let Some(values) = self.compiled.inline_scans.get(&cursor_info.scan_id) {
                self.cursors[cursor_id] = Some(DataSourceImpl::Inline(InlineDataSource::new(
                    values.clone(),
                )));
                continue;
            }

            // Skip expression scans — they are bound at runtime by MaterializeCursor
            if self.compiled.expr_scan_ids.contains(&cursor_info.scan_id) {
                continue;
            }

            // Skip table function scans — they are bound at runtime by CreateTableFnCursor
            if self
                .compiled
                .table_fn_scans
                .contains_key(&cursor_info.scan_id)
            {
                continue;
            }

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
        for bank in &self.banks {
            bank.reset();
        }
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
        self.table_functions = exec_context.table_functions().clone();
        self.bind_cursors(exec_context)?;
        self.ip = 0;
        self.halted = false;
        for bank in &self.banks {
            bank.reset();
        }
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
                    // Reset the row-level bank for the new row
                    self.vm.banks[1].reset();

                    let cursor = match self.vm.cursors.get_mut(*cursor_id as usize) {
                        Some(Some(c)) => c,
                        _ => {
                            return Some(Err(EngineError::IllegalState(format!(
                                "cursor {} not bound",
                                cursor_id
                            ))))
                        }
                    };

                    // Write next row into registers via RegisterWriter (row bank)
                    let has_row = {
                        let mut writer = RegisterWriter::new(regs, &self.vm.banks[1]);
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

                Inst::AssertCollection { src } => {
                    let value = regs[*src as usize];
                    if !matches!(value, ValueRef::Bag(_) | ValueRef::List(_)) {
                        return Some(Err(EngineError::StrictModeViolation(
                            "FROM clause requires collection value".to_string(),
                        )));
                    }
                }

                Inst::MaterializeCursor { src, cursor_id } => {
                    let collection = regs[*src as usize];
                    let values = match collection {
                        ValueRef::Bag(items) | ValueRef::List(items) => {
                            items.iter().map(|item| value_ref_to_owned(*item)).collect()
                        }
                        other => vec![value_ref_to_owned(other)],
                    };
                    self.vm.cursors[*cursor_id as usize] =
                        Some(DataSourceImpl::Inline(InlineDataSource::new(values)));
                }

                Inst::CreateTableFnCursor {
                    cursor_id,
                    func_name_idx,
                } => {
                    let func_name = &program.keys[*func_name_idx as usize];
                    let table_fn = match self.vm.table_functions.get(func_name) {
                        Some(f) => f.clone(),
                        None => {
                            return Some(Err(EngineError::IllegalState(format!(
                                "table function '{}' not registered",
                                func_name
                            ))))
                        }
                    };

                    let scan_id = self.vm.compiled.cursors[*cursor_id as usize].scan_id;
                    let tf_meta = &self.vm.compiled.table_fn_scans[&scan_id];

                    let reader = RegisterReader::new(regs);
                    let data_source =
                        match table_fn.create(&reader, &tf_meta.arg_slots, &tf_meta.layout) {
                            Ok(ds) => ds,
                            Err(e) => return Some(Err(e)),
                        };

                    self.vm.cursors[*cursor_id as usize] =
                        Some(DataSourceImpl::Catalog(data_source));
                }

                // === Sorter Instructions ===

                Inst::SorterInsert {
                    sorter_id,
                    src_reg,
                    reg_count,
                } => {
                    let sorter = match self.vm.sorters.get_mut(*sorter_id as usize) {
                        Some(Some(s)) => s,
                        _ => {
                            return Some(Err(EngineError::IllegalState(format!(
                                "sorter {} not bound",
                                sorter_id
                            ))))
                        }
                    };
                    let bank = &self.vm.banks[sorter.bank_id];
                    let start = *src_reg as usize;
                    let count = *reg_count as usize;
                    let mut fields = Vec::with_capacity(count);
                    for i in start..start + count {
                        // Safety: sorter bank outlives the sorter records.
                        // Same contract as the VM's register/arena relationship.
                        let copied = copy_value_ref_to_bank(regs[i], bank);
                        let copied: ValueRef<'static> = unsafe { std::mem::transmute(copied) };
                        fields.push(copied);
                    }
                    sorter.insert(crate::engine::sorter::SorterRecord { fields });
                }

                Inst::SorterSort {
                    sorter_id,
                    eof_target,
                } => {
                    let sorter = match self.vm.sorters.get_mut(*sorter_id as usize) {
                        Some(Some(s)) => s,
                        _ => {
                            return Some(Err(EngineError::IllegalState(format!(
                                "sorter {} not bound",
                                sorter_id
                            ))))
                        }
                    };
                    if !sorter.sort() {
                        self.vm.ip = *eof_target as usize;
                    }
                }

                Inst::SorterData { sorter_id, dst_reg } => {
                    let sorter = match self.vm.sorters.get(*sorter_id as usize) {
                        Some(Some(s)) => s,
                        _ => {
                            return Some(Err(EngineError::IllegalState(format!(
                                "sorter {} not bound",
                                sorter_id
                            ))))
                        }
                    };
                    let fields = match sorter.current() {
                        Some(f) => f,
                        None => {
                            return Some(Err(EngineError::IllegalState(
                                "sorter positioned past end".to_string(),
                            )))
                        }
                    };
                    let dst = *dst_reg as usize;
                    for (i, field) in fields.iter().enumerate() {
                        regs[dst + i] = *field;
                    }
                }

                Inst::SorterNext { sorter_id, target } => {
                    let sorter = match self.vm.sorters.get_mut(*sorter_id as usize) {
                        Some(Some(s)) => s,
                        _ => {
                            return Some(Err(EngineError::IllegalState(format!(
                                "sorter {} not bound",
                                sorter_id
                            ))))
                        }
                    };
                    if sorter.advance() {
                        self.vm.ip = *target as usize;
                    }
                }

                // === Aggregation Instructions ===

                Inst::AggStep {
                    func,
                    accum_reg,
                    input_reg,
                } => {
                    let accum = regs[*accum_reg as usize];
                    let input = regs[*input_reg as usize];
                    regs[*accum_reg as usize] = agg_step(*func, accum, input);
                }

                Inst::AggFinal {
                    func,
                    accum_reg,
                    dst_reg,
                } => {
                    let accum = regs[*accum_reg as usize];
                    regs[*dst_reg as usize] = agg_final(*func, accum);
                }

                Inst::AggReset { accum_start, count } => {
                    let start = *accum_start as usize;
                    let cnt = *count as usize;
                    for reg in &mut regs[start..start + cnt] {
                        *reg = ValueRef::Missing;
                    }
                }

                // === Subroutine Instructions ===

                Inst::Gosub { ret_reg, target } => {
                    regs[*ret_reg as usize] = ValueRef::I64(self.vm.ip as i64);
                    self.vm.ip = *target as usize;
                }

                Inst::Return { ret_reg } => {
                    match regs[*ret_reg as usize] {
                        ValueRef::I64(addr) => {
                            self.vm.ip = addr as usize;
                        }
                        _ => {
                            return Some(Err(EngineError::IllegalState(
                                "Return: ret_reg does not contain a valid address".to_string(),
                            )))
                        }
                    }
                }

                Inst::Copy { dst, src } => {
                    regs[*dst as usize] = regs[*src as usize];
                }

                // === Comparison ===

                Inst::CompareEq {
                    lhs_reg,
                    rhs_reg,
                    dst_reg,
                } => {
                    let lhs = regs[*lhs_reg as usize];
                    let rhs = regs[*rhs_reg as usize];
                    regs[*dst_reg as usize] =
                        ValueRef::Bool(crate::engine::sorter::value_ref_eq(&lhs, &rhs));
                }

                // All scalar instructions delegate to eval_inst
                other => {
                    if let Err(e) =
                        program.eval_inst(other, &self.vm.banks[1], regs, Some(&self.vm.builtins))
                    {
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
        for bank in &self.vm.banks {
            bank.reset();
        }
        self.vm.halted = true;
    }
}

/// Copy a ValueRef's data into the given arena bank, returning a new ValueRef
/// that borrows from that bank. For inline types (I64, F64, Bool, etc.) this is
/// a no-op copy. For heap types (Str, Bytes), it allocates into the bank.
fn copy_value_ref_to_bank<'a>(value: ValueRef<'_>, bank: &'a Arena) -> ValueRef<'a> {
    match value {
        ValueRef::Missing => ValueRef::Missing,
        ValueRef::Null => ValueRef::Null,
        ValueRef::Bool(b) => ValueRef::Bool(b),
        ValueRef::I64(n) => ValueRef::I64(n),
        ValueRef::F64(f) => ValueRef::F64(f),
        ValueRef::Decimal(d) => ValueRef::Decimal(d),
        ValueRef::Str(s) => {
            let bytes = bank.alloc_slice(s.as_bytes());
            let copied = unsafe { std::str::from_utf8_unchecked(bytes) };
            ValueRef::Str(copied)
        }
        ValueRef::Bytes(b) => {
            let copied = bank.alloc_slice(b);
            ValueRef::Bytes(copied)
        }
        // Tuple/List/Bag: deep copy into bank
        ValueRef::Tuple(t) => {
            use crate::engine::value::{TupleField, TupleRef};
            let fields: Vec<TupleField<'a>> = t
                .fields
                .iter()
                .map(|f| TupleField {
                    name: {
                        let bytes = bank.alloc_slice(f.name.as_bytes());
                        unsafe { std::str::from_utf8_unchecked(bytes) }
                    },
                    value: copy_value_ref_to_bank(f.value, bank),
                })
                .collect();
            let fields_slice = bank.alloc_slice(&fields);
            let tuple_ref = bank.alloc_tuple_ref(TupleRef {
                fields: fields_slice,
            });
            ValueRef::Tuple(tuple_ref)
        }
        ValueRef::List(items) => {
            let refs: Vec<ValueRef<'a>> =
                items.iter().map(|v| copy_value_ref_to_bank(*v, bank)).collect();
            let refs_slice = bank.alloc_slice(&refs);
            ValueRef::List(refs_slice)
        }
        ValueRef::Bag(items) => {
            let refs: Vec<ValueRef<'a>> =
                items.iter().map(|v| copy_value_ref_to_bank(*v, bank)).collect();
            let refs_slice = bank.alloc_slice(&refs);
            ValueRef::Bag(refs_slice)
        }
    }
}

/// Perform one step of an aggregate function: update the accumulator with a new input.
fn agg_step(func: AggFunc, accum: ValueRef<'_>, input: ValueRef<'_>) -> ValueRef<'static> {
    match func {
        AggFunc::Sum => match (accum, input) {
            (_, ValueRef::Null | ValueRef::Missing) => unsafe { std::mem::transmute(accum) },
            (ValueRef::Missing, v) => unsafe { std::mem::transmute(v) },
            (ValueRef::I64(a), ValueRef::I64(b)) => ValueRef::I64(a + b),
            (ValueRef::F64(a), ValueRef::F64(b)) => ValueRef::F64(a + b),
            (ValueRef::Decimal(a), ValueRef::Decimal(b)) => ValueRef::Decimal(a + b),
            // Mixed numeric: promote to f64
            (ValueRef::I64(a), ValueRef::F64(b)) => ValueRef::F64(a as f64 + b),
            (ValueRef::F64(a), ValueRef::I64(b)) => ValueRef::F64(a + b as f64),
            _ => unsafe { std::mem::transmute(accum) },
        },
        AggFunc::Count => match accum {
            ValueRef::Missing => ValueRef::I64(1),
            ValueRef::I64(n) => ValueRef::I64(n + 1),
            _ => ValueRef::I64(1),
        },
        AggFunc::Min => match (accum, input) {
            (_, ValueRef::Null | ValueRef::Missing) => unsafe { std::mem::transmute(accum) },
            (ValueRef::Missing, v) => unsafe { std::mem::transmute(v) },
            (ValueRef::I64(a), ValueRef::I64(b)) => ValueRef::I64(a.min(b)),
            (ValueRef::F64(a), ValueRef::F64(b)) => ValueRef::F64(a.min(b)),
            (ValueRef::Decimal(a), ValueRef::Decimal(b)) => {
                ValueRef::Decimal(if a <= b { a } else { b })
            }
            _ => unsafe { std::mem::transmute(accum) },
        },
        AggFunc::Max => match (accum, input) {
            (_, ValueRef::Null | ValueRef::Missing) => unsafe { std::mem::transmute(accum) },
            (ValueRef::Missing, v) => unsafe { std::mem::transmute(v) },
            (ValueRef::I64(a), ValueRef::I64(b)) => ValueRef::I64(a.max(b)),
            (ValueRef::F64(a), ValueRef::F64(b)) => ValueRef::F64(a.max(b)),
            (ValueRef::Decimal(a), ValueRef::Decimal(b)) => {
                ValueRef::Decimal(if a >= b { a } else { b })
            }
            _ => unsafe { std::mem::transmute(accum) },
        },
        AggFunc::Any => match (accum, input) {
            (ValueRef::Bool(true), _) => ValueRef::Bool(true),
            (_, ValueRef::Bool(true)) => ValueRef::Bool(true),
            (ValueRef::Missing, ValueRef::Bool(b)) => ValueRef::Bool(b),
            (ValueRef::Bool(a), _) => ValueRef::Bool(a),
            _ => unsafe { std::mem::transmute(accum) },
        },
        AggFunc::Every => match (accum, input) {
            (ValueRef::Bool(false), _) => ValueRef::Bool(false),
            (_, ValueRef::Bool(false)) => ValueRef::Bool(false),
            (ValueRef::Missing, ValueRef::Bool(b)) => ValueRef::Bool(b),
            (ValueRef::Bool(a), _) => ValueRef::Bool(a),
            _ => unsafe { std::mem::transmute(accum) },
        },
        AggFunc::Avg => {
            // Avg stores (count, sum) encoded as two I64 values packed in the accum.
            // For simplicity, we'll accumulate sum and count separately:
            // The compiler should allocate TWO accum registers for Avg.
            // For now, treat as Sum — finalize will need count.
            // TODO: proper Avg support with paired registers
            agg_step(AggFunc::Sum, accum, input)
        }
    }
}

/// Finalize an aggregate: convert the accumulator state to the final result.
fn agg_final<'a>(func: AggFunc, accum: ValueRef<'a>) -> ValueRef<'a> {
    match func {
        AggFunc::Count => match accum {
            ValueRef::Missing => ValueRef::I64(0),
            other => other,
        },
        AggFunc::Sum | AggFunc::Min | AggFunc::Max | AggFunc::Any | AggFunc::Every => {
            match accum {
                ValueRef::Missing => ValueRef::Null,
                other => other,
            }
        }
        AggFunc::Avg => {
            // TODO: proper Avg with count/sum pair
            accum
        }
    }
}
