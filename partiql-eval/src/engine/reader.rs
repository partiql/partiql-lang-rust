use crate::engine::error::{EngineError, Result};
use crate::engine::row::{RowFrame, SlotId, SlotValue};
use crate::engine::value::{value_get_field_ref, value_ref_from_value_in_arena, ValueRef};
use partiql_extension_ion::boxed_ion::BoxedIonType;
use partiql_value::{BindingsName, Value};
use std::fs::File;
use std::io::BufReader;

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
    fn caps(&self) -> ReaderCaps;
    fn set_projection(&mut self, layout: ScanLayout) -> Result<()>;
    fn open(&mut self) -> Result<()>;
    fn next_row(&mut self, out: &mut RowFrame<'_>) -> Result<bool>;
    fn resolve(&self, field_name: &str) -> Option<ScanSource>;
    fn close(&mut self) -> Result<()>;
}

pub trait RowReaderFactory {
    fn create(&self) -> Result<Box<dyn RowReader>>;
}

pub struct ValueRowReader {
    rows: Vec<Value>,
    pos: usize,
    layout: ScanLayout,
    caps: ReaderCaps,
}

#[derive(Clone)]
pub struct ValueRowReaderFactory {
    rows: Vec<Value>,
}

impl ValueRowReaderFactory {
    pub fn new(rows: Vec<Value>) -> Self {
        ValueRowReaderFactory { rows }
    }
}

impl RowReaderFactory for ValueRowReaderFactory {
    fn create(&self) -> Result<Box<dyn RowReader>> {
        Ok(Box::new(ValueRowReader::new(self.rows.clone())))
    }
}

pub struct IonRowReader {
    path: String,
    iter: Option<partiql_extension_ion::boxed_ion::BoxedIonIterator>,
    layout: ScanLayout,
    caps: ReaderCaps,
}

impl IonRowReader {
    pub fn new(path: String) -> Self {
        IonRowReader {
            path,
            iter: None,
            layout: ScanLayout::base_row(),
            caps: ReaderCaps {
                stability: BufferStability::UntilNext,
                can_project: true,
                can_return_opaque: true,
            },
        }
    }
}

impl RowReader for IonRowReader {
    fn caps(&self) -> ReaderCaps {
        self.caps
    }

    fn set_projection(&mut self, layout: ScanLayout) -> Result<()> {
        if !self.caps.can_project && !layout.is_base_row_only() {
            return Err(EngineError::ProjectionNotSupported(
                "reader does not support projection",
            ));
        }
        self.layout = layout;
        Ok(())
    }

    fn open(&mut self) -> Result<()> {
        let file = File::open(&self.path)
            .map_err(|e| EngineError::ReaderError(format!("ion open failed: {e}")))?;
        let reader = BufReader::new(file);
        let boxed = BoxedIonType {}
            .stream_from_read(reader)
            .map_err(|e| EngineError::ReaderError(format!("ion parse failed: {e}")))?;
        let iter = boxed
            .try_into_iter()
            .map_err(|e| EngineError::ReaderError(format!("ion iter failed: {e}")))?;
        self.iter = Some(iter);
        Ok(())
    }

    fn next_row(&mut self, out: &mut RowFrame<'_>) -> Result<bool> {
        let iter = match self.iter.as_mut() {
            Some(iter) => iter,
            None => return Ok(false),
        };
        let boxed = match iter.next() {
            Some(Ok(value)) => value,
            Some(Err(err)) => {
                return Err(EngineError::ReaderError(format!(
                    "ion read failed: {err}"
                )))
            }
            None => return Ok(false),
        };

        let base_value = boxed.into_value();
        let base_ref = value_ref_from_value_in_arena(&base_value, out.arena);

        for proj in &self.layout.projections {
            let target = proj.target_slot as usize;
            if target >= out.slots.len() {
                continue;
            }
            let value = match &proj.source {
                ScanSource::BaseRow => base_ref,
                ScanSource::FieldPath(path) => {
                    let mut current = base_ref;
                    for part in path.split('.') {
                        current = value_get_field_ref(current, part, out.arena);
                    }
                    current
                }
                ScanSource::ColumnIndex(_) => ValueRef::Missing,
            };
            out.slots[target] = SlotValue::Val(value);
        }

        Ok(true)
    }

    fn resolve(&self, field_name: &str) -> Option<ScanSource> {
        Some(ScanSource::FieldPath(field_name.to_string()))
    }

    fn close(&mut self) -> Result<()> {
        self.iter = None;
        Ok(())
    }
}

pub struct IonRowReaderFactory {
    path: String,
}

impl IonRowReaderFactory {
    pub fn new(path: String) -> Self {
        IonRowReaderFactory { path }
    }
}

impl RowReaderFactory for IonRowReaderFactory {
    fn create(&self) -> Result<Box<dyn RowReader>> {
        Ok(Box::new(IonRowReader::new(self.path.clone())))
    }
}

impl ValueRowReader {
    pub fn new(rows: Vec<Value>) -> Self {
        ValueRowReader {
            rows,
            pos: 0,
            layout: ScanLayout::base_row(),
            caps: ReaderCaps {
                stability: BufferStability::UntilClose,
                can_project: true,
                can_return_opaque: false,
            },
        }
    }
}

impl RowReader for ValueRowReader {
    fn caps(&self) -> ReaderCaps {
        self.caps
    }

    fn set_projection(&mut self, layout: ScanLayout) -> Result<()> {
        if !self.caps.can_project && !layout.is_base_row_only() {
            return Err(EngineError::ProjectionNotSupported(
                "reader does not support projection",
            ));
        }
        self.layout = layout;
        Ok(())
    }

    fn open(&mut self) -> Result<()> {
        self.pos = 0;
        Ok(())
    }

    fn next_row(&mut self, out: &mut RowFrame<'_>) -> Result<bool> {
        if self.pos >= self.rows.len() {
            return Ok(false);
        }
        let row = &self.rows[self.pos];
        self.pos += 1;

        for proj in &self.layout.projections {
            let target = proj.target_slot as usize;
            if target >= out.slots.len() {
                continue;
            }
            let value = match &proj.source {
                ScanSource::BaseRow => Some(row),
                ScanSource::FieldPath(path) => get_field_path(row, path),
                ScanSource::ColumnIndex(index) => get_column_index(row, *index),
            };
            let value = match value {
                Some(value) => value_ref_from_value_in_arena(value, out.arena),
                None => ValueRef::Missing,
            };
            out.slots[target] = SlotValue::Val(value);
        }
        Ok(true)
    }

    fn resolve(&self, field_name: &str) -> Option<ScanSource> {
        Some(ScanSource::FieldPath(field_name.to_string()))
    }

    fn close(&mut self) -> Result<()> {
        Ok(())
    }
}

fn get_field_path<'a>(row: &'a Value, path: &str) -> Option<&'a Value> {
    let mut current = row;
    for part in path.split('.') {
        let key = BindingsName::CaseInsensitive(part.into());
        match current {
            Value::Tuple(tuple) => {
                if let Some(value) = tuple.get(&key) {
                    current = value;
                } else {
                    return None;
                }
            }
            _ => return None,
        }
    }
    Some(current)
}

fn get_column_index<'a>(row: &'a Value, index: usize) -> Option<&'a Value> {
    match row {
        Value::Tuple(tuple) => tuple.values().nth(index),
        _ => None,
    }
}
