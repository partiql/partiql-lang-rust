# Compiler-Reader Integration Plan

## Goal
Add `resolve()` method to BatchReader to map field names to ProjectionSource, enabling the compiler to translate logical field references into physical access patterns.

## 1. Define ProjectionSource

```rust
// src/reader/batch_reader.rs
pub enum ProjectionSource {
    ColumnIndex(usize),    // For columnar readers
    FieldPath(String),     // For row/struct readers
}
```

## 2. Update BatchReader Trait

```rust
pub trait BatchReader {
    fn next_batch(&mut self) -> Result<Option<VectorizedBatch>, EvalError>;
    
    // New: Replace schema() with resolve()
    fn resolve(&self, field_name: &str) -> Option<ProjectionSource>;
}
```

## 3. Implement for TupleIteratorReader

```rust
impl BatchReader for TupleIteratorReader {
    fn resolve(&self, field_name: &str) -> Option<ProjectionSource> {
        self.schema
            .fields()
            .iter()
            .position(|f| f.name == field_name)
            .map(ProjectionSource::ColumnIndex)
    }
}
```

## 4. Update Compiler

**Before (hardcoded):**
```rust
CompiledExpr {
    op: ExprOp::Identity,
    inputs: smallvec![ExprInput::InputCol(0)],  // Hardcoded index
    output: 0,
}
```

**After (resolved):**
```rust
let source = reader.resolve("a")
    .ok_or(PlanError::General("Field 'a' not found"))?;

let col_idx = match source {
    ProjectionSource::ColumnIndex(idx) => idx,
    ProjectionSource::FieldPath(_) => return Err(...),
};

CompiledExpr {
    op: ExprOp::Identity,
    inputs: smallvec![ExprInput::InputCol(col_idx)],
    output: 0,
}
```

## 5. Migration Checklist

- [ ] Add `ProjectionSource` to `batch_reader.rs`
- [ ] Update `BatchReader` trait (remove `schema()`, add `resolve()`)
- [ ] Implement `resolve()` in `TupleIteratorReader`
- [ ] Export `ProjectionSource` from `reader/mod.rs`
- [ ] Update compiler to use `resolve()` instead of hardcoded indices
- [ ] Search codebase for `.schema()` calls and update
- [ ] Update tests

## Future Evolution

This incremental change naturally extends to the full Phase 0 design:
- `ProjectionSource` → used in `Projection` and `ProjectionSpec`
- `resolve()` → validates what `set_projection()` can accept
- Supports both columnar and row-based readers
