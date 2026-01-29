# Hybrid Execution Approach: Solving Batch Size = 1 Performance

## Problem: Why Vectorized Performs Worse with Batch Size = 1

### Root Cause Analysis

When `BATCH_SIZE = 1`, the vectorized implementation underperforms the legacy implementation due to per-batch overhead dominating execution time.

#### Breakdown of Overhead (10,000 batches × 1 row each)

1. **Iterator Protocol Overhead**
   - 10,000 calls to `next_batch()`
   - 10,000 × Result unwrapping and error checking
   - 10,000 × batch ownership transfers

2. **Batch Infrastructure Overhead**
   - **Scratch vector management**: Resizing/recreating vectors on each batch
   - **Selection vector handling**: Checking for selection on every operation
   - **Phase separation**: Two-phase execution (compute → transfer to output)
   - **Memory operations**: Cloning Arc references for zero-copy transfers

3. **Lost Vectorization Benefits**
   - **SIMD operations**: Meaningless with 1 element per batch
   - **Cache locality**: Destroyed when processing 1 row at a time
   - **Amortized costs**: Can't amortize function call overhead over just 1 row

#### Legacy Implementation Advantage

The legacy evaluator processes all 10,000 rows as a single `Value::Bag`:

```rust
match plan.execute(&ctx) {
    Ok(evaluated) => {
        match evaluated.result {
            Value::Bag(bag) => {  // All rows processed at once
                non_vec_row_count = bag.len();
            }
        }
    }
}
```

This approach:
- Avoids all batching infrastructure overhead
- Single pass through the data
- No iterator protocol overhead

### Performance Crossover Point

**Vectorized becomes faster when:**
- Batch size ≥ 64-256 rows (enough for SIMD and cache efficiency)
- Number of operations per row is high (amortizes overhead)
- Data is stored in columnar format

---

## Solution: Hybrid Execution with Capability-Based Readers

### Architecture Overview

```
┌─────────────────────────────────────────────────────┐
│              DataSource Trait                        │
│  - capabilities() → supports vectorized/value mode   │
│  - execution_hints() → row count, batch size, etc.   │
│  - Vectorized API: set_projection(), next_batch()    │
│  - Value API: read_as_value()                        │
└─────────────────────────────────────────────────────┘
                         ↓
        ┌────────────────┴────────────────┐
        ↓                                  ↓
┌──────────────────┐            ┌──────────────────┐
│  Vectorized Mode │            │   Value Mode     │
│  (batch size ≥64)│            │  (small/batch=1) │
│                  │            │                  │
│  • SIMD ops      │            │  • No batching   │
│  • Cache friendly│            │  • Single pass   │
│  • Columnar      │            │  • Simple        │
└──────────────────┘            └──────────────────┘
```

### Key Design Principles

1. **Compile-Time Decision**: Choose execution mode during `compile()`, not during execution
   - Avoids wasting resources building unused infrastructure
   - Based on reader capabilities + execution hints

2. **Capability-Based Readers**: Readers declare what they support
   - `supports_vectorized`: Can produce `VectorizedBatch`es
   - `supports_value_mode`: Can produce `Value` directly
   - No forced conversions or adapters

3. **Transparent to Consumers**: Same API regardless of internal execution path
   ```rust
   let plan = compiler.compile(&logical)?;
   let result = plan.execute()?;  // Internally picks best mode
   ```

---

## Reader API Design

### Core DataSource Trait

```rust
/// Unified data source trait that can operate in multiple modes
pub trait DataSource {
    /// Declare which execution modes this source supports
    fn capabilities(&self) -> SourceCapabilities;
    
    /// Get execution hints for compile-time decisions
    fn execution_hints(&self) -> ExecutionHints {
        ExecutionHints::default()
    }
    
    /// Initialize the data source (common for both modes)
    fn open(&mut self) -> Result<(), EvalError>;
    
    /// Clean up resources (common for both modes)
    fn close(&mut self) -> Result<(), EvalError>;
    
    // ===== Vectorized Mode API =====
    /// Configure projection for vectorized mode
    fn set_projection(&mut self, spec: ProjectionSpec) -> Result<(), EvalError> {
        if !self.capabilities().supports_vectorized {
            return Err(EvalError::UnsupportedOperation(
                "Vectorized mode not supported".into()
            ));
        }
        Err(EvalError::NotImplemented)
    }
    
    /// Read next batch of vectorized data
    fn next_batch(&mut self) -> Result<Option<VectorizedBatch>, EvalError> {
        if !self.capabilities().supports_vectorized {
            return Err(EvalError::UnsupportedOperation(
                "Vectorized mode not supported".into()
            ));
        }
        Err(EvalError::NotImplemented)
    }
    
    fn resolve(&self, field_name: &str) -> Option<ProjectionSource> {
        None
    }
    
    // ===== Value Mode API (Legacy) =====
    /// Read all data as a PartiQL Value (for legacy execution)
    fn read_as_value(&mut self) -> Result<Value, EvalError> {
        if !self.capabilities().supports_value_mode {
            return Err(EvalError::UnsupportedOperation(
                "Value mode not supported".into()
            ));
        }
        Err(EvalError::NotImplemented)
    }
}
```

### Capability Structures

```rust
/// Describes what execution modes a data source supports
#[derive(Clone, Debug)]
pub struct SourceCapabilities {
    /// Can produce vectorized batches
    pub supports_vectorized: bool,
    
    /// Can produce Value directly
    pub supports_value_mode: bool,
    
    /// Prefer vectorized even for small datasets
    pub prefer_vectorized: bool,
}

impl Default for SourceCapabilities {
    fn default() -> Self {
        Self {
            supports_vectorized: false,
            supports_value_mode: true,  // Safe default
            prefer_vectorized: false,
        }
    }
}

/// Execution hints for compile-time optimization decisions
#[derive(Clone, Debug, Default)]
pub struct ExecutionHints {
    pub estimated_row_count: Option<usize>,
    pub preferred_batch_size: Option<usize>,
    pub has_columnar_storage: bool,
    pub supports_pushdown: bool,
}
```

---

## Example Reader Implementations

### Dual-Mode Reader (Parquet)

```rust
pub struct ParquetReader {
    file_path: String,
    batch_size: usize,
    arrow_reader: Option<ArrowFileReader>,
    // ... other fields
}

impl DataSource for ParquetReader {
    fn capabilities(&self) -> SourceCapabilities {
        SourceCapabilities {
            supports_vectorized: true,
            supports_value_mode: true,
            prefer_vectorized: true,  // Columnar format
        }
    }
    
    fn execution_hints(&self) -> ExecutionHints {
        ExecutionHints {
            estimated_row_count: Some(self.metadata.num_rows),
            preferred_batch_size: Some(self.batch_size),
            has_columnar_storage: true,
            ..Default::default()
        }
    }
    
    fn open(&mut self) -> Result<(), EvalError> {
        // Open file, initialize arrow reader
        Ok(())
    }
    
    // Vectorized mode
    fn set_projection(&mut self, spec: ProjectionSpec) -> Result<(), EvalError> {
        // Configure Arrow projection
        Ok(())
    }
    
    fn next_batch(&mut self) -> Result<Option<VectorizedBatch>, EvalError> {
        // Read Arrow batch and convert to VectorizedBatch
        Ok(None)
    }
    
    // Value mode
    fn read_as_value(&mut self) -> Result<Value, EvalError> {
        // Read all batches and convert to Value::Bag
        let mut rows = Vec::new();
        while let Some(batch) = self.next_batch()? {
            rows.extend(batch_to_values(batch)?);
        }
        Ok(Value::Bag(Box::new(rows.into())))
    }
    
    fn close(&mut self) -> Result<(), EvalError> {
        Ok(())
    }
}
```

### Vectorized-Only Reader (Arrow)

```rust
pub struct ArrowReader {
    // ... fields
}

impl DataSource for ArrowReader {
    fn capabilities(&self) -> SourceCapabilities {
        SourceCapabilities {
            supports_vectorized: true,
            supports_value_mode: false,  // Won't implement value mode
            prefer_vectorized: true,
        }
    }
    
    // Only implement vectorized methods
    fn set_projection(&mut self, spec: ProjectionSpec) -> Result<(), EvalError> {
        // Arrow-specific projection
        Ok(())
    }
    
    fn next_batch(&mut self) -> Result<Option<VectorizedBatch>, EvalError> {
        // Native Arrow batching
        Ok(None)
    }
    
    // Don't implement read_as_value - use default error
}
```

### Value-Only Reader (JSON)

```rust
pub struct JsonReader {
    data: Vec<Value>,
}

impl DataSource for JsonReader {
    fn capabilities(&self) -> SourceCapabilities {
        SourceCapabilities {
            supports_vectorized: false,
            supports_value_mode: true,
            prefer_vectorized: false,
        }
    }
    
    fn execution_hints(&self) -> ExecutionHints {
        ExecutionHints {
            estimated_row_count: Some(self.data.len()),
            ..Default::default()
        }
    }
    
    fn open(&mut self) -> Result<(), EvalError> { Ok(()) }
    
    // Only implement value mode
    fn read_as_value(&mut self) -> Result<Value, EvalError> {
        Ok(Value::Bag(Box::new(self.data.clone().into())))
    }
    
    fn close(&mut self) -> Result<(), EvalError> { Ok(()) }
}
```

---

## Compiler Integration

### Compile-Time Mode Selection

```rust
impl Compiler {
    pub fn compile(
        &mut self,
        logical: &LogicalPlan<BindingsOp>,
    ) -> Result<ExecutionPlan, PlanError> {
        // DECISION POINT: Check hints from all data sources
        let execution_mode = self.determine_execution_mode(logical)?;
        
        match execution_mode {
            ExecutionMode::Legacy => self.compile_legacy(logical),
            ExecutionMode::Vectorized => self.compile_vectorized(logical),
        }
    }
    
    fn determine_execution_mode(
        &self,
        logical: &LogicalPlan<BindingsOp>,
    ) -> Result<ExecutionMode, PlanError> {
        // Check if ALL sources support the desired mode
        let mut can_vectorize = true;
        let mut can_legacy = true;
        
        for (_name, source) in &self.context.data_sources {
            let caps = source.capabilities();
            can_vectorize &= caps.supports_vectorized;
            can_legacy &= caps.supports_value_mode;
        }
        
        // If only one mode is available, use it
        if !can_vectorize && can_legacy {
            return Ok(ExecutionMode::Legacy);
        }
        if can_vectorize && !can_legacy {
            return Ok(ExecutionMode::Vectorized);
        }
        if !can_vectorize && !can_legacy {
            return Err(PlanError::NoSupportedExecutionMode);
        }
        
        // Both modes available - make decision based on hints
        let hints = self.collect_hints();
        
        // Decision criteria
        if hints.total_estimated_rows < 1000 {
            return Ok(ExecutionMode::Legacy);
        }
        
        if hints.min_batch_size == 1 && hints.total_estimated_rows > 10_000 {
            return Ok(ExecutionMode::Legacy);
        }
        
        if !hints.has_columnar && hints.min_batch_size < 64 {
            return Ok(ExecutionMode::Legacy);
        }
        
        // Check if any source prefers vectorized
        let any_prefer_vectorized = self.context.data_sources
            .values()
            .any(|s| s.capabilities().prefer_vectorized);
            
        if any_prefer_vectorized {
            return Ok(ExecutionMode::Vectorized);
        }
        
        // Default to vectorized for medium-large datasets
        Ok(ExecutionMode::Vectorized)
    }
    
    fn collect_hints(&self) -> AggregateHints {
        let mut total_estimated_rows = 0;
        let mut min_batch_size = usize::MAX;
        let mut has_columnar = false;
        
        for (_name, source) in &self.context.data_sources {
            let hints = source.execution_hints();
            
            if let Some(rows) = hints.estimated_row_count {
                total_estimated_rows += rows;
            }
            if let Some(batch_size) = hints.preferred_batch_size {
                min_batch_size = min_batch_size.min(batch_size);
            }
            has_columnar |= hints.has_columnar_storage;
        }
        
        AggregateHints {
            total_estimated_rows,
            min_batch_size,
            has_columnar,
        }
    }
}
```

### Unified Execution Plan

```rust
pub enum ExecutionPlan {
    Legacy(partiql_eval::plan::EvaluablePlan),
    Vectorized(VectorizedPlan),
}

impl ExecutionPlan {
    pub fn execute(&mut self, ctx: &dyn Context) -> Result<Value, EvalError> {
        match self {
            ExecutionPlan::Legacy(plan) => {
                // Execute with legacy evaluator
                let result = plan.execute(ctx)?;
                Ok(result.result)
            }
            ExecutionPlan::Vectorized(plan) => {
                // Execute vectorized and collect results
                let mut results = Vec::new();
                for batch_result in plan.execute() {
                    let batch = batch_result?;
                    results.extend(batch_to_values(batch));
                }
                Ok(Value::Bag(Box::new(results.into())))
            }
        }
    }
}
```

---

## Decision Criteria

### Use Legacy Mode When:

1. **Small Datasets**: Total rows < 1,000
2. **Pathological Batching**: Batch size = 1 with many batches (>10,000 rows)
3. **Row-Oriented with Small Batches**: Not columnar AND batch size < 64
4. **Reader Limitation**: Some readers only support value mode

### Use Vectorized Mode When:

1. **Columnar Storage**: Parquet, Arrow formats
2. **Efficient Batching**: Batch size ≥ 64
3. **Large Datasets**: Total rows > 1,000
4. **Reader Preference**: Source explicitly prefers vectorized
5. **Reader Limitation**: Some readers only support vectorized mode

---

## Implementation Roadmap

### Phase 1: Add Capability Infrastructure (Non-Breaking)
- [ ] Define `DataSource` trait with capabilities
- [ ] Add `SourceCapabilities` and `ExecutionHints` structs
- [ ] Provide default implementations for all optional methods
- [ ] Add blanket impl for existing `BatchReader` (backward compatibility)

### Phase 2: Update Compiler (Core Change)
- [ ] Add `ExecutionMode` enum (Legacy | Vectorized)
- [ ] Implement `determine_execution_mode()` with decision logic
- [ ] Create `ExecutionPlan` enum to hold either plan type
- [ ] Add compile-time mode selection in `Compiler::compile()`
- [ ] Implement `collect_hints()` for aggregating reader metadata

### Phase 3: Update Readers (Progressive)
- [ ] Convert `InMemoryGeneratedReader` to dual-mode
- [ ] Convert `ParquetReader` to dual-mode
- [ ] Keep `ArrowReader` as vectorized-only
- [ ] Add `read_as_value()` implementations where beneficial
- [ ] Update reader tests for both modes

### Phase 4: Testing & Validation
- [ ] Benchmark with various batch sizes (1, 64, 256, 1024)
- [ ] Verify decision logic chooses correctly
- [ ] Test with mixed reader types in single query
- [ ] Measure overhead of capability checking
- [ ] Add integration tests for both execution paths

### Phase 5: Documentation & Migration
- [ ] Update reader authoring guide
- [ ] Document capability system
- [ ] Provide migration examples for custom readers
- [ ] Add logging for mode selection (observability)

---

## Benefits Summary

1. **Performance**: Right execution path for the workload
   - No wasted overhead for small datasets or batch size = 1
   - Full vectorization benefits for appropriate workloads

2. **Simplicity**: Readers only implement what makes sense
   - Parquet: Both modes (native columnar + fallback)
   - Arrow: Vectorized only (always efficient)
   - JSON: Value mode only (simple parsing)

3. **Efficiency**: No wasted initialization or conversion
   - Compile-time decision avoids building unused infrastructure
   - No runtime adapters or conversions

4. **Transparency**: Consumers see unified API
   - Same `compile()` and `execute()` interface
   - Mode selection is automatic and internal

5. **Measurability**: Can log which path was chosen
   - Observability for performance tuning
   - Easy to benchmark both modes

6. **Future-Proof**: Easy to add new execution modes
   - Capability system extends naturally
   - Could add specialized modes (e.g., GPU, distributed)

---

## Execution Flow Comparison

### Current Flow (Always Vectorized)

```
CompilerContext::new()
  .with_data_source("data", reader)
↓
Compiler::new(context)
↓
compiler.compile(logical)
  → Builds VectorizedPlan (ALWAYS)
  → Creates scratch vectors, operators, etc.
↓
plan.execute()
  → Iterator over batches
  → 10,000 iterations for batch_size=1 (SLOW!)
```

### New Flow (Hybrid with Compile-Time Decision)

```
CompilerContext::new()
  .with_data_source("data", reader)  ← Reader has capabilities
↓
Compiler::new(context)
↓
compiler.compile(logical)
  → reader.capabilities() + execution_hints()  ← DECISION HERE
  → IF small/pathological:
       compile_legacy() → ExecutionPlan::Legacy
     ELSE:
       compile_vectorized() → ExecutionPlan::Vectorized
↓
plan.execute(ctx)
  → Dispatches to appropriate executor
  → Legacy: Single pass, no batching
  → Vectorized: Efficient batching with proper size
```

---

## Migration Path for Existing Code

### For Reader Authors

**Before** (Only vectorized):
```rust
impl BatchReader for MyReader {
    fn set_projection(&mut self, spec: ProjectionSpec) -> Result<(), EvalError> { ... }
    fn next_batch(&mut self) -> Result<Option<VectorizedBatch>, EvalError> { ... }
    // ...
}
```

**After** (Dual-mode):
```rust
impl DataSource for MyReader {
    fn capabilities(&self) -> SourceCapabilities {
        SourceCapabilities {
            supports_vectorized: true,
            supports_value_mode: true,  // NEW: Now supports both!
            prefer_vectorized: true,
        }
    }
    
    fn execution_hints(&self) -> ExecutionHints {
        ExecutionHints {
            estimated_row_count: Some(self.row_count),
            preferred_batch_size: Some(self.batch_size),
            has_columnar_storage: true,
            ..Default::default()
        }
    }
    
    // Keep existing vectorized methods
    fn set_projection(&mut self, spec: ProjectionSpec) -> Result<(), EvalError> { ... }
    fn next_batch(&mut self) -> Result<Option<VectorizedBatch>, EvalError> { ... }
    
    // ADD: Value mode for small datasets
    fn read_as_value(&mut self) -> Result<Value, EvalError> {
        let mut rows = Vec::new();
        while let Some(batch) = self.next_batch()? {
            rows.extend(batch_to_values(batch)?);
        }
        Ok(Value::Bag(Box::new(rows.into())))
    }
}
```

### For Query Engine Users

**No changes required!** The API remains the same:

```rust
let context = CompilerContext::new()
    .with_data_source("data".to_string(), reader);

let mut compiler = Compiler::new(context);
let plan = compiler.compile(&logical)?;

// Execute - internally chooses best strategy
let result = plan.execute(ctx)?;
```

---

## Performance Expectations

### Batch Size = 1 (10,000 rows)

**Before (Always Vectorized)**:
- 10,000 batch iterations
- Significant overhead per batch
- **Result**: Slower than legacy

**After (Hybrid with Legacy Mode)**:
- Single value collection
- No batching overhead
- **Result**: Fast, matches legacy performance

### Batch Size = 256 (10,000 rows)

**Before (Always Vectorized)**:
- ~40 batch iterations
- SIMD benefits realized
- **Result**: Fast

**After (Hybrid with Vectorized Mode)**:
- ~40 batch iterations
- SIMD benefits realized
- **Result**: Same fast performance

### Columnar Data (Parquet, any size)

**After (Hybrid with Vectorized Mode)**:
- Leverages columnar layout
- Efficient projection pushdown
- **Result**: Optimal performance

---

## Conclusion

The hybrid execution approach solves the batch size = 1 performance problem by:

1. **Recognizing the fundamental difference** between row-at-a-time and batch processing
2. **Making intelligent decisions at compile time** to avoid wasted work
3. **Giving readers control** over their capabilities and preferences
4. **Maintaining API simplicity** for both reader authors and query engine users
5. **Ensuring optimal performance** across different workload characteristics

This design is realistic, implementable in phases, and provides a clean path forward for the PartiQL vectorized execution engine.
