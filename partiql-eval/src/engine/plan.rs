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
use partiql_logical::JoinKind;

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
            nodes: self.nodes.to_vec(),
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
    NestedLoopJoin(NestedLoopJoinSpec),
    HashJoin(HashJoinSpec),
    HashAgg(HashAggSpec),
    Sort(SortSpec),
    Custom(Box<dyn BlockingOperatorSpec>),
}

impl Clone for RelOpSpec {
    fn clone(&self) -> Self {
        match self {
            RelOpSpec::Pipeline(spec) => RelOpSpec::Pipeline(spec.clone_pipeline()),
            RelOpSpec::ExprQuery(spec) => RelOpSpec::ExprQuery(spec.clone()),
            RelOpSpec::NestedLoopJoin(spec) => RelOpSpec::NestedLoopJoin(spec.clone()),
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

/// Specification for a nested-loop join operator.
///
/// Contains all compile-time information needed to instantiate a join at execution time.
/// The left and right children are full operator specs (typically PipelineSpecs for scans),
/// making the plan a composable tree of operators.
#[derive(Clone)]
pub struct NestedLoopJoinSpec {
    /// Join type (Cross, Inner, Left, Right, Full)
    pub kind: JoinKind,
    /// Left (outer) child operator spec
    pub left: Box<RelOpSpec>,
    /// Right (inner) child operator spec
    pub right: Box<RelOpSpec>,
    /// Compiled join condition (ON clause), None for Cross join
    pub condition: Option<Program>,
    /// Register slot where the condition result is stored
    pub condition_slot: Option<SlotId>,
    /// Start of right-side input slots in the register array
    pub right_input_start: usize,
    /// Number of right-side input slots (1 for whole-value mode)
    pub right_input_count: usize,
    /// Post-join steps (projection, filter, limit)
    pub steps: Vec<StepSpec>,
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
    NestedLoopJoin(NestedLoopJoinOp),
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
            RelOp::NestedLoopJoin(op) => op.next_row(arena, regs, slot_count),
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
            RelOp::NestedLoopJoin(op) => op.open(),
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
            RelOp::NestedLoopJoin(op) => op.close(),
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
                let mut writer = RegisterWriter::new(regs_reborrow, arena);
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

/// Runtime state for a nested-loop join operator.
///
/// The left and right children are full `RelOp` operators (typically PipelineOps
/// wrapping data source scans). This makes the operator tree composable —
/// a join's child could be another join, a pipeline with filters, etc.
///
/// Implements lateral join semantics: for each LHS row, the RHS is iterated fully.
/// Supports Cross, Inner, and Left joins.
pub struct NestedLoopJoinOp {
    kind: JoinKind,
    left: Box<RelOp>,
    right: Box<RelOp>,
    condition: Option<Program>,
    condition_slot: Option<SlotId>,
    right_input_start: usize,
    right_input_count: usize,
    steps: Vec<Step>,
    udf: Option<Arc<dyn UdfRegistry>>,
    // Iteration state
    opened: bool,
    has_left_row: bool,
    right_opened: bool,
    left_had_match: bool,
    #[allow(dead_code)]
    slot_count: usize,
}

/// Configuration for constructing a `NestedLoopJoinOp`.
pub(crate) struct NestedLoopJoinConfig {
    pub kind: JoinKind,
    pub left: Box<RelOp>,
    pub right: Box<RelOp>,
    pub condition: Option<Program>,
    pub condition_slot: Option<SlotId>,
    pub right_input_start: usize,
    pub right_input_count: usize,
    pub steps: Vec<Step>,
    pub slot_count: usize,
    pub udf: Option<Arc<dyn UdfRegistry>>,
}

impl NestedLoopJoinOp {
    pub(crate) fn new(config: NestedLoopJoinConfig) -> Self {
        NestedLoopJoinOp {
            kind: config.kind,
            left: config.left,
            right: config.right,
            condition: config.condition,
            condition_slot: config.condition_slot,
            right_input_start: config.right_input_start,
            right_input_count: config.right_input_count,
            steps: config.steps,
            udf: config.udf,
            opened: false,
            has_left_row: false,
            right_opened: false,
            left_had_match: false,
            slot_count: config.slot_count,
        }
    }

    pub fn open(&mut self) -> Result<()> {
        if !self.opened {
            self.left.open()?;
            self.opened = true;
            self.has_left_row = false;
            self.right_opened = false;
            self.left_had_match = false;
        }
        Ok(())
    }

    pub fn close(&mut self) -> Result<()> {
        if self.opened {
            if self.right_opened {
                let _ = self.right.close();
                self.right_opened = false;
            }
            self.left.close()?;
            self.opened = false;
            self.has_left_row = false;
            self.left_had_match = false;
            for step in &mut self.steps {
                if let Step::Limit { limit, remaining } = step {
                    *remaining = *limit;
                }
            }
        }
        Ok(())
    }

    /// Produce next joined row.
    ///
    /// # Safety within this method
    /// We use unsafe reborrows when calling child `next_row` in a loop.
    /// This is safe because:
    /// 1. We immediately discard the `RegisterReader` (`.is_some()`) — no aliased references persist
    /// 2. Children write into `regs` via side effects; we only need to know if a row was produced
    /// 3. The arena and regs remain valid for the full `'a` lifetime
    pub fn next_row<'a>(
        &'a mut self,
        arena: &'a Arena,
        regs: &'a mut [ValueRef<'a>],
        slot_count: usize,
    ) -> Result<Option<RegisterReader<'a>>> {
        let udf = self.udf.as_deref();

        loop {
            // Step 1: Ensure we have a LHS row
            if !self.has_left_row {
                // Safety: We reborrow self.left, arena, regs with shorter lifetimes.
                // The returned RegisterReader is immediately consumed (.is_some()),
                // so no aliased references persist past this block.
                let has_left = unsafe {
                    let left_ptr = &mut *self.left as *mut RelOp;
                    let regs_ptr = regs as *mut [ValueRef<'a>];
                    (*left_ptr)
                        .next_row(arena, &mut *regs_ptr, slot_count)?
                        .is_some()
                };
                if !has_left {
                    return Ok(None);
                }
                self.has_left_row = true;
                self.left_had_match = false;

                if self.right_opened {
                    self.right.close()?;
                }
                self.right.open()?;
                self.right_opened = true;
            }

            // Step 2: Try to read next RHS row
            let has_right = unsafe {
                let right_ptr = &mut *self.right as *mut RelOp;
                let regs_ptr = regs as *mut [ValueRef<'a>];
                (*right_ptr)
                    .next_row(arena, &mut *regs_ptr, slot_count)?
                    .is_some()
            };

            if !has_right {
                // RHS exhausted for current LHS row
                if matches!(self.kind, JoinKind::Left) && !self.left_had_match {
                    for i in 0..self.right_input_count {
                        regs[self.right_input_start + i] = ValueRef::Null;
                    }
                    self.has_left_row = false;

                    match PipelineOp::run_steps(&mut self.steps, arena, regs, udf)? {
                        StepOutcome::Emit => {
                            let slots = &regs[0..slot_count];
                            return Ok(Some(RegisterReader::new(slots)));
                        }
                        StepOutcome::Skip => continue,
                        StepOutcome::Halt => return Ok(None),
                    }
                }

                self.has_left_row = false;
                continue;
            }

            // Step 3: Evaluate join condition (if any)
            let pass = match (&self.condition, self.condition_slot) {
                (Some(condition), Some(cond_slot)) => {
                    condition.eval(arena, regs, udf)?;
                    matches!(regs.get(cond_slot as usize), Some(&ValueRef::Bool(true)))
                }
                _ => true,
            };

            if !pass {
                continue;
            }

            self.left_had_match = true;

            // Step 4: Run post-join steps (projection, filter, limit)
            match PipelineOp::run_steps(&mut self.steps, arena, regs, udf)? {
                StepOutcome::Emit => {
                    let slots = &regs[0..slot_count];
                    return Ok(Some(RegisterReader::new(slots)));
                }
                StepOutcome::Skip => continue,
                StepOutcome::Halt => return Ok(None),
            }
        }
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
    /// Recursively instantiate a RelOpSpec into a RelOp.
    fn instantiate_op(
        spec: &RelOpSpec,
        compiled: &CompiledPlan,
        exec_context: &ExecutionContext,
    ) -> Result<RelOp> {
        match spec {
            RelOpSpec::Pipeline(pspec) => {
                let scan_meta = compiled.get_scan(pspec.scan_id).ok_or_else(|| {
                    EngineError::IllegalState(format!("Unknown scan_id: {:?}", pspec.scan_id))
                })?;
                let catalog = exec_context
                    .get_catalog(scan_meta.object_id.catalog_id())
                    .ok_or_else(|| {
                        EngineError::IllegalState(format!(
                            "Catalog {:?} not found",
                            scan_meta.object_id.catalog_id()
                        ))
                    })?;
                let data_source = catalog.create(pspec.scan_id)?;
                let reader = DataSourceImpl::Catalog(data_source);
                let steps = pspec.steps.iter().cloned().map(Step::from_spec).collect();
                Ok(RelOp::Pipeline(PipelineOp::new(steps, reader, None)))
            }
            RelOpSpec::ExprQuery(espec) => Ok(RelOp::ExprQuery(ExprQueryOp::new(
                espec.program.clone(),
                None,
            ))),
            RelOpSpec::NestedLoopJoin(jspec) => {
                // Recursively instantiate left and right children
                let left = Box::new(Self::instantiate_op(&jspec.left, compiled, exec_context)?);
                let right = Box::new(Self::instantiate_op(&jspec.right, compiled, exec_context)?);
                let steps = jspec.steps.iter().cloned().map(Step::from_spec).collect();
                Ok(RelOp::NestedLoopJoin(NestedLoopJoinOp::new(
                    NestedLoopJoinConfig {
                        kind: jspec.kind.clone(),
                        left,
                        right,
                        condition: jspec.condition.clone(),
                        condition_slot: jspec.condition_slot,
                        right_input_start: jspec.right_input_start,
                        right_input_count: jspec.right_input_count,
                        steps,
                        slot_count: compiled.slot_count,
                        udf: None,
                    },
                )))
            }
            _ => Err(EngineError::InvalidPlan(
                "unsupported operator spec".to_string(),
            )),
        }
    }

    pub fn new(compiled: CompiledPlan, exec_context: &ExecutionContext) -> Result<Self> {
        let compiled = Arc::new(compiled);
        let slot_count = compiled.slot_count;
        let root = compiled.root;

        let mut operators = Vec::with_capacity(compiled.nodes.len());
        for node in &compiled.nodes {
            operators.push(Self::instantiate_op(node, &compiled, exec_context)?);
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
        // Re-instantiate all operators using the same recursive helper
        let mut operators = Vec::with_capacity(self.compiled.nodes.len());
        for node in &self.compiled.nodes {
            operators.push(Self::instantiate_op(node, &self.compiled, exec_context)?);
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

// ---------------------------------------------------------------------------
// Display implementations for plan debugging
// ---------------------------------------------------------------------------

impl std::fmt::Display for CompiledPlan {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        writeln!(f, "CompiledPlan {{")?;
        writeln!(f, "  slot_count: {}", self.slot_count)?;
        writeln!(f, "  max_registers: {}", self.max_registers)?;
        writeln!(f, "  total_regs: {}", self.slot_count + self.max_registers)?;
        writeln!(f, "  root: {}", self.root)?;
        writeln!(f, "  shape: {:?}", self.shape)?;
        writeln!(f)?;

        // Print nodes
        for (i, node) in self.nodes.iter().enumerate() {
            let marker = if i == self.root { " (ROOT)" } else { "" };
            writeln!(f, "  Node[{}]{}:", i, marker)?;
            write_relop_spec(f, node, 4)?;
        }

        // Print scan metadata
        if !self.scan_metadata.is_empty() {
            writeln!(f)?;
            writeln!(f, "  Scans:")?;
            let mut scans: Vec<_> = self.scan_metadata.iter().collect();
            scans.sort_by_key(|(id, _)| id.as_u64());
            for (scan_id, meta) in scans {
                writeln!(
                    f,
                    "    scan_id={} → object(catalog={:?}, entry={:?})",
                    scan_id.as_u64(),
                    meta.object_id.catalog_id(),
                    meta.object_id.entry_id(),
                )?;
                for proj in &meta.layout.projections {
                    writeln!(
                        f,
                        "      projection: source={:?} → slot {}",
                        proj.source, proj.target_slot
                    )?;
                }
            }
        }

        writeln!(f, "}}")
    }
}

fn write_relop_spec(
    f: &mut std::fmt::Formatter<'_>,
    spec: &RelOpSpec,
    indent: usize,
) -> std::fmt::Result {
    let pad = " ".repeat(indent);
    match spec {
        RelOpSpec::Pipeline(p) => {
            writeln!(f, "{}Pipeline(scan_id={})", pad, p.scan_id.as_u64())?;
            write_steps(f, &p.steps, indent + 2)?;
        }
        RelOpSpec::ExprQuery(eq) => {
            writeln!(f, "{}ExprQuery", pad)?;
            write_program(f, &eq.program, indent + 2)?;
        }
        RelOpSpec::NestedLoopJoin(j) => {
            writeln!(
                f,
                "{}NestedLoopJoin(kind={:?}, cond_slot={:?}, right_start={}, right_count={})",
                pad, j.kind, j.condition_slot, j.right_input_start, j.right_input_count
            )?;
            writeln!(f, "{}  Left:", pad)?;
            write_relop_spec(f, &j.left, indent + 4)?;
            writeln!(f, "{}  Right:", pad)?;
            write_relop_spec(f, &j.right, indent + 4)?;
            if let Some(ref cond) = j.condition {
                writeln!(f, "{}  Condition program:", pad)?;
                write_program(f, cond, indent + 4)?;
            }
            write_steps(f, &j.steps, indent + 2)?;
        }
        RelOpSpec::HashJoin(_) => writeln!(f, "{}HashJoin (not implemented)", pad)?,
        RelOpSpec::HashAgg(_) => writeln!(f, "{}HashAgg (not implemented)", pad)?,
        RelOpSpec::Sort(_) => writeln!(f, "{}Sort (not implemented)", pad)?,
        RelOpSpec::Custom(_) => writeln!(f, "{}Custom", pad)?,
    }
    Ok(())
}

fn write_steps(
    f: &mut std::fmt::Formatter<'_>,
    steps: &[StepSpec],
    indent: usize,
) -> std::fmt::Result {
    if steps.is_empty() {
        return Ok(());
    }
    let pad = " ".repeat(indent);
    writeln!(f, "{}Steps:", pad)?;
    for (i, step) in steps.iter().enumerate() {
        match step {
            StepSpec::Filter {
                program,
                predicate_slot,
            } => {
                writeln!(
                    f,
                    "{}  [{}] Filter(predicate_slot={})",
                    pad, i, predicate_slot
                )?;
                write_program(f, program, indent + 4)?;
            }
            StepSpec::Project { program } => {
                writeln!(f, "{}  [{}] Project", pad, i)?;
                write_program(f, program, indent + 4)?;
            }
            StepSpec::Limit { limit } => {
                writeln!(f, "{}  [{}] Limit({})", pad, i, limit)?;
            }
        }
    }
    Ok(())
}

fn write_program(
    f: &mut std::fmt::Formatter<'_>,
    program: &Program,
    indent: usize,
) -> std::fmt::Result {
    let pad = " ".repeat(indent);
    writeln!(
        f,
        "{}Program(reg_count={}, slot_count={}, consts={}, keys={:?})",
        pad,
        program.reg_count,
        program.slot_count,
        program.consts.len(),
        program.keys,
    )?;
    for (i, inst) in program.insts.iter().enumerate() {
        writeln!(f, "{}  [{}] {:?}", pad, i, inst)?;
    }
    Ok(())
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
