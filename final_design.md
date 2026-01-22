# Performance Improvements Design Document

## 1. Executive Summary

The current PartiQL engine (partiql-eval + partiql-planner) frequently copies and materializes data, has no projection pushdown support due to its data model, and pays high per-row overhead. This is misaligned with our primary workload: streaming, per-row simple SFW queries. The near-term goal is to deliver a streaming-first engine with minimal per-row overhead, explicit zero-copy contracts, and projection pushdown. At the same time, we want to hide engine internals behind a stable interface so we can later add analytical optimizations (batch/vectorized execution) without exposing internal changes to callers.

This design proposes a streaming-first execution engine that fuses Scan/Filter/Project/Limit into a single pipeline runner, uses a compact scalar bytecode VM for expressions, and defines a strict reader contract to minimize copies. Batch execution is explicitly deferred to a future extension (Appendix A).

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

### 5.2 Plan Instance

Each execution creates a `PlanInstance` that owns all mutable state (operators, arenas, readers). Specs map 1:1 to runtime operators by index.

```rust
struct PlanInstance {
    compiled: Arc<CompiledPlan>,
    operators: Vec<RelOp>,
    arenas: ExecArenaSet,
}
```

### 5.3 Relational Operators

A streaming relational operator produces output rows without requiring unbounded buffering. Examples include Scan, Filter, Project, and Limit. A blocking relational operator must retain input (or significant state) before producing output, such as HashJoin, HashAgg, and Sort.

Operators exchange data through `RowFrame` values. A parent calls `next_row` on its child, passing a mutable `RowFrame` that is filled by the child. Rows are pointer-sized slot carriers, so passing between operators is typically just slot assignment and does not require copying.

```rust
pub type SlotId = u16;

pub enum SlotValue<'a> {
    Val(ValueRef<'a>),
    Owned(&'a ValueOwned),
}

pub struct RowFrame<'a> {
    pub slots: &'a mut [SlotValue<'a>],
    pub arena: &'a mut RowArena,
}
```

`RelOp` is the unified runtime operator enum. It exposes a single `next_row` entrypoint, with variants for streaming pipelines and blocking operators.

```rust
enum RelOp {
    Pipeline(PipelineOp),
    HashJoin(HashJoinState),
    HashAgg(HashAggState),
    Sort(SortState),
    Custom(Box<dyn BlockingOperator>),
}

impl RelOp {
    fn next_row(&mut self, layout: &ScanLayout, out: &mut RowFrame) -> Result<bool> {
        match self {
            RelOp::Pipeline(op) => op.next_row(layout, out),
            RelOp::HashJoin(op) => op.next_row(out),
            RelOp::HashAgg(op) => op.next_row(out),
            RelOp::Sort(op) => op.next_row(out),
            RelOp::Custom(op) => op.next_row(out),
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
    fn next_row(&mut self, layout: &ScanLayout, out: &mut RowFrame) -> Result<bool> {
        loop {
            if !self.reader.read_next(layout, out)? {
                return Ok(false);
            }
            if self.run_steps(out)? {
                return Ok(true);
            }
        }
    }

    fn run_steps(&self, frame: &mut RowFrame) -> Result<bool> {
        for step in &self.steps {
            if !step.eval(frame)? {
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
    fn eval(&mut self, frame: &mut RowFrame) -> Result<bool> {
        match self {
            Step::Filter { program } => eval_row(program, frame),
            Step::Project { program } => {
                eval_row(program, frame)?;
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

Blocking operators (HashJoin, HashAgg, Sort) remain classic stateful operators and must respect buffer stability. They are direct variants of `RelOp` for performance, with an optional dynamic variant for customer-provided implementations. Because they retain data beyond a single pull, they may need to materialize values, which limits zero-copy benefits.

Materialization policy for buffering operators:

```rust
fn store_build_row(row: &RowFrame, caps: &ReaderCaps, arena: &mut ExecArena) {
    match caps.stability {
        BufferStability::UntilClose => store_borrowed(row),
        BufferStability::UntilNext => store_owned_copy(row, arena),
    }
}
```

### 5.4 Data Source Readers

Readers are third-party data providers. Similar to the experimental `BatchReader`, they are configured with a fixed `ScanLayout` (projection) before reading. They must declare buffer stability and capabilities, and they must honor the borrowing rule (valid until the next pull).

```rust
pub enum BufferStability {
    UntilNext,
    UntilClose,
}

pub struct ReaderCaps {
    pub stability: BufferStability,
    pub can_project: bool,
    pub can_return_opaque: bool,
}

pub trait RowReader {
    fn caps(&self) -> ReaderCaps;

    fn set_projection(&mut self, layout: ScanLayout) -> anyhow::Result<()>;

    fn open(&mut self) -> anyhow::Result<()>;

    fn next_row<'a>(&'a mut self, out: &mut RowFrame<'a>) -> anyhow::Result<bool>;

    fn resolve(&self, field_name: &str) -> Option<ScanSource>;

    fn close(&mut self) -> anyhow::Result<()>;
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

The execution model uses borrowed values for zero-copy and a strict ownership tiering model. Borrowed values are never owned and must not be retained beyond their validity window.

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

Row execution uses slot-based frames and a per-row arena for computed outputs. `RowArena` is necessary to avoid heap allocation per computed value: it provides a bump-allocated lifetime that matches a single row pull, allowing derived values (casts, arithmetic, string ops) to be stored without copying into long-lived buffers.

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
}

fn eval_row(program: &Program, frame: &mut RowFrame) -> Result<bool> {
    // Interpret bytecode over frame slots and registers.
    // Store results back into output slots.
    Ok(true)
}
```

Registers are modeled as `ValueRef` so they can point directly at borrowed inputs without copying. When an instruction produces a new value (e.g., arithmetic), it allocates into `RowArena` and stores a `ValueRef` to that owned value. This keeps register values uniform and avoids heap allocation per register.

An instruction set is expressed as a Rust enum for fast dispatch:

```rust
enum Inst {
    LoadSlot { dst: u16, slot: SlotId },
    ConstI64 { dst: u16, imm: i64 },
    AddI64 { dst: u16, a: u16, b: u16 },
    StoreSlot { slot: SlotId, src: u16 },
    CallUdf { dst: u16, func_id: u16, args: Vec<u16> },
}
```

Example evaluation of `AddI64` with slot references:

```rust
fn eval_inst(inst: &Inst, regs: &mut [ValueRef], frame: &mut RowFrame) -> Result<()> {
    match *inst {
        Inst::LoadSlot { dst, slot } => {
            regs[dst as usize] = match frame.slots[slot as usize] {
                SlotValue::Val(v) => v,
                SlotValue::Owned(v) => ValueRef::from_owned(v),
            };
        }
        Inst::AddI64 { dst, a, b } => {
            let av = regs[a as usize].as_i64()?;
            let bv = regs[b as usize].as_i64()?;
            let owned = frame.arena.alloc(ValueOwned::I64(av + bv));
            regs[dst as usize] = ValueRef::from_owned(owned);
        }
        Inst::StoreSlot { slot, src } => {
            frame.slots[slot as usize] = SlotValue::Val(regs[src as usize]);
        }
        _ => {}
    }
    Ok(())
}
```

The VM loads slot values into registers, computes over registers, and writes results back into slots or registers. Computed values are allocated in `RowArena` to keep lifetimes local to the row.

### 5.8 Consuming Results

Results are pulled from the root operator through a `ResultStream` returned by `PlanInstance::execute`. Borrowed outputs remain valid until the next pull on the same stream.

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
    instance: PlanInstance,
}

impl ResultStream {
    pub fn next_row(&mut self) -> Result<Option<RowView<'_>>> { /* ... */ }
}

impl PlanInstance {
    pub fn execute(self) -> Result<ResultStream> {
        Ok(ResultStream {
            schema: self.compiled.result_schema(),
            root: 0,
            instance: self,
        })
    }
}
```

Callers map column names to indices using the `ResultStream.schema`.

### 5.9 Public APIs

Public APIs expose views, not internal representations. Internal types such as `ValueRef`, arenas, and VM registers are not exposed.

- Public `Plan` or `PreparedQuery` API to compile plans from queries.
- Public `ResultStream` API returned by `PlanInstance::execute`, with `schema` metadata.
- Public `RowView` and `ValueView` types for zero-copy access to results, plus typed getters (`get_i64`, `get_str`) and `get_value` for owned materialization.
- Public owned conversion helpers (`to_owned`, serialization helpers) for callers that need retention.
- Internal-only types: `ValueRef`, `RowFrame`, arenas, VM registers, `RelOp`/operator states.

## 6. Performance Analysis

### 6.1 Benchmarking Methodology

Benchmark the streaming-first engine against the current implementation on:

- Streaming SFW queries (dominant workload).
- Early-exit queries with LIMIT.
- Basic joins/aggregations with projection pushdown.

Collect throughput (rows/sec), CPU utilization, allocation rates, and materialization counts.

### 6.2 Expected Results

We expect significant latency reductions and throughput gains for streaming SFW queries due to fewer copies, lower per-row overhead, and projection pushdown. Joins/aggregations improve primarily from reduced materialization and better layout planning.

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

The proof-of-concept vectorized engine demonstrated substantial gains for analytical workloads (hundreds of times faster), but degraded performance when the batch size was reduced to 1, likely due to per-batch overhead. This motivates a streaming-first focus for v1. By hiding internal details behind stable public APIs, we preserve the ability to add vectorized operators and readers later, as outlined in Appendix A.

The pipeline runner adds complexity but provides significant per-row savings. Zero-copy reduces memory traffic but depends on strict buffer lifetime guarantees.

## 8. Compatibility Strategy

Note: This section is optional and can be omitted if backward compatibility is handled elsewhere.

We need a low-friction path for existing customers using `partiql-eval` APIs (e.g., `EvalPlan::execute` returning a single `Value`). The compatibility strategy is to provide adapters that bridge the legacy `Value`-based evaluation into the new streaming engine without forcing immediate API migration.

Proposed approach:

- Legacy operator variant: add a `RelOp::LegacyEval(Box<dyn Evaluable>)` (and `RelOpSpec::LegacyEval`) that wraps a legacy `Evaluable` node. The wrapper materializes input `RowFrame` values into a `Value`, calls `evaluate`, then maps the `Value` result back into a row slot. This enables incremental adoption while keeping new pipelines intact.
- Value conversion helpers: add explicit adapters between `Value` and `ValueRef`/`ValueOwned`:
  - `Value` -> `ValueRef`: borrow when the `Value` is stable and owned by the caller; otherwise copy into a `RowArena` and return a `ValueRef` to the owned value.
  - `ValueRef` -> `ValueOwned`: materialize on demand for legacy evaluables or for API consumers that need owned retention.
- Result compatibility: the `ResultStream` can expose a convenience method that materializes the stream into a single `Value` for legacy callers. This preserves the legacy behavior of “one evaluated Value” while allowing new consumers to stream rows.

This keeps legacy APIs functional while allowing the new engine to be introduced behind the same public surface. Over time, legacy `Evaluable` nodes can be replaced by native `RelOp` variants without breaking callers.

## 9. Implementation Plan

### Phase 1: Foundations (Data Model + Plan Scaffolding)

- Define `ValueRef`, `ValueOwned`, and `RowArena` with clear lifetime rules.
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

## 11. Monitoring & Observability

Track per-operator rows/sec, materialization rate (copy vs borrow), and arena allocation behavior. Alert on regressions in throughput or sudden increases in materialization.

## 12. Risks & Mitigations

- Incorrect borrow lifetimes: enforce stability contract and add tests around reader validity.
- Performance regressions for small queries: optimize pipeline runner and keep row mode minimal.
- Complexity in planner/executor: keep boundaries explicit and limit scope in v1.

## 13. Open Questions

- Are there strict latency SLAs for streaming workloads that should bound per-row overhead?
- Which readers (JSON, Arrow, custom) should be first-class for projection pushdown?
- What data sizes and distributions are most representative of current streaming traffic?
- Graph execution (GPML): can we retain existing graph functionality via adapters around the legacy graph APIs without significant engineering effort? Further investigation is required; current customer usage of GPML is minimal.

## 14. FAQs

Q: What problem are we solving?
A: Reduce per-row overhead and excessive copying in the current engine, while enabling future analytical optimizations without exposing internal details.

Q: Why not adopt a full vectorized engine now?
A: The POC vectorized engine excelled for large analytical workloads but performed worse at batch size 1 due to overhead. Our dominant use case is streaming, so v1 is row-first.

Q: How does zero-copy actually work here?
A: Borrowed values flow through operators via `RowFrame` slots and remain valid until the next pull. Buffer stability contracts ensure operators only retain borrows when safe.

Q: Will this break existing customers?
A: Yes, however the compatibility strategy preserves legacy `partiql-eval` behavior via adapters and optional `RelOp::LegacyEval` wrappers. We will attempt to make it as easy as possible for users to upgrade.

Q: How do I know what the result means (table vs scalar)?
A: `ResultStream` exposes a schema and shape metadata so callers can tell whether results are a collection, struct, or scalar, and map columns accordingly.

Q: When will vectorized execution arrive?
A: It is deferred to a future phase (Appendix A) once the streaming-first core is stable.

## Appendix A. Future Batch/Vectorized Execution (Not in v1)

This appendix outlines how batch execution can be added later without changing external interfaces. The goal is to enable analytical workloads while keeping engine internals hidden.

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
