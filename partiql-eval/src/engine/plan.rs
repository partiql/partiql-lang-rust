use std::sync::Arc;

use crate::engine::error::{EngineError, Result};
use crate::engine::expr::Program;
use crate::engine::reader::{RowReader, RowReaderFactory, ScanLayout};
use crate::engine::row::{Arena, SlotId};
use crate::engine::value::ValueRef;
use crate::engine::UdfRegistry;

#[derive(Clone, Debug, Default)]
pub struct Schema {
    pub columns: Vec<Column>,
}

#[derive(Clone, Debug)]
pub struct Column {
    pub name: String,
}

#[derive(Default)]
pub struct CompiledPlan {
    pub(crate) nodes: Vec<RelOpSpec>,
    pub(crate) root: usize,
    pub(crate) schema: Schema,
    pub(crate) slot_count: usize,
    pub(crate) max_registers: usize,
}

// Conditional bounds ensure CompiledPlan is only Send/Sync when all fields are.
// This provides compile-time verification - the compiler verifies the bounds.
// Safety: The bounds ensure all fields are Send/Sync, making this impl safe.
unsafe impl Send for CompiledPlan
where
    Vec<RelOpSpec>: Send,
    Schema: Send,
{
}

unsafe impl Sync for CompiledPlan
where
    Vec<RelOpSpec>: Sync,
    Schema: Sync,
{
}

impl Clone for CompiledPlan {
    fn clone(&self) -> Self {
        Self {
            nodes: self.nodes.iter().map(|node| node.clone_spec()).collect(),
            root: self.root,
            schema: self.schema.clone(),
            slot_count: self.slot_count,
            max_registers: self.max_registers,
        }
    }
}

impl CompiledPlan {
    pub fn result_schema(&self) -> Schema {
        self.schema.clone()
    }
}

// TODO: Actually implement HashJoin and whatnot.
#[allow(dead_code)]
pub enum RelOpSpec {
    Pipeline(PipelineSpec),
    HashJoin(HashJoinSpec),
    HashAgg(HashAggSpec),
    Sort(SortSpec),
    Custom(Box<dyn BlockingOperatorSpec>),
}

impl RelOpSpec {
    pub(crate) fn clone_spec(&self) -> Self {
        match self {
            RelOpSpec::Pipeline(spec) => RelOpSpec::Pipeline(spec.clone_pipeline()),
            RelOpSpec::HashJoin(_) => RelOpSpec::HashJoin(HashJoinSpec),
            RelOpSpec::HashAgg(_) => RelOpSpec::HashAgg(HashAggSpec),
            RelOpSpec::Sort(_) => RelOpSpec::Sort(SortSpec),
            RelOpSpec::Custom(_) => panic!("Cannot clone custom operator spec"),
        }
    }
}

pub struct PipelineSpec {
    pub layout: ScanLayout,
    pub steps: Vec<StepSpec>,
    pub reader_factory: crate::engine::reader::ReaderFactory,
}

impl PipelineSpec {
    pub(crate) fn clone_pipeline(&self) -> Self {
        Self {
            layout: self.layout.clone(),
            steps: self.steps.clone(),
            reader_factory: self.reader_factory.clone(),
        }
    }
}
pub struct HashJoinSpec;
pub struct HashAggSpec;
pub struct SortSpec;

pub trait BlockingOperatorSpec: Send + Sync {
    fn instantiate(&self) -> Box<dyn BlockingOperator>;
}

pub trait BlockingOperator {
    fn next_row(&mut self, arena: &Arena, regs: &mut [ValueRef<'_>]) -> Result<bool>;
    fn open(&mut self) -> Result<()>;
    fn close(&mut self) -> Result<()>;
}

// TODO: Actually implement HashJoin and whatnot.
#[allow(dead_code)]
pub(crate) enum RelOp {
    Pipeline(PipelineOp),
    HashJoin(HashJoinState),
    HashAgg(HashAggState),
    Sort(SortState),
    Custom(Box<dyn BlockingOperator>),
}

impl RelOp {
    pub fn next_row<'a>(&'a mut self, arena: &'a Arena, regs: &mut [ValueRef<'a>]) -> Result<bool> {
        match self {
            RelOp::Pipeline(op) => op.next_row(arena, regs),
            RelOp::HashJoin(op) => op.next_row(arena, regs),
            RelOp::HashAgg(op) => op.next_row(arena, regs),
            RelOp::Sort(op) => op.next_row(arena, regs),
            RelOp::Custom(op) => op.next_row(arena, regs),
        }
    }

    /// Open operator and allocate resources
    pub fn open(&mut self) -> Result<()> {
        match self {
            RelOp::Pipeline(op) => op.open(),
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
            RelOp::HashJoin(op) => op.close(),
            RelOp::HashAgg(op) => op.close(),
            RelOp::Sort(op) => op.close(),
            RelOp::Custom(op) => op.close(),
        }
    }
}

pub struct PipelineOp {
    layout: ScanLayout,
    steps: Vec<Step>,
    reader: Box<dyn RowReader>,
    opened: bool,
    udf: Option<Arc<dyn UdfRegistry>>,
}
pub struct HashJoinState;
pub struct HashAggState;
pub struct SortState;

impl PipelineOp {
    pub(crate) fn new(
        layout: ScanLayout,
        steps: Vec<Step>,
        reader: Box<dyn RowReader>,
        udf: Option<Arc<dyn UdfRegistry>>,
    ) -> Self {
        PipelineOp {
            layout,
            steps,
            reader,
            opened: false,
            udf,
        }
    }

    pub fn open(&mut self) -> Result<()> {
        if !self.opened {
            self.reader.set_projection(self.layout.clone())?;
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

    pub fn next_row<'a>(&'a mut self, arena: &'a Arena, regs: &mut [ValueRef<'a>]) -> Result<bool> {
        let udf = self.udf.as_deref();
        loop {
            if !self.reader.next_row(regs)? {
                return Ok(false);
            }
            match Self::run_steps(&mut self.steps, arena, regs, udf)? {
                StepOutcome::Emit => return Ok(true),
                StepOutcome::Skip => continue,
                StepOutcome::Halt => return Ok(false),
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

    pub fn next_row(&mut self, _arena: &Arena, _regs: &mut [ValueRef<'_>]) -> Result<bool> {
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

    pub fn next_row(&mut self, _arena: &Arena, _regs: &mut [ValueRef<'_>]) -> Result<bool> {
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

    pub fn next_row(&mut self, _arena: &Arena, _regs: &mut [ValueRef<'_>]) -> Result<bool> {
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
    /// Create a new VM instance from a compiled plan
    ///
    /// # Arguments
    /// * `compiled` - The compiled query plan to execute
    /// * `udf_registry` - Optional registry for user-defined functions
    ///
    /// # Returns
    /// A new PartiQLVM ready to execute the plan
    pub fn new(compiled: CompiledPlan, udf_registry: Option<Arc<dyn UdfRegistry>>) -> Result<Self> {
        let compiled = Arc::new(compiled);
        let slot_count = compiled.slot_count;
        let root = compiled.root;

        // Instantiate operators from specs
        let mut operators = Vec::with_capacity(compiled.nodes.len());
        for node in &compiled.nodes {
            match node {
                RelOpSpec::Pipeline(spec) => {
                    let reader = spec.reader_factory.create()?;
                    let steps = spec.steps.iter().cloned().map(Step::from_spec).collect();
                    operators.push(RelOp::Pipeline(PipelineOp::new(
                        spec.layout.clone(),
                        steps,
                        reader,
                        udf_registry.clone(),
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

    /// Get the result schema for this VM's query
    pub fn schema(&self) -> Schema {
        self.compiled.result_schema()
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
    fn next_row_internal(&mut self) -> Option<Result<crate::engine::row::RowView<'_>>> {
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

        match op.next_row(&self.vm.arena, regs) {
            Ok(true) => {
                // Return view of slot region
                let slots = &regs[0..self.vm.slot_count];
                Some(Ok(crate::engine::row::RowView::new(slots)))
            }
            Ok(false) => None,
            Err(e) => Some(Err(e)),
        }
    }
}

impl<'vm> Iterator for QueryIterator<'vm> {
    type Item = Result<crate::engine::row::RowView<'vm>>;

    fn next(&mut self) -> Option<Self::Item> {
        // Delegate to the lending next_row_internal() method
        // Safety: We extend the lifetime to 'vm, which is safe because:
        // 1. The VM owns the arena and registers
        // 2. The iterator has exclusive access to the VM via &mut
        // 3. Callers must not hold references across next() calls (standard iterator contract)
        match self.next_row_internal() {
            Some(Ok(row)) => {
                let row = unsafe {
                    std::mem::transmute::<
                        crate::engine::row::RowView<'_>,
                        crate::engine::row::RowView<'vm>,
                    >(row)
                };
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
