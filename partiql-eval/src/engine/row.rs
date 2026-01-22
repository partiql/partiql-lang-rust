use crate::engine::value::{ValueOwned, ValueRef, ValueView};
use std::cell::UnsafeCell;

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

    pub fn as_ref(&self) -> ValueRef<'a> {
        match *self {
            SlotValue::Val(v) => v,
            SlotValue::Owned(v) => ValueRef::from_owned(v),
        }
    }
}

#[derive(Debug, Default)]
pub struct RowArena {
    values: UnsafeCell<Vec<Box<ValueOwned>>>,
}

impl RowArena {
    pub fn alloc(&self, value: ValueOwned) -> &ValueOwned {
        let values = unsafe { &mut *self.values.get() };
        values.push(Box::new(value));
        values
            .last()
            .map(|v| v.as_ref())
            .expect("row arena should have at least one value")
    }

    pub fn reset(&self) {
        let values = unsafe { &mut *self.values.get() };
        values.clear();
    }
}

pub struct RowFrame<'a> {
    pub slots: &'a mut [SlotValue<'a>],
    pub arena: &'a RowArena,
}

pub struct RowFrameScratch {
    slots: Vec<SlotValue<'static>>,
    arena: RowArena,
}

impl RowFrameScratch {
    pub fn new(slot_count: usize) -> Self {
        let mut slots = Vec::with_capacity(slot_count);
        for _ in 0..slot_count {
            slots.push(SlotValue::Val(ValueRef::Missing));
        }
        RowFrameScratch {
            slots,
            arena: RowArena::default(),
        }
    }

    pub fn reset(&mut self) {
        self.arena.reset();
        for slot in &mut self.slots {
            *slot = SlotValue::Val(ValueRef::Missing);
        }
    }

    pub fn frame(&mut self) -> RowFrame<'_> {
        let slots = unsafe {
            std::mem::transmute::<&mut [SlotValue<'static>], &mut [SlotValue<'_>]>(
                self.slots.as_mut_slice(),
            )
        };
        RowFrame {
            slots,
            arena: &self.arena,
        }
    }
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
