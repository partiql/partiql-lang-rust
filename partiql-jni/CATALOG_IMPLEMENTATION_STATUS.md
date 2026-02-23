# PartiQL JNI Catalog Implementation Status

> **Last Updated:** 2026-02-23  
> **Current State:** Fully Functional — Buffered Catalog Only

## Executive Summary

The JNI bindings support a **buffered catalog** approach where Java pre-populates data into a `DirectByteBuffer` at registration time, and Rust reads from that buffer during query execution — eliminating Rust-to-Java callback overhead.

**What Works:**
- ✅ Compile and execute PartiQL queries with custom catalogs
- ✅ `CompilationCatalog` — Java interface for compile-time table resolution (JNI callbacks)
- ✅ `BufferedExecutionCatalog` — Java pre-populates data into a buffer; Rust reads it directly
- ✅ `RegisterReader` for direct column access in query results
- ✅ Complex value support (tuples, lists, bags) via `BufferValueWriter`

## Architecture

### Compilation Phase
```
Java CompilationCatalog.getTable(path) → DataSourceHandle(entryId, config)
                                         ↓
Rust calls Java via JNI callback to resolve table names at compile time
```

### Execution Phase (Buffered)
```
Java: BufferedExecutionCatalog pre-populates DirectByteBuffer with all row data
                                         ↓
Rust: Reads buffer directly — zero JNI callbacks during execution
```

## Key Java Classes

### Compilation
- **CompilationContext** — Registry mapping catalog names to `CompilationCatalog` instances
- **CompilationCatalog** — Interface for compile-time table resolution
- **DataSourceHandle** — Contains `entryId` + `DataSourceConfig`
- **DataSourceConfig** — Compile-time metadata (capabilities, field resolution)

### Execution
- **ExecutionContext** — Registry mapping `CatalogId` to `BufferedExecutionCatalog`
- **BufferedExecutionCatalog** — Pre-populates data into a `DirectByteBuffer`
- **BufferWriter** — Writes scalar values (i64, f64, bool, string, null, missing) to buffer
- **BufferValueWriter** — Writes complex values (tuples, lists, bags) to buffer
- **BufferedMemoryPool** — Manages reusable `DirectByteBuffer` instances

### Supporting Types
- **BindingsName** — Case-sensitive/insensitive name binding
- **ScanSource** — Column index, field path, or whole value
- **ScanCapabilities** / **BufferStability** — Data source metadata
- **ScanLayout** / **ScanProjection** — Projection descriptors

## Wire Format

The buffer uses a compact binary format:
- **Row**: `[slot: u16][Value]* MARKER_ROW_END(0x12)`
- **Scalar**: `[type_tag: u8][data]`
- **Container**: `[TYPE_TUPLE|TYPE_LIST|TYPE_BAG] content* MARKER_CONTAINER_END(0x11)`
- **Tuple field**: `MARKER_FIELD_NAME(0x10) [len: i32] [utf8_bytes] [Value]`

## Rust Implementation

- **`catalog_bridge.rs`** — `JavaCompilationCatalog` (JNI callbacks), `JavaBufferedExecutionCatalog` (buffer reader), `BufferDataSource`, `JavaDataSourceConfig`
- **`context.rs`** — Handle management for `CompilationContext` and `ExecutionContext`, JNI exports, buffer address caching

## Usage Example

```java
// 1. Compile
CompilationContext compCtx = new CompilationContext();
long catalogId = compCtx.addCatalog("main", myCompilationCatalog);
CompiledPlan plan = new PlanCompiler().compile("SELECT a, b FROM data", compCtx);

// 2. Build buffered execution catalog
BufferedMemoryPool pool = new BufferedMemoryPool();
BufferedExecutionCatalog execCatalog = new BufferedExecutionCatalog(entryId, pool);
BufferWriter writer = execCatalog.getWriter();
// Write rows: writer.writeI64(slot, value), writer.endRow(), etc.
execCatalog.seal();

// 3. Execute
ExecutionContext execCtx = new ExecutionContext();
execCtx.addBufferedCatalog(catalogId, execCatalog);
try (PartiQLVM vm = new PartiQLVM(plan, execCtx)) {
    ExecutionResult result = vm.execute();
    // iterate results...
}