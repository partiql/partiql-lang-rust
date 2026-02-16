use std::collections::HashMap;
use std::sync::Arc;

use crate::engine::arena::{Arena, SlotId};
use crate::engine::catalog::ExecutionContext;
use crate::engine::error::{EngineError, Result};
use crate::engine::expr::Program;
use crate::engine::source::RegisterWriter;
use crate::engine::source::{DataSourceImpl, ScanLayout};
use crate::engine::value::{RegisterReader, Shape, ValueRef};
use crate::engine::UdfRegistry;

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

/// Compiled query execution plan.
///
/// # Thread Safety Warning
/// **IMPORTANT**: CompiledPlan is NOT thread-safe if it contains Legacy operators
/// (RelOpSpec::Legacy with old Evaluable-based EvalPlan). The Legacy support is
/// temporary for migration purposes only.
///
/// Legacy operators:
/// - Should NOT be shared across threads
/// - Should NOT be executed concurrently
/// - Are safe only for single-threaded execution
///
/// Pipeline-based plans (without Legacy operators) remain fully Send + Sync.
#[derive(Default)]
pub struct CompiledPlan {
    pub(crate) nodes: Vec<RelOpSpec>,
    pub(crate) root: usize,
    pub(crate) shape: Shape,
    pub(crate) slot_count: usize,
    pub(crate) max_registers: usize,
    pub(crate) scan_metadata: HashMap<ScanId, ScanMetadata>,
}

// Conditional bounds ensure CompiledPlan is only Send/Sync when all fields are.
// This provides compile-time verification - the compiler verifies the bounds.
// Safety: The bounds ensure all fields are Send/Sync, making this impl safe.
unsafe impl Send for CompiledPlan
where
    Vec<RelOpSpec>: Send,
    Shape: Send,
{
}

unsafe impl Sync for CompiledPlan
where
    Vec<RelOpSpec>: Sync,
    Shape: Sync,
{
}

impl Clone for CompiledPlan {
    fn clone(&self) -> Self {
        Self {
            nodes: self.nodes.iter().map(|node| node.clone_spec()).collect(),
            root: self.root,
            shape: self.shape.clone(),
            slot_count: self.slot_count,
            max_registers: self.max_registers,
            scan_metadata: self.scan_metadata.clone(),
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
    ///
    /// # Example
    /// ```ignore
    /// let catalog_scans = compiled.scans_for_catalog(catalog_id);
    /// exec_catalog.prepare(&catalog_scans);
    /// ```
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

// TODO: Actually implement HashJoin and whatnot.
#[allow(dead_code)]
pub(crate) enum RelOpSpec {
    Pipeline(PipelineSpec),
    ExprQuery(ExprQuerySpec),
    HashJoin(HashJoinSpec),
    HashAgg(HashAggSpec),
    Sort(SortSpec),
    Custom(Box<dyn BlockingOperatorSpec>),
}

impl RelOpSpec {
    pub(crate) fn clone_spec(&self) -> Self {
        match self {
            RelOpSpec::Pipeline(spec) => RelOpSpec::Pipeline(spec.clone_pipeline()),
            RelOpSpec::ExprQuery(spec) => RelOpSpec::ExprQuery(spec.clone()),
            RelOpSpec::HashJoin(_) => RelOpSpec::HashJoin(HashJoinSpec),
            RelOpSpec::HashAgg(_) => RelOpSpec::HashAgg(HashAggSpec),
            RelOpSpec::Sort(_) => RelOpSpec::Sort(SortSpec),
            RelOpSpec::Custom(_) => panic!("Cannot clone custom operator spec"),
        }
    }
}

pub struct PipelineSpec {
    pub scan_id: ScanId,
    pub steps: Vec<StepSpec>,
}

impl PipelineSpec {
    pub(crate) fn clone_pipeline(&self) -> Self {
        Self {
            scan_id: self.scan_id,
            steps: self.steps.clone(),
        }
    }
}

#[derive(Clone)]
pub struct ExprQuerySpec {
    pub program: Program,
}

pub struct HashJoinSpec;
pub struct HashAggSpec;
pub struct SortSpec;

pub(crate) trait BlockingOperatorSpec: Send + Sync {
    #[allow(dead_code)]
    fn instantiate(&self) -> Box<dyn BlockingOperator>;
}

pub(crate) trait BlockingOperator {
    #[allow(dead_code)]
    fn next_row(&mut self, arena: &Arena, regs: &mut [ValueRef<'_>]) -> Result<bool>;
    fn open(&mut self) -> Result<()>;
    fn close(&mut self) -> Result<()>;
}

// TODO: Actually implement HashJoin and whatnot.
#[allow(dead_code)]
pub(crate) enum RelOp {
    Pipeline(PipelineOp),
    ExprQuery(ExprQueryOp),
    HashJoin(HashJoinState),
    HashAgg(HashAggState),
    Sort(SortState),
    Custom(Box<dyn BlockingOperator>),
}

impl RelOp {
    pub fn next_row<'a>(
        &'a mut self,
        arena: &'a Arena,
        regs: &'a mut [ValueRef<'a>],
        slot_count: usize,
    ) -> Result<Option<RegisterReader<'a>>> {
        match self {
            RelOp::Pipeline(op) => op.next_row(arena, regs, slot_count),
            RelOp::ExprQuery(op) => op.next_row(arena, regs, slot_count),
            RelOp::HashJoin(_op) => Err(EngineError::NotImplemented),
            RelOp::HashAgg(_op) => Err(EngineError::NotImplemented),
            RelOp::Sort(_op) => Err(EngineError::NotImplemented),
            RelOp::Custom(_op) => Err(EngineError::NotImplemented),
        }
    }

    /// Open operator and allocate resources
    pub fn open(&mut self) -> Result<()> {
        match self {
            RelOp::Pipeline(op) => op.open(),
            RelOp::ExprQuery(op) => op.open(),
            RelOp::HashJoin(op) => op.open(),
            RelOp::HashAgg(op) => op.open(),
            RelOp::Sort(op) => op.open(),
            RelOp::Custom(op) => op.open(),
        }
    }

    /// Close operator and release resources
    pub fn close(&mut self) -> Result<()> {
        match self {
            RelOp::Pipeline(op) => op.close(),
            RelOp::ExprQuery(op) => op.close(),
            RelOp::HashJoin(op) => op.close(),
            RelOp::HashAgg(op) => op.close(),
            RelOp::Sort(op) => op.close(),
            RelOp::Custom(op) => op.close(),
        }
    }
}

pub struct PipelineOp {
    steps: Vec<Step>,
    reader: DataSourceImpl,
    opened: bool,
    udf: Option<Arc<dyn UdfRegistry>>,
}

/// Operator for expression-only queries (no scan)
pub struct ExprQueryOp {
    program: Program,
    yielded: bool,
    udf: Option<Arc<dyn UdfRegistry>>,
}

impl ExprQueryOp {
    pub(crate) fn new(program: Program, udf: Option<Arc<dyn UdfRegistry>>) -> Self {
        ExprQueryOp {
            program,
            yielded: false,
            udf,
        }
    }

    pub fn open(&mut self) -> Result<()> {
        // No resources to open for expression queries
        Ok(())
    }

    pub fn close(&mut self) -> Result<()> {
        // Reset state for potential reuse
        self.yielded = false;
        Ok(())
    }

    pub fn next_row<'a>(
        &'a mut self,
        arena: &'a Arena,
        regs: &'a mut [ValueRef<'a>],
        slot_count: usize,
    ) -> Result<Option<RegisterReader<'a>>> {
        if self.yielded {
            return Ok(None);
        }

        // Evaluate expression into register 0
        let udf = self.udf.as_deref();
        self.program.eval(arena, regs, udf)?;
        self.yielded = true;

        // Return view of slot region
        let slots = &regs[0..slot_count];
        Ok(Some(RegisterReader::new(slots)))
    }
}

pub struct HashJoinState;
pub struct HashAggState;
pub struct SortState;

impl PipelineOp {
    pub(crate) fn new(
        steps: Vec<Step>,
        reader: DataSourceImpl,
        udf: Option<Arc<dyn UdfRegistry>>,
    ) -> Self {
        PipelineOp {
            steps,
            reader,
            opened: false,
            udf,
        }
    }

    pub fn open(&mut self) -> Result<()> {
        if !self.opened {
            self.reader.open()?;
            self.opened = true;
        }
        Ok(())
    }

    pub fn close(&mut self) -> Result<()> {
        if self.opened {
            self.reader.close()?;
            self.opened = false;
            // Reset step state to allow VM reuse across multiple execute() calls
            for step in &mut self.steps {
                if let Step::Limit { limit, remaining } = step {
                    *remaining = *limit;
                }
            }
        }
        Ok(())
    }

    pub fn next_row<'a>(
        &'a mut self,
        arena: &'a Arena,
        regs: &'a mut [ValueRef<'a>],
        slot_count: usize,
    ) -> Result<Option<RegisterReader<'a>>> {
        let udf = self.udf.as_deref();
        loop {
            // Read next row into registers via RegisterWriter
            // Use a shorter-lived reborrow to ensure the mutable borrow ends
            let has_row = {
                // Reborrow with shorter lifetime to prevent lifetime extension
                let regs_reborrow = &mut *regs;
                let mut writer = RegisterWriter::new(regs_reborrow);
                self.reader.next_row(&mut writer)?
            };

            if !has_row {
                return Ok(None);
            }

            // Mutable borrow has ended, now we can read from regs
            match Self::run_steps(&mut self.steps, arena, regs, udf)? {
                StepOutcome::Emit => {
                    // Return immutable view of slot region
                    let slots = &regs[0..slot_count];
                    return Ok(Some(RegisterReader::new(slots)));
                }
                StepOutcome::Skip => continue,
                StepOutcome::Halt => return Ok(None),
            }
        }
    }

    fn run_steps<'a>(
        steps: &mut [Step],
        arena: &'a Arena,
        regs: &mut [ValueRef<'a>],
        udf: Option<&'a dyn UdfRegistry>,
    ) -> Result<StepOutcome> {
        for step in steps {
            match step.eval(arena, regs, udf)? {
                StepOutcome::Emit => {}
                StepOutcome::Skip => return Ok(StepOutcome::Skip),
                StepOutcome::Halt => return Ok(StepOutcome::Halt),
            }
        }
        Ok(StepOutcome::Emit)
    }
}

impl HashJoinState {
    pub fn open(&mut self) -> Result<()> {
        Err(EngineError::NotImplemented)
    }

    pub fn close(&mut self) -> Result<()> {
        Err(EngineError::NotImplemented)
    }
}

impl HashAggState {
    pub fn open(&mut self) -> Result<()> {
        Err(EngineError::NotImplemented)
    }

    pub fn close(&mut self) -> Result<()> {
        Err(EngineError::NotImplemented)
    }
}

impl SortState {
    pub fn open(&mut self) -> Result<()> {
        Err(EngineError::NotImplemented)
    }

    pub fn close(&mut self) -> Result<()> {
        Err(EngineError::NotImplemented)
    }
}

/// Single-threaded virtual machine for executing a compiled PartiQL plan
///
/// The VM owns all execution state including:
/// - Operator instances
/// - Memory arena for intermediate values  
/// - Row processing scratch space
/// - Register array for expression evaluation
///
/// The VM is fully reusable - call `reset()` to prepare for another execution.
/// Multiple VMs can be created from the same CompiledPlan for concurrent execution.
pub struct PartiQLVM {
    compiled: Arc<CompiledPlan>,
    operators: Vec<RelOp>,
    /// Per-row memory arena for computed values
    ///
    /// Reset on each call to `next_row()`.
    /// All operators in the pipeline (readers, filters, projects) allocate
    /// computed values into this shared arena.
    ///
    /// **Lifetime**: Values valid only until next `next_row()` call.
    ///
    /// **Note**: Blocking operators (HashJoin, HashAgg) maintain separate arenas
    /// for data that must persist across multiple rows.
    arena: Arena,
    /// Unified register array: [0..slot_count] are slots, [slot_count..] are temporaries
    ///
    /// Allocated once at VM creation, sized to slot_count + max_registers.
    /// The first slot_count registers hold row data (replacing RowFrameScratch),
    /// and the remaining registers are used for expression evaluation temporaries.
    ///
    /// **Performance**: Eliminates LoadSlot instructions and heap allocations,
    /// maintains perfect cache locality across rows.
    registers: Vec<ValueRef<'static>>,
    root: usize,
    slot_count: usize,
}

impl PartiQLVM {
    /// Create a new VM instance from a compiled plan with ExecutionContext
    ///
    /// # Arguments
    /// * `compiled` - The compiled query plan to execute
    /// * `exec_context` - ExecutionContext for resolving catalog-based data sources
    ///
    /// # Returns
    /// A new PartiQLVM ready to execute the plan
    pub fn new(compiled: CompiledPlan, exec_context: &ExecutionContext) -> Result<Self> {
        let compiled = Arc::new(compiled);
        let slot_count = compiled.slot_count;
        let root = compiled.root;

        // Instantiate operators from specs
        let mut operators = Vec::with_capacity(compiled.nodes.len());
        for node in &compiled.nodes {
            match node {
                RelOpSpec::Pipeline(spec) => {
                    // Get scan metadata for this scan_id
                    let scan_meta = compiled.get_scan(spec.scan_id).ok_or_else(|| {
                        EngineError::IllegalState(format!("Unknown scan_id: {:?}", spec.scan_id))
                    })?;

                    // Get the catalog for this scan's object
                    let catalog = exec_context
                        .get_catalog(scan_meta.object_id.catalog_id())
                        .ok_or_else(|| {
                            EngineError::IllegalState(format!(
                                "Catalog {:?} not found in ExecutionContext",
                                scan_meta.object_id.catalog_id()
                            ))
                        })?;

                    // Catalog creates DataSource using just the ScanId
                    let data_source = catalog.create(spec.scan_id)?;
                    let reader = DataSourceImpl::Catalog(data_source);

                    let steps = spec.steps.iter().cloned().map(Step::from_spec).collect();
                    operators.push(RelOp::Pipeline(PipelineOp::new(
                        steps, reader, None, // UDF registry not supported yet
                    )));
                }
                RelOpSpec::ExprQuery(spec) => {
                    operators.push(RelOp::ExprQuery(ExprQueryOp::new(
                        spec.program.clone(),
                        None, // UDF registry not supported yet
                    )));
                }
                _ => {
                    return Err(EngineError::InvalidPlan(
                        "unsupported operator spec".to_string(),
                    ));
                }
            }
        }

        // Allocate unified register array: slot_count + max_registers
        // First slot_count registers are for slots, rest for temporaries
        let max_regs = compiled.max_registers;
        let total_regs = slot_count + max_regs;
        let registers = vec![ValueRef::Missing; total_regs];

        Ok(PartiQLVM {
            compiled,
            operators,
            arena: Arena::new(16384), // 16KB arena - tune based on workload
            registers,
            root,
            slot_count,
        })
    }

    /// Get the result shape for this VM's query
    pub fn shape(&self) -> &Shape {
        self.compiled.result_shape()
    }

    /// Execute the plan and return streaming results
    ///
    /// Returns an `ExecutionResult::Query` containing an iterator over result rows.
    /// The iterator implements RAII - operators are lazily opened on first iteration
    /// and automatically closed when the iterator is dropped.
    ///
    /// # Example
    /// ```ignore
    /// let mut vm = compiler.instantiate(compiled, None)?;
    /// match vm.execute()? {
    ///     ExecutionResult::Query(iter) => {
    ///         for row in iter {
    ///             let row = row?;
    ///             // Process row
    ///         }
    ///     }
    /// }
    /// ```
    pub fn execute(&mut self) -> Result<ExecutionResult<'_>> {
        Ok(ExecutionResult::Query(QueryIterator::new(self)))
    }

    /// Update the ExecutionContext for this VM
    ///
    /// This allows reusing the same VM instance with different data sources
    /// by updating the catalog mappings. The VM must not have any active
    /// iterators when updating context.
    ///
    /// # Arguments
    /// * `exec_context` - New ExecutionContext with updated catalog mappings
    ///
    /// # Returns
    /// Result indicating success or error
    pub fn set_context(&mut self, exec_context: &ExecutionContext) -> Result<()> {
        // Re-instantiate operators with new execution context
        let mut operators = Vec::with_capacity(self.compiled.nodes.len());
        for node in &self.compiled.nodes {
            match node {
                RelOpSpec::Pipeline(spec) => {
                    // Get scan metadata for this scan_id
                    let scan_meta = self.compiled.get_scan(spec.scan_id).ok_or_else(|| {
                        EngineError::IllegalState(format!("Unknown scan_id: {:?}", spec.scan_id))
                    })?;

                    // Get the catalog for this scan's object
                    let catalog = exec_context
                        .get_catalog(scan_meta.object_id.catalog_id())
                        .ok_or_else(|| {
                            EngineError::IllegalState(format!(
                                "Catalog {:?} not found in ExecutionContext",
                                scan_meta.object_id.catalog_id()
                            ))
                        })?;

                    // Catalog creates DataSource using just the ScanId
                    let data_source = catalog.create(spec.scan_id)?;
                    let reader = DataSourceImpl::Catalog(data_source);

                    let steps = spec.steps.iter().cloned().map(Step::from_spec).collect();
                    operators.push(RelOp::Pipeline(PipelineOp::new(steps, reader, None)));
                }
                RelOpSpec::ExprQuery(spec) => {
                    operators.push(RelOp::ExprQuery(ExprQueryOp::new(
                        spec.program.clone(),
                        None,
                    )));
                }
                _ => {
                    return Err(EngineError::InvalidPlan(
                        "unsupported operator spec".to_string(),
                    ));
                }
            }
        }

        self.operators = operators;
        self.arena.reset();
        Ok(())
    }

    /// Open all operators in the execution tree
    fn open_operators(&mut self) -> Result<()> {
        for op in &mut self.operators {
            op.open()?;
        }
        Ok(())
    }

    /// Close all operators in the execution tree
    fn close_operators(&mut self) -> Result<()> {
        for op in &mut self.operators {
            op.close()?;
        }
        Ok(())
    }
}

/// Result of query execution
///
/// Currently only supports Query results (SELECT statements).
/// Future extensions will add Mutation (DML) and Definition (DDL) variants.
pub enum ExecutionResult<'vm> {
    /// Query results - streaming iterator over rows
    Query(QueryIterator<'vm>),
}

/// Iterator over query result rows with RAII resource management
///
/// Operators are lazily opened on first iteration and automatically
/// closed when the iterator is dropped, even on early exit.
///
/// **Note**: Each row is only valid until the next call to `next()`.
/// This is a lending iterator pattern required for zero-copy semantics.
///
/// # Example
/// ```ignore
/// match vm.execute()? {
///     ExecutionResult::Query(iter) => {
///         for row in iter {
///             let row = row?;
///             println!("{:?}", row);
///             // Row data invalidated on next iteration
///         }
///         // Operators automatically closed here
///     }
/// }
/// ```
pub struct QueryIterator<'vm> {
    vm: &'vm mut PartiQLVM,
    opened: bool,
}

impl<'vm> QueryIterator<'vm> {
    fn new(vm: &'vm mut PartiQLVM) -> Self {
        QueryIterator { vm, opened: false }
    }

    /// Get the next row, with lifetime tied to the iterator borrow
    ///
    /// Returns `Some(Ok(row))` if a row is available, `None` if complete,
    /// or `Some(Err(...))` if an error occurred.
    fn next_row_internal(&mut self) -> Option<Result<RegisterReader<'_>>> {
        // Lazy open on first iteration
        if !self.opened {
            if let Err(e) = self.vm.open_operators() {
                return Some(Err(e));
            }
            self.opened = true;
        }

        // Reset arena for this row
        self.vm.arena.reset();

        // Borrow registers from VM (transmute lifetime to match arena)
        let regs = unsafe {
            std::mem::transmute::<&mut [ValueRef<'static>], &mut [ValueRef<'_>]>(
                self.vm.registers.as_mut_slice(),
            )
        };

        let op = match self.vm.operators.get_mut(self.vm.root) {
            Some(op) => op,
            None => {
                return Some(Err(EngineError::IllegalState(
                    "invalid root operator".to_string(),
                )))
            }
        };

        // Call next_row - it now returns Option<RegisterReader> directly
        match op.next_row(&self.vm.arena, regs, self.vm.slot_count) {
            Ok(Some(row)) => Some(Ok(row)),
            Ok(None) => None,
            Err(e) => Some(Err(e)),
        }
    }
}

impl<'vm> Iterator for QueryIterator<'vm> {
    type Item = Result<RegisterReader<'vm>>;

    fn next(&mut self) -> Option<Self::Item> {
        // Delegate to the lending next_row_internal() method
        // Safety: We extend the lifetime to 'vm, which is safe because:
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
        if self.opened {
            // Best-effort close, ignore errors in Drop
            let _ = self.vm.close_operators();
            // Reset arena to reclaim memory
            self.vm.arena.reset();
        }
    }
}

#[derive(Clone)]
pub enum StepSpec {
    Filter {
        program: Program,
        predicate_slot: SlotId,
    },
    Project {
        program: Program,
    },
    Limit {
        limit: usize,
    },
}

pub enum Step {
    Filter {
        program: Program,
        predicate_slot: SlotId,
    },
    Project {
        program: Program,
    },
    Limit {
        limit: usize,
        remaining: usize,
    },
}

enum StepOutcome {
    Emit,
    Skip,
    Halt,
}

impl Step {
    pub(crate) fn from_spec(spec: StepSpec) -> Self {
        match spec {
            StepSpec::Filter {
                program,
                predicate_slot,
            } => Step::Filter {
                program,
                predicate_slot,
            },
            StepSpec::Project { program } => Step::Project { program },
            StepSpec::Limit { limit } => Step::Limit {
                limit,
                remaining: limit,
            },
        }
    }

    fn eval<'a>(
        &mut self,
        arena: &'a Arena,
        regs: &mut [ValueRef<'a>],
        udf: Option<&'a dyn UdfRegistry>,
    ) -> Result<StepOutcome> {
        match self {
            Step::Filter {
                program,
                predicate_slot,
            } => {
                program.eval(arena, regs, udf)?;
                // Predicate result is now in the register at predicate_slot index
                match regs.get(*predicate_slot as usize) {
                    Some(&ValueRef::Bool(true)) => Ok(StepOutcome::Emit),
                    Some(&ValueRef::Bool(false))
                    | Some(&ValueRef::Missing)
                    | Some(&ValueRef::Null) => Ok(StepOutcome::Skip),
                    Some(_) => Err(EngineError::TypeError(
                        "filter predicate must be bool".to_string(),
                    )),
                    None => Err(EngineError::IllegalState(
                        "filter predicate slot missing".to_string(),
                    )),
                }
            }
            Step::Project { program } => {
                program.eval(arena, regs, udf)?;
                Ok(StepOutcome::Emit)
            }
            Step::Limit { remaining, .. } => {
                if *remaining == 0 {
                    return Ok(StepOutcome::Halt);
                }
                *remaining -= 1;
                Ok(StepOutcome::Emit)
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    /// Compile-time assertion to ensure CompiledPlan remains Send + Sync.
    /// This will cause a test compilation error if these traits are ever lost,
    /// providing better error messages than deep trait bound failures.
    #[test]
    fn compiled_plan_is_thread_safe() {
        fn assert_send<T: Send>() {}
        fn assert_sync<T: Sync>() {}

        assert_send::<CompiledPlan>();
        assert_sync::<CompiledPlan>();
    }
}
