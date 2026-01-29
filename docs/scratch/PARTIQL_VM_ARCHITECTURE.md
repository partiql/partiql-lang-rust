# PartiQLVM Architecture

## Overview

PartiQLVM is the execution engine for compiled PartiQL queries. It represents a single-threaded virtual machine that owns all execution state and memory for query processing.

## Architecture

### Design Principles

1. **VM owns all memory** - Single centralized arena for the entire query
2. **Single-threaded execution** - Each VM processes one query at a time
3. **Lightweight instantiation** - Can create multiple VMs from same CompiledPlan
4. **Clear ownership model** - Memory, operators, and state all owned by VM

### Components

```rust
pub struct PartiQLVM {
    compiled: Arc<CompiledPlan>,   // Shared plan (can have multiple VMs)
    operators: Vec<RelOp>,         // Operator instances
    arena: RowArena,               // VM-level memory arena (16KB default)
    scratch: RowFrameScratch,      // Row processing slots
}
```

## API Usage

### Basic Query Execution

```rust
use partiql_eval::engine::{PlanCompiler, PartiQLVM};

// 1. Compile the plan
let compiler = PlanCompiler::new(&scan_provider);
let compiled = compiler.compile(&logical_plan)?;

// 2. Create VM instance
let vm = PartiQLVM::new(compiled, Some(udf_registry))?;

// 3. Execute and iterate results
let mut stream = vm.execute()?;
while let Some(row) = stream.next_row()? {
    println!("Value: {:?}", row.get_i64(0));
}
```

### Convenience Method

The compiler also provides a convenience method:

```rust
// Equivalent to PartiQLVM::new()
let vm = compiler.instantiate(compiled, Some(udf_registry))?;
```

## Memory Management

### VM-Level Arena

The VM owns a single `RowArena` that is used for all allocations during query execution:

```
┌─────────────────────────────────────────┐
│           PartiQLVM                     │
│  ┌───────────────────────────────────┐  │
│  │  RowArena (16KB buffer)           │  │
│  │  [Row1][Row2][Row3]...            │  │
│  │  Contiguous, cache-friendly       │  │
│  └───────────────────────────────────┘  │
│                                          │
│  Operators (Pipeline, Filter, etc.)     │
│  RowFrameScratch (slots)                │
└─────────────────────────────────────────┘
```

**Benefits:**
- ✅ Single contiguous allocation for entire query
- ✅ Perfect cache locality
- ✅ O(1) reset between rows
- ✅ Memory reuse across operators in pipeline

### Per-Row Processing

```rust
// Inside ResultStream::next_row()
self.vm.arena.reset();           // O(1) - just reset offset
self.vm.scratch.reset();         // Clear slots
let frame = self.vm.scratch.frame(&self.vm.arena);  // Borrow arena
// Process row...
```

## Advanced Usage

### Concurrent Execution

Create multiple VMs from the same CompiledPlan:

```rust
let compiled = Arc::new(compiler.compile(&plan)?);

// Create VMs for parallel execution
let vm1 = PartiQLVM::new(compiled.clone(), udf.clone())?;
let vm2 = PartiQLVM::new(compiled.clone(), udf.clone())?;

// Execute in different threads
let handle1 = thread::spawn(move || {
    let mut stream = vm1.execute()?;
    // Process results...
});

let handle2 = thread::spawn(move || {
    let mut stream = vm2.execute()?;
    // Process results...
});
```

### VM Pooling

Reuse VMs across queries:

```rust
pub struct VMPool {
    compiled: Arc<CompiledPlan>,
    pool: Vec<PartiQLVM>,
    udf: Option<Arc<dyn UdfRegistry>>,
}

impl VMPool {
    pub fn execute_query(&mut self) -> Result<ResultStream> {
        // Get or create VM
        let vm = self.pool.pop()
            .unwrap_or_else(|| {
                PartiQLVM::new(self.compiled.clone(), self.udf.clone())
                    .expect("VM creation failed")
            });
        
        Ok(vm.execute()?)
    }
    
    pub fn return_vm(&mut self, stream: ResultStream) {
        // Extract VM from stream and return to pool
        // (requires exposing vm from ResultStream)
    }
}
```

## Performance Characteristics

### Memory Efficiency

**Before (Per-Operator Arena):**
- N separate arenas × 8KB = potentially 100s of KB
- Scattered allocations across heap
- Multiple reset operations

**After (VM-Level Arena):**
- 1 arena × 16KB = single allocation
- Contiguous memory (perfect cache locality)
- Single O(1) reset per row

### Cache Behavior

All query processing happens in one contiguous memory region:
```
VM Arena: [Scan data][Filter temps][Project results]
          ^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^
          All sequential in memory - great for CPU cache
```

### Comparison to Other Engines

| Engine     | Memory Model           | Approach                |
|------------|------------------------|-------------------------|
| DuckDB     | Per-thread arena       | Similar to PartiQLVM    |
| DataFusion | Arrow RecordBatch      | Columnar, pre-allocated |
| PostgreSQL | Memory contexts        | Hierarchical arenas     |
| PartiQL    | VM-level arena         | Single contiguous buffer|

## Implementation Details

### RowFrame Borrowing

```rust
pub struct RowFrameScratch {
    slots: Vec<SlotValue<'static>>,
    // No arena - borrowed from VM
}

impl RowFrameScratch {
    pub fn frame<'a>(&'a mut self, arena: &'a RowArena) -> RowFrame<'a> {
        RowFrame {
            slots: self.slots.as_mut_slice(),
            arena,  // Borrowed from VM
        }
    }
}
```

This design ensures:
- Arena lifetime tied to VM
- No arena cloning or duplication
- Clear ownership model

### ResultStream Ownership

```rust
pub struct ResultStream {
    pub schema: Schema,
    root: usize,
    vm: PartiQLVM,  // Owns the VM
}
```

The VM is moved into ResultStream, ensuring:
- VM lives as long as results are being consumed
- Arena remains valid for all returned rows
- Clean resource management

## Migration from PlanInstance

### Old API (Deprecated)
```rust
let instance = compiler.instantiate(compiled, udf)?;
let stream = instance.execute()?;
```

### New API (Current)
```rust
let vm = PartiQLVM::new(compiled, udf)?;
let stream = vm.execute()?;
```

**Changes:**
- `PlanInstance` → `PartiQLVM` (clearer naming)
- Arena moved from `RowFrameScratch` to `PartiQLVM`
- `RowFrameScratch::frame()` now takes arena parameter

## Future Optimizations

### 1. Batch Processing
```rust
impl ResultStream {
    pub fn next_batch(&mut self, size: usize) -> Result<Vec<RowView>> {
        // Process multiple rows before arena reset
        // Better amortization of reset cost
    }
}
```

### 2. Arena Size Tuning
```rust
impl PartiQLVM {
    pub fn with_arena_size(mut self, size: usize) -> Self {
        self.arena = RowArena::new(size);
        self
    }
}
```

### 3. Memory Pooling
```rust
pub struct ArenaPool {
    free_arenas: Vec<RowArena>,
}

impl PartiQLVM {
    pub fn with_pooled_arena(pool: &mut ArenaPool) -> Self {
        // Reuse arenas across queries
    }
}
```

## Testing

### Basic Correctness
```rust
#[test]
fn test_vm_execution() {
    let vm = PartiQLVM::new(compiled_plan, None)?;
    let mut stream = vm.execute()?;
    
    assert_eq!(stream.next_row()?.unwrap().get_i64(0), Some(42));
    assert!(stream.next_row()?.is_none());
}
```

### Concurrent Execution
```rust
#[test]
fn test_concurrent_vms() {
    let compiled = Arc::new(compile_plan());
    
    let handles: Vec<_> = (0..4).map(|_| {
        let plan = compiled.clone();
        thread::spawn(move || {
            let vm = PartiQLVM::new(plan.as_ref().clone(), None).unwrap();
            execute_and_verify(vm);
        })
    }).collect();
    
    for handle in handles {
        handle.join().unwrap();
    }
}
```

## Summary

PartiQLVM provides:
- ✅ Clear, intuitive API
- ✅ Efficient memory management
- ✅ Excellent cache locality
- ✅ Support for concurrent execution
- ✅ Foundation for future optimizations

The VM-level arena design aligns with industry best practices and provides a solid foundation for high-performance query execution.
