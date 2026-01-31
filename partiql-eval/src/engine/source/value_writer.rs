use crate::engine::error::Result;
use crate::engine::row::SlotId;
use crate::engine::value::{ValueOwned, ValueRef};

/// Type-safe interface for readers to populate row data
///
/// RegisterWriter encapsulates register array access, providing type-safe methods
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
/// Readers use RegisterWriter methods to populate projected columns:
/// ```ignore
/// impl DataSource for MyReader {
///     fn next_row(&mut self, writer: &mut RegisterWriter<'_>) -> Result<bool> {
///         writer.put_i64(0, 42)?;
///         writer.put_str(1, "hello")?;
///         Ok(true)
///     }
/// }
/// ```
pub struct RegisterWriter<'w, 'a> {
    pub(crate) regs: &'w mut [ValueRef<'a>],
}

impl<'w, 'a> RegisterWriter<'w, 'a> {
    /// Create a new RegisterWriter wrapping the register array
    ///
    /// # Safety
    /// The register array must remain valid for the lifetime 'a.
    /// Internal use only - readers receive RegisterWriter from the VM.
    pub(crate) fn new(regs: &'w mut [ValueRef<'a>]) -> Self {
        RegisterWriter { regs }
    }

    /// Write an i64 value to the specified slot
    ///
    /// # Errors
    /// Returns error if slot index is out of bounds
    #[inline]
    pub fn put_i64(&mut self, slot: SlotId, value: i64) -> Result<()> {
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
    pub fn put_f64(&mut self, slot: SlotId, value: f64) -> Result<()> {
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
    pub fn put_bool(&mut self, slot: SlotId, value: bool) -> Result<()> {
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
    pub fn put_str(&mut self, slot: SlotId, value: &'a str) -> Result<()> {
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
    pub fn put_bytes(&mut self, slot: SlotId, value: &'a [u8]) -> Result<()> {
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
    pub fn put_null(&mut self, slot: SlotId) -> Result<()> {
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
    pub fn put_missing(&mut self, slot: SlotId) -> Result<()> {
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
    pub fn put_owned(&mut self, slot: SlotId, value: &'a ValueOwned) -> Result<()> {
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
