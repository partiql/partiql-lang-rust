# Future Roadmap: Batch and Vectorized Execution

**Status**: Not in v1 - Future Extension

This document details how vectorized execution integrates into the v1 architecture without modifying public APIs. The abstractions established in v1 are designed to enable this evolution.

## Strategic Context

As part of BDT (an analytical organization), the long-term roadmap includes high-throughput analytical workloads. Proof-of-concept experiments with vectorized execution demonstrated 150-300x performance improvements on analytical queries with columnar data formats. This roadmap outlines how to transparently integrate vectorized performance while maintaining compatibility with streaming workloads.

## Proof-of-Concept Results

The POC vectorized engine demonstrated substantial performance gains for analytical workloads:

- **Columnar data (Arrow/Parquet)**: 150-300x throughput improvement over row-at-a-time execution
- **Analytical queries** (filters, projections, aggregations): Near-linear scaling with data volume due to SIMD operations and cache efficiency
- **Degradation at batch_size=1**: 2-3x slower than pure row mode due to per-batch coordination overhead and selection vector management

**Analysis**: These results validate two requirements:
1. Vectorized execution provides transformational performance for analytical workloads (relevant to BDT's mission)
2. Row-mode execution is necessary for low-latency streaming workloads

Rather than selecting one execution model, the v1 design establishes abstractions that support both modes behind a unified API. The planner can detect data characteristics (columnar formats, large batch sizes) and select appropriate operators transparently.

## Evolution Path to Analytical Performance

### Phase 1 (v1 - Current): Streaming-First Foundation

- Row-mode execution with pipeline fusion for SELECT queries
- Reader abstraction supporting projection pushdown
- Public APIs independent of execution strategy
- Immutable compiled plans enabling caching and concurrent execution
- `ExecutionResult::Query` for SELECT statements

**Key Achievement**: Establishes abstraction boundaries that enable future vectorized performance without API changes.

**Deferred to Future Phases**:
- `ExecutionResult::Mutation` (INSERT, UPDATE, DELETE support)
- `ExecutionResult::Definition` (CREATE, DROP, DDL support)

### Phase 2 (Future): DML and DDL Support

**Goal**: Extend execution model to support data manipulation and definition statements.

**Features**:
- `ExecutionResult::Mutation` for INSERT, UPDATE, DELETE statements
  - Immediate execution with summary results (rows affected)
  - Integration with existing transactional semantics
- `ExecutionResult::Definition` for CREATE, DROP, ALTER statements
  - Schema modification operations
  - Metadata catalog integration

**API Consistency**: All statement types use the unified `PartiQLVM::execute()` interface, with polymorphic result handling through the `ExecutionResult` enum.

### Phase 3 (Future): Transparent Vectorized Execution

**Goal**: Enable vectorized performance for analytical workloads without requiring user code changes.

**Architecture Changes**:

1. **Batch Data Model**:
```rust
pub struct DataChunk<'a> {
    pub len: usize,
    pub sel: Option<&'a [u16]>,  // Selection vector for filtering
    pub cols: Vec<Column<'a>>,
}

pub enum Column<'a> {
    I64(Vec<Option<i64>>),
    F64(Vec<Option<f64>>),
    Bool(Vec<Option<bool>>),
    Str(Vec<Option<&'a str>>),
    Any(Vec<ValueRef<'a>>),  // Fallback for complex types
}
```

2. **BatchReader Implementations**:
- Arrow reader returning `DataChunk` batches
- Parquet reader with columnar data extraction
- Existing readers continue supporting row-at-a-time

3. **Vectorized Operator Variants**:
```rust
enum RelOp {
    Pipeline(PipelineOp),              // Row-mode (v1)
    VectorizedPipeline(VectorizedOp),  // Batch-mode (Phase 2)
    HashJoin(HashJoinState),
    BatchHashJoin(BatchHashJoinState), // Vectorized variant
    // ... other operators
}
```

4. **Compiler Strategy Selection**:
- Detect columnar input formats (Arrow, Parquet)
- Select vectorized operators for analytical queries
- Maintain row-mode operators for streaming queries
- Decision made at compile time based on data characteristics

**API Compatibility**:

The public `ResultStream` interface remains unchanged:
```rust
impl ResultStream {
    pub fn next_row(&mut self) -> Result<Option<RowView<'_>>>
}
```

Internal implementation may buffer batches and emit rows, or expose optional batch API:
```rust
impl ResultStream {
    pub fn next_batch(&mut self) -> Result<Option<BatchView<'_>>>  // Optional performance API
}
```

**Technical Feasibility**:

1. **Reader abstraction**: `RowReader` can wrap `BatchReader`, exposing row-at-a-time iteration over internal batches
2. **Operator polymorphism**: `RelOp::VectorizedPipeline` is an internal implementation variant invisible to callers
3. **Result abstraction**: `ResultStream` can buffer batches internally while maintaining row iteration semantics

**Migration Strategy**: No user code changes required. Queries automatically benefit from vectorized execution when operating on columnar data.

### Phase 4 (Future): Adaptive Execution

**Goal**: Dynamic strategy selection based on runtime characteristics.

**Features**:
- Runtime workload profiling
- Dynamic mode switching (row ↔ batch) within query execution
- Hybrid operators with runtime strategy selection
- Cost-based model for execution strategy selection

**Example Use Cases**:
- Start with vectorized execution, fall back to row-mode for selective filters
- Switch to batch mode when data volume exceeds threshold
- Maintain row-mode for low-latency requirements

## Batch Execution Model Details

### Typed Kernels for Common Operations

Vectorized execution leverages typed kernels optimized for specific data types:

```rust
// Example: Vectorized addition for i64 columns
fn add_i64_kernel(
    left: &[Option<i64>],
    right: &[Option<i64>],
    output: &mut [Option<i64>],
    sel: Option<&[u16]>
) {
    // SIMD-optimized addition with null handling
    // Selection vector for sparse computation
}
```

### Batch Pipeline Execution

Reuse the same scalar bytecode VM with a batch evaluator:

```rust
impl Program {
    fn eval_batch(
        &self,
        chunk: &mut DataChunk,
        arena: &Arena
    ) -> Result<()> {
        // Execute bytecode over entire batch
        // Use typed kernels where possible
        // Fall back to scalar evaluation for complex operations
    }
}
```

### Boundary Adapters

Insert explicit `Batcher`/`Unbatcher` adapters at execution strategy boundaries:

```rust
enum RelOp {
    Pipeline(PipelineOp),           // Row-mode
    VectorizedPipeline(VecOp),      // Batch-mode
    Batcher(BatcherOp),             // Row → Batch adapter
    Unbatcher(UnbatcherOp),         // Batch → Row adapter
}
```

This enables mixing execution strategies within a single query plan:
- Vectorized scan/filter/project over columnar data
- Unbatcher before row-mode user-defined functions
- Re-batcher before vectorized aggregation

## Planner Integration

### Static Mode Selection (Phase 2)

Compile-time strategy selection based on:
- Input data format (columnar → vectorized, row-oriented → row-mode)
- Query characteristics (analytical patterns → vectorized, simple SFW → row-mode)
- Operator capabilities (all operators support batching → vectorized pipeline)

```rust
impl Compiler {
    fn select_execution_strategy(&self, logical_plan: &LogicalPlan) -> Strategy {
        if self.has_columnar_input() && self.is_analytical_query() {
            Strategy::Vectorized
        } else {
            Strategy::RowMode
        }
    }
}
```

### Adaptive Mode Selection (Phase 3)

Runtime strategy switching based on:
- Observed selectivity (high selectivity → row-mode, low → batch-mode)
- Data volume (small batches → row-mode, large → batch-mode)
- Performance metrics (actual throughput vs expected)

## Performance Expectations

Based on POC results, expected improvements for analytical workloads:

- **Columnar scans**: 100-200x improvement (Arrow/Parquet direct column access)
- **Filters on primitives**: 50-100x improvement (SIMD operations, selection vectors)
- **Aggregations**: 150-300x improvement (columnar processing, cache efficiency)
- **Complex projections**: 10-50x improvement (reduced per-row overhead)

Row-mode execution remains optimal for:
- Low-latency streaming queries (< 1ms per query)
- Highly selective filters (< 1% of rows pass)
- Small result sets (< 100 rows)

## Migration Timeline

**Phase 2 Prerequisites** (DML/DDL Support):
- v1 streaming SELECT engine stable and deployed
- Transactional semantics defined
- Catalog integration design complete

**Phase 2 Deliverables**:
- Mutation result type and DML operators
- Definition result type and DDL operators
- Catalog modification framework

**Phase 3 Prerequisites** (Vectorized Execution):
- v1 streaming engine stable and deployed
- Comprehensive benchmarking suite validated
- Reader contract proven with multiple implementations

**Phase 3 Deliverables**:
- `BatchReader` trait and implementations (Arrow, Parquet)
- Vectorized pipeline operators
- Typed kernel library for common operations
- Compiler integration for strategy selection

**Phase 4 Prerequisites** (Adaptive Execution):
- Phase 3 vectorized execution deployed and validated
- Performance profiling infrastructure in place
- Runtime metrics collection framework

**Phase 4 Deliverables**:
- Adaptive execution framework
- Runtime strategy switching
- Cost-based execution model
- Hybrid operator implementations

## Open Questions for Future Phases

1. **Batch Size Selection**: What heuristics should determine optimal batch size for different workloads?
2. **Memory Management**: Should batch execution use separate memory pools or reuse the VM arena?
3. **UDF Integration**: How should user-defined functions interact with vectorized execution?
4. **Incremental Adoption**: What phasing strategy allows gradual rollout of vectorized operators?
5. **Monitoring**: What metrics should track vectorized vs row-mode execution in production?

## Strategic Value

This evolution path enables:
- **Immediate value**: v1 addresses streaming performance issues for existing customers
- **Future capability**: Transparent access to 150-300x analytical performance improvements
- **Unified codebase**: Single engine serving both streaming and analytical workloads
- **No breaking changes**: Public APIs remain stable throughout evolution

The abstraction strategy preserves architectural options while delivering immediate performance improvements, aligning with both current customer needs and long-term organizational goals.
