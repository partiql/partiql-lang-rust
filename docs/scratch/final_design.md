# Performance Improvements Design Document

## 1. Executive Summary

The current PartiQL engine (partiql-eval + partiql-planner) frequently copies and materializes data, has no projection pushdown support due to its data model, and pays high per-row overhead. This is misaligned with our primary workload: streaming, per-row simple SFW queries. The near-term goal is to deliver a streaming-first engine with minimal per-row overhead, explicit zero-copy contracts, and projection pushdown. At the same time, we want to hide engine internals behind a stable interface so we can later add analytical optimizations (batch/vectorized execution) without exposing internal changes to callers.

This design proposes a streaming-first execution engine that fuses Scan/Filter/Project/Limit into a single pipeline runner, uses a compact scalar bytecode VM for expressions, and defines a strict reader contract to minimize copies. Note that while zero-copy is possible for simple projections, computed values (arithmetic, type conversions) necessarily require allocation. Batch execution is explicitly deferred to a future extension (Appendix A).

**Strategic Context**: As part of BDT (an analytical organization), the long-term roadmap includes high-throughput analytical workloads. Proof-of-concept experiments with vectorized execution demonstrated 150-300x performance improvements on analytical queries with columnar data formats. This design establishes abstraction boundaries that enable transparent migration to vectorized execution for analytical workloads without requiring API changes, while immediately addressing performance issues in the existing streaming use case.

## 2. Background & Context

### 2.1 Current Baseline

Execution is row-at-a-time via partiql-eval. Scalar expressions are interpreted per row, and relational operators are chained with nested `next()` calls. The current data model requires materialization, which in turn prevents projection pushdown: readers must produce full rows/objects even when only a subset of fields are needed.

### 2.2 Pain Points And Bottlenecks

- Excessive copying/materialization at operator boundaries and during expression evaluation.
- No projection pushdown support due to the current data model, so readers return more data than needed.
- High per-row overhead in the streaming SFW path (our dominant workload).

### 2.3 Business Impact

Streaming workloads require more CPU per row than necessary, which increases latency and cost. Performance fixes often require exposing internal engine details to callers, which we want to avoid so we can preserve evolution paths for future analytical workloads.

## 3. Goals & Non-Goals

### 3.1 Goals

- Reduce per-row overhead for streaming SFW queries.
- Minimize materialization and preserve zero-copy where safe.
- Enable projection pushdown via a reader contract that is independent of internal operator implementation.
- Keep internals hidden so future analytical optimizations can be added without public API changes.

### 3.1.1 API Stability as a Design Constraint

A primary design constraint is that future execution strategies—including vectorized operators for analytical workloads—must be implementable without changing public APIs. This is achieved through:

1. **Opaque result types**: Public APIs expose `ResultStream`, `RowView`, and `ValueView` abstractions rather than internal execution representations (`RowFrame`, `ValueRef`, operator states).

2. **Reader contract polymorphism**: The `RowReader` trait supports both row-at-a-time and batch-capable implementations through the same interface. `BufferStability` and `ReaderCaps` provide execution hints without exposing implementation details.

3. **Operator extensibility**: `RelOp` as an enum enables adding new operator variants (e.g., `VectorizedPipeline`, `BatchHashJoin`) without affecting call sites.

4. **Spec/state separation**: `CompiledPlan` immutability allows the compiler to select execution strategies (row-mode vs batch-mode operators) based on data characteristics while maintaining interface stability.

This constraint aligns with BDT's analytical mission: v1 addresses streaming performance for existing customers, while the architecture enables future vectorized execution for large-scale analytical workloads.

### 3.2 Non-Goals

- No full relational VM or cost-based optimizer rewrite in v1.
- No changes to PartiQL language semantics.
- No new external storage formats.
- Batch/vectorized execution is not in v1; it is a future extension (Appendix A).

## 4. Proposed Solution

The architecture follows a classic Volcano-style model: relational operators pull rows from their children and pass rows upstream using a shared `RowFrame`. Expressions are compiled separately from relational operators; each top-level scalar expression becomes its own register-based bytecode virtual machine instance that reads from input slots and writes results back to output slots. The data model centers on borrowed views (`ValueRef`) and slot-based row frames so streaming operators can pass values without copying, preserving zero-copy semantics wherever lifetimes allow.

The design emphasizes zero-copy, explicit data ownership, and cacheable compiled artifacts. Data ownership is explicit through buffer stability contracts and arena lifetimes; caching is enabled by a spec/state split with immutable compiled plans and per-execution runtime state. Gradual typing is supported without polluting fast paths by keeping typed slots when possible and falling back to `Any` only when needed. Hot paths are implemented as Rust enums for predictable dispatch and performance, while public APIs expose only views and results, hiding engine internals to keep the system evolvable.

## 5. Detailed Design

### 5.1 Compiled Plan

Compiled artifacts are immutable and reusable. A compiled plan contains the full operator graph, layouts, and scalar programs.

```rust
struct CompiledPlan {
    nodes: Vec<RelOpSpec>,
    bytecode: Vec<Program>,
    layouts: Vec<ScanLayout>,
}

enum RelOpSpec {
    Pipeline(PipelineSpec),
    HashJoin(HashJoinSpec),
    HashAgg(HashAggSpec),
    Sort(SortSpec),
    Custom(Box<dyn BlockingOperatorSpec>),
}
```

The compiled plan is shared across threads (`Arc`, `Send + Sync`) and never mutated.

### 5.2 PartiQLVM: Single-Threaded Execution Context

Each query execution instantiates a `PartiQLVM`, which serves as a single-threaded virtual machine owning all execution state. The VM encapsulates operators, a unified memory arena, and a reusable register array. This design centralizes memory management at the VM level rather than distributing it across operators or per-row allocations.

**Design Goal**: The VM is designed for reusability—create once and execute multiple queries over its lifetime without re-allocating core data structures. This reduces per-execution overhead while maintaining clean separation between thread-safe compiled plans and single-threaded execution state.

```rust
pub struct PartiQLVM {
    compiled: Arc<CompiledPlan>,
    operators: Vec<RelOp>,
    arena: Arena,              // VM-level memory for computed values
    registers: Vec<ValueRef<'static>>,  // Unified register array: slots + temporaries
}

impl PartiQLVM {
    /// Create VM with initial plan
    pub fn new(compiled: CompiledPlan, udf_registry: Option<Arc<dyn UdfRegistry>>) -> Result<Self>;
    
    /// Execute loaded plan, returning polymorphic results
    pub fn execute(&mut self) -> Result<ExecutionResult<'_>>;
    
    /// Load new plan for execution (VM must not be executing)
    pub fn load_plan(&mut self, compiled: CompiledPlan, udf_registry: Option<Arc<dyn UdfRegistry>>) -> Result<()>;
}
```

**Unified Register Array Architecture**: The VM uses a single contiguous register array where registers [0..slot_count] serve as row slots, and registers [slot_count..] are used for expression evaluation temporaries. This eliminates data movement between separate slot and register storage, providing better cache locality and simpler architecture.

During instantiation, each `RelOpSpec` is transformed into its corresponding runtime `RelOp`. The compiled plan tracks both slot_count and the maximum register count across all programs. The VM allocates a register array sized to `slot_count + max_registers`, allocated once and reused across all rows. The VM is lightweight enough to create one per query, while the compiled plan remains shared (`Arc`) across VMs for concurrent execution.

### 5.2.1 VM Reusability and Lifecycle

The VM supports three reusability patterns without requiring re-initialization:

**Pattern 1: Re-execution of same plan**
```rust
let plan_arc = Arc::new(compiled_plan);
let mut vm = PartiQLVM::new((*plan_arc).clone(), None)?;

// First execution
match vm.execute()? {
    ExecutionResult::Query(iter) => { /* consume */ }
    _ => {}
}

// Re-execute same plan
vm.load_plan((*plan_arc).clone(), None)?;
match vm.execute()? { /* ... */ }
```

**Pattern 2: Multiple different plans (sequential)**
```rust
let mut vm = PartiQLVM::new(plan1, None)?;

// Execute first plan
match vm.execute()? {
    ExecutionResult::Query(iter) => { /* consume */ }
    _ => {}
}

// Load and execute different plan
vm.load_plan(plan2, None)?;
match vm.execute()? { /* ... */ }
```

**Pattern 3: Multi-statement batch processing**
```rust
let mut vm = PartiQLVM::new(initial_plan, None)?;

for compiled_plan in compiled_statements {
    vm.load_plan(compiled_plan, None)?;
    match vm.execute()? {
        ExecutionResult::Query(iter) => { /* SELECT results */ }
        ExecutionResult::Mutation(summary) => { /* INSERT/UPDATE/DELETE */ }
        ExecutionResult::Definition(summary) => { /* CREATE/DROP */ }
    }
}
```

**Thread Safety Model**: `CompiledPlan` is wrapped in `Arc` and implements `Send + Sync`, allowing multiple VMs to share the same plan concurrently. Individual VMs are single-threaded but multiple VMs can execute the same plan on different threads:

```rust
let plan = Arc::new(compiled_plan);

// Spawn concurrent workers
let handles: Vec<_> = (0..num_threads).map(|_| {
    let plan = Arc::clone(&plan);
    thread::spawn(move || {
        let mut vm = PartiQLVM::new((*plan).clone(), None)?;
        vm.execute()
    })
}).collect();
```

**Memory Reuse Strategy**: When `load_plan()` is called, the VM intelligently manages its internal resources:

1. **Register Array Resizing**: Registers grow if the new plan needs more, but never shrink. This maximizes reusability since most queries have similar register requirements.

```rust
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
    
    // Re-instantiate operators from new plan specs
    self.operators = instantiate_operators(&compiled, udf)?;
    self.compiled = Arc::new(compiled);
    self.slot_count = compiled.slot_count;
    
    Ok(())
}
```

2. **Arena Persistence**: The arena buffer persists across `load_plan()` calls—it's just a memory pool that gets reset between rows, not between queries. This avoids repeated large allocations.

3. **Operator Re-instantiation**: Operators are re-created from the new plan's specs, but this is cheap compared to re-allocating the entire VM infrastructure.

**Execution State Lifecycle**: The VM enforces that execution state (active iterators) must be consumed before loading a new plan. This is guaranteed at compile time through Rust's borrow checker—`execute()` returns `ExecutionResult<'_>` with a lifetime tied to the VM, preventing `load_plan()` while an iterator exists.

**Memory Ownership Model**: The VM owns two critical execution resources:

1. **Arena** (default 16KB): Bump allocator for computed values that need heap storage (strings, complex objects, type conversions). Rather than each operator or row maintaining separate arenas, all allocations flow through the VM's arena. This provides:

- **Cache locality**: All intermediate values allocated sequentially in a single contiguous buffer
- **O(1) reset**: Arena resets via offset adjustment between rows, avoiding per-value deallocation
- **Reduced fragmentation**: Single allocation replaces scattered per-operator or per-value heap allocations
- **Predictable performance**: Memory access patterns are sequential, optimizing CPU cache utilization

2. **Registers**: Pre-allocated array sized to maximum register count across all programs. Eliminates per-row register allocation overhead and maintains perfect cache locality. Borrowed by `Program::eval()` during expression evaluation.

Both resources are borrowed by execution contexts (`RowFrame` for arena, `Program::eval()` for registers), ensuring that all operators in the pipeline share the same memory regions. This design aligns with high-performance query engines (DuckDB, DataFusion) that centralize memory management for streaming execution.

### 5.3 Relational Operators

A streaming relational operator produces output rows without requiring unbounded buffering. Examples include Scan, Filter, Project, and Limit. A blocking relational operator must retain input (or significant state) before producing output, such as HashJoin, HashAgg, and Sort.

Operators exchange data through the arena and unified register array. A parent calls `next_row` on its child, passing the arena reference (for computed value allocation) and the register array (for row data). Row data resides in registers [0..slot_count], so passing between operators involves register value assignment without copying.

```rust
pub type SlotId = u16;
```

`RelOp` is the unified runtime operator enum. It exposes a single `next_row` entrypoint that accepts both the arena and register array, with variants for streaming pipelines and blocking operators.

```rust
enum RelOp {
    Pipeline(PipelineOp),
    HashJoin(HashJoinState),
    HashAgg(HashAggState),
    Sort(SortState),
    Custom(Box<dyn BlockingOperator>),
}

impl RelOp {
    fn next_row<'a>(
        &'a mut self,
        arena: &'a Arena,
        regs: &mut [ValueRef<'a>]
    ) -> Result<bool> {
        match self {
            RelOp::Pipeline(op) => op.next_row(arena, regs),
            RelOp::HashJoin(op) => op.next_row(arena, regs),
            RelOp::HashAgg(op) => op.next_row(arena, regs),
            RelOp::Sort(op) => op.next_row(arena, regs),
            RelOp::Custom(op) => op.next_row(arena, regs),
        }
    }
}
```

### 5.3.1 Streaming Relational Operators

Streaming operators are fused into a `PipelineOp`. The runner executes a tight loop over rows to minimize call overhead.

```rust
struct PipelineOp {
    steps: Vec<Step>,
    reader: Box<dyn RowReader>,
}

impl PipelineOp {
    fn next_row<'a>(&'a mut self, arena: &'a Arena, regs: &mut [ValueRef<'a>]) -> Result<bool> {
        loop {
            if !self.reader.next_row(regs)? {
                return Ok(false);
            }
            if self.run_steps(arena, regs)? {
                return Ok(true);
            }
        }
    }

    fn run_steps<'a>(&mut self, arena: &'a Arena, regs: &mut [ValueRef<'a>]) -> Result<bool> {
        for step in &mut self.steps {
            if !step.eval(arena, regs)? {
                return Ok(false);
            }
        }
        Ok(true)
    }
}
```

Pipeline steps are native Rust enums for performance, not trait objects. Some steps (such as Filter) only read slots and do not mutate them, so downstream steps can safely share borrowed values without materialization. This preserves zero-copy for streaming paths.

```rust
enum Step {
    Filter { program: Program },
    Project { program: Program },
    Limit { remaining: usize },
}

impl Step {
    fn eval<'a>(&mut self, arena: &'a Arena, regs: &mut [ValueRef<'a>], udf: Option<&'a dyn UdfRegistry>) -> Result<bool> {
        match self {
            Step::Filter { program } => {
                program.eval(arena, regs, udf)?;
                Ok(regs[0].as_bool()?) // Check predicate in first slot
            }
            Step::Project { program } => {
                program.eval(arena, regs, udf)?;
                Ok(true)
            }
            Step::Limit { remaining } => {
                if *remaining == 0 {
                    return Ok(false);
                }
                *remaining -= 1;
                Ok(true)
            }
        }
    }
}
```

### 5.3.2 Blocking Relational Operators

Blocking operators (HashJoin, HashAgg, Sort) remain classic stateful operators and must respect buffer stability. They are direct variants of `RelOp` for performance, with an optional dynamic variant for customer-provided implementations.

**Materialization Policy**: Blocking operators must consider the reader's buffer stability when storing rows:

```rust
fn store_build_row(row: &RowFrame, caps: &ReaderCaps) {
    match caps.stability {
        BufferStability::UntilClose => {
            // Safe to store ValueRef directly - reader guarantees 
            // borrowed data remains valid until close()
            store_borrowed(row)
        }
        BufferStability::UntilNext => {
            // Must materialize to owned values - reader may reuse
            // buffers on next pull, invalidating borrows
            store_owned_copy(row)
        }
    }
}
```

This policy enables zero-copy for blocking operators when readers provide `UntilClose` stability, while ensuring correctness when readers reuse buffers.

### 5.4 Data Source Readers

**Design Rationale**: The `RowReader` contract is designed as an abstraction layer that decouples the execution model from data access patterns. Row-oriented readers (JSON, Ion) and batch-oriented readers (Arrow, Parquet) implement the same interface. This enables execution strategy selection at compile time: a query over columnar data can use vectorized operators internally while exposing the same `ResultStream` API externally. The `BufferStability` and `ReaderCaps` metadata provide materialization hints without exposing whether the reader operates row-at-a-time or in batches.

Readers are third-party data providers that manage their own memory. They are configured with a fixed `ScanLayout` (projection) before reading, and they populate row data directly into the unified register array (slots [0..slot_count]). They must declare buffer stability and capabilities, and they must honor the borrowing rule (valid until the next pull).

**Memory Management**: Readers are independent data sources and do not receive arena access. They manage their own storage for borrowed data:
- Primitives (i64, f64, bool) are generated on-the-fly with no storage needed
- Strings and complex values are stored in reader-owned buffers (e.g., `IonRowReader::string_storage`)

This design cleanly separates concerns: readers produce input data from external sources, while the arena (accessed via `RowFrame`) is exclusively for computed intermediate values during expression evaluation.

```rust
pub enum BufferStability {
    UntilNext,    // Reader may reuse buffers on next next_row() call
    UntilClose,   // Reader guarantees stability until close()
}

pub struct ReaderCaps {
    pub stability: BufferStability,
    pub can_project: bool,
    pub can_return_opaque: bool,
}

pub trait RowReader {
    fn caps(&self) -> ReaderCaps;
    fn set_projection(&mut self, layout: ScanLayout) -> Result<()>;
    fn open(&mut self) -> Result<()>;
    
    /// Populate row data into registers [0..slot_count]
    /// Readers manage their own memory; borrowed references must remain
    /// valid according to BufferStability contract
    fn next_row(&mut self, regs: &mut [ValueRef<'_>]) -> Result<bool>;
    
    fn resolve(&self, field_name: &str) -> Option<ScanSource>;
    fn close(&mut self) -> Result<()>;
}
```

### 5.5 Projection Pushdown

Projection pushdown is enabled through `ScanLayout`. To align with the experimental vectorized `ProjectionSpec`, `ScanLayout` explicitly captures a projection source, a target slot index, and a declared type. Target indices are contiguous from 0 to keep row layouts compact and consistent across readers.

```rust
pub struct ScanLayout {
    pub projections: Vec<ScanProjection>,
}

pub struct ScanProjection {
    pub source: ScanSource,
    pub target_slot: SlotId,
    pub type_hint: TypeHint,
}

pub enum ScanSource {
    ColumnIndex(usize),
    FieldPath(String),
    BaseRow,
}
```

### 5.6 Execution Data Model

The execution model uses borrowed values where possible, with a strict ownership tiering model. Borrowed values must not be retained beyond their validity window as defined by `BufferStability`.

```rust
pub enum ValueRef<'a> {
    Missing,
    Null,
    Bool(bool),
    I64(i64),
    F64(f64),
    Str(&'a str),
    Bytes(&'a [u8]),
    Obj(ObjRef<'a>),
    Arr(ArrRef<'a>),
    Opaque(OpaqueRef<'a>),
}
```

**Slot Values**: Rows are represented as slot arrays, where each slot contains either a borrowed or owned value:

```rust
pub enum SlotValue<'a> {
    Val(ValueRef<'a>),        // Borrowed from reader or arena-allocated
    Owned(&'a ValueOwned),    // Reference to arena-owned value
}
```

**VM-Level Arena Architecture**: The `Arena` is owned by the `PartiQLVM` rather than individual operators or row frames. This centralized ownership provides several advantages:

```rust
pub struct Arena {
    buffer: Vec<u8>,          // Contiguous 16KB buffer (default)
    offset: usize,            // Current allocation offset
}

impl Arena {
    pub fn alloc(&mut self, value: ValueOwned) -> &ValueOwned {
        // Bump allocate into contiguous buffer
        // Returns reference valid until arena reset
    }
    
    pub fn reset(&mut self) {
        // O(1) reset via offset = 0
        // No per-value deallocation
    }
}
```

The arena serves all operators in a pipeline through a borrowed reference in `RowFrame`. When `ResultStream::next_row()` executes:

1. **Arena reset**: `vm.arena.reset()` (O(1) operation)
2. **Frame creation**: `vm.scratch.frame(&vm.arena)` borrows the arena
3. **Pipeline execution**: All operators (readers, filters, projects) allocate through `frame.arena`
4. **Result return**: Borrowed values remain valid until next `next_row()` call

This design eliminates scattered heap allocations that would occur with per-operator or per-value allocation strategies. All intermediate values—computed results, type conversions, string operations—reside in sequential memory addresses within the VM's arena. CPU cache prefetchers can predict access patterns, and L1/L2 cache utilization improves significantly compared to pointer-chasing across heap fragments.

**Memory Flow Example**:
```rust
// In ResultStream::next_row()
self.vm.arena.reset();                    // Clear previous row allocations
let mut frame = self.vm.scratch.frame(&self.vm.arena);  // Borrow VM arena

// Pipeline execution
self.vm.operators[root].next_row(&mut frame)?;

// All computed values now in vm.arena's contiguous buffer:
// [Row1_Int][Row1_String][Row1_Computed] <- Sequential in memory
```

The arena lifetime is tied to the VM, ensuring all borrowed references remain valid throughout query execution. Between rows, the arena resets without deallocating the underlying buffer, maintaining allocation locality across the entire result set.

### 5.7 Scalar Bytecode Virtual Machine

Scalar expressions are represented as a compact Rust enum for fast pattern matching and compilation. This is the primary in-memory representation used by the compiler and optimizer. User-defined function calls remain possible via a dedicated variant.

```rust
enum Expr {
    Literal(ValueOwned),
    SlotRef(SlotId),
    Add(Box<Expr>, Box<Expr>),
    Eq(Box<Expr>, Box<Expr>),
    GetField(Box<Expr>, String),
    UdfCall { name: String, args: Vec<Expr> },
}
```

Scalar expressions compile to register-based bytecode. The VM is intentionally minimal and reused across all operators.

```rust
pub struct Program {
    pub insts: Vec<Inst>,
    pub consts: Vec<ValueOwned>,
    pub keys: Vec<String>,
    pub reg_count: u16,  // Maximum registers needed by this program
}

fn eval_row(
    program: &Program, 
    frame: &mut RowFrame,
    regs: &mut [ValueRef],  // Borrowed from VM
) -> Result<bool> {
    // Interpret bytecode over frame slots and borrowed registers.
    // Store results back into output slots.
    Ok(true)
}
```

**VM Register Model**: Registers are allocated once at VM creation time, sized to the maximum register count across all programs in the compiled plan. The register array is borrowed by `Program::eval()` during expression evaluation, eliminating per-row heap allocations. This provides:

- **Zero allocations per row**: Register array allocated once, reused for entire query execution
- **Perfect cache locality**: Contiguous register array keeps all intermediate values in CPU cache
- **Zero-copy primitives**: i64, f64, bool values stored directly in registers without boxing
- **Zero-copy constants**: Program constants (from `Program.consts`) referenced directly without cloning

The compiled plan tracks `max_registers` across all programs:

```rust
struct CompiledPlan {
    nodes: Vec<RelOpSpec>,
    max_registers: usize,  // Maximum across all Programs in plan
    // ...
}
```

When `PartiQLVM` is instantiated, it allocates the register array:

```rust
impl PartiQLVM {
    pub fn new(compiled: CompiledPlan, udf: Option<Arc<dyn UdfRegistry>>) -> Result<Self> {
        let max_regs = compiled.max_register_count();
        let registers = vec![ValueRef::Missing; max_regs];
        
        Ok(PartiQLVM {
            compiled: Arc::new(compiled),
            operators,
            arena: Arena::new(16384),
            scratch: RowFrameScratch::new(slot_count),
            registers,  // Allocated once, reused for all rows
        })
    }
}
```

During execution, programs borrow the VM's register array with lifetime transmutation to match the row frame's lifetime. Registers can reference borrowed input values, point to arena-allocated computed values, or reference program constants directly. When an instruction produces a new value requiring heap storage (e.g., string concatenation), it allocates into `Arena`; primitive operations (arithmetic on i64/f64) store results directly in registers without allocation.

An instruction set is expressed as a Rust enum for fast dispatch:

```rust
enum Inst {
    LoadConst { dst: u16, const_idx: u16 },
    AddI64 { dst: u16, a: u16, b: u16 },
    SubI64 { dst: u16, a: u16, b: u16 },
    ModI64 { dst: u16, a: u16, b: u16 },
    EqI64 { dst: u16, a: u16, b: u16 },
    AndBool { dst: u16, a: u16, b: u16 },
    OrBool { dst: u16, a: u16, b: u16 },
    NotBool { dst: u16, src: u16 },
    GetField { dst: u16, base: u16, key_idx: u16 },
    StoreSlot { slot: SlotId, src: u16 },
    CallUdf { dst: u16, func_idx: u16, args: Vec<u16> },
}
```

**Slot Access**: Slot references (`Expr::SlotRef(slot_id)`) compile directly to register indices. Since slots occupy registers [0..slot_count], a reference to slot 3 is simply register 3 - no load instruction needed. This eliminates an entire instruction category from the bytecode.

Example evaluation demonstrating zero-copy primitive handling and direct slot access:

```rust
fn eval_inst(inst: &Inst, regs: &mut [ValueRef], frame: &mut RowFrame) -> Result<()> {
    match *inst {
        Inst::AddI64 { dst, a, b } => {
            let av = regs[a as usize].as_i64()?;  // May reference slot directly
            let bv = regs[b as usize].as_i64()?;
            // Zero-copy: store primitive directly in register
            regs[dst as usize] = ValueRef::I64(av + bv);
        }
        Inst::LoadConst { dst, const_idx } => {
            let value = &program.consts[const_idx as usize];
            // Zero-copy: reference constant from program's const pool
            regs[dst as usize] = ValueRef::from_owned(value);
        }
        Inst::StoreSlot { slot, src } => {
            // Copy from temporary register to slot register
            regs[slot as usize] = regs[src as usize];
        }
        _ => {}
    }
    Ok(())
}
```

The VM accesses slots directly through register indices, computes over registers, and writes results back. Primitive operations (arithmetic, comparisons, logical ops) store results directly in registers without heap allocation. Only operations producing complex values (strings, objects, type conversions) allocate into `Arena`.

**Performance Impact**: This register reuse strategy eliminates a major source of per-row overhead. For expression-heavy queries (filters with complex predicates, computed projections), the VM now performs:
- Zero heap allocations for register management (vs. one Vec allocation per expression per row)
- Sequential register access patterns optimized for CPU cache prefetching
- Reduced memory allocator pressure and fragmentation

Combined with the VM-level arena for computed values, the engine achieves near-optimal memory behavior for streaming execution.

### 5.8 Consuming Results and Statement Polymorphism

The execution model supports multiple statement types (SELECT, INSERT, UPDATE, DELETE, CREATE, DROP) through a unified API. `execute()` returns an `ExecutionResult` enum that handles all statement types polymorphically:

```rust
pub enum ExecutionResult<'vm> {
    Query(QueryIterator<'vm>),       // SELECT - streaming results
    Mutation(MutationSummary),       // INSERT/UPDATE/DELETE - immediate summary
    Definition(DefinitionSummary),   // CREATE/DROP - immediate summary
}

pub struct MutationSummary {
    pub rows_affected: usize,
}

pub struct DefinitionSummary {
    pub objects_created: usize,
}
```

**Query Execution**: SELECT statements return a `QueryIterator` that streams results. The iterator borrows the VM mutably, ensuring safe access to shared execution state (arena, registers).

**Mutation/Definition Execution**: DML and DDL statements return immediately with summary information, allowing instant VM reuse without waiting for iteration completion.

### 5.8.1 RAII-Based Resource Management

Operator resources (file handles, network connections, temporary buffers) are managed automatically via RAII through the `QueryIterator`. This eliminates manual resource lifecycle management:

```rust
pub struct QueryIterator<'vm> {
    vm: &'vm mut PartiQLVM,
    opened: bool,  // Track operator resource state
}

impl QueryIterator<'_> {
    pub fn next(&mut self) -> Result<Option<RowView<'_>>> {
        // Lazy open on first iteration
        if !self.opened {
            self.vm.open_operators()?;  // Open files, connections, etc.
            self.opened = true;
        }
        
        // Arena reset and row processing
        self.vm.arena.reset();
        let regs = /* borrow registers */;
        let op = &mut self.vm.operators[self.vm.root];
        
        if op.next_row(&self.vm.arena, regs)? {
            let slots = &regs[0..self.vm.slot_count];
            Ok(Some(RowView::new(slots)))
        } else {
            Ok(None)
        }
    }
}

impl Drop for QueryIterator<'_> {
    fn drop(&mut self) {
        if self.opened {
            // Best-effort close, ignore errors in Drop
            let _ = self.vm.close_operators();
        }
    }
}
```

**Benefits of RAII Lifecycle**:
- **Lazy Initialization**: Resources acquired only if iteration begins, not at `execute()`
- **Automatic Cleanup**: Drop trait guarantees resource release even on early exit (`break`) or errors
- **Zero Boilerplate**: Users never call `open()`/`close()` manually
- **Error Handling**: First `next()` can return errors if resource acquisition fails

**Operator Resource Interface**:
```rust
impl RelOp {
    fn open(&mut self) -> Result<()> {
        match self {
            RelOp::Pipeline(op) => op.open(),  // Open file readers
            RelOp::HashJoin(op) => op.open(),  // Acquire temp buffers
            RelOp::HashAgg(op) => op.open(),   // Acquire temp buffers
            RelOp::Sort(op) => op.open(),      // Acquire temp buffers
            RelOp::Custom(op) => op.open(),
        }
    }
    
    fn close(&mut self) -> Result<()> {
        // Mirror open() for each operator variant
    }
}
```

Each operator type implements resource-specific lifecycle:
- `PipelineOp`: Delegates to `RowReader::open()`/`close()` for file handles
- `HashJoin/HashAgg/Sort`: Manages temporary buffer allocation/deallocation
- Custom operators: User-defined resource management

### 5.8.2 Result Streaming API

Results are pulled from the root operator through the borrowed VM. The lifetime guarantees ensure all borrowed values remain valid:


```rust
pub struct RowView<'a> { /* opaque */ }
pub struct ValueView<'a> { /* opaque */ }

impl<'a> RowView<'a> {
    pub fn get(&self, col: usize) -> ValueView<'a> { /* ... */ }
    pub fn get_i64(&self, col: usize) -> Option<i64> { /* ... */ }
    pub fn get_str(&self, col: usize) -> Option<&'a str> { /* ... */ }
    pub fn get_value(&self, col: usize) -> ValueOwned { /* ... */ }
}

pub struct ResultStream {
    pub schema: Schema,
    root: usize,
    vm: PartiQLVM,  // Owns the VM and its arena
}

impl ResultStream {
    pub fn next_row(&mut self) -> Result<Option<RowView<'_>>> {
        // Reset arena for this row
        self.vm.arena.reset();
        self.vm.scratch.reset();
        
        // Borrow arena from VM for execution
        let mut frame = self.vm.scratch.frame(&self.vm.arena);
        
        // Pull row from operator
        let op = &mut self.vm.operators[self.root];
        if op.next_row(&mut frame)? {
            Ok(Some(RowView::new(frame.slots)))
        } else {
            Ok(None)
        }
    }
}
```

**Usage Examples**:

```rust
// Single query execution
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
// VM immediately reusable for load_plan()

// Multi-statement batch
let mut vm = PartiQLVM::new(initial_plan, None)?;
for plan in compiled_statements {
    vm.load_plan(plan, None)?;
    match vm.execute()? {
        ExecutionResult::Query(mut iter) => {
            while let Some(row) = iter.next()? { /* process */ }
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

**Execution Flow**: When `next()` is called on `QueryIterator`:
1. VM arena resets (O(1) offset adjustment)
2. Scratch slots reset to default values  
3. `RowFrame` created with borrowed VM arena
4. Operator pipeline executes, allocating into VM arena
5. Result row returned with borrowed references valid until next call

Callers map column names to indices using `compiled.result_schema()`. Borrowed outputs (`RowView`, `ValueView`) remain valid until the next `next()` pull, which resets the arena and invalidates previous borrows.

**Borrowing Constraint**: The iterator borrows the VM mutably during its lifetime. This prevents calling `load_plan()` while an iterator exists—the borrow checker enforces this at compile time:

```rust
let mut iter = vm.execute()?;  // Iterator borrows VM
vm.load_plan(new_plan)?;       // ERROR: VM is borrowed

// Correct usage:
{
    let iter = vm.execute()?;
    // consume iterator
} // Iterator dropped, VM available
vm.load_plan(new_plan)?;  // OK
```

**Rationale**: Zero-copy results reference VM-owned memory (arena, registers). The lifetime `'vm` on `ExecutionResult` enforces memory safety—users must consume or drop the iterator before the VM can be reused.

### 5.9 Public APIs

Public APIs are designed for execution strategy independence. Internal execution models (row-at-a-time vs vectorized) remain encapsulated, enabling performance improvements without interface changes.

**Public Surface:**
- `Plan` or `PreparedQuery`: Query compilation interface
- `ResultStream`: Execution result iterator
  - `schema`: Column metadata for result mapping
  - `next_row()`: Pull-based row iteration
- `RowView` and `ValueView`: Zero-copy result accessors
  - Typed getters: `get_i64()`, `get_str()`, `get_bool()`
  - Materialization: `get_value()` for owned value extraction
- Owned conversion helpers for serialization and retention

**Internal-Only Types** (not exposed):
- `ValueRef`, `RowFrame`: Execution data model primitives
- `RelOp`, `PipelineOp`: Operator implementations
- Arenas, VM registers, bytecode structures

**Open Question**: The visibility of `ValueRef` and `ValueOwned` may need to be reconsidered for extensibility. Customer-provided readers, UDFs, and potentially custom relational operators may require direct access to these types. This trade-off between encapsulation and extensibility should be evaluated as implementation progresses.

**Evolution Guarantee**: The public API design enables adding vectorized operators, batch readers, or adaptive execution without user code changes. A query executing row-at-a-time today can transparently switch to vectorized execution when operating on columnar data—the public interface (`ResultStream.next_row()`) remains identical.

## 6. Performance Analysis

### 6.1 Benchmarking Methodology

Benchmark the streaming-first engine against the current implementation on:

- Streaming SFW queries (dominant workload).
- Early-exit queries with LIMIT.
- Basic joins/aggregations with projection pushdown.

Collect throughput (rows/sec), CPU utilization, allocation rates, and materialization counts.

### 6.2 Expected Results

Based on proof-of-concept testing, we observed approximately 40% latency reduction for streaming SFW queries with projection pushdown. Memory utilization is expected to decrease due to reduced copying and arena-based allocation for computed values.

**Performance Drivers**:
- Pipeline fusion eliminates per-operator virtual dispatch overhead
- Projection pushdown reduces data volume from readers
- Arena allocation amortizes allocation cost across rows
- Bytecode VM avoids repeated AST traversal

Actual results will vary based on query complexity, data characteristics, and reader implementations. Comprehensive benchmarking is required to validate these improvements across representative workloads.

### 6.3 Anticipated Hot Spots

- Scalar expression evaluation.
- HashAgg/HashJoin key evaluation and storage.
- Pipeline runner overhead if not carefully minimized.

### 6.4 Scalability

Row-mode streaming pipelines scale linearly with input size. Blocking operators scale with input size but require careful materialization policy to manage memory usage.

## 7. Trade-Offs & Alternatives

- Pure vectorized engine: rejected for v1 due to complexity and misalignment with streaming-first goals.
- Pure row engine without structural changes: rejected due to copying/materialization costs.
- Relational VM: rejected for higher complexity and longer roadmap.

The proof-of-concept vectorized engine demonstrated substantial performance gains for analytical workloads:

- **Columnar data (Arrow/Parquet)**: 150-300x throughput improvement over row-at-a-time execution
- **Analytical queries** (filters, projections, aggregations): Near-linear scaling with data volume due to SIMD operations and cache efficiency
- **Degradation at batch_size=1**: 2-3x slower than pure row mode due to per-batch coordination overhead and selection vector management

**Analysis**: These results validate two requirements:
1. Vectorized execution provides transformational performance for analytical workloads (relevant to BDT's mission)
2. Row-mode execution is necessary for low-latency streaming workloads

Rather than selecting one execution model, this design establishes abstractions that support both modes behind a unified API. The planner can detect data characteristics (columnar formats, large batch sizes) and select appropriate operators transparently.

The pipeline runner adds complexity but provides significant per-row savings. Reduced copying for passthrough operations lowers memory traffic, though computed values require allocation.

### 7.5 Evolution Path to Analytical Performance

While v1 implements streaming execution, the architecture explicitly enables vectorized performance for analytical workloads without API changes.

**Phase 1 (v1 - This Document)**: Streaming-first foundation
- Row-mode execution with pipeline fusion
- Reader abstraction supporting projection pushdown
- Public APIs independent of execution strategy

**Phase 2 (Future - Appendix A)**: Transparent vectorized execution
- `BatchReader` implementations (Arrow, Parquet)
- Vectorized operator variants (`VectorizedPipeline`, `BatchHashJoin`)
- Compiler detects columnar data and selects batch operators
- API compatibility: `ResultStream.next_row()` unchanged
  - Internal implementation may buffer batches and emit rows
  - Optional `next_batch()` performance API for batch-aware consumers

**Phase 3 (Future)**: Adaptive execution
- Runtime workload profiling
- Dynamic mode switching (row ↔ batch) within query execution
- Hybrid operators with runtime strategy selection

**Technical Feasibility:**

1. **Reader abstraction**: `RowReader` can wrap `BatchReader`, exposing row-at-a-time iteration over internal batches
2. **Operator polymorphism**: `RelOp::VectorizedPipeline` is an internal implementation variant invisible to callers
3. **Result abstraction**: `ResultStream` can buffer batches internally while maintaining row iteration semantics

**Strategic Value**: This evolution path enables serving streaming workloads (existing customers) and analytical workloads (vectorized performance) from a unified codebase. The POC demonstrated 150-300x improvements for analytical queries—the abstraction strategy preserves access to this performance while maintaining compatibility.

## 8. Compatibility Strategy

Existing customers use `partiql-eval` APIs (e.g., `EvalPlan::execute` returning a single `Value`). While we aim for compatibility where feasible, some breaking changes are expected. The compatibility strategy provides adapters for the most common use cases, but customers may need to make small code adjustments during migration.

Proposed approach:

- Legacy operator variant: add a `RelOp::LegacyEval(Box<dyn Evaluable>)` (and `RelOpSpec::LegacyEval`) that wraps a legacy `Evaluable` node. The wrapper materializes input `RowFrame` values into a `Value`, calls `evaluate`, then maps the `Value` result back into a row slot. This enables incremental adoption while keeping new pipelines intact.
- Value conversion helpers: add explicit adapters between `Value` and `ValueRef`/`ValueOwned`:
  - `Value` -> `ValueRef`: borrow when the `Value` is stable and owned by the caller; otherwise copy into a `Arena` and return a `ValueRef` to the owned value.
  - `ValueRef` -> `ValueOwned`: materialize on demand for legacy evaluables or for API consumers that need owned retention.
- Result compatibility: the `ResultStream` can expose a convenience method that materializes the stream into a single `Value` for legacy callers. This preserves the legacy behavior of “one evaluated Value” while allowing new consumers to stream rows.

**Expected Breaking Changes**:
- API signatures may change to accommodate new execution model
- Some legacy evaluation patterns may not map cleanly to streaming
- Performance characteristics may differ from legacy engine

The adapters provide a migration path for common cases, but full compatibility cannot be guaranteed. We will document breaking changes and provide migration guidance as part of the release process.

## 9. Implementation Plan

### Phase 1: Foundations (Data Model + Plan Scaffolding)

- Define `ValueRef`, `ValueOwned`, and `Arena` with clear lifetime rules.
- Implement `RowFrame` and slot semantics; add `RowView`/`ValueView` mapping.
- Create `RelOpSpec`/`RelOp` scaffolding and compiled plan storage.
- Build `PlanInstance::execute` and `ResultStream` with schema metadata.

### Phase 2: Reader Contracts + Projection Pushdown

- Implement `ReaderCaps` and `BufferStability` contracts.
- Define `ScanLayout` and integrate into planner output.
- Add row readers that honor `can_project` and fill slots directly.
- Add `Value` ↔ `ValueRef` conversion helpers for legacy adapters.

### Phase 3: Scalar Bytecode VM

- Define `Expr` enum and compilation to `Program` (bytecode).
- Implement row VM evaluator (registers, slot loads/stores, arena writes).
- Add UDF call support and error propagation.
- Integrate scalar VM into Filter/Project steps.

### Phase 4: Streaming Pipelines

- Implement `PipelineOp` runner and native `Step` enum (Filter/Project/Limit).
- Fuse Scan + Filter + Project + Limit in row mode.
- Validate zero-copy behavior through pipeline steps.
- Add microbenchmarks for streaming SFW queries.

### Phase 5: Blocking Operators (Minimal Viable)

- Implement HashAgg/HashJoin/Sort with materialization policy.
- Use buffer stability to decide borrow vs copy.
- Integrate blocking operators into `RelOp` dispatch.
- Add correctness tests for joins/aggregations.

### Phase 6: Compatibility Layer (Optional)

- Add `RelOp::LegacyEval` wrapper around `Evaluable`.
- Bridge `Value` inputs/outputs to `RowFrame` and `RowView`.
- Provide result materialization into a single `Value` for legacy callers.

### Future Phases (Appendix A)

- Batch execution for streaming pipelines.
- Typed kernels and runtime adaptivity.

## 10. Testing & Validation

- Microbenchmarks for streaming SFW with synthetic data.
- End-to-end streaming queries on representative datasets (JSON, Arrow if available).
- Conformance tests for correctness parity with existing partiql-eval.

## 11. Risks & Mitigations

- Incorrect borrow lifetimes: enforce stability contract and add tests around reader validity.
- Performance regressions for small queries: optimize pipeline runner and keep row mode minimal.
- Complexity in planner/executor: keep boundaries explicit and limit scope in v1.

## 12. Open Questions

- Are there strict latency SLAs for streaming workloads that should bound per-row overhead?
- Which readers (JSON, Arrow, custom) should be first-class for projection pushdown?
- What data sizes and distributions are most representative of current streaming traffic?
- Graph execution (GPML): can we retain existing graph functionality via adapters around the legacy graph APIs without significant engineering effort? Further investigation is required; current customer usage of GPML is minimal.

## 13. FAQs

Q: What problem are we solving?
A: Reduce per-row overhead and excessive copying in the current engine, while enabling future analytical optimizations without exposing internal details.

Q: Why not adopt a full vectorized engine now?
A: The POC vectorized engine excelled for large analytical workloads but performed worse at batch size 1 due to overhead. Our dominant use case is streaming, so v1 is row-first.

Q: How does zero-copy work in this design?
A: Simple projections that pass reader values directly to output slots avoid copying. Computed values (arithmetic, type conversions) require allocation into `Arena`. Buffer stability contracts ensure borrowed values remain valid appropriately.

Q: Will this break existing customers?
A: Some breaking changes are expected as the execution model fundamentally differs from the legacy engine. The compatibility strategy (Section 8) provides adapters for common use cases, but customers may need to make small code adjustments. We will provide migration documentation and support to ease the transition.

Q: How do I know what the result means (table vs scalar)?
A: `ResultStream` exposes a schema and shape metadata so callers can tell whether results are a collection, struct, or scalar, and map columns accordingly.

Q: When will vectorized execution arrive?
A: It is deferred to a future phase (Appendix A) once the streaming-first core is stable.

Q: How do API abstractions enable future vectorized performance?
A: Public APIs expose high-level abstractions (`ResultStream`, `RowView`) rather than internal execution types (`RowFrame`, `ValueRef`). This maintains interface stability regardless of internal execution strategy. `ResultStream.next_row()` can pull from row-mode pipelines today or vectorized pipelines tomorrow—consumers observe no difference. The POC vectorized engine demonstrated 150-300x improvements for analytical workloads; the abstraction strategy preserves access to this performance without requiring code changes.

## Appendix A. Future Batch/Vectorized Execution (Not in v1)

This appendix details how vectorized execution integrates into the v1 architecture without modifying public APIs. The abstractions established in v1 (Section 5.9) are designed to enable this evolution.

**Design Continuity**: The batch execution model reuses v1 abstractions:
- Readers implement projection contracts (applying `ScanLayout` to batches)
- Operators expose pull-based interfaces (yielding `DataChunk` instead of `RowFrame`)
- Public APIs remain unchanged (`ResultStream.next_row()` maintains identical semantics)

The distinction is internal: v1 operators process individual rows, while vectorized operators process batches using typed kernels.

### A.1 Batch Data Model

```rust
pub struct DataChunk<'a> {
    pub len: usize,
    pub sel: Option<&'a [u16]>,
    pub cols: Vec<Column<'a>>,
}

pub enum Column<'a> {
    I64(Vec<Option<i64>>),
    F64(Vec<Option<f64>>),
    Bool(Vec<Option<bool>>),
    Str(Vec<Option<&'a str>>),
    Any(Vec<ValueRef<'a>>),
}
```

### A.2 Batch Execution Model

- Reuse the same scalar bytecode VM with a batch evaluator.
- Add typed kernels for common scalar ops.
- Insert explicit `Batcher`/`Unbatcher` adapters at boundaries.

### A.3 Planner/Executor Changes

- Pipeline segments can be marked as batch-capable.
- Mode selection starts static (compile-time), with optional runtime adaptivity later.
