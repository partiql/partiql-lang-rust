use crate::engine::value::{ValueOwned, ValueRef, ValueView};
use std::cell::{Cell, UnsafeCell};

pub type SlotId = u16;

/// Bump allocator for temporary query values
///
/// Provides O(1) allocation and O(1) bulk deallocation via `reset()`.
/// Values allocated in an arena remain valid until the arena is reset.
///
/// # Usage Patterns
/// - **Per-row**: VM resets arena between output rows
/// - **Per-phase**: Blocking operators maintain separate arenas for build phases
/// - **Per-query**: Could be used for query-scoped allocations
///
/// The arena uses a simple bump pointer strategy: allocations increment an offset
/// into a contiguous buffer, and `reset()` returns the offset to zero without
/// touching individual values. This provides excellent cache locality and minimal
/// per-allocation overhead.
#[derive(Debug)]
pub struct Arena {
    // Contiguous buffer for all allocations
    buffer: UnsafeCell<Vec<u8>>,
    // Current allocation offset into buffer
    offset: Cell<usize>,
}

impl Default for Arena {
    fn default() -> Self {
        Self::new(8192) // 8KB default capacity
    }
}

impl Arena {
    pub fn new(capacity: usize) -> Self {
        Arena {
            buffer: UnsafeCell::new(Vec::with_capacity(capacity)),
            offset: Cell::new(0),
        }
    }

    /// Allocate a value in the arena and return a reference to it
    ///
    /// All allocations are sequential in memory for perfect cache locality.
    /// The returned reference is valid until the next reset() call.
    pub fn alloc(&self, value: ValueOwned) -> &ValueOwned {
        let size = std::mem::size_of::<ValueOwned>();
        let align = std::mem::align_of::<ValueOwned>();

        // Align the current offset
        let offset = self.offset.get();
        let aligned_offset = (offset + align - 1) & !(align - 1);

        let buffer = unsafe { &mut *self.buffer.get() };

        // Calculate new offset after this allocation
        let new_offset = aligned_offset + size;

        // Ensure we have enough capacity
        if new_offset > buffer.capacity() {
            // Double the capacity when we run out
            let new_capacity = buffer
                .capacity()
                .max(size)
                .checked_mul(2)
                .expect("arena capacity overflow");
            buffer.reserve(new_capacity - buffer.capacity());
        }

        // Extend buffer length if needed
        if new_offset > buffer.len() {
            buffer.resize(new_offset, 0);
        }

        // Write the value at the aligned offset
        unsafe {
            let ptr = buffer.as_mut_ptr().add(aligned_offset) as *mut ValueOwned;
            std::ptr::write(ptr, value);
            self.offset.set(new_offset);
            &*ptr
        }
    }

    /// Reset the arena for reuse
    ///
    /// This is O(1) - just resets the offset pointer. The buffer memory
    /// is retained for reuse, avoiding deallocation/reallocation overhead.
    pub fn reset(&self) {
        self.offset.set(0);
        // Note: We don't clear the buffer contents - they'll be overwritten
        // on the next allocation. This is safe because we only hand out
        // references to properly initialized values.
    }
}

pub struct RowView<'a> {
    slots: &'a [ValueRef<'a>],
}

impl<'a> RowView<'a> {
    pub(crate) fn new(slots: &'a [ValueRef<'a>]) -> Self {
        RowView { slots }
    }

    pub fn get(&self, col: usize) -> ValueView<'a> {
        self.slots
            .get(col)
            .map(|v| ValueView::from(*v))
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
