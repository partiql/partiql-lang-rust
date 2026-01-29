# SQL++ Query Engine Design (Rust) — Hybrid Row + Vectorized, Zero-Copy, Scalar Bytecode VM

## High-level overview

This design describes a SQL++ query engine in Rust that:

- Evaluates queries **row-by-row** via a Volcano-style pull interface.
- Also supports **vectorized (batch) execution** for high throughput on large inputs.
- Uses a **register-based bytecode VM for scalar expressions** in both row and batch execution.
- Preserves **zero-copy** data flow where possible by passing borrowed views from readers through streaming operators.

Relational execution is implemented as **Rust operator states** (not a relational VM). Hybrid behavior is handled by a **pipeline executor** that chooses row vs batch per pipeline segment and inserts **Batcher/Unbatcher adapters** when mode conversions are needed.

## Design decisions

1. **Only scalar expressions use bytecode.** Relational operators are Rust structs with row and/or batch entrypoints.
2. **Hybrid mode selection happens at the pipeline level.** Streaming chains like Scan+Filter+Project can be fused and run in one mode.
3. **Zero-copy by default, copy only at necessary boundaries.** Streaming operators alias references; buffering operators materialize only what they must retain.
4. **Gradual typing is first-class.** Fast typed columns/slots are used when known; a generic `Any` representation is used when unknown.
5. **Projection pushdown is a contract between planner and reader.** A `ScanLayout` describes which fields to produce and how.

## Architecture

### Components

- **Planner/Compiler**
  - Builds a physical operator graph.
  - Computes layouts (slot/column mapping) per operator.
  - Compiles scalar expressions into VM programs.
  - Generates `ScanLayout` for sources (projection pushdown + type hints).

- **Pipeline executor**
  - Identifies pipeline segments (maximal streaming chains without blocking operators).
  - Chooses row or batch mode per segment.
  - Inserts adapters for row/batch boundary crossings.

- **Relational operators**
  - Implement streaming transforms (Scan, Filter, Project, Limit) and stateful operators (HashJoin, HashAgg, Sort).
  - May support row, batch, or both.
  - Own internal state (hash tables, aggregate states, sorter buffers, spill state).

- **Scalar VM**
  - Register-based bytecode for expressions.
  - Two evaluators: row evaluation and batch evaluation.

## Data model

### Borrowed values for zero-copy: `ValueRef<'a>`

This represents values without copying, including SQL++ concepts like `Missing`.

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

### Object/array views

Readers expose objects and arrays via view traits so navigation can be zero-copy.

```rust
pub trait ObjectView<'a> {
    fn get(&self, key: &str) -> ValueRef<'a>;          // Missing if absent
    fn get_path(&self, path: &[&str]) -> ValueRef<'a>; // optional convenience
}

pub struct ObjRef<'a>(pub &'a dyn ObjectView<'a>);
```

This supports multiple backings:
- Arrow/columnar: constant-time column lookup.
- Custom structs: field access without serialization.
- JSON text: start simple; later upgrade to an indexed/tape representation to return borrowed slices.

### Owned values and arenas

Computed results and buffered values live in owned memory. Use arenas to avoid per-value heap churn.

```rust
pub enum ValueOwned {
    Null,
    Bool(bool),
    I64(i64),
    F64(f64),
    Str(String),
    Bytes(Vec<u8>),
}

pub struct RowArena { /* bump allocator */ }
impl RowArena {
    pub fn reset(&mut self) {}
    pub fn alloc(&mut self, v: ValueOwned) -> &ValueOwned { todo!() }
}
```

## Row mode representation

Rows are represented as integer-indexed slots.

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

Passing rows between streaming operators is typically just copying `SlotValue` (pointer-sized), which is zero-copy for borrowed data.

## Batch (vectorized) representation

A batch is a `DataChunk` containing columns (lanes). Lanes can be typed (fast path) or `Any` (`ValueRef`) for gradual typing.

```rust
pub struct DataChunk<'a> {
    pub len: usize,
    pub sel: Option<&'a [u16]>,
    pub cols: Vec<Column<'a>>,
}

pub enum Column<'a> {
    I64(ColI64<'a>),
    F64(ColF64<'a>),
    Bool(ColBool<'a>),
    Str(ColStr<'a>),
    Any(ColAny<'a>),
}

pub struct ColAny<'a> {
    pub vals: Vec<ValueRef<'a>>,
}
```

Zero-copy in batch mode is maximized when columns borrow from stable backing storage (Arrow buffers, mmap files, or reader-owned buffers with documented stability).

## Reader APIs

### Buffer stability

Readers declare how long borrows remain valid.

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
```

This is essential for buffering operators (join/agg/sort) to decide whether they can keep borrowed references or must materialize.

### ScanLayout (projection pushdown)

The planner describes what the scan should output.

```rust
pub struct ScanLayout {
    pub base: Option<SlotId>,
    pub projections: Vec<(PathId, SlotId, TypeHint)>,
}
```

- `base` is a slot that can carry the whole row/document as `Obj` or `Opaque` for late materialization.
- `projections` asks the reader to produce specific fields directly into slots/columns when possible.

### Row reader

```rust
pub trait RowReader {
    fn caps(&self) -> ReaderCaps;

    fn read_next<'a>(
        &'a mut self,
        layout: &ScanLayout,
        out: &mut RowFrame<'a>,
    ) -> anyhow::Result<bool>;
}
```

### Batch reader

```rust
pub trait BatchReader {
    fn caps(&self) -> ReaderCaps;

    fn read_next_batch<'a>(
        &'a mut self,
        layout: &ScanLayout,
        out: &mut DataChunk<'a>,
        max_rows: usize,
    ) -> anyhow::Result<bool>;
}
```

Readers may implement one or both. If only row is provided, the engine can adapt with a `Batcher`.

## Relational operators

### Operator interface

Operators can support row mode, batch mode, or both.

```rust
pub enum ExecMode { Row, Batch }

pub trait Operator {
    fn open(&mut self) -> anyhow::Result<()>;
    fn close(&mut self) -> anyhow::Result<()>;

    fn supports_row(&self) -> bool;
    fn supports_batch(&self) -> bool;

    fn next_row(&mut self, out: &mut RowFrame) -> anyhow::Result<bool>;
    fn next_batch(&mut self, out: &mut DataChunk, max: usize) -> anyhow::Result<bool>;
}
```

### Are there vectorized relational operators as well as per-row relational operators?

Yes, in three forms:

1. **Row-only operators** (early prototypes or inherently row-ish operations).
2. **Dual-mode streaming operators**: Scan, Filter, Project, Limit. These are the primary targets for vectorization.
3. **Stateful/buffering operators** that often consume batches internally even if they output rows:
   - HashJoin: batch key evaluation + probing loop; can emit rows or batches.
   - HashAgg: batch updates into group states; emit results at end.
   - Sort: batch input, potentially external sorting/spill, emit results.

## Pipeline executor and hybrid mode selection

The executor:

- Identifies pipeline segments (streaming chains between blocking operators).
- Chooses row vs batch mode for each segment based on:
  - consumer behavior (early exit, small LIMIT, interactive streaming)
  - reader capabilities (native batch sources such as Arrow/Parquet)
  - batch fill rate (if batches are tiny, prefer row)
  - operator capabilities (only run batch if all operators in the segment support it)

### Mode adapters

- **Batcher (row to batch)**: collects rows from an upstream row-only operator until it fills a chunk (or hits EOF).
- **Unbatcher (batch to row)**: iterates within a held chunk to satisfy row-oriented consumers.

These adapters localize hybrid complexity and allow most operators to remain simple.

## Scalar expression VM

### VM programs

Expressions are compiled to register-based bytecode referencing:
- slots/columns
- constants
- interned field keys
- builtin function ids

The VM is mostly stateless between evaluations. Temporary allocations go into per-row or per-chunk arenas.

```rust
pub struct Program {
    pub insts: Vec<Inst>,
    pub consts: Vec<ValueOwned>,
    pub keys: Vec<String>,
}

pub struct Vm<'vm> {
    pub program: &'vm Program,
    pub builtins: &'vm Builtins,
}
```

### Row evaluation

Row evaluation loads from `RowFrame` slots into registers and writes results back to slots or registers.

### Batch evaluation

Batch evaluation uses the same logical bytecode but operates on lanes:

- Initial implementation: interpret per-row inside a tight loop over the chunk using the row interpreter.
- Optimized implementation: add typed batched kernels for common operations (comparisons, boolean ops, arithmetic, simple casts) to reduce per-row dispatch.

## Zero-copy rules and buffering operators

Streaming operators (scan/filter/project/limit) can pass borrowed `ValueRef` downstream because those borrows only need to remain valid until the next pull.

Buffering operators must not keep ephemeral borrows unless the reader declares `BufferStability::UntilClose`.

### HashJoin storage policy

- If build-side data is stable until close: store borrowed references or row ids into the stable backing.
- Otherwise: materialize only required fields (keys + payload projection) into an owned arena and store those.

### HashAgg and Sort

- HashAgg stores distinct keys and aggregate state; it should copy only distinct keys when stability is not guaranteed.
- Sort stores sort keys and either row ids (stable backing) or a materialized projection (ephemeral backing), and may spill to disk.

## Examples

### Example 1: Streaming query with early exit (row mode)

```sql
SELECT t.user_id
FROM t
WHERE t.country = "US"
LIMIT 1;
```

Plan:
- Fused `Pipeline(Scan + Filter + Project)` in row mode.
- Predicate and projection are scalar VM programs.

Why row mode:
- early exit makes batching overhead dominate

### Example 2: Large scan/filter/project (batch mode)

```sql
SELECT t.user_id, t.order.total
FROM t
WHERE t.order.total > 100;
```

Plan:
- Fused pipeline in batch mode.
- Reader provides projected columns if possible; otherwise a batcher collects rows.
- Filter produces selection vector; project outputs columns.

### Example 3: GROUP BY boundary (buffering)

```sql
SELECT t.country, COUNT(*)
FROM t
GROUP BY t.country;
```

Plan:
- Batch pipeline into HashAgg.
- HashAgg consumes batches; key evaluation uses batch evaluation of the scalar VM.
- Key storage is borrowed only if reader stability is `UntilClose`, otherwise keys are interned/copied once per distinct group.

## Suggested implementation stages

1. Row mode engine with scalar bytecode VM.
2. Batch types (`DataChunk`) + Batcher/Unbatcher adapters.
3. Batch execution for Scan+Filter+Project (fused pipeline), with batch evaluation implemented as a row-loop initially.
4. Add typed batched kernels for hot scalar ops.
5. Batch-friendly HashAgg and HashJoin (batch key evaluation, internal loops).
6. Sort/external sort and spilling if needed.


## Data model

### Borrowed values: `ValueRef<'a>`

The core zero-copy representation is a borrowed value enum that can reference reader-owned memory.

```rust
pub enum ValueRef<'a> {
    Missing, // SQL++ missing
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

#### Objects and arrays as views (not DOMs)

To avoid copying/parsing into an owned tree, objects and arrays are exposed as “views” with lazy field/index access.

```rust
pub trait ObjectView<'a> {
    fn get(&self, key: &str) -> ValueRef<'a>; // returns Missing if absent
    fn get_path(&self, path: &[&str]) -> ValueRef<'a> { /* default impl */ }
}

pub struct ObjRef<'a>(pub &'a dyn ObjectView<'a>);
```

`OpaqueRef` is an escape hatch for customer-defined representations (e.g., pointer + vtable), letting the engine carry complex structures without unpacking.

### Owned values: `ValueOwned`

Any computed value (concat, casts, arithmetic results), and any value that must survive beyond the reader’s “borrow window,” is stored in an owned representation. Allocation is typically done via a bump arena.

```rust
pub enum ValueOwned {
    Null,
    Bool(bool),
    I64(i64),
    F64(f64),
    Str(String),
    Bytes(Vec<u8>),
}

pub struct RowArena { /* bump allocator */ }
impl RowArena {
    pub fn reset(&mut self) {}
    pub fn alloc(&mut self, v: ValueOwned) -> &ValueOwned { /* ... */ }
}
```

## Row execution model

### Slot-based frames

Rows are represented as a fixed set of integer-indexed slots.

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

Passing a row through a streaming operator is typically pointer-sized slot assignment (zero-copy), for example `out.slots[o] = in.slots[i]`.

## Batch execution model

Batches are represented as `DataChunk` values containing a row count and a set of columns (lanes). Columns can be typed (fast) or `Any` (gradual typing).

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

Notes:
- The exact column layout is flexible. You can store validity bitmaps instead of `Option<T>`.
- `sel` (selection vector) lets filters avoid copying; downstream operators respect `sel`.

## Reader interfaces

Readers provide data to scans and are the primary zero-copy integration point.

### Buffer stability contract

```rust
pub enum BufferStability {
    UntilNext,  // borrows valid until the next read call
    UntilClose, // borrows valid until close (Arrow/mmap-like)
}

pub struct ReaderCaps {
    pub stability: BufferStability,
    pub can_project: bool,
    pub can_return_opaque: bool,
}
```

### Projection pushdown: `ScanLayout`

The planner passes a `ScanLayout` describing which paths to produce and where to place them.

```rust
pub struct ScanLayout {
    pub base_slot: Option<SlotId>,
    pub projections: Vec<(PathId, SlotId, TypeHint)>,
}
```

### Row reader

```rust
pub trait RowReader {
    fn caps(&self) -> ReaderCaps;

    fn read_next<'a>(
        &'a mut self,
        layout: &ScanLayout,
        out: &mut RowFrame<'a>,
    ) -> anyhow::Result<bool>;
}
```

### Batch reader

```rust
pub trait BatchReader {
    fn caps(&self) -> ReaderCaps;

    fn read_next_batch<'a>(
        &'a mut self,
        layout: &ScanLayout,
        out: &mut DataChunk<'a>,
        max_rows: usize,
    ) -> anyhow::Result<bool>;
}
```

A source may implement one or both traits. If only `RowReader` is available, the engine can insert a `Batcher` operator.

## Relational operators

Operators may support row mode, batch mode, or both.

```rust
pub enum ExecMode { Row, Batch }

pub trait Operator {
    fn open(&mut self) -> anyhow::Result<()>;
    fn close(&mut self) -> anyhow::Result<()>;

    fn supports_row(&self) -> bool;
    fn supports_batch(&self) -> bool;

    fn next_row(&mut self, out: &mut RowFrame) -> anyhow::Result<bool>;
    fn next_batch(&mut self, out: &mut DataChunk, max: usize) -> anyhow::Result<bool>;
}
```

### Vectorized vs per-row relational operators

- **Dual streaming operators**: Scan, Filter, Project, Limit should support both. These are the highest payoff for vectorization.
- **Stateful/buffering operators**: HashJoin, HashAgg, Sort can consume batches internally even if they output rows.
- **Row-only initial implementations** are acceptable for v1; batch support can be added incrementally.

### Adapters

- **Batcher** (row to batch): pulls rows from an upstream row operator, fills a `DataChunk`.
- **Unbatcher** (batch to row): holds one `DataChunk` and yields row views by indexing into columns.

These isolate mode transitions and keep hybrid complexity localized.

## Scalar expression VM

### Bytecode structure

Scalar expressions compile to a register-based instruction stream plus side tables (constants, interned keys, builtins).

Illustrative instructions:
- `LoadSlot dst, slot`
- `ConstI64 dst, imm`
- `Add dst, a, b`
- `Eq dst, a, b`
- `And dst, a, b`
- `GetField dst, obj_reg, key_id`
- `JumpIfFalse reg, target`
- `StoreSlot slot, src`
- `Return reg`

### VM state

The VM is largely stateless per evaluation aside from:
- program (instructions + constants + interned key table)
- builtin/UDF registry
- an arena for temporaries (row arena or chunk arena)

```rust
pub struct Program {
    pub insts: Vec<Inst>,
    pub consts: Vec<ValueOwned>,
    pub keys: Vec<String>,
}

pub struct Vm<'vm> {
    pub program: &'vm Program,
    pub builtins: &'vm Builtins,
}
```

### Row and batch evaluation

- `eval_row(vm, frame, regs)` evaluates once for a row.
- `eval_batch(vm, chunk, regs_or_cols)` evaluates over a chunk.

Recommended progression:
1. v1 batch eval runs the row VM in a tight loop over `chunk.len` rows (still a big win because relational overhead is reduced).
2. v2 introduces typed batched kernels for common opcodes (comparisons, boolean ops, numeric ops).

## Zero-copy rules and buffering boundaries

### Streaming operators

Filter/Project/Limit can remain zero-copy by aliasing borrowed values in slots/columns. Their outputs remain valid until the next call to the upstream operator, matching the reader/operator stability contract.

### Buffering operators

Operators that retain data beyond a single pull call must not keep ephemeral borrows unless the source stability is `UntilClose`.

- **HashAgg**: stores group keys (interned or owned) and aggregate state. Only distinct keys are copied when needed.
- **HashJoin**: build side must be stored (keys + required payload projection). If stable, can store row ids or borrows.
- **Sort**: stores sort keys + output projection (or row ids if stable) and may spill.

## Examples

### Example 1: Streaming query with early exit (row mode)

Query:

```sql
SELECT t.user_id
FROM t
WHERE t.country = "US"
LIMIT 1;
```

Plan:
- Fused pipeline: `Scan + Filter + Project + Limit` in row mode.
- Predicate and projection compiled to scalar bytecode.

Why row mode:
- batch fill overhead dominates when only a few rows are needed.

### Example 2: Large scan (batch mode)

Query:

```sql
SELECT t.user_id, t.order.total
FROM t
WHERE t.order.total > 100;
```

Plan:
- Fused pipeline in batch mode.
- If the reader can project `order.total` as a typed `F64` column, filtering uses a fast typed lane.
- Otherwise, filtering uses `Any` values and `GetField` from a base object lane.

### Example 3: Group by boundary (materialization)

Query:

```sql
SELECT t.country, COUNT(*)
FROM t
GROUP BY t.country;
```

Plan:
- Batch pipeline into `HashAgg`.
- Keys computed via bytecode. The group key is stored as:
  - a borrowed `&str` when stability is `UntilClose`, or
  - an interned owned string when stability is `UntilNext`.

## Implementation roadmap

- v1: row engine + scalar bytecode VM; fused Scan+Filter+Project; Batcher/Unbatcher; basic batch pipeline.
- v2: typed batched kernels for hot scalar ops; batch-native HashAgg update; batch probe for HashJoin.
- v3: spilling (sort/agg/join), more sophisticated type feedback, and optional micro-specialization in stateful operators.
