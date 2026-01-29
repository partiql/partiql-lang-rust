# PartiQL VM Reusability Design

## Design Goals

The PartiQLVM is designed to balance **reusability** with **simplicity** and **zero-copy performance**:

1. **Create Once, Execute Many**: A single VM instance can execute multiple queries over its lifetime, avoiding repeated memory allocation overhead
2. **Thread-Safe Plans**: `CompiledPlan` is wrapped in `Arc`, allowing multiple VMs to share the same plan concurrently
3. **Automatic Resource Management**: RAII (Resource Acquisition Is Initialization) handles file handles, connections, and other resources without user intervention
4. **Zero-Copy Streaming**: Results reference VM-owned memory (arena, registers) for maximum performance
5. **Statement Polymorphism**: Single execution API handles SELECT, INSERT, UPDATE, DELETE, CREATE, etc.

## Core API

```rust
// Thread-safe compiled plan
pub struct CompiledPlan { ... }

impl CompiledPlan {
    pub fn result_schema(&self) -> Schema { ... }
}

// Single-threaded execution engine
pub struct PartiQLVM { ... }

impl PartiQLVM {
    /// Create VM with initial plan
    pub fn new(
        compiled: CompiledPlan, 
        udf_registry: Option<Arc<dyn UdfRegistry>>
    ) -> Result<Self>;
    
    /// Execute loaded plan
    pub fn execute(&mut self) -> Result<ExecutionResult<'_>>;
    
    /// Load new plan (VM must not be executing)
    pub fn load_plan(
        &mut self, 
        compiled: CompiledPlan, 
        udf_registry: Option<Arc<dyn UdfRegistry>>
    ) -> Result<()>;
}

// Statement execution results
pub enum ExecutionResult<'vm> {
    Query(QueryIterator<'vm>),       // SELECT
    Mutation(MutationSummary),       // INSERT/UPDATE/DELETE
    Definition(DefinitionSummary),   // CREATE/DROP
}

// Streaming query results
pub struct QueryIterator<'vm> {
    vm: &'vm mut PartiQLVM,
    opened: bool,
}

impl QueryIterator<'_> {
    pub fn next(&mut self) -> Result<Option<RowView<'_>>>;
}

pub struct MutationSummary {
    pub rows_affected: usize,
}

pub struct DefinitionSummary {
    pub objects_created: usize,
}
```

## Lifecycle Management

### RAII for Operators

Operator resources (file handles, network connections) are managed automatically via RAII:

```rust
impl QueryIterator<'_> {
    pub fn next(&mut self) -> Result<Option<RowView<'_>>> {
        // Lazy open on first iteration
        if !self.opened {
            self.vm.open_operators()?;  // Open files, connections, etc.
            self.opened = true;
        }
        
        // Row processing...
    }
}

impl Drop for QueryIterator<'_> {
    fn drop(&mut self) {
        if self.opened {
            let _ = self.vm.close_operators();  // Close all resources
        }
    }
}
```

**Benefits**:
- Resources acquired lazily (only if iteration begins)
- Resources released automatically when iterator dropped
- Works correctly even with early exit (`break`) or errors
- Zero user boilerplate

### Operator Interface

```rust
pub trait RelOp {
    fn open(&mut self) -> Result<()>;
    fn close(&mut self) -> Result<()>;
    fn next_row(&mut self, arena: &Arena, regs: &mut [ValueRef]) -> Result<bool>;
}
```

Each operator implements resource management:
- `PipelineOp`: Opens file readers, closes when done
- `HashJoinState`: Acquires temporary buffers, releases on close
- `HashAggState`: Similar resource management
- `SortState`: Similar resource management

## Usage Patterns

### Pattern 1: Single Query

```rust
let mut vm = PartiQLVM::new(compiled_select, None)?;

match vm.execute()? {
    ExecutionResult::Query(mut iter) => {
        while let Some(row) = iter.next()? {
            println!("{:?}", row);
        }
        // Drop closes operators automatically
    }
    _ => {}
}
```

### Pattern 2: Multiple Queries (Same Thread)

```rust
let mut vm = PartiQLVM::new(plan1, None)?;

// Execute first query
{
    if let ExecutionResult::Query(mut iter) = vm.execute()? {
        while let Some(row) = iter.next()? { 
            // process row
        }
    } // Iterator dropped, operators closed
}

// Load and execute different query
vm.load_plan(plan2, None)?;
{
    if let ExecutionResult::Query(mut iter) = vm.execute()? {
        while let Some(row) = iter.next()? {
            // process row
        }
    }
}
```

### Pattern 3: Multi-Statement Batch

```rust
let mut vm = PartiQLVM::new(initial_plan, None)?;

for compiled_plan in compiled_statements {
    vm.load_plan(compiled_plan, None)?;
    
    match vm.execute()? {
        ExecutionResult::Query(mut iter) => {
            while let Some(row) = iter.next()? {
                // Process SELECT results
            }
        }
        ExecutionResult::Mutation(summary) => {
            println!("Modified {} rows", summary.rows_affected);
        }
        ExecutionResult::Definition(summary) => {
            println!("Created {} objects", summary.objects_created);
        }
    }
}
```

### Pattern 4: Concurrent Execution (Multiple VMs)

```rust
// Share plan across threads
let plan = Arc::new(compiled_plan);

// Spawn multiple workers
let handles: Vec<_> = (0..num_threads).map(|_| {
    let plan = Arc::clone(&plan);
    thread::spawn(move || {
        let mut vm = PartiQLVM::new((*plan).clone(), None)?;
        // Each thread executes independently
        vm.execute()?;
        Ok(())
    })
}).collect();

for handle in handles {
    handle.join().unwrap()?;
}
```

## Implementation Details

### Memory Management

```rust
pub struct PartiQLVM {
    compiled: Arc<CompiledPlan>,
    operators: Vec<RelOp>,
    arena: Arena,                     // Per-row scratch space
    registers: Vec<ValueRef<'static>>,  // Unified slot + temp storage
    root: usize,
    slot_count: usize,
}
```

**Arena**: 
- Reset on each `next_row()` call
- Holds computed values for current row only
- Persistent across loads (just a memory pool)

**Registers**:
- Layout: `[slots... | temps...]`
- First `slot_count` registers hold row data
- Remaining registers are expression temporaries
- Resized intelligently during `load_plan()`

### Register Array Resizing

```rust
impl PartiQLVM {
    pub fn load_plan(&mut self, compiled: CompiledPlan, udf: Option<Arc<dyn UdfRegistry>>) -> Result<()> {
        let needed = compiled.slot_count + compiled.max_register_count();
        
        if needed > self.registers.len() {
            // Grow if needed
            self.registers.resize(needed, ValueRef::Missing);
        } else {
            // Reuse existing capacity, just clear
            for reg in &mut self.registers[..needed] {
                *reg = ValueRef::Missing;
            }
        }
        
        // Re-instantiate operators from new plan
        self.operators = instantiate_operators(&compiled, udf)?;
        self.compiled = Arc::new(compiled);
        self.root = compiled.root;
        self.slot_count = compiled.slot_count;
        
        Ok(())
    }
}
```

**Strategy**: Registers never shrink, only grow. This maximizes reusability since most queries will have similar register requirements.

### State Tracking

```rust
pub struct QueryIterator<'vm> {
    vm: &'vm mut PartiQLVM,
    opened: bool,  // Track if operators are open
}
```

The `opened` flag prevents double-open and ensures Drop only closes if resources were acquired.

## Trade-offs

### Iterator Borrows VM

**Design Decision**: `QueryIterator<'vm>` borrows the VM mutably during iteration.

**Implication**: Cannot call `load_plan()` while iterator exists.

```rust
let mut iter = vm.execute()?;  // Iterator borrows VM
vm.load_plan(new_plan)?;       // ERROR: VM is borrowed
```

**Rationale**: 
- Zero-copy results reference VM-owned memory (arena, registers)
- Lifetime `'vm` enforces memory safety at compile time
- User must consume/drop iterator before loading new plan

**Workaround** (if needed):
```rust
{
    let iter = vm.execute()?;
    // consume iterator
} // Iterator dropped, VM available
vm.load_plan(new_plan)?;  // OK
```

### No Reset Method

**Previous Design**: Had explicit `reset()` to clear state between executions.

**Current Design**: No `reset()` method - use `load_plan()` with same plan for re-execution.

**Rationale**:
- `load_plan()` subsumes `reset()` functionality
- Simpler API (fewer methods)
- RAII handles resource lifecycle automatically

**Re-execution Pattern**:
```rust
let plan = vm.compiled.clone();  // Clone Arc (cheap)
vm.load_plan(plan, udf)?;         // Effectively resets
```

### Mutation/Definition Results

DML and DDL statements return immediately (no streaming):

```rust
match vm.execute()? {
    ExecutionResult::Mutation(summary) => {
        // No iteration needed, summary available immediately
        println!("Rows affected: {}", summary.rows_affected);
    }
    _ => {}
}
```

This allows immediate VM reuse without waiting for iteration completion.

## Thread Safety

### CompiledPlan

```rust
unsafe impl Send for CompiledPlan {}
unsafe impl Sync for CompiledPlan {}
```

`CompiledPlan` is immutable after creation, allowing safe sharing via `Arc`.

### PartiQLVM

**Not thread-safe**: A single VM instance is bound to one thread. For concurrent execution, create multiple VM instances sharing the same `CompiledPlan`.

```rust
// Thread 1
let mut vm1 = PartiQLVM::new(plan.clone(), None)?;

// Thread 2
let mut vm2 = PartiQLVM::new(plan.clone(), None)?;

// Both VMs share the plan data but have separate execution state
```

## Summary

The PartiQLVM design achieves reusability through:

1. **Separation of Plan and Execution**: `CompiledPlan` (thread-safe, shareable) vs `PartiQLVM` (single-threaded, stateful)
2. **RAII Resource Management**: Automatic open/close via `QueryIterator` Drop
3. **Lazy Resource Acquisition**: Resources opened on first iteration, not at execute()
4. **Memory Reuse**: Arena and registers persist across `load_plan()` calls
5. **Statement Polymorphism**: Single `execute()` API handles all statement types
6. **Explicit Lifecycle**: Lifetime `'vm` enforces correct usage at compile time

This design provides a clean, safe, and efficient execution model that scales from single queries to complex multi-statement workloads.
