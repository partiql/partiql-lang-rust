# Hybrid Query Engine Design — Addendum

This addendum summarizes key architectural findings and decisions around **cacheability**, **hybrid row/batch execution**, **pipeline runners**, and the interaction between **blocking** and **streaming** relational operators.

It is intended to complement `hybrid_design.md` and clarify *why* certain structures exist and *where* responsibilities live.

---

## 1. Cacheability and Multi-threaded Reuse

### 1.1 Spec vs State split (core principle)

The system is deliberately split into:

### Compiled, cacheable (immutable, thread-safe)
These objects can be safely reused across threads and executions:
- Scalar expression bytecode programs (`Program`)
- Operator specifications (`PipelineSpec`, `HashJoinSpec`, etc.)
- Layouts and scan layouts
- The full `CompiledPlan`

All compiled artifacts are:
- immutable
- stored behind `Arc`
- `Send + Sync`

They can be cached globally (e.g., plan cache) and reused by many threads.

### Runtime, per-thread/per-execution state
These objects **must not** be shared:
- Operator states (scan cursors, hash tables, sort buffers)
- Scratch row frames / data chunks
- Row and chunk arenas
- Reader instances

Each thread creates a fresh `PlanInstance` from the same `CompiledPlan`.

This gives:
- zero shared mutable state
- safe parallel execution
- cheap plan reuse

---

## 2. Why a Hybrid Row + Batch Design

### Observed behavior
- Vectorized execution is **very fast for large data**
- Vectorized execution is often **slower for single-row / early-exit queries**
- Pure row engines suffer overhead on large scans
- Pure vector engines suffer overhead on tiny inputs

### Goal
Support **both** execution styles in one engine:
- row-at-a-time for streaming, LIMIT, interactive queries
- batch/vectorized for scans, analytics, heavy filtering

---

## 3. Pipeline Operators vs Blocking Operators

### 3.1 Pipeline operators (runner-based)

**Pipeline operators** are *streaming*, non-blocking chains such as:
- Scan
- Filter
- Project
- Compute/Extend
- (sometimes) Limit

These operators:
- do not need to buffer unbounded input
- can be fused into a single execution unit
- benefit most from batch execution

In the hybrid design, they are compiled into a **PipelineOp**:
- internally uses a *runner* (step list)
- owns scratch buffers
- can run in **row mode** or **batch mode**
- exposes `next_row` and/or `next_batch` like a normal operator

**Key point**:  
Inside a pipeline, steps do **not call each other**.  
The runner executes them in a tight loop over a row or batch.

This:
- reduces nested `next()` calls
- improves cache locality
- makes hybrid switching easier
- simplifies selection-vector-based filtering

---

### 3.2 Blocking operators (classic Volcano-style)

**Blocking operators** include:
- HashJoin
- HashAgg (GROUP BY)
- Sort / Order By
- Window functions

These operators:
- require internal state machines
- often need to consume all or many input rows before producing output
- may emit multiple outputs per input
- often have build/probe or consume/produce phases

They remain **normal stateful operators**:
- they own their internal state
- they pull input from children via `next_row` / `next_batch`
- they may internally consume batches even if outputting rows

**Important**:
Blocking operators do *not* use the runner.
They treat `PipelineOp` as just another child operator.

This avoids turning the system into a full relational VM.

---

## 4. How the Runner Fits with Blocking Operators

A `PipelineOp` is itself an operator node.

Example plan shape:
```
HashJoin
  left:  PipelineOp (scan/filter/project, batch)
  right: PipelineOp (scan/filter/project, batch)
```

Interaction:
- HashJoin calls `child.next_row()` or `child.next_batch()`
- If the child is a pipeline, the pipeline runner executes
- No special scheduling or callbacks are required

This keeps:
- join logic self-contained
- pipelines fast
- the overall system understandable

---

## 5. Why the Runner Can Be Faster

### Benefits
- Fewer function calls in hot loops
- Fewer scratch buffers and less memory traffic
- Easier operator fusion (filter + project)
- Efficient selection-vector handling in batch mode
- Less dynamic dispatch

### But not always faster
The runner can be slower if:
- it forces row→batch conversions with copying
- batches are tiny (vector overhead dominates)
- data is ephemeral and cannot be safely borrowed
- the runner loop becomes overly branchy or abstract

**Conclusion**:  
The runner is ideal for *streaming pipelines*, not for everything.

---

## 6. Row vs Batch Mode Selection

### Compile-time choice (v1)
The compiler:
- decides row vs batch per pipeline
- inserts explicit adapters:
  - `RowToBatch`
  - `BatchToRow`

This keeps execution simple and predictable.

### Runtime adaptivity (optional future)
Later, the executor may:
- fall back to row mode for tiny batches
- switch to batch when enough rows accumulate
- adapt based on selectivity or LIMIT behavior

This can be added without changing the plan format.

---

## 7. Adapters and Boundaries

Adapters exist only at **mode boundaries**:
- before/after joins
- before row-only consumers
- when sources do not natively produce batches

They are explicit plan nodes, not implicit behavior.

This makes:
- performance costs visible
- plans easier to reason about
- zero-copy behavior more predictable

---

## 8. Zero-copy Guarantees and Limits

### Streaming pipelines
- Rows and batches can borrow directly from readers
- Slot/column passing is shallow (pointer-sized)
- Validity follows the reader contract:
  - `UntilNext` or `UntilClose`

### Blocking operators
- Must not store ephemeral borrows
- Must materialize:
  - keys
  - payload projections
  - or row identifiers
- Can avoid copying if reader stability is `UntilClose`

### Arenas
- `RowArena`: per-row temporary allocations
- `ChunkArena`: per-batch allocations
- `ExecArena`: long-lived allocations for buffering ops

Arenas allow allocation without heap churn while preserving safety.

---

## 9. Why Operators Don’t Reference Each Other Directly (Everywhere)

Pipeline steps do not reference each other because:
- they are executed by a runner
- this reduces call depth and overhead

Blocking operators *do* reference children (via NodeId dispatch):
- this keeps their state machines simple
- avoids building a relational scheduler/VM

This hybrid gives the best of both worlds.

---

## 10. Summary

- Compiled plans are immutable, cacheable, and thread-safe
- Execution state is per-thread and isolated
- Streaming operators are fused into runner-based pipelines
- Blocking operators remain classic stateful operators
- Hybrid row/batch execution is explicit and predictable
- Zero-copy is preserved wherever lifetimes allow
- The design scales from simple v1 to more adaptive future execution

This structure avoids premature complexity while leaving clear paths for optimization.
