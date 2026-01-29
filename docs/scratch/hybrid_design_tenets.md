# Hybrid Query Engine Design — Core Tenets & Ownership Model

This document captures the **core design tenets**, **ownership rules**, and **API contracts**
for the hybrid row + vectorized SQL++ query engine.  
It is intended as a *normative reference* for contributors and reviewers.

It complements:
- `hybrid_design.md`
- `hybrid_design_addendum.md`

---

## 1. Core Tenets (Non‑negotiable)

### T1) Spec vs State Separation
All engine components are split into:

**Compiled / Spec (immutable, cacheable, shareable)**
- Scalar bytecode programs
- Operator specifications (PipelineSpec, HashJoinSpec, etc.)
- Layouts and scan layouts
- CompiledPlan

**Runtime / State (mutable, per‑thread, per‑execution)**
- Operator states (scan cursors, join tables, agg maps)
- Scratch row frames and data chunks
- Arenas
- Reader instances

Compiled artifacts are `Arc`‑backed, `Send + Sync`, and reusable across threads.
Runtime state is never shared.

---

### T2) One Simple Borrowing Rule
Any borrowed data returned by the engine (rows, batches, strings, bytes, object views)
is valid:

> **Until the next call to `next_row()` / `next_batch()` on the same stream.**

If a consumer needs to keep data longer, it must explicitly materialize it.

---

### T3) Copying Is Explicit and Localized
- Streaming execution prefers pointer‑sized aliasing (zero‑copy).
- Allocation happens only when unavoidable:
  - computed expressions
  - buffering operators (join / group by / sort)
  - mode adapters (row↔batch)
- Consumers opt into copying via `to_owned()` or serialization helpers.

---

### T4) Buffer Stability Is a First‑Class Capability
All data producers declare buffer stability:

- `UntilNext` — borrowed values invalid after next read
- `UntilClose` — borrowed values valid for the operator lifetime

Buffering operators must **never** retain ephemeral borrows.

---

### T5) Public APIs Expose Views, Not Representations
Internal types (`ValueRef`, arenas, VM registers) are not exposed publicly.

Consumers interact through:
- `RowView<'a>` / `BatchView<'a>`
- `ValueView<'a>` (borrowed)
- Optional owned conversion helpers

This preserves zero‑copy while allowing internal evolution.

---

### T6) Gradual Typing Without Polluting Fast Paths
- Typed slots/columns are used whenever possible.
- Untyped or complex values flow through `Any` lanes (`ValueRef`).
- The planner decides projection and pushdown; execution supports both paths.

---

## 2. Ownership Tiers (Mental Model)

Every value observed during execution belongs to exactly one tier:

### Tier A — Borrowed from Input (Zero‑copy)
- `&str`, `&[u8]`
- object/array views
- opaque user data

Validity depends on upstream buffer stability.

---

### Tier B — Borrowed from an Arena
- Computed results allocated in a bump arena
- Borrowed, but not input‑owned

Validity lasts until the arena is reset.

---

### Tier C — Owned / Persistent
- Keys and payloads stored by buffering operators
- Aggregation state
- Join build tables
- Sort buffers

Validity lasts until operator close or PlanInstance drop.

---

## 3. Core Internal Types and Contracts

### 3.1 `ValueRef<'a>` (internal only)
**Purpose:** Represent any borrowed value without copying.

**Rules:**
- Never owns memory
- May reference reader buffers, arenas, or opaque data
- Must not be stored beyond its validity window

---

### 3.2 `ValueOwned` (internal only)
**Purpose:** Represent materialized values.

**Rules:**
- Stored in arenas or owned containers
- Lifetime defined by the owning arena or operator
- Referenced via borrows in row/batch carriers

---

### 3.3 Arenas

#### `RowArena`
- Reset once per produced row
- Used for computed scalar outputs
- Never referenced by buffering operators

#### `ChunkArena`
- Reset once per batch
- Used for computed batch columns

#### `ExecArena`
- Lives for operator execution
- Used by joins, aggregations, sorts
- Stores persistent state

**Invariant:**  
Arena lifetime must outlive all borrows derived from it.

---

### 3.4 Row and Batch Carriers

#### RowFrame
- Holds slot values (scalars, borrows, arena references)
- Does not own underlying data

#### DataChunk
- Holds columns (typed or Any)
- Columns may borrow or be arena‑owned

---

### 3.5 Scalar VM

**Program (Spec)**
- Immutable bytecode + constants
- Cacheable and shareable

**Execution**
- Per‑call registers and scratch
- Reads from carriers, writes to carriers/arenas
- Never stores references across calls

---

## 4. Operators

### 4.1 Pipeline Operators (Streaming)
- Represent fused scan/filter/project chains
- Executed by a runner (step list)
- Can run in row or batch mode
- Own scratch buffers and ephemeral arenas

**Guarantee:**  
Produced rows/batches follow the standard borrowing rule.

---

### 4.2 Blocking Operators (Join / Agg / Sort)
- Own internal state machines
- May buffer input
- Must materialize ephemeral data unless stability is `UntilClose`

**Guarantee:**  
Output values are valid until the next pull, unless explicitly documented otherwise.

---

## 5. Readers

Readers are third‑party data providers.

**Required declarations:**
- buffer stability
- batch support
- projection capability

**Rules:**
- May return borrowed values
- Must honor declared stability
- Must not assume values are retained by the engine

---

## 6. Public Result Consumption API

### Public Views
- `ResultStream` / `ResultBatchStream`
- `RowView<'a>` / `BatchView<'a>`
- `ValueView<'a>`

### Guarantees
- Borrowed validity: until next call
- No exposure of arenas or VM internals
- Explicit escape hatch to owned values

---

## 7. Adapter Boundaries

Row↔Batch adapters exist only at explicit plan boundaries:
- around joins
- before row‑only consumers
- when sources cannot natively batch

This makes copying visible and predictable.

---

## 8. Do / Don’t Rules

### DO
- Reset arenas deterministically
- Keep compiled plans immutable
- Use explicit adapters
- Enforce buffer stability checks

### DON’T
- Store ephemeral borrows in buffered state
- Expose internal value representations
- Allocate per row on the heap
- Allow implicit lifetime extension

---

## 9. Canonical Terminology

Use these terms consistently:
- **Borrowed output** — valid until next pull
- **Stable borrowed output** — valid until close
- **Materialized** — copied into owned memory
- **Ephemeral arena** — reset every row/batch
- **Execution arena** — lives for operator execution

---

## 10. Summary

This ownership model:
- enables zero‑copy where possible
- keeps lifetimes predictable
- allows plan caching and parallel execution
- prevents accidental retention of invalid borrows
- supports gradual typing and hybrid execution

It is the foundation that keeps the engine fast, safe, and evolvable.
