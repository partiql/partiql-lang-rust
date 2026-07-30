use crate::engine::value::{TupleField, TupleRef, ValueRef};
use std::cell::{Cell, UnsafeCell};

pub type SlotId = u16;

/// Bump allocator for temporary query values.
///
/// Values allocated in an arena remain valid until the arena is reset.
///
/// # Usage Patterns
/// - **Per-row**: VM resets arena between output rows
/// - **Per-phase**: Blocking operators maintain separate arenas for build phases
/// - **Per-query**: Could be used for query-scoped allocations
///
/// Allocations bump within the current chunk; when it fills, another chunk is
/// appended. Chunks are never resized, so a pointer handed out from one stays
/// valid until `reset`. The row bank holds the ingested row alongside the
/// per-field temporaries computed from it, so growing the bank for a temporary
/// must not move the row still being read.
#[derive(Debug)]
pub struct Arena {
    chunks: UnsafeCell<Vec<Vec<u8>>>,
    current: Cell<usize>,
    offset: Cell<usize>,
}

impl Default for Arena {
    fn default() -> Self {
        Self::new(8192)
    }
}

impl Arena {
    pub fn new(capacity: usize) -> Self {
        Arena {
            chunks: UnsafeCell::new(vec![vec![0u8; capacity.max(1)]]),
            current: Cell::new(0),
            offset: Cell::new(0),
        }
    }

    /// Reserve `size` bytes aligned to `align` and return a pointer to them.
    ///
    /// Walks retained chunks before appending a new one, so a reset arena reuses
    /// its existing storage.
    fn bump(&self, size: usize, align: usize) -> *mut u8 {
        let chunks = unsafe { &mut *self.chunks.get() };

        // Align on the chunk's real address, not its start offset: `Vec<u8>` is
        // only 1-byte aligned, so a chunk-relative offset that looks aligned can
        // still yield a misaligned pointer.
        let aligned_offset = |chunk: &[u8], from: usize| {
            from + unsafe { chunk.as_ptr().add(from) }.align_offset(align)
        };

        let mut idx = self.current.get();
        let mut from = self.offset.get();
        while idx < chunks.len() {
            let start = aligned_offset(&chunks[idx], from);
            if start
                .checked_add(size)
                .is_some_and(|end| end <= chunks[idx].len())
            {
                self.current.set(idx);
                self.offset.set(start + size);
                return unsafe { chunks[idx].as_mut_ptr().add(start) };
            }
            idx += 1;
            from = 0;
        }

        let needed = size
            .checked_add(align)
            .expect("arena allocation size overflows usize");
        let next = chunks
            .last()
            .map_or(needed, |last| last.len().max(needed))
            .saturating_mul(2);
        chunks.push(vec![0u8; next]);
        let idx = chunks.len() - 1;
        let start = aligned_offset(&chunks[idx], 0);
        debug_assert!(
            start + size <= chunks[idx].len(),
            "fresh chunk must fit its request after alignment padding"
        );
        self.current.set(idx);
        self.offset.set(start + size);
        unsafe { chunks[idx].as_mut_ptr().add(start) }
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

        let ptr = self.bump(std::mem::size_of_val(items), std::mem::align_of::<T>()) as *mut T;
        unsafe {
            std::ptr::copy_nonoverlapping(items.as_ptr(), ptr, items.len());
            std::slice::from_raw_parts(ptr, items.len())
        }
    }

    /// Allocate a TupleRef in the arena
    pub(crate) fn alloc_tuple_ref<'a>(&'a self, tuple_ref: TupleRef<'a>) -> &'a TupleRef<'a> {
        let ptr = self.bump(
            std::mem::size_of::<TupleRef<'_>>(),
            std::mem::align_of::<TupleRef<'_>>(),
        ) as *mut TupleRef<'_>;
        unsafe {
            std::ptr::write(ptr, tuple_ref);
            &*ptr
        }
    }

    /// Reset the arena for reuse
    ///
    /// Rewinds the allocation cursor; chunks are retained. Callers must drop
    /// every reference handed out by this arena before calling `reset` — the
    /// next allocation overwrites those bytes.
    pub fn reset(&self) {
        self.current.set(0);
        self.offset.set(0);
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    /// Chunks currently held. Asserts a test actually reaches the multi-chunk
    /// path, where the reallocation bug lived.
    fn chunk_count(arena: &Arena) -> usize {
        unsafe { (*arena.chunks.get()).len() }
    }

    /// Every allocation must still read back correctly after later ones outgrow
    /// the chunk they landed in.
    #[test]
    fn allocations_stay_valid_across_growth() {
        // (initial capacity, allocation count, u64s per allocation). Each case
        // allocates at least twice so a request larger than the first chunk
        // still crosses a boundary rather than just sizing the first chunk to fit.
        let cases: &[(usize, u64, usize)] =
            &[(64, 2, 512), (32, 200, 4), (16, 2, 1250), (1, 50, 1)];
        for &(capacity, count, width) in cases {
            let arena = Arena::new(capacity);
            let mut handles = Vec::new();
            for i in 0..count {
                let data: Vec<u64> = (0..width).map(|j| i * 10_000 + j as u64).collect();
                handles.push((i, arena.alloc_slice(&data)));
            }
            assert!(
                chunk_count(&arena) > 1,
                "cap {capacity}: never grew past one chunk, so it proves nothing"
            );
            for (i, h) in handles {
                let want: Vec<u64> = (0..width).map(|j| i * 10_000 + j as u64).collect();
                assert_eq!(
                    h,
                    &want[..],
                    "cap {capacity}: allocation {i} was invalidated by a later one"
                );
            }
        }
    }

    /// Alignment comes entirely from the bump arithmetic, since `Vec<u8>` gives
    /// no guarantee. The odd-sized filler forces each `u128` to start unaligned.
    #[test]
    fn allocations_are_aligned_across_chunks() {
        let arena = Arena::new(1);
        for i in 0..64u128 {
            let _pad: &[u8] = arena.alloc_slice(&[0xABu8; 3]);
            let s: &[u128] = arena.alloc_slice(&[i, i + 1]);
            assert_eq!(
                s.as_ptr().align_offset(std::mem::align_of::<u128>()),
                0,
                "u128 slice was misaligned"
            );
            assert_eq!(s, &[i, i + 1]);
        }
    }

    /// `reset` must retain chunks, and repeated cycles must reuse them.
    #[test]
    fn reset_retains_chunks_and_reuses_them() {
        let arena = Arena::new(16);
        for i in 0..50u64 {
            let _ = arena.alloc_slice(&[i; 8]);
        }
        let grown = chunk_count(&arena);
        assert!(grown > 1, "expected multiple chunks, got {grown}");

        arena.reset();
        assert_eq!(
            chunk_count(&arena),
            grown,
            "reset must retain every chunk, not free them"
        );

        for round in 0..50u64 {
            let mut held = Vec::new();
            for i in 0..40u64 {
                held.push(arena.alloc_slice(&[round * 1000 + i; 8]));
            }
            for (i, h) in held.iter().enumerate() {
                let want = round * 1000 + i as u64;
                assert_eq!(*h, &[want; 8], "round {round} alloc {i} corrupted");
            }
            arena.reset();
        }
        assert_eq!(
            chunk_count(&arena),
            grown,
            "repeated reset cycles should reuse chunks, not accumulate them"
        );
    }

    /// The `TupleRef` wrapper is forced onto a new chunk while the field slice
    /// is still live — the shape that would move the fields on a single buffer.
    #[test]
    fn alloc_tuple_fields_survive_a_chunk_boundary() {
        // Fields fit with alignment slack; TupleRef (needs 16 more bytes) does not.
        let fields_bytes = 8 * std::mem::size_of::<TupleField<'_>>();
        let capacity = fields_bytes + std::mem::align_of::<TupleField<'_>>();

        let arena = Arena::new(capacity);
        let chunks_before = chunk_count(&arena);

        let names: Vec<String> = (0..8).map(|i| format!("field{i}")).collect();
        let fields: Vec<(ValueRef<'_>, ValueRef<'_>)> = names
            .iter()
            .enumerate()
            .map(|(i, n)| (ValueRef::Str(n.as_str()), ValueRef::I64(i as i64)))
            .collect();
        let tuple = arena.alloc_tuple(fields);

        assert!(
            chunk_count(&arena) > chunks_before,
            "expected the TupleRef wrapper to push a new chunk, got {} chunks",
            chunk_count(&arena),
        );
        assert_eq!(tuple.fields.len(), 8);
        for (i, f) in tuple.fields.iter().enumerate() {
            assert_eq!(f.name, format!("field{i}"), "name {i}");
            match f.value {
                ValueRef::I64(v) => assert_eq!(v, i as i64, "value {i}"),
                ref other => panic!("field {i} wrong variant: {other:?}"),
            }
        }
    }
}
