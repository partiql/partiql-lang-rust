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

/// Type-safe interface for readers to populate row data
///
/// ValueWriter encapsulates register array access, providing type-safe methods
/// for writing values to row slots. This abstraction hides internal `ValueRef`
/// representation and register array details from reader implementations.
///
/// # Design Goals
/// - **Type Safety**: Compile-time guarantees about value types via typed methods
/// - **Encapsulation**: Hides ValueRef internals from reader authors
/// - **Evolution**: Internal representation can change without breaking readers
/// - **Safety**: Bounds checking prevents out-of-bounds slot access
///
/// # Usage
/// Readers use ValueWriter methods to populate projected columns:
/// ```ignore
/// impl RowReader for MyReader {
///     fn next_row(&mut self, writer: &mut ValueWriter<'_>) -> Result<bool> {
///         writer.put_i64(0, 42)?;
///         writer.put_str(1, "hello")?;
///         Ok(true)
///     }
/// }
/// ```
pub struct ValueWriter<'w, 'a> {
    regs: &'w mut [ValueRef<'a>],
}

impl<'w, 'a> ValueWriter<'w, 'a> {
    /// Create a new ValueWriter wrapping the register array
    ///
    /// # Safety
    /// The register array must remain valid for the lifetime 'a.
    /// Internal use only - readers receive ValueWriter from the VM.
    pub(crate) fn new(regs: &'w mut [ValueRef<'a>]) -> Self {
        ValueWriter { regs }
    }

    /// Write an i64 value to the specified slot
    ///
    /// # Errors
    /// Returns error if slot index is out of bounds
    #[inline]
    pub fn put_i64(&mut self, slot: SlotId, value: i64) -> crate::engine::error::Result<()> {
        let idx = slot as usize;
        if idx >= self.regs.len() {
            return Err(crate::engine::error::EngineError::SlotOutOfBounds(slot));
        }
        self.regs[idx] = ValueRef::I64(value);
        Ok(())
    }

    /// Write an f64 value to the specified slot
    ///
    /// # Errors
    /// Returns error if slot index is out of bounds
    #[inline]
    pub fn put_f64(&mut self, slot: SlotId, value: f64) -> crate::engine::error::Result<()> {
        let idx = slot as usize;
        if idx >= self.regs.len() {
            return Err(crate::engine::error::EngineError::SlotOutOfBounds(slot));
        }
        self.regs[idx] = ValueRef::F64(value);
        Ok(())
    }

    /// Write a bool value to the specified slot
    ///
    /// # Errors
    /// Returns error if slot index is out of bounds
    #[inline]
    pub fn put_bool(&mut self, slot: SlotId, value: bool) -> crate::engine::error::Result<()> {
        let idx = slot as usize;
        if idx >= self.regs.len() {
            return Err(crate::engine::error::EngineError::SlotOutOfBounds(slot));
        }
        self.regs[idx] = ValueRef::Bool(value);
        Ok(())
    }

    /// Write a string reference to the specified slot
    ///
    /// The string reference must remain valid according to the reader's
    /// BufferStability contract (UntilNext or UntilClose).
    ///
    /// # Errors
    /// Returns error if slot index is out of bounds
    #[inline]
    pub fn put_str(&mut self, slot: SlotId, value: &'a str) -> crate::engine::error::Result<()> {
        let idx = slot as usize;
        if idx >= self.regs.len() {
            return Err(crate::engine::error::EngineError::SlotOutOfBounds(slot));
        }
        self.regs[idx] = ValueRef::Str(value);
        Ok(())
    }

    /// Write a byte slice reference to the specified slot
    ///
    /// The byte slice must remain valid according to the reader's
    /// BufferStability contract (UntilNext or UntilClose).
    ///
    /// # Errors
    /// Returns error if slot index is out of bounds
    #[inline]
    pub fn put_bytes(&mut self, slot: SlotId, value: &'a [u8]) -> crate::engine::error::Result<()> {
        let idx = slot as usize;
        if idx >= self.regs.len() {
            return Err(crate::engine::error::EngineError::SlotOutOfBounds(slot));
        }
        self.regs[idx] = ValueRef::Bytes(value);
        Ok(())
    }

    /// Write NULL to the specified slot
    ///
    /// # Errors
    /// Returns error if slot index is out of bounds
    #[inline]
    pub fn put_null(&mut self, slot: SlotId) -> crate::engine::error::Result<()> {
        let idx = slot as usize;
        if idx >= self.regs.len() {
            return Err(crate::engine::error::EngineError::SlotOutOfBounds(slot));
        }
        self.regs[idx] = ValueRef::Null;
        Ok(())
    }

    /// Write MISSING to the specified slot
    ///
    /// # Errors
    /// Returns error if slot index is out of bounds
    #[inline]
    pub fn put_missing(&mut self, slot: SlotId) -> crate::engine::error::Result<()> {
        let idx = slot as usize;
        if idx >= self.regs.len() {
            return Err(crate::engine::error::EngineError::SlotOutOfBounds(slot));
        }
        self.regs[idx] = ValueRef::Missing;
        Ok(())
    }

    /// Write an owned value reference to the specified slot
    ///
    /// Used for complex values (tuples, lists, bags) that require
    /// materialization to ValueOwned.
    ///
    /// # Errors
    /// Returns error if slot index is out of bounds
    #[inline]
    pub fn put_owned(
        &mut self,
        slot: SlotId,
        value: &'a ValueOwned,
    ) -> crate::engine::error::Result<()> {
        let idx = slot as usize;
        if idx >= self.regs.len() {
            return Err(crate::engine::error::EngineError::SlotOutOfBounds(slot));
        }
        self.regs[idx] = ValueRef::Owned(value);
        Ok(())
    }

    /// Get the number of available slots
    ///
    /// Useful for readers that need to validate their projection layout.
    #[inline]
    pub fn slot_count(&self) -> usize {
        self.regs.len()
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
