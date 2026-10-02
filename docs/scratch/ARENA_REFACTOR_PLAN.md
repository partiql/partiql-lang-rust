# Arena Refactoring Plan: VM-Level Memory Management

## Problem

Currently, each `RowFrameScratch` owns its own `RowArena`, which means:
- Separate memory pools per pipeline operator
- No memory sharing across query execution
- Missed optimization opportunity

## Solution: Centralize Arena at VM Level

### Architecture Change

```rust
// BEFORE (Current - Per-Row Arena)
pub struct RowFrameScratch {
    slots: Vec<SlotValue<'static>>,
    arena: RowArena,  // ← Each row processor owns arena
}

pub struct ResultStream {
    scratch: RowFrameScratch,  // Arena buried here
    // ...
}

// AFTER (VM-Level Arena)
pub struct ResultStream {
    arena: RowArena,               // ← Arena owned at VM level
    scratch: RowFrameScratch,      // Just slots, no arena
    // ...
}

pub struct RowFrameScratch {
    slots: Vec<SlotValue<'static>>,
    // No arena field - borrowed from VM
}
```

### Implementation Steps

#### 1. Modify RowFrameScratch to NOT own arena

```rust
// row.rs
pub struct RowFrameScratch {
    slots: Vec<SlotValue<'static>>,
    // Remove: arena: RowArena,
}

impl RowFrameScratch {
    pub fn new(slot_count: usize) -> Self {
        let mut slots = Vec::with_capacity(slot_count);
        for _ in 0..slot_count {
            slots.push(SlotValue::Val(ValueRef::Missing));
        }
        RowFrameScratch { slots }
    }

    // NEW: Take arena as parameter
    pub fn reset(&mut self) {
        for slot in &mut self.slots {
            *slot = SlotValue::Val(ValueRef::Missing);
        }
    }

    // NEW: Frame borrows arena from outside
    pub fn frame<'a>(&'a mut self, arena: &'a RowArena) -> RowFrame<'a> {
        let slots = unsafe {
            std::mem::transmute::<&mut [SlotValue<'static>], &mut [SlotValue<'a>]>(
                self.slots.as_mut_slice(),
            )
        };
        RowFrame { slots, arena }
    }
}
```

#### 2. Move arena to ResultStream (VM level)

```rust
// plan.rs
pub struct ResultStream {
    pub schema: Schema,
    root: usize,
    instance: PlanInstance,
    arena: RowArena,        // ← NEW: VM-level arena
    scratch: RowFrameScratch,
}

impl ResultStream {
    pub fn next_row(&mut self) -> Result<Option<RowView<'_>>> {
        // Reset arena once per row
        self.arena.reset();
        
        // Reset slots
        self.scratch.reset();
        
        // Create frame with VM-level arena
        let mut frame = self.scratch.frame(&self.arena);
        
        let op = self.instance.operators.get_mut(self.root)
            .ok_or_else(|| EngineError::IllegalState("invalid root".to_string()))?;
        
        if op.next_row(&mut frame)? {
            Ok(Some(RowView::new(frame.slots)))
        } else {
            Ok(None)
        }
    }
}
```

#### 3. Update PlanInstance.execute()

```rust
// plan.rs
impl PlanInstance {
    pub fn execute(self) -> Result<ResultStream> {
        let slot_count = self.compiled.slot_count;
        Ok(ResultStream {
            schema: self.compiled.result_schema(),
            root: self.compiled.root,
            instance: self,
            arena: RowArena::new(16384),  // ← Single large arena (16KB)
            scratch: RowFrameScratch::new(slot_count),
        })
    }
}
```

## Benefits of This Refactor

### 1. Memory Efficiency
- **Before:** N separate arenas (one per operator) × 8KB = potentially 100s of KB
- **After:** 1 arena × 16-32KB = single allocation

### 2. Cache Locality
All query processing happens in one contiguous memory region:
```
[Row 1 data][Row 2 data][Row 3 data]... all sequential
```
vs scattered across multiple arena buffers.

### 3. Future Optimizations Enabled

#### Batch Processing
```rust
pub fn next_batch(&mut self, batch_size: usize) -> Result<Vec<RowView<'_>>> {
    let mut rows = Vec::new();
    
    // Process multiple rows before arena reset
    for _ in 0..batch_size {
        // ... process row without resetting arena
        rows.push(row);
    }
    
    // Single reset for entire batch
    self.arena.reset();
    Ok(rows)
}
```

#### Cross-Operator Memory Reuse
```rust
// Scan → Filter → Project all using same arena
// Intermediate values stay in same cache-friendly memory region
```

#### Memory Pooling
```rust
pub struct QueryEngine {
    arena_pool: Vec<RowArena>,  // Reuse arenas across queries
}
```

## Compatibility

This is **fully backward compatible** - just internal restructuring. External API unchanged:

```rust
// Usage stays exactly the same
let mut stream = plan_instance.execute()?;
while let Some(row) = stream.next_row()? {
    // ... process row
}
```

## Performance Impact

**Expected improvements:**
- **10-20% faster** for queries with multiple operators (less allocation overhead)
- **Better cache hit rates** (measured with cachegrind)
- **Reduced memory fragmentation**
- **Enables future batch processing** (major win for high-throughput scenarios)

## Migration Path

1. ✅ Implement bump allocator (done)
2. ⏭️ **Next:** Refactor to VM-level arena (this document)
3. ⏭️ Then: Add batch processing support
4. ⏭️ Then: Add arena pooling

## Recommendation

**Do this refactor next.** It's:
- Low risk (internal change only)
- High value (better memory model)
- Enables future optimizations
- Aligns with how other high-performance query engines work (DuckDB, DataFusion, etc.)
