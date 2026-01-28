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

pub struct CompiledPlan {
    pub nodes: Vec<RelOpSpec>,
    pub root: usize,
    pub schema: Schema,
    pub slot_count: usize,
    pub max_registers: usize,
}

impl Default for CompiledPlan {
    fn default() -> Self {
        Self {
            nodes: Vec::new(),
            root: 0,
            schema: Schema::default(),
            slot_count: 0,
            max_registers: 0,
        }
    }
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
    
    /// Returns the maximum number of registers needed across all programs in this plan
    pub fn max_register_count(&self) -> usize {
        self.max_registers
    }
}

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
    pub reader_factory: ReaderFactoryEnum,
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

#[derive(Clone)]
pub enum ReaderFactoryEnum {
    InMem(crate::engine::reader::InMemGeneratedReaderFactory),
    Ion(crate::engine::reader::IonRowReaderFactory),
}

impl ReaderFactoryEnum {
    pub fn create(&self) -> crate::engine::error::Result<Box<dyn crate::engine::reader::RowReader>> {
        use crate::engine::reader::RowReaderFactory;
        match self {
            ReaderFactoryEnum::InMem(factory) => factory.create(),
            ReaderFactoryEnum::Ion(factory) => factory.create(),
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
}

pub enum RelOp {
    Pipeline(PipelineOp),
    HashJoin(HashJoinState),
    HashAgg(HashAggState),
    Sort(SortState),
    Custom(Box<dyn BlockingOperator>),
}

impl RelOp {
    pub fn next_row<'a>(
        &'a mut self,
        arena: &'a Arena,
        regs: &mut [ValueRef<'a>],
    ) -> Result<bool> {
        match self {
            RelOp::Pipeline(op) => op.next_row(arena, regs),
            RelOp::HashJoin(op) => op.next_row(arena, regs),
            RelOp::HashAgg(op) => op.next_row(arena, regs),
            RelOp::Sort(op) => op.next_row(arena, regs),
            RelOp::Custom(op) => op.next_row(arena, regs),
        }
    }

    /// Reset operator state for reuse
    pub fn reset(&mut self) -> Result<()> {
        match self {
            RelOp::Pipeline(op) => op.reset(),
            RelOp::HashJoin(op) => op.reset(),
            RelOp::HashAgg(op) => op.reset(),
            RelOp::Sort(op) => op.reset(),
            RelOp::Custom(_) => Ok(()), // Custom operators must handle their own reset
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

    pub fn next_row<'a>(
        &'a mut self,
        arena: &'a Arena,
        regs: &mut [ValueRef<'a>],
    ) -> Result<bool> {
        if !self.opened {
            self.reader.set_projection(self.layout.clone())?;
            self.reader.open()?;
            self.opened = true;
        }

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

    /// Reset pipeline state for reuse
    pub fn reset(&mut self) -> Result<()> {
        self.opened = false;
        // Reset step state (e.g., Limit counter)
        for step in &mut self.steps {
            step.reset();
        }
        Ok(())
    }
}

impl HashJoinState {
    pub fn next_row(&mut self, _arena: &Arena, _regs: &mut [ValueRef<'_>]) -> Result<bool> {
        Err(EngineError::NotImplemented)
    }

    pub fn reset(&mut self) -> Result<()> {
        Err(EngineError::NotImplemented)
    }
}

impl HashAggState {
    pub fn next_row(&mut self, _arena: &Arena, _regs: &mut [ValueRef<'_>]) -> Result<bool> {
        Err(EngineError::NotImplemented)
    }

    pub fn reset(&mut self) -> Result<()> {
        Err(EngineError::NotImplemented)
    }
}

impl SortState {
    pub fn next_row(&mut self, _arena: &Arena, _regs: &mut [ValueRef<'_>]) -> Result<bool> {
        Err(EngineError::NotImplemented)
    }

    pub fn reset(&mut self) -> Result<()> {
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
    pub fn new(
        compiled: CompiledPlan,
        udf_registry: Option<Arc<dyn UdfRegistry>>,
    ) -> Result<Self> {
        let compiled = Arc::new(compiled);
        let slot_count = compiled.slot_count;
        let root = compiled.root;
        
        // Instantiate operators from specs
        let mut operators = Vec::with_capacity(compiled.nodes.len());
        for node in &compiled.nodes {
            match node {
                RelOpSpec::Pipeline(spec) => {
                    let reader = spec.reader_factory.create()?;
                    let steps = spec
                        .steps
                        .iter()
                        .cloned()
                        .map(Step::from_spec)
                        .collect();
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
        let max_regs = compiled.max_register_count();
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
    
    /// Get the next row from query execution
    /// 
    /// Returns `Ok(Some(row))` if a row is available, `Ok(None)` if the query is complete,
    /// or `Err` if an error occurred during execution.
    /// 
    /// # Example
    /// ```ignore
    /// let mut vm = compiler.instantiate(compiled, None)?;
    /// while let Some(row) = vm.next_row()? {
    ///     // Process row
    /// }
    /// ```
    pub fn next_row(&mut self) -> Result<Option<crate::engine::row::RowView<'_>>> {
        // Reset arena for this row
        self.arena.reset();
        
        // Borrow registers from VM (transmute lifetime to match arena)
        let regs = unsafe {
            std::mem::transmute::<&mut [ValueRef<'static>], &mut [ValueRef<'_>]>(
                self.registers.as_mut_slice()
            )
        };
        
        let op = self
            .operators
            .get_mut(self.root)
            .ok_or_else(|| EngineError::IllegalState("invalid root operator".to_string()))?;
        
        if op.next_row(&self.arena, regs)? {
            // Return view of slot region
            let slots = &regs[0..self.slot_count];
            Ok(Some(crate::engine::row::RowView::new(slots)))
        } else {
            Ok(None)
        }
    }
    
    /// Reset the VM to initial state for reuse
    /// 
    /// After calling `reset()`, the VM can be used for another execution
    /// of the same query. This is more efficient than creating a new VM.
    /// 
    /// # Example
    /// ```ignore
    /// let mut vm = compiler.instantiate(compiled, None)?;
    /// 
    /// // First execution
    /// while vm.next_row()?.is_some() { }
    /// 
    /// // Reset and execute again
    /// vm.reset()?;
    /// while vm.next_row()?.is_some() { }
    /// ```
    pub fn reset(&mut self) -> Result<()> {
        // Reset all operators
        for op in &mut self.operators {
            op.reset()?;
        }
        
        // Clear arena
        self.arena.reset();
        
        // Clear registers
        for reg in &mut self.registers {
            *reg = ValueRef::Missing;
        }
        
        Ok(())
    }
    
    /// Execute the plan and return a result stream (deprecated)
    /// 
    /// **Deprecated**: Use `next_row()` directly instead. This method is kept
    /// for backward compatibility but will be removed in a future version.
    /// 
    /// # Migration
    /// Old code:
    /// ```ignore
    /// let mut stream = vm.execute()?;
    /// while let Some(row) = stream.next_row()? { }
    /// ```
    /// 
    /// New code:
    /// ```ignore
    /// while let Some(row) = vm.next_row()? { }
    /// ```
    #[deprecated(since = "0.1.0", note = "Use next_row() directly instead")]
    pub fn execute(self) -> Result<ResultStream> {
        Ok(ResultStream {
            schema: self.compiled.result_schema(),
            vm: self,
        })
    }
}

/// Stream of results from query execution (deprecated)
/// 
/// **Deprecated**: Use `PartiQLVM::next_row()` directly instead.
#[deprecated(since = "0.1.0", note = "Use PartiQLVM::next_row() directly")]
pub struct ResultStream {
    pub schema: Schema,
    vm: PartiQLVM,
}

#[allow(deprecated)]
impl ResultStream {
    pub fn next_row(&mut self) -> Result<Option<crate::engine::row::RowView<'_>>> {
        self.vm.next_row()
    }
}

#[derive(Clone)]
pub enum StepSpec {
    Filter { program: Program, predicate_slot: SlotId },
    Project { program: Program },
    Limit { limit: usize },
}

pub enum Step {
    Filter { program: Program, predicate_slot: SlotId },
    Project { program: Program },
    Limit { limit: usize, remaining: usize },
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
            StepSpec::Limit { limit } => Step::Limit { limit, remaining: limit },
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
                    Some(&ValueRef::Bool(false)) | Some(&ValueRef::Missing) | Some(&ValueRef::Null) => {
                        Ok(StepOutcome::Skip)
                    }
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

    /// Reset step state for reuse
    fn reset(&mut self) {
        if let Step::Limit { limit, remaining } = self {
            // Reset remaining counter to original limit
            *remaining = *limit;
        }
    }
}
