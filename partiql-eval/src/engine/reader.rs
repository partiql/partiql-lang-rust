use crate::engine::error::{EngineError, Result};
use crate::engine::row::{RowFrame, SlotId, SlotValue};
use crate::engine::value::{value_ref_from_value, ValueRef};
use partiql_value::{BindingsName, Value};

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
    fn next_row<'a>(&'a mut self, out: &mut RowFrame<'a>) -> Result<bool>;
    fn resolve(&self, field_name: &str) -> Option<ScanSource>;
    fn close(&mut self) -> Result<()>;
}

pub struct ValueRowReader {
    rows: Vec<Value>,
    pos: usize,
    layout: ScanLayout,
    caps: ReaderCaps,
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

    fn next_row<'a>(&'a mut self, out: &mut RowFrame<'a>) -> Result<bool> {
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
                ScanSource::BaseRow => value_ref_from_value(row),
                ScanSource::FieldPath(path) => get_field_path(row, path),
                ScanSource::ColumnIndex(index) => get_column_index(row, *index),
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

fn get_field_path<'a>(row: &'a Value, path: &str) -> ValueRef<'a> {
    let mut current = row;
    for part in path.split('.') {
        let key = BindingsName::CaseInsensitive(part.into());
        match current {
            Value::Tuple(tuple) => {
                if let Some(value) = tuple.get(&key) {
                    current = value;
                } else {
                    return ValueRef::Missing;
                }
            }
            _ => return ValueRef::Missing,
        }
    }
    value_ref_from_value(current)
}

fn get_column_index<'a>(row: &'a Value, index: usize) -> ValueRef<'a> {
    match row {
        Value::Tuple(tuple) => tuple
            .values()
            .nth(index)
            .map(value_ref_from_value)
            .unwrap_or(ValueRef::Missing),
        _ => ValueRef::Missing,
    }
}
