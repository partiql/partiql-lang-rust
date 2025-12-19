# PartiQL Vectorized Evaluator - PoC

A proof-of-concept vectorized evaluation engine for PartiQL that processes data in batches using columnar storage.

## Project Status

✅ **Structure Created** - All placeholder interfaces and modules are in place
⚠️ **Implementation Pending** - Core logic marked with `todo!()` for parallel development

## What's Implemented

### Phase 1: Core Abstractions ✅
- ✅ `PVector` - Type-specific columnar storage (Vec-based, pre-allocated)
- ✅ `TypeInfo` - Type system (Int64, Float64, Boolean, String)
- ✅ `SourceTypeDef` & `Field` - Schema definition
- ✅ `VectorizedBatch` - Batch container
- ✅ `VectorizedFn` trait - Function interface with pre-allocated output contract
- ✅ `VectorizedFnRegistry` - Function registration and lookup

### Phase 2: Data Ingestion ✅
- ✅ `BatchReader` trait - Iterator interface for batches
- ✅ `TupleIteratorReader` - Placeholder for tuple-to-columnar conversion
- ✅ `VectorizedBatch` - Columnar batch container

### Phase 3: Expressions ✅
- ✅ `VectorizedExpr` trait - Expression evaluation interface
- ✅ `ColumnRef` - Column reference expression
- ✅ `LiteralExpr` - Literal value expression (broadcast pending)
- ✅ `FnCallExpr` - Function call expression

### Phase 4: Functions (Placeholders) ⚠️
- ⚠️ Comparison: `VecGtInt64`, `VecLtInt64` (marked `todo!()`)
- ⚠️ Logical: `VecAnd`, `VecOr`, `VecNot` (marked `todo!()`)
- ⚠️ Arithmetic: `VecAddInt64` (marked `todo!()`)

### Phase 5: Operators ✅
- ✅ `VectorizedOperator` trait - Volcano-style interface
- ✅ `VectorizedScan` - Data source reader (complete)
- ✅ `VectorizedFilter` - Predicate filtering (marked `todo!()`)
- ✅ `VectorizedProject` - Column projection (marked `todo!()`)

## Building

```bash
cargo build
```

Currently compiles with warnings about unused code (expected for placeholders).

## Implementation Tasks

All functions and operators marked with `todo!()` need implementation:

### Priority 1: Functions (Phase 3)
- [ ] Implement comparison functions (Gt, Lt, Gte, Lte, Eq, Neq)
- [ ] Implement logical functions (And, Or, Not)
- [ ] Implement arithmetic functions (Add, Sub, Mul, Div)

### Priority 2: Operators (Phase 5)
- [ ] Implement `VectorizedFilter::next_batch` with selection vector
- [ ] Implement `VectorizedProject::next_batch` with expression evaluation

### Priority 3: Data Ingestion (Phase 2)
- [ ] Implement `TupleIteratorReader::next_batch` (Value → PVector conversion)
- [ ] Implement `LiteralExpr::eval` broadcast logic

### Priority 4: Testing (Phase 6)
- [ ] Unit tests for each function
- [ ] Integration test for PoC query: `SELECT a, b FROM data WHERE a > 10 AND b < 100`
- [ ] Benchmark comparing with current evaluator

## PoC Query

Target query for validation:
```sql
SELECT a, b FROM data WHERE a > 10 AND b < 100
```

This exercises:
- Scan (read data in batches)
- Filter (a > 10 AND b < 100)
- Project (select columns a, b)

## Design Principles

1. **Pre-allocated outputs**: All functions write to caller-allocated buffers
2. **No NULL/MISSING**: Simplified for PoC
3. **Batch size**: 1024 rows default
4. **Type specialization**: Separate implementations per type
5. **Vec-based storage**: Enables SIMD later
6. **Volcano interface**: Iterator-based `next_batch()`

## Next Steps

1. Implement comparison functions (start with `VecGtInt64`)
2. Implement logical functions (`VecAnd`)
3. Implement `VectorizedFilter::next_batch`
4. Implement `VectorizedProject::next_batch`
5. Create integration test
6. Add benchmarks

See `../poc_plan.md` for detailed implementation guide.
