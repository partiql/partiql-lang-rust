use crate::engine::value::{TupleField, TupleRef, ValueRef};
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

    /// Allocate a tuple from name and value ValueRefs
    ///
    /// Takes pairs of (name ValueRef, value ValueRef) where:
    /// - Name ValueRef should be ValueRef::Str (already a valid pointer - zero copy!)
    /// - Value ValueRef can be any variant
    ///
    /// Only allocates the TupleField array and TupleRef wrapper.
    /// String pointers from ValueRef::Str are used directly without copying.
    pub fn alloc_tuple<'a>(
        &'a self,
        fields: impl IntoIterator<Item = (ValueRef<'a>, ValueRef<'a>)>,
    ) -> &'a TupleRef<'a> {
        let fields_vec: Vec<TupleField<'a>> = fields
            .into_iter()
            .map(|(name_ref, value)| {
                // Extract name - assume it's already a string pointer
                let name = match name_ref {
                    ValueRef::Str(s) => s,
                    _ => panic!("tuple field names must be ValueRef::Str"),
                };
                TupleField { name, value }
            })
            .collect();

        let fields_slice = self.alloc_slice(&fields_vec);
        let tuple_ref = TupleRef {
            fields: fields_slice,
        };
        self.alloc_tuple_ref(tuple_ref)
    }

    /// Allocate a slice in the arena
    pub(crate) fn alloc_slice<T: Copy>(&self, items: &[T]) -> &[T] {
        if items.is_empty() {
            return &[];
        }

        let size = std::mem::size_of_val(items);
        let align = std::mem::align_of::<T>();

        let offset = self.offset.get();
        let aligned_offset = (offset + align - 1) & !(align - 1);

        let buffer = unsafe { &mut *self.buffer.get() };
        let new_offset = aligned_offset + size;

        if new_offset > buffer.capacity() {
            let new_capacity = buffer
                .capacity()
                .max(size)
                .checked_mul(2)
                .expect("arena capacity overflow");
            buffer.reserve(new_capacity - buffer.capacity());
        }

        if new_offset > buffer.len() {
            buffer.resize(new_offset, 0);
        }

        unsafe {
            let ptr = buffer.as_mut_ptr().add(aligned_offset) as *mut T;
            std::ptr::copy_nonoverlapping(items.as_ptr(), ptr, items.len());
            self.offset.set(new_offset);
            std::slice::from_raw_parts(ptr, items.len())
        }
    }

    /// Allocate a TupleRef in the arena
    pub(crate) fn alloc_tuple_ref<'a>(&'a self, tuple_ref: TupleRef<'a>) -> &'a TupleRef<'a> {
        let size = std::mem::size_of::<TupleRef<'_>>();
        let align = std::mem::align_of::<TupleRef<'_>>();

        let offset = self.offset.get();
        let aligned_offset = (offset + align - 1) & !(align - 1);

        let buffer = unsafe { &mut *self.buffer.get() };
        let new_offset = aligned_offset + size;

        if new_offset > buffer.capacity() {
            let new_capacity = buffer
                .capacity()
                .max(size)
                .checked_mul(2)
                .expect("arena capacity overflow");
            buffer.reserve(new_capacity - buffer.capacity());
        }

        if new_offset > buffer.len() {
            buffer.resize(new_offset, 0);
        }

        unsafe {
            let ptr = buffer.as_mut_ptr().add(aligned_offset) as *mut TupleRef<'_>;
            std::ptr::write(ptr, tuple_ref);
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
