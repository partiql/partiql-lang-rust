use crate::engine::value::{ValueOwned, ValueRef, ValueView};

pub type SlotId = u16;

#[derive(Clone, Copy, Debug)]
pub enum SlotValue<'a> {
    Val(ValueRef<'a>),
    Owned(&'a ValueOwned),
}

impl<'a> SlotValue<'a> {
    pub fn as_view(&self) -> ValueView<'a> {
        match *self {
            SlotValue::Val(v) => v.into(),
            SlotValue::Owned(v) => ValueView::from_owned(v),
        }
    }
}

#[derive(Debug, Default)]
pub struct RowArena {
    values: Vec<ValueOwned>,
}

impl RowArena {
    pub fn alloc(&mut self, value: ValueOwned) -> &ValueOwned {
        self.values.push(value);
        self.values
            .last()
            .expect("row arena should have at least one value")
    }

    pub fn reset(&mut self) {
        self.values.clear();
    }
}

pub struct RowFrame<'a> {
    pub slots: &'a mut [SlotValue<'a>],
    pub arena: &'a mut RowArena,
}

pub struct RowView<'a> {
    slots: &'a [SlotValue<'a>],
}

impl<'a> RowView<'a> {
    pub(crate) fn new(slots: &'a [SlotValue<'a>]) -> Self {
        RowView { slots }
    }

    pub fn get(&self, col: usize) -> ValueView<'a> {
        self.slots
            .get(col)
            .map(SlotValue::as_view)
            .unwrap_or(ValueView::Missing)
    }

    pub fn get_i64(&self, col: usize) -> Option<i64> {
        self.get(col).as_i64()
    }

    pub fn get_str(&self, col: usize) -> Option<&'a str> {
        self.get(col).as_str()
    }

    pub fn get_value(&self, col: usize) -> ValueOwned {
        self.get(col).to_owned()
    }
}
