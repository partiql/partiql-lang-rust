# Performance Improvements Design Document

>Author: [John Quinn](https://quip-amazon.com/aKf9EACWmkk)
Review Date: 2026-01-26

## 1. Executive Summary

The current PartiQL Rust engine frequently copies and materializes data, lacks projection pushdown, and incurs high per-row overhead. This misaligns with our primary workload: streaming, per-row SFW queries. This design proposes a streaming-first execution engine that fuses Scan/Filter/Project/Limit into a single pipeline, uses a compact bytecode VM for expressions, and defines a strict reader contract to minimize copies.

## 2. Background & Context

**Current State**: Execution is collection-at-a-time via partiql-eval. Relational operators fully materialize collections before passing them to the next operator. The data model uses fragmented memory allocations: each collection is a Vec of Values, and each row/tuple is a Value containing a Vec of fields, resulting in extensive pointer indirection, scattered data, and poor cache locality. Projection pushdown is prevented because collections pass full Values (logical value = physical value) rather than indexable records/rows where specific fields may be selected.

**Business Impact**: Streaming workloads require more CPU per row than necessary, increasing latency and cost. Performance fixes often expose internal engine details to callers, limiting evolution paths for future analytical workloads.

## 3. Goals / Tenets

* **Reduce per-row overhead** for streaming SFW queries
* **Minimize materialization and preserve zero-copy** where safe
* **Enable projection pushdown** via a reader contract independent of internal operator implementation
* **Keep internals hidden** so future analytical optimizations can be added without public API changes
* **Maintain API stability** through opaque result types, polymorphic reader contracts, and immutable compiled plans that allow internal execution strategy changes without major breaking changes

## 4. Architecture Overview

The proposed architecture centers around the **PartiQLVM**, an opaque interpreter that manages a **single contiguous register array** and **memory arena**. Relational operators operate directly on VM-level registers, with row data residing in the first *n* slots of the VM’s register array. Scalar expressions compile to register-based bytecode that shares this same register array, using indices greater than *n* for temporary values during expression evaluation. This unified memory model eliminates data movement between storage layers—values flow from readers through relational operators to expression evaluation without copying between separate memory spaces.

Custom readers populate row data directly into register slots using a **ScanLayout** contract that enables projection pushdown at the data source. Readers declare a **BufferStability** contract (**UntilNext or UntilClose**) that governs when borrowed data remains valid, allowing operators to make informed decisions about materialization versus zero-copy preservation. The VM-owned arena provides centralized memory for computed intermediate values with O(1) reset between rows, while reader-owned buffers maintain input data independently. This separation of concerns, combined with borrowed value types and the register-based execution model, enables zero-copy data flow for simple projections and passthrough operations while allocating only computed results into the arena.
[Image: partiqlvm.png]The architecture follows a classic **Volcano-style model** with key innovations:

1. **Unified Register Array**: Slots [0..n] for relational row data + registers [n..] for expression temporaries, eliminating data movement between storage layers
2. **VM-Level Memory Management**: Single arena owned by VM rather than per-operator allocation, providing cache locality and O(1) reset
3. **Pipeline Fusion**: Scan/Filter/Project/Limit fused into single tight loop with native Rust enum dispatch. Discussed later.
4. **Zero-Copy Contracts**: Readers declare **buffer stability**, operators preserve borrowed values where possible
5. **Spec/State Separation**: Immutable compiled plans enable concurrent execution and caching
6. **First-Class Operator Implementations**: Internal operators handle common cases with static dispatch, delegating to customer-provided trait implementations only when necessary to minimize dynamic dispatch overhead

## 5. Implementation Details

### 5.1 Compiled Plan & Execution Context

**Compiled artifacts are immutable and reusable**. A compiled plan contains the operator graph, bytecode programs, and layouts. The plan is shared across threads (`Arc`, `Send + Sync`) and never mutated.

Each query execution leverages a `PartiQLVM`, a single-threaded virtual machine owning execution state. The VM encapsulates operators, a unified memory arena, and a reusable register array. The VM is designed for reusability—create once and execute multiple queries over its lifetime.

**Thread Safety**: `CompiledPlan` wrapped in `Arc` allows multiple VMs to share the same plan concurrently. Individual VMs are single-threaded but multiple VMs can execute the same plan on different threads.

### 5.2 Relational Operators

Operators are categorized as **streaming** (Scan, Filter, Project, Limit) or **blocking** (HashJoin, HashAgg, Sort). Streaming operators are fused into a `PipelineOp` that executes a tight loop over rows to minimize call overhead. Blocking operators remain as separate stateful operators.

Operators exchange data through the unified register array. A parent calls `next_row` on its child, passing arena reference (for computed value allocation) and register array (for row data). Row data resides in registers [0..slot_count], so passing between operators involves register value assignment without copying.

**Materialization Policy**: Blocking operators consider the reader's buffer stability when storing rows. For `UntilClose` stability, operators can store borrowed references directly. For `UntilNext` stability, operators must materialize to owned values.

### 5.3 Data Source Readers

Readers are third-party data providers that manage their own memory. They are configured with a fixed `ScanLayout` (projection) before reading and populate row data through a `ValueWriter` abstraction that encapsulates register access.

**ValueWriter Abstraction**: Readers interact with row slots through a `ValueWriter` struct rather than directly manipulating registers. This provides type-safe methods for writing values (`put_i64`, `put_str`, `put_bool`, etc.) while hiding internal `ValueRef` representation and register array details from reader implementations.
Readers are third-party data providers that manage their own memory. They are configured with a fixed `ScanLayout` (projection) before reading and populate row data directly into the unified register array (slots [0..slot_count]).

**Buffer Stability Contract**: Readers must declare `BufferStability` to communicate how long borrowed data remains valid. This contract enables operators to make informed decisions about when to materialize values versus preserving zero-copy borrows:

* `BufferStability::UntilNext`: Reader may reuse buffers on the next `next_row()` call. Borrowed references are invalidated when the reader produces the next row. Blocking operators must materialize these values to owned storage.
* `BufferStability::UntilClose`: Reader guarantees borrowed data remains valid until `close()` is called. This allows blocking operators to store direct references without materialization, enabling zero-copy for accumulation phases (e.g., HashJoin build side, HashAgg accumulation).

**Memory Management**: Readers are independent data sources and do not receive arena access. They manage their own storage for borrowed data (e.g., strings stored in reader-owned buffers). This design cleanly separates concerns: readers produce input data from external sources, while the arena is exclusively for computed intermediate values.

### 5.4 Projection Pushdown

Projection pushdown is enabled through `ScanLayout`, which explicitly captures a projection source, target slot index, and declared type. Target indices are contiguous from 0 to keep row layouts compact.

The reader contract (`RowReader` trait) supports both row-at-a-time and batch-capable implementations through the same interface. This abstraction layer decouples the execution model from data access patterns, enabling execution strategy selection at compile time.

### 5.5 Arena Memory Management

The Arena is a [bump allocator](https://docs.rs/bump-scope/latest/bump_scope/#what-is-bump-allocation) shared by all operators for intermediate values. It uses a contiguous buffer (default 16KB) with a single offset pointer that advances on each allocation—an
O(1) operation with zero fragmentation.

Between rows, arena.reset() performs an O(1) offset adjustment to reclaim all memory instantly. Due to the structured nature of SQL (Structured Query Language), data flow is tightly
coupled with memory allocation and de-allocation. This means we don't need a comprehensive garbage collector, as data can be fully flushed between iterations.

**What uses the Arena**:

* Expression results (concatenation, conversions)
* Intermediate evaluation values
* Constructed complex objects

**Benefits**:

* Cache locality from sequential allocation
* Predictable access patterns enable CPU prefetching
* Single-instruction cleanup via offset reset

**Note**: Readers manage their own storage independently—they own input data while the arena owns computed values.

### 5.6 Execution Data Model

The execution model uses borrowed values (internally `ValueRef`) where possible, with strict ownership tiering:

* Primitives (i64, f64, bool) stored directly in registers without allocation
* Strings and complex values reference reader-owned buffers **or** arena-allocated storage **or** memory-mapped files
* Borrowed values must not be retained beyond their validity window as defined by `BufferStability`

### 5.7 Scalar Bytecode VM

Scalar expressions compile to register-based bytecode. The VM is intentionally minimal and reused across all operators. Instructions are expressed as Rust enums for fast dispatch.

**VM Register Model**: Registers are allocated once at VM creation time, sized to the maximum register count across all programs. The register array is borrowed by expression evaluation, eliminating per-row heap allocations. This provides zero allocations per row, perfect cache locality, and zero-copy primitives.

Slot references compile directly to register indices. Since slots occupy registers [0..slot_count], a reference to slot 3 is simply register 3—no load instruction needed.

**Performance Impact**: Primitive operations (arithmetic, comparisons, logical ops) store results directly in registers without heap allocation. Only operations producing complex values (strings, objects, type conversions) allocate into the arena.

### 5.8 Result Streaming & Statement Types

The execution model will support multiple statement types (SELECT, INSERT, UPDATE, DELETE, CREATE, DROP) through a unified API. `execute()` returns an `ExecutionResult` enum:


* **Query**: SELECT statements return an iterator that streams results
* **Mutation**: DML statements return immediate summary (rows affected) - *future extension*
* **Definition**: DDL statements return immediate summary (objects created) - *future extension*

**Note**: This proposal focuses on SELECT queries. Mutation and Definition result types will be added in future phases as DML and DDL support is implemented.

**RAII Resource Management**: Operator resources (file handles, network connections) are managed automatically through the iterator's Drop implementation. Resources are lazily opened on first iteration and automatically closed when the iterator is dropped. This makes public consumption trivial.

**Execution Flow**: When iterating results, the VM arena resets (O(1)), operators execute allocating into the VM arena, and the result row is returned with borrowed references valid until the next iteration.

### 5.9 Public APIs

Public APIs are designed for execution strategy independence. Internal execution models (row-at-a-time vs vectorized) remain encapsulated.

**Public Surface**:

* `PartiQLCompiler` / `CompiledPlan`: Query compilation interface
* `Evaluable`: An implementation of a relational operator (legacy, for backwards-compatibility)
* `PartiQLVM`: PartiQL's virtual machine (interpreter)
    * We will need a way for users to provide a way to create relational operator implementations from custom plan nodes (this will be figured out during implementation)
* `ValueWriter`: Type-safe interface for readers to populate row data
* `ExecutionResult` (and variants): Execution result iterator
* `RowView` and `ValueView`: Zero-copy result accessors with typed getters
* Owned conversion helpers for serialization (and to/from legacy APIs)

**Internal-Only Types**: `ValueRef`, `RowFrame`, `RelOp`, `PipelineOp`, arenas, VM registers, bytecode structures

**Evolution Guarantee**: The public API design enables adding vectorized operators, batch readers, or adaptive execution without drastic user code changes. By hiding the VM’s internal details, a query executing row-at-a-time today can under-the-hood switch to vectorized execution when operating on appropriate data.

## 6. Performance Analysis

### Expected Results

Based on proof-of-concept testing, we observed approximately 40-100% latency reduction for streaming SFW queries with the VM model. Memory utilization is expected to decrease due to reduced copying and arena-based allocation.

**Performance Drivers**:

* Pipeline fusion eliminates per-operator virtual dispatch overhead
* Projection pushdown reduces data volume from readers
* Arena allocation amortizes allocation cost across rows
* Bytecode register-based VM avoids repeated AST traversal

### Benchmarking Methodology

Benchmark the streaming-first engine against the current implementation on:

* Streaming SFW queries (dominant workload)
* Using Criterion (Rust benchmarking crate)

### Scalability

Row-mode streaming pipelines scale linearly with input size. Blocking operators scale with input size but require careful materialization policy to manage memory usage.

## 7. Trade-Offs & Design Decisions

**Rejected Alternatives**:

* Pure vectorized engine: Rejected due to misalignment with streaming-first goals. With an opaque VM, we can add vectorization in the future.
* Pure row engine without structural changes: Rejected due to copying/materialization costs
* Relational bytecode VM: Rejected for higher complexity and longer roadmap

**Proof-of-Concept Vectorized Results**: The POC vectorized engine demonstrated 150-300x throughput improvement for columnar data (Arrow/Parquet) but degraded performance at batch_size=1 due to per-batch overhead. This validates two requirements: vectorized execution provides transformational performance for analytical workloads, and row-mode execution is necessary for low-latency streaming workloads.

Rather than selecting one execution model, this design establishes abstractions that support both modes behind a unified API. See the Future Roadmap document for evolution path details.

**Pipeline Complexity Trade-off**: The pipeline runner adds complexity but provides significant per-row savings. Reduced copying for passthrough operations lowers memory traffic, though computed values require allocation.

## 8. Compatibility Strategy

Existing customers use `partiql-eval` APIs. While we aim for compatibility where feasible, some breaking changes are expected. The compatibility strategy provides adapters for the most common use cases.

**Proposed Approach**:

* Dedicated (internal) relational operator variant to wrap existing `Evaluable` nodes
* Value conversion helpers between `Value` (legacy) and `ValueRef`
* `ResultStream` exposes convenience method to materialize stream into single `Value` for legacy callers

**Expected Breaking Changes**: API signatures may change, some legacy patterns may not map cleanly to streaming, and performance characteristics may differ. Migration documentation will be provided.

## 9. Implementation Plan

1. **Foundations**: Define data model (`ValueRef`, `Arena`), implement `RowFrame` and slot semantics, create operator scaffolding
2. **Reader Contracts**: Implement `ReaderCaps` and `BufferStability`, define `ScanLayout`, add row readers honoring projection
3. **Scalar Bytecode VM**: Define `Expr` enum, implement bytecode compilation and evaluator, integrate into Filter/Project
4. **Streaming Pipelines**: Implement `PipelineOp` runner, fuse Scan/Filter/Project/Limit, validate zero-copy behavior
5. **Blocking Operators**: Implement HashAgg/HashJoin/Sort with materialization policy
6. **Compatibility Layer** (Optional): Add legacy `Evaluable` wrapper, bridge `Value` inputs/outputs

## 10. Testing & Validation

* Microbenchmarks for streaming SFW with synthetic data
* End-to-end streaming queries on representative datasets (JSON, Arrow)
* Conformance tests for correctness parity with existing partiql-eval

## 11. Risks & Mitigations

* **Incorrect borrow lifetimes**: Enforce stability contract and add tests around reader validity
* **Performance regressions for small queries**: Optimize pipeline runner and keep row mode minimal
* **Complexity in planner/executor**: Keep boundaries explicit and limit scope in this proposal

## 12. Open Questions

* What should be the initial size of the arena? Should it be configurable via public API?
* Which readers (JSON, Arrow, custom) should be first-class for projection pushdown?
* What data sizes and distributions are most representative of current streaming traffic?
* Can GPML graph functionality be retained via adapters without significant engineering effort?
* Should readers have access to the arena (or a view of it) to enable more efficient allocation for non-memory-mappable file types? This could benefit readers that need to materialize string data but currently must manage separate buffers.
* Should `ValueWriter` provide batch methods (`put_i64s`) immediately, or defer until vectorized reader implementations exist?

## 13. FAQs

**Q1: What problem are we solving?**  
A: Reduce per-row overhead and excessive copying in the current engine, while enabling future analytical optimizations without exposing internal details.

**Q2: Why not adopt a full vectorized engine now?**  
A: The POC vectorized engine excelled for large analytical workloads but performed worse at batch size 1 due to overhead. Our dominant use case is streaming, so this proposal is row-first.

**Q3: How does zero-copy work in this design?**  
A: Simple projections that pass reader values directly to output slots avoid copying. Computed values (arithmetic, type conversions) require allocation into the arena. Buffer stability contracts ensure borrowed values remain valid appropriately.

**Q4: Why is API stability a primary design constraint?**  
A: This enables serving streaming workloads (existing customers) and analytical workloads (vectorized performance) from a unified codebase. Public APIs expose high-level abstractions rather than internal execution types, maintaining interface stability regardless of internal execution strategy. The POC demonstrated 150-300x improvements for analytical queries—the abstraction strategy preserves access to this performance without requiring code changes.

**Q5: How does the VM memory model work?**  
A: The VM owns two critical resources: (1) Arena for computed values with O(1) reset between rows, and (2) Pre-allocated register array reused across all rows. All operators share these resources through borrowed references, providing cache locality and eliminating per-row allocations.

**Q6: Why separate compiled plan from VM?**  
A: Compiled plans are immutable and shared (`Arc`, `Send + Sync`), enabling concurrent execution across threads. VMs are single-threaded execution contexts that borrow the plan, allowing multiple VMs to execute the same plan simultaneously while maintaining clean separation between cacheable artifacts and mutable state.

**Q7: Will this break existing customers?**  
A: Some breaking changes are expected as the execution model fundamentally differs. The compatibility strategy provides adapters for common use cases, but customers may need small code adjustments. Migration documentation will be provided.

**Q8: How do I know what the result means (table vs scalar)?**  
A: `ResultStream` exposes schema and shape metadata so callers can determine whether results are a collection, struct, or scalar, and map columns accordingly.

**Q9: When will vectorized execution arrive?**  
A: It is deferred to a future phase (see Future Roadmap document) once the streaming-first core is stable.
