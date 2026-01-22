# Hybrid Query Engine Design — RowView / Result Consumption API

This document specifies a **public-facing result consumption API** for third-party consumers.
It focuses on `RowView` (and optionally `BatchView`) as a stable, ergonomic façade that preserves
**zero-copy** behavior while hiding internal engine representations (`ValueRef`, arenas, VM registers).

This complements:
- `hybrid_design.md`
- `hybrid_design_addendum.md`
- `hybrid_design_tenets.md`

---

## 1. Goals

1. **Zero-copy by default**
   - Scalars, strings, bytes, object/array views should be readable without copying when backed by
     reader buffers or stable engine storage.

2. **Low surface-area / low complexity**
   - Consumers should not need to understand engine internals (slot layouts, arenas, `ValueOwned`).

3. **Predictable lifetimes**
   - One simple rule for validity of borrows.

4. **Gradual typing support**
   - Typed getters for common cases.
   - A generic path for unknown/complex values.

5. **Future-proofing**
   - Allow internal representation changes (JSON tape, Arrow, custom readers) without breaking consumers.

---

## 2. Core Public Types

### 2.1 `ResultStream` (row-oriented)
A cursor-like interface for pulling rows.

```rust
pub trait ResultStream {
    fn schema(&self) -> &Schema;
    fn next_row(&mut self) -> anyhow::Result<Option<RowView<'_>>>;
}
```

### 2.2 `ResultBatchStream` (optional, batch-oriented)
Useful for consumers that can exploit columnar batches.

```rust
pub trait ResultBatchStream {
    fn schema(&self) -> &Schema;
    fn next_batch(&mut self, max_rows: usize) -> anyhow::Result<Option<BatchView<'_>>>;
}
```

> You may expose only `ResultStream` initially, and add `ResultBatchStream` later.

---

## 3. Lifetime & Validity Contract (Zero-copy rule)

All borrowed data returned from `RowView` / `BatchView` is valid:

> **until the next call to `next_row()` / `next_batch()` on the same stream**.

This includes:
- `&str`, `&[u8]`
- nested object/array iterators/views
- opaque references

If a consumer needs to retain data beyond that, they must:
- materialize (`to_owned()`), or
- copy into their own buffers.

This rule matches typical DB cursor semantics and keeps the engine free to reuse buffers.

---

## 4. Schema

A schema describes output columns and (optionally) type information.

```rust
pub struct Schema {
    pub columns: Vec<ColumnSchema>,
}

pub struct ColumnSchema {
    pub name: String,
    pub hint: TypeHintPublic,   // optional/gradual; can be Unknown
    pub nullable: bool,
    pub may_be_missing: bool,   // SQL++ Missing vs Null
}
```

> Type hints are best-effort and may be `Unknown`. Consumers should treat them as advisory.

---

## 5. RowView API

### 5.1 Typed getters (ergonomic fast path)

```rust
pub struct RowView<'a> { /* opaque */ }

impl<'a> RowView<'a> {
    pub fn len(&self) -> usize;

    pub fn is_null(&self, col: usize) -> bool;
    pub fn is_missing(&self, col: usize) -> bool;

    pub fn get_bool(&self, col: usize) -> Option<bool>;
    pub fn get_i64(&self, col: usize) -> Option<i64>;
    pub fn get_f64(&self, col: usize) -> Option<f64>;
    pub fn get_str(&self, col: usize) -> Option<&'a str>;
    pub fn get_bytes(&self, col: usize) -> Option<&'a [u8]>;

    /// Generic access for unknown/complex values.
    pub fn get_value(&self, col: usize) -> ValueView<'a>;
}
```

Notes:
- Typed getters return `Option<T>`:
  - `None` if value is `Null` or `Missing` or wrong type (your choice—see below).
- Keep `get_value()` as the canonical, always-correct access path.

### 5.2 Behavior on type mismatch (choose one policy)
Pick a clear policy and document it:

**Policy A (strict):** typed getter returns `None` on mismatch  
**Policy B (error):** typed getter returns `Result<Option<T>>`  

For v1 ergonomics, Policy A is common:
- cheap and non-throwing in hot loops
- callers can fallback to `get_value()` if needed

---

## 6. ValueView API (generic access)

`ValueView<'a>` is the public borrowed value representation. It should remain stable even if internal
representation changes.

```rust
pub enum ValueView<'a> {
    Missing,
    Null,
    Bool(bool),
    I64(i64),
    F64(f64),
    Str(&'a str),
    Bytes(&'a [u8]),
    Array(ArrayView<'a>),
    Object(ObjectView<'a>),
    Opaque(OpaqueView<'a>), // optional escape hatch
}
```

### 6.1 ArrayView / ObjectView (views, not representations)
These are opaque views with methods to navigate without allocation.

```rust
pub struct ArrayView<'a> { /* opaque */ }
impl<'a> ArrayView<'a> {
    pub fn len(&self) -> usize;
    pub fn get(&self, idx: usize) -> ValueView<'a>;
    pub fn iter(&self) -> impl Iterator<Item = ValueView<'a>> + 'a;
}

pub struct ObjectView<'a> { /* opaque */ }
impl<'a> ObjectView<'a> {
    pub fn get(&self, key: &str) -> ValueView<'a>;
    pub fn iter(&self) -> impl Iterator<Item = (&'a str, ValueView<'a>)> + 'a;
}
```

### 6.2 Owned conversion
Provide a single explicit conversion to owned form:

```rust
impl<'a> ValueView<'a> {
    pub fn to_owned(&self) -> ValueOwnedPublic;
}
```

`ValueOwnedPublic` should be a stable owned representation appropriate for consumers.

### 6.3 Optional JSON serialization helper
If many consumers just want JSON output for complex values:

```rust
impl<'a> ValueView<'a> {
    pub fn write_json(&self, out: &mut dyn std::io::Write) -> anyhow::Result<()>;
}
```

This avoids exposing internal object representations while remaining functional.

---

## 7. BatchView API (optional)

Batch view is useful for exporters and analytics integrations.

```rust
pub struct BatchView<'a> { /* opaque */ }

impl<'a> BatchView<'a> {
    pub fn len(&self) -> usize;
    pub fn column(&self, col: usize) -> ColumnView<'a>;
}

pub enum ColumnView<'a> {
    I64(&'a [i64], Validity<'a>),
    F64(&'a [f64], Validity<'a>),
    Bool(&'a [bool], Validity<'a>),
    Str(StrColumnView<'a>, Validity<'a>),
    Any(&'a [ValueView<'a>]),
}

pub struct Validity<'a>(pub &'a [u8]); // bitmap or byte validity; implementation-defined
```

Notes:
- `Any` columns are allowed for gradual typing.
- For strings, consider either:
  - `&'a [&'a str]` (simple), or
  - offset buffer + data buffer (Arrow-like), if you plan for Arrow interop.

---

## 8. Integration with PlanInstance

### 8.1 PlanStream: a concrete ResultStream wrapper
A third-party consumer should not pull from `PlanInstance` directly. Provide a wrapper:

```rust
pub struct PlanStream {
    inst: PlanInstance,
    tctx: ThreadCtx,
    out: RowFrameScratch,
    schema: Schema,
}

impl ResultStream for PlanStream {
    fn schema(&self) -> &Schema { &self.schema }

    fn next_row(&mut self) -> anyhow::Result<Option<RowView<'_>>> {
        self.tctx.row_arena.reset();
        let ok = self.inst.next_row(self.inst.root, self.out.as_mut(), &mut self.tctx)?;
        if !ok { return Ok(None); }
        Ok(Some(RowView::new(self.out.as_ref(), &self.schema)))
    }
}
```

Key points:
- `RowFrameScratch` is reused each call.
- `row_arena.reset()` occurs once per produced row.
- Returned `RowView` borrows from `out` and any upstream borrowed buffers.

### 8.2 BatchStream wrapper (optional)
Similar, but reuses a `DataChunkScratch`:

```rust
pub struct PlanBatchStream {
    inst: PlanInstance,
    tctx: ThreadCtx,
    out: DataChunkScratch,
    schema: Schema,
}

impl ResultBatchStream for PlanBatchStream {
    fn schema(&self) -> &Schema { &self.schema }

    fn next_batch(&mut self, max: usize) -> anyhow::Result<Option<BatchView<'_>>> {
        self.tctx.chunk_arena.reset();
        let ok = self.inst.next_batch(self.inst.root, self.out.as_mut(), max, &mut self.tctx)?;
        if !ok { return Ok(None); }
        Ok(Some(BatchView::new(self.out.as_ref(), &self.schema)))
    }
}
```

---

## 9. Example Usage (Consumer)

### 9.1 Typed scalar consumption (fast path)

```rust
let mut stream = engine.execute(query, env)?;
while let Some(row) = stream.next_row()? {
    if let Some(id) = row.get_i64(0) {
        // zero-copy scalar
        process_id(id);
    }
    if let Some(country) = row.get_str(1) {
        // zero-copy &str
        process_country(country);
    }
}
```

### 9.2 Generic nested value access

```rust
while let Some(row) = stream.next_row()? {
    match row.get_value(2) {
        ValueView::Object(obj) => {
            let city = obj.get("city");
            // ...
        }
        ValueView::Missing | ValueView::Null => { /* ... */ }
        other => { /* type mismatch */ }
    }
}
```

### 9.3 Holding onto results (explicit copy)

```rust
let mut saved = Vec::new();
while let Some(row) = stream.next_row()? {
    let v = row.get_value(0).to_owned();
    saved.push(v);
}
// safe after stream continues
```

---

## 10. Zero-copy Guidance for Consumers

Consumers should assume:
- row views are transient
- strings/bytes are borrowed
- nested iterators borrow too

If they need:
- persistence: call `to_owned()`
- cross-thread ownership: materialize into owned types

---

## 11. Non-goals / Intentional Omissions

- Exposing internal `ValueRef` / `ValueOwned`
- Exposing slot layouts, VM registers, arena lifetimes
- Promising stable borrows across multiple `next_*` calls (except via explicit “stable source” APIs)

These remain internal to preserve flexibility.

---

## 12. Implementation Notes (for engine authors)

- `RowView` should be a thin wrapper around an internal row carrier (RowFrame).
- `ValueView` is constructed on-demand; avoid allocations.
- For object/array views, prefer lightweight wrappers over trait objects in hot paths.
- Ensure that `RowArena.reset()` (and `ChunkArena.reset()`) happens at the correct boundary:
  - once per produced row/batch
  - before writing computed outputs

---

## 13. Summary

`RowView` provides a stable, ergonomic, zero-copy result API by:
- exposing typed getters for common cases
- supporting generic nested values through views
- enforcing a simple lifetime rule
- allowing explicit materialization when needed

It decouples consumers from internal engine representations, enabling performance optimizations and representation changes without breaking API contracts.
