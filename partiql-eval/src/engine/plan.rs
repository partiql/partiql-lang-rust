use std::sync::Arc;

use crate::engine::error::{EngineError, Result};
use crate::engine::expr::Program;
use crate::engine::reader::{RowReader, RowReaderFactory, ScanLayout};
use crate::engine::row::{RowFrame, RowFrameScratch, SlotId};
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
    pub nodes: Vec<RelOpSpec>,
    pub root: usize,
    pub schema: Schema,
    pub slot_count: usize,
}

impl CompiledPlan {
    pub fn result_schema(&self) -> Schema {
        self.schema.clone()
    }
}

pub enum RelOpSpec {
    Pipeline(PipelineSpec),
    HashJoin(HashJoinSpec),
    HashAgg(HashAggSpec),
    Sort(SortSpec),
    Custom(Box<dyn BlockingOperatorSpec>),
}

pub struct PipelineSpec {
    pub layout: ScanLayout,
    pub steps: Vec<StepSpec>,
    pub reader_factory: Box<dyn RowReaderFactory>,
}
pub struct HashJoinSpec;
pub struct HashAggSpec;
pub struct SortSpec;

pub trait BlockingOperatorSpec: Send + Sync {
    fn instantiate(&self) -> Box<dyn BlockingOperator>;
}

pub trait BlockingOperator {
    fn next_row(&mut self, out: &mut RowFrame<'_>) -> Result<bool>;
}

pub enum RelOp {
    Pipeline(PipelineOp),
    HashJoin(HashJoinState),
    HashAgg(HashAggState),
    Sort(SortState),
    Custom(Box<dyn BlockingOperator>),
}

impl RelOp {
    pub fn next_row<'a>(&'a mut self, out: &mut RowFrame<'a>) -> Result<bool> {
        match self {
            RelOp::Pipeline(op) => op.next_row(out),
            RelOp::HashJoin(op) => op.next_row(out),
            RelOp::HashAgg(op) => op.next_row(out),
            RelOp::Sort(op) => op.next_row(out),
            RelOp::Custom(op) => op.next_row(out),
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

    pub fn next_row<'a>(&'a mut self, out: &mut RowFrame<'a>) -> Result<bool> {
        if !self.opened {
            self.reader.set_projection(self.layout.clone())?;
            self.reader.open()?;
            self.opened = true;
        }

        let udf = self.udf.as_deref();
        loop {
            if !self.reader.next_row(out)? {
                return Ok(false);
            }
            match Self::run_steps(&mut self.steps, out, udf)? {
                StepOutcome::Emit => return Ok(true),
                StepOutcome::Skip => continue,
                StepOutcome::Halt => return Ok(false),
            }
        }
    }

    fn run_steps<'a>(
        steps: &mut [Step],
        frame: &mut RowFrame<'a>,
        udf: Option<&'a dyn UdfRegistry>,
    ) -> Result<StepOutcome> {
        for step in steps {
            match step.eval(frame, udf)? {
                StepOutcome::Emit => {}
                StepOutcome::Skip => return Ok(StepOutcome::Skip),
                StepOutcome::Halt => return Ok(StepOutcome::Halt),
            }
        }
        Ok(StepOutcome::Emit)
    }
}

impl HashJoinState {
    pub fn next_row(&mut self, _out: &mut RowFrame<'_>) -> Result<bool> {
        Err(EngineError::NotImplemented)
    }
}

impl HashAggState {
    pub fn next_row(&mut self, _out: &mut RowFrame<'_>) -> Result<bool> {
        Err(EngineError::NotImplemented)
    }
}

impl SortState {
    pub fn next_row(&mut self, _out: &mut RowFrame<'_>) -> Result<bool> {
        Err(EngineError::NotImplemented)
    }
}

pub struct PlanInstance {
    pub compiled: Arc<CompiledPlan>,
    pub operators: Vec<RelOp>,
}

impl PlanInstance {
    pub fn execute(self) -> Result<ResultStream> {
        let slot_count = self.compiled.slot_count;
        Ok(ResultStream {
            schema: self.compiled.result_schema(),
            root: self.compiled.root,
            instance: self,
            scratch: RowFrameScratch::new(slot_count),
        })
    }
}

pub struct ResultStream {
    pub schema: Schema,
    root: usize,
    instance: PlanInstance,
    scratch: RowFrameScratch,
}

impl ResultStream {
    pub fn next_row(&mut self) -> Result<Option<crate::engine::row::RowView<'_>>> {
        self.scratch.reset();
        let mut frame = self.scratch.frame();
        let op = self
            .instance
            .operators
            .get_mut(self.root)
            .ok_or_else(|| EngineError::IllegalState("invalid root operator".to_string()))?;
        if op.next_row(&mut frame)? {
            Ok(Some(crate::engine::row::RowView::new(frame.slots)))
        } else {
            Ok(None)
        }
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
    Limit { remaining: usize },
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
            StepSpec::Limit { limit } => Step::Limit { remaining: limit },
        }
    }

    fn eval<'a>(
        &mut self,
        frame: &mut RowFrame<'a>,
        udf: Option<&'a dyn UdfRegistry>,
    ) -> Result<StepOutcome> {
        match self {
            Step::Filter {
                program,
                predicate_slot,
            } => {
                program.eval(frame, udf)?;
                match frame.slots.get(*predicate_slot as usize) {
                    Some(slot) => match slot.as_ref() {
                        ValueRef::Bool(true) => Ok(StepOutcome::Emit),
                        ValueRef::Bool(false) | ValueRef::Missing | ValueRef::Null => {
                            Ok(StepOutcome::Skip)
                        }
                        _ => Err(EngineError::TypeError(
                            "filter predicate must be bool".to_string(),
                        )),
                    },
                    None => Err(EngineError::IllegalState(
                        "filter predicate slot missing".to_string(),
                    )),
                }
            }
            Step::Project { program } => {
                program.eval(frame, udf)?;
                Ok(StepOutcome::Emit)
            }
            Step::Limit { remaining } => {
                if *remaining == 0 {
                    return Ok(StepOutcome::Halt);
                }
                *remaining -= 1;
                Ok(StepOutcome::Emit)
            }
        }
    }
}
