use crate::engine::arena::{Arena, SlotId};
use crate::engine::error::{EngineError, Result};
use crate::engine::value::{TupleField, TupleRef, ValueRef};

/// Type-safe interface for readers to populate row data
///
/// RegisterWriter provides two modes of operation:
///
/// ## Register Writes (direct, slot-targeted)
/// For writing scalar values directly into register slots:
/// ```ignore
/// writer.write_i64(0, 42)?;
/// writer.write_str(1, "hello")?;
/// ```
///
/// ## Complex Value Construction (via ValueWriter)
/// For building structured values, borrow a ValueWriter:
/// ```ignore
/// let mut vw = writer.value_writer(0)?;
/// vw.step_in_tuple()?;
/// vw.put_field_name("name")?;
/// vw.put_str("Alice")?;
/// vw.put_field_name("scores")?;
/// vw.step_in_list()?;
/// vw.put_i64(95)?;
/// vw.put_i64(87)?;
/// vw.step_out()?;
/// vw.step_out()?;
/// vw.finish()?;
/// ```
///
/// The ValueWriter mirrors `ValueView`'s read cursor symmetrically.
/// It borrows RegisterWriter exclusively — you cannot interleave
/// register writes with value construction. This is enforced at compile time.
pub struct RegisterWriter<'w, 'a> {
    pub(crate) regs: &'w mut [ValueRef<'a>],
    pub(crate) arena: &'a Arena,
}

impl<'w, 'a> RegisterWriter<'w, 'a> {
    /// Create a new RegisterWriter wrapping the register array and arena
    ///
    /// Internal use only — readers receive RegisterWriter from the VM.
    pub(crate) fn new(regs: &'w mut [ValueRef<'a>], arena: &'a Arena) -> Self {
        RegisterWriter { regs, arena }
    }

    // =========================================================================
    // Register writes — direct, slot-targeted
    // =========================================================================

    /// Write an i64 value to the specified slot
    #[inline]
    pub fn write_i64(&mut self, slot: SlotId, value: i64) -> Result<()> {
        self.write_to_slot(slot, ValueRef::I64(value))
    }

    /// Write an f64 value to the specified slot
    #[inline]
    pub fn write_f64(&mut self, slot: SlotId, value: f64) -> Result<()> {
        self.write_to_slot(slot, ValueRef::F64(value))
    }

    /// Write a bool value to the specified slot
    #[inline]
    pub fn write_bool(&mut self, slot: SlotId, value: bool) -> Result<()> {
        self.write_to_slot(slot, ValueRef::Bool(value))
    }

    /// Write a string reference to the specified slot
    #[inline]
    pub fn write_str(&mut self, slot: SlotId, value: &'a str) -> Result<()> {
        self.write_to_slot(slot, ValueRef::Str(value))
    }

    /// Write a byte slice reference to the specified slot
    #[inline]
    pub fn write_bytes(&mut self, slot: SlotId, value: &'a [u8]) -> Result<()> {
        self.write_to_slot(slot, ValueRef::Bytes(value))
    }

    /// Write NULL to the specified slot
    #[inline]
    pub fn write_null(&mut self, slot: SlotId) -> Result<()> {
        self.write_to_slot(slot, ValueRef::Null)
    }

    /// Write MISSING to the specified slot
    #[inline]
    pub fn write_missing(&mut self, slot: SlotId) -> Result<()> {
        self.write_to_slot(slot, ValueRef::Missing)
    }

    /// Write a decimal value to the specified slot
    #[inline]
    pub fn write_decimal(&mut self, slot: SlotId, value: rust_decimal::Decimal) -> Result<()> {
        self.write_to_slot(slot, ValueRef::Decimal(value))
    }

    /// Internal helper for slot-targeted writes with bounds checking
    #[inline]
    fn write_to_slot(&mut self, slot: SlotId, value: ValueRef<'a>) -> Result<()> {
        let idx = slot as usize;
        if idx >= self.regs.len() {
            return Err(EngineError::SlotOutOfBounds(slot));
        }
        self.regs[idx] = value;
        Ok(())
    }

    // =========================================================================
    // Complex value construction — via ValueWriter
    // =========================================================================

    /// Create a ValueWriter for constructing a complex value in the given slot
    ///
    /// The returned ValueWriter borrows this RegisterWriter exclusively.
    /// Use `step_in_tuple()`, `step_in_list()`, or `step_in_bag()` to begin,
    /// add content, then `step_out()` and `finish()` to commit.
    #[inline]
    pub fn value_writer(&mut self, slot: SlotId) -> Result<ValueWriter<'_, 'a>> {
        let idx = slot as usize;
        if idx >= self.regs.len() {
            return Err(EngineError::SlotOutOfBounds(slot));
        }
        Ok(ValueWriter {
            regs: self.regs,
            arena: self.arena,
            target_slot: slot,
            stack: Vec::new(),
            result: None,
        })
    }

    /// Get the number of available register slots
    #[inline]
    pub fn slot_count(&self) -> usize {
        self.regs.len()
    }
}

// =============================================================================
// ValueWriter — unified cursor-based writer for complex values
// =============================================================================

/// Stack frame for tracking nested container construction
enum ContainerFrame<'a> {
    /// Building a tuple — accumulates named fields
    Tuple {
        pending_name: Option<&'a str>,
        fields: Vec<TupleField<'a>>,
    },
    /// Building a list — accumulates ordered elements
    List { items: Vec<ValueRef<'a>> },
    /// Building a bag — accumulates unordered elements
    Bag { items: Vec<ValueRef<'a>> },
}

/// Unified cursor-based writer for constructing complex PartiQL values
///
/// Mirrors `ValueView`'s read cursor symmetrically:
/// - `step_in_tuple()` / `step_in_list()` / `step_in_bag()` to open containers
/// - `put_field_name()` + `put_*()` for tuple fields
/// - `put_*()` directly for list/bag elements
/// - `step_out()` to close and commit containers to their parent
/// - `finish()` to commit the final value to the register
///
/// Supports arbitrary nesting depth via an internal stack.
///
/// # Example: Tuple with nested list
/// ```ignore
/// let mut vw = writer.value_writer(slot)?;
/// vw.step_in_tuple()?;
/// vw.put_field_name("name")?;
/// vw.put_str("Alice")?;
/// vw.put_field_name("scores")?;
/// vw.step_in_list()?;
/// vw.put_i64(95)?;
/// vw.put_i64(87)?;
/// vw.step_out()?;     // closes list → commits as "scores" field value
/// vw.step_out()?;     // closes tuple → stored as result
/// vw.finish()?;        // commits result to register
/// ```
pub struct ValueWriter<'w, 'a> {
    regs: &'w mut [ValueRef<'a>],
    arena: &'a Arena,
    target_slot: SlotId,
    stack: Vec<ContainerFrame<'a>>,
    result: Option<ValueRef<'a>>,
}

impl<'w, 'a> ValueWriter<'w, 'a> {
    // =========================================================================
    // Container navigation
    // =========================================================================

    /// Step into a new tuple container
    ///
    /// If inside a tuple, `put_field_name` must have been called first.
    /// If inside a list/bag, the tuple becomes the next element.
    /// If at top level, this starts the root value.
    pub fn step_in_tuple(&mut self) -> Result<()> {
        self.prepare_for_container()?;
        self.stack.push(ContainerFrame::Tuple {
            pending_name: None,
            fields: Vec::new(),
        });
        Ok(())
    }

    /// Step into a new list container
    pub fn step_in_list(&mut self) -> Result<()> {
        self.prepare_for_container()?;
        self.stack.push(ContainerFrame::List { items: Vec::new() });
        Ok(())
    }

    /// Step into a new bag container
    pub fn step_in_bag(&mut self) -> Result<()> {
        self.prepare_for_container()?;
        self.stack.push(ContainerFrame::Bag { items: Vec::new() });
        Ok(())
    }

    /// Step out of the current container
    ///
    /// Finalizes the current container (arena-allocates its contents),
    /// then commits the resulting value to the parent context:
    /// - If parent is a tuple: commits as the current field's value
    /// - If parent is a list/bag: commits as the next element
    /// - If no parent (root level): stores as the result for `finish()`
    pub fn step_out(&mut self) -> Result<()> {
        let frame = self.stack.pop().ok_or_else(|| {
            EngineError::IllegalState("step_out called with empty stack".to_string())
        })?;

        let value = self.finalize_frame(frame)?;

        // Commit to parent or store as result
        if let Some(parent) = self.stack.last_mut() {
            Self::commit_to_frame(parent, value)?;
        } else {
            // Root level — store as the final result
            self.result = Some(value);
        }
        Ok(())
    }

    // =========================================================================
    // Scalar puts — context-dependent
    // =========================================================================

    /// Set the field name for the next value in a tuple
    ///
    /// Must be called before each `put_*` or `step_in_*` inside a tuple.
    pub fn put_field_name(&mut self, name: &'a str) -> Result<()> {
        match self.stack.last_mut() {
            Some(ContainerFrame::Tuple { pending_name, .. }) => {
                if pending_name.is_some() {
                    return Err(EngineError::IllegalState(
                        "put_field_name called twice without value between".to_string(),
                    ));
                }
                *pending_name = Some(name);
                Ok(())
            }
            Some(_) => Err(EngineError::IllegalState(
                "put_field_name called inside list/bag (not a tuple)".to_string(),
            )),
            None => Err(EngineError::IllegalState(
                "put_field_name called at top level (call step_in_tuple first)".to_string(),
            )),
        }
    }

    /// Put an i64 value
    #[inline]
    pub fn put_i64(&mut self, value: i64) -> Result<()> {
        self.put_scalar(ValueRef::I64(value))
    }

    /// Put an f64 value
    #[inline]
    pub fn put_f64(&mut self, value: f64) -> Result<()> {
        self.put_scalar(ValueRef::F64(value))
    }

    /// Put a bool value
    #[inline]
    pub fn put_bool(&mut self, value: bool) -> Result<()> {
        self.put_scalar(ValueRef::Bool(value))
    }

    /// Put a string value
    #[inline]
    pub fn put_str(&mut self, value: &'a str) -> Result<()> {
        self.put_scalar(ValueRef::Str(value))
    }

    /// Put a byte slice
    #[inline]
    pub fn put_bytes(&mut self, value: &'a [u8]) -> Result<()> {
        self.put_scalar(ValueRef::Bytes(value))
    }

    /// Put NULL
    #[inline]
    pub fn put_null(&mut self) -> Result<()> {
        self.put_scalar(ValueRef::Null)
    }

    /// Put MISSING
    #[inline]
    pub fn put_missing(&mut self) -> Result<()> {
        self.put_scalar(ValueRef::Missing)
    }

    /// Put a decimal value
    #[inline]
    pub fn put_decimal(&mut self, value: rust_decimal::Decimal) -> Result<()> {
        self.put_scalar(ValueRef::Decimal(value))
    }

    // =========================================================================
    // Finalization
    // =========================================================================

    /// Commit the constructed value to the target register slot
    ///
    /// The stack must be empty (all containers closed via `step_out`).
    /// Consumes self — the compiler enforces that you finalize.
    pub fn finish(self) -> Result<()> {
        if !self.stack.is_empty() {
            return Err(EngineError::IllegalState(format!(
                "finish called with {} unclosed container(s) — call step_out first",
                self.stack.len()
            )));
        }
        let value = self.result.ok_or_else(|| {
            EngineError::IllegalState(
                "finish called with no value built (did you step_in and step_out?)".to_string(),
            )
        })?;
        self.regs[self.target_slot as usize] = value;
        Ok(())
    }

    // =========================================================================
    // Internal helpers
    // =========================================================================

    /// Validate and prepare for opening a new container
    ///
    /// Inside a tuple: ensures a field name has been set
    /// Inside a list/bag or at top level: no preparation needed
    fn prepare_for_container(&self) -> Result<()> {
        if let Some(ContainerFrame::Tuple { pending_name, .. }) = self.stack.last() {
            if pending_name.is_none() {
                return Err(EngineError::IllegalState(
                    "step_in_* inside tuple requires put_field_name first".to_string(),
                ));
            }
        }
        if self.result.is_some() {
            return Err(EngineError::IllegalState(
                "value already built — call finish() or don't step_in again".to_string(),
            ));
        }
        Ok(())
    }

    /// Put a scalar value into the current container context
    fn put_scalar(&mut self, value: ValueRef<'a>) -> Result<()> {
        match self.stack.last_mut() {
            Some(frame) => Self::commit_to_frame(frame, value),
            None => Err(EngineError::IllegalState(
                "put_* called at top level (call step_in_* first)".to_string(),
            )),
        }
    }

    /// Commit a value into a container frame
    fn commit_to_frame(frame: &mut ContainerFrame<'a>, value: ValueRef<'a>) -> Result<()> {
        match frame {
            ContainerFrame::Tuple {
                pending_name,
                fields,
            } => {
                let name = pending_name.take().ok_or_else(|| {
                    EngineError::IllegalState(
                        "value added to tuple without preceding put_field_name".to_string(),
                    )
                })?;
                fields.push(TupleField { name, value });
                Ok(())
            }
            ContainerFrame::List { items } | ContainerFrame::Bag { items } => {
                items.push(value);
                Ok(())
            }
        }
    }

    /// Arena-allocate a container frame and return the resulting ValueRef
    fn finalize_frame(&self, frame: ContainerFrame<'a>) -> Result<ValueRef<'a>> {
        match frame {
            ContainerFrame::Tuple {
                pending_name,
                fields,
            } => {
                if pending_name.is_some() {
                    return Err(EngineError::IllegalState(
                        "step_out from tuple with pending field name (no value)".to_string(),
                    ));
                }
                let fields_slice = self.arena.alloc_slice(&fields);
                let tuple_ref = self.arena.alloc_tuple_ref(TupleRef {
                    fields: fields_slice,
                });
                Ok(ValueRef::Tuple(tuple_ref))
            }
            ContainerFrame::List { items } => {
                let items_slice = self.arena.alloc_slice(&items);
                Ok(ValueRef::List(items_slice))
            }
            ContainerFrame::Bag { items } => {
                let items_slice = self.arena.alloc_slice(&items);
                Ok(ValueRef::Bag(items_slice))
            }
        }
    }
}
