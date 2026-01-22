use std::sync::Arc;

use crate::engine::error::{EngineError, Result};
use crate::engine::row::RowFrame;

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

pub struct PipelineSpec;
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
    pub fn next_row(&mut self, out: &mut RowFrame<'_>) -> Result<bool> {
        match self {
            RelOp::Pipeline(op) => op.next_row(out),
            RelOp::HashJoin(op) => op.next_row(out),
            RelOp::HashAgg(op) => op.next_row(out),
            RelOp::Sort(op) => op.next_row(out),
            RelOp::Custom(op) => op.next_row(out),
        }
    }
}

pub struct PipelineOp;
pub struct HashJoinState;
pub struct HashAggState;
pub struct SortState;

impl PipelineOp {
    pub fn next_row(&mut self, _out: &mut RowFrame<'_>) -> Result<bool> {
        Err(EngineError::NotImplemented)
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
        Ok(ResultStream {
            schema: self.compiled.result_schema(),
            root: self.compiled.root,
            instance: self,
        })
    }
}

pub struct ResultStream {
    pub schema: Schema,
    root: usize,
    instance: PlanInstance,
}

impl ResultStream {
    pub fn next_row(&mut self) -> Result<Option<crate::engine::row::RowView<'_>>> {
        let _ = (&self.root, &self.instance);
        Ok(None)
    }
}
