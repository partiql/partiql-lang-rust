use crate::engine::error::Result;
use crate::engine::row::SlotId;

#[derive(Clone, Copy, Debug)]
pub enum BufferStability {
    UntilNext,
    UntilClose,
}

#[derive(Clone, Copy, Debug)]
pub struct ReaderCaps {
    pub stability: BufferStability,
    pub can_project: bool,
    pub can_return_opaque: bool,
}

#[derive(Clone, Debug, Default)]
pub struct ScanLayout {
    pub projections: Vec<ScanProjection>,
}

impl ScanLayout {
    pub fn base_row() -> Self {
        ScanLayout {
            projections: vec![ScanProjection {
                source: ScanSource::BaseRow,
                target_slot: 0,
                type_hint: TypeHint::Any,
            }],
        }
    }

    pub fn is_base_row_only(&self) -> bool {
        self.projections.len() == 1
            && matches!(self.projections[0].source, ScanSource::BaseRow)
            && self.projections[0].target_slot == 0
    }
}

#[derive(Clone, Debug)]
pub struct ScanProjection {
    pub source: ScanSource,
    pub target_slot: SlotId,
    pub type_hint: TypeHint,
}

#[derive(Clone, Debug)]
pub enum ScanSource {
    ColumnIndex(usize),
    FieldPath(String),
    BaseRow,
}

#[derive(Clone, Copy, Debug)]
pub enum TypeHint {
    Any,
}

pub trait RowReader {
    fn open(&mut self) -> Result<()>;
    fn next_row(&mut self, writer: &mut crate::engine::row::ValueWriter<'_, '_>) -> Result<bool>;
    fn close(&mut self) -> Result<()>;
}

pub trait RowReaderFactory: Send + Sync {
    fn create(&self, layout: ScanLayout) -> Result<Box<dyn RowReader>>;
    fn caps(&self) -> ReaderCaps;
    fn resolve(&self, field_name: &str) -> Option<ScanSource>;
}
