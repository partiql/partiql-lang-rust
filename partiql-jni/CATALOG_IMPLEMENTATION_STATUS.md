# PartiQL JNI Catalog Implementation Status

> **Last Updated:** 2026-02-03  
> **Current State:** Java API Complete, Rust Implementation Partial (Stubs)

## Executive Summary

The JNI bindings are **functionally complete for basic queries** but catalog integration is **designed but not yet implemented**. The Java API layer is complete with all 14 catalog classes, but the Rust native layer currently ignores context parameters and uses default catalogs.

**What Works:**
- ✅ Compile and execute simple PartiQL queries
- ✅ Basic VM/Compiler/Plan operations
- ✅ RegisterReader for direct column access
- ✅ All catalog Java classes defined with proper APIs

**What Doesn't Work:**
- ❌ Custom catalog registration (uses default catalog only)
- ❌ Runtime data source binding
- ❌ Java → Rust catalog callbacks

---

## ✅ Phase 1: Java Catalog API (COMPLETE)

All Java interfaces and classes for the catalog system have been successfully implemented and compile correctly.

### Core Types (14 files)

#### 1. Name Binding
- **BindingsName.java** - Final class with `isDelimited()` method
  - `getName()` - returns the name string
  - `isDelimited()` - returns true for delimited (case-sensitive) names
  - Factory methods: `delimited()`, `undelimited()`

#### 2. Scan Configuration
- **ScanSource.java** - Abstract class with 3 variants:
  - `ColumnIndex` - column at specific index
  - `FieldPath` - field accessed by path
  - `BaseRow` - entire row without projection
  
- **BufferStability.java** - Enum:
  - `UNTIL_NEXT` - buffer valid until next() call
  - `UNTIL_CLOSE` - buffer valid until close
  
- **ScanCapabilities.java** - Data source capabilities
- **TypeHint.java** - Type hints for projections (currently just `ANY`)
- **ScanProjection.java** - Single field projection descriptor
- **ScanLayout.java** - Complete scan layout with projections

#### 3. Catalog Interfaces
- **DataSourceConfig.java** - Compile-time metadata interface
  - `getCapabilities()` - returns ScanCapabilities
  - `resolveField(String)` - resolves field name to ScanSource
  
- **DataSourceHandle.java** - Handle with entryId + config
- **DataSource.java** - Execution-time data reading interface
  - `next()` - returns RegisterReader or null
  - `hasNext()` - checks if more rows available
  - `close()` - releases resources

- **CompilationCatalog.java** - Compilation-time catalog interface
  - `getTable(List<BindingsName>)` - resolve table by path

- **ExecutionCatalog.java** - Execution-time catalog interface
  - `create(entryId, layout)` - creates DataSource for execution

#### 4. Context Classes
- **CompilationContext.java** - Compilation catalog registry
  - `addCatalog(name, catalog)` - returns catalogId
  - `getNativeHandle()` - returns handle for JNI
  - Manages catalog name → CompilationCatalog mappings
  - **Has native method stubs declared but NOT IMPLEMENTED in Rust**
  
- **ExecutionContext.java** - Execution catalog registry
  - `addCatalog(catalogId, catalog)` - maps catalogId to ExecutionCatalog
  - `getNativeHandle()` - returns handle for JNI
  - Enables different datasets for same compiled plan
  - **Has native method stubs declared but NOT IMPLEMENTED in Rust**

### Updated Core Classes

#### PlanCompiler.java
```java
public CompiledPlan compile(String sql, CompilationContext context)
```
- ✅ Java signature requires CompilationContext parameter
- ✅ Calls `nativeCompile(sql, context.getNativeHandle())`
- ❌ **Rust implementation IGNORES the context parameter**

#### PartiQLVM.java
```java
public PartiQLVM(CompiledPlan plan, ExecutionContext context)
```
- ✅ Java signature requires ExecutionContext parameter  
- ✅ Calls `nativeNew(plan.getNativeHandle(), context.getNativeHandle())`
- ❌ **Rust implementation IGNORES the context parameter**

---

## 🟡 Phase 1.5: Basic JNI Bindings (COMPLETE)

Core JNI bindings work for simple queries but don't support custom catalogs.

### What's Implemented

#### Handle Management (`handles.rs`) ✅
```rust
- create_vm_handle() / get_vm() / remove_vm_handle()
- create_plan_handle() / get_plan() / remove_plan_handle()
- create_result_handle() / get_result() / remove_result_handle()
- create_iterator_handle() / get_iterator() / remove_iterator_handle()
```

#### VM Wrapper (`vm.rs`) ⚠️
```rust
#[no_mangle]
pub extern "system" fn Java_org_partiql_jni_PartiQLVM_nativeNew(
    plan_handle: jlong,
    _exec_context_handle: jlong,  // ⚠️ IGNORED (underscore prefix)
) -> jlong {
    // TODO: Get ExecutionContext from handle
    let exec_context = ExecutionContext::default();  // Uses default!
    let vm = PartiQLVM::new((*plan).clone(), &exec_context)?;
    Ok(create_vm_handle(vm) as jlong)
}
```

#### Compiler Wrapper (`compiler.rs`) ❌
```rust
#[no_mangle]
pub extern "system" fn Java_org_partiql_jni_PlanCompiler_nativeCompile(
    sql: JString<'_>,
    // ❌ MISSING: contextHandle parameter not in signature!
) -> jlong {
    let compilation_context = partiql_eval::CompilationContext::default();
    let plan_compiler = PlanCompiler::new(&compilation_context);
    // Uses default catalog, ignores Java context completely
}
```

#### Other Wrappers ✅
- `plan.rs` - CompiledPlan lifecycle (complete)
- `result.rs` - ExecutionResult and QueryIterator (complete)
- `register_reader.rs` - Direct column access (complete)
- `error.rs` - Exception mapping (complete)
- `conversion.rs` - Value conversion stubs (future work)

### Build Status
- ✅ Rust: `cargo clippy --all-features -- -D warnings` PASSES
- ✅ Rust: `cargo build --release` SUCCEEDS
- ✅ Gradle: `./gradlew build` SUCCEEDS
- ⚠️ Javadoc: 38 warnings (missing @param/@return, cosmetic only)

---

## 📋 Phase 2: Catalog Integration (TODO)

The catalog integration design is complete but implementation is pending.

### Missing Implementations

#### 1. Context Native Methods

**File:** `partiql-jni/src/context.rs` (does not exist yet)

Need to implement:
```rust
// CompilationContext native methods
#[no_mangle]
pub extern "system" fn Java_org_partiql_jni_CompilationContext_nativeNew(
    env: JNIEnv,
    _class: JClass,
) -> jlong {
    // Create CompilationContext handle
}

#[no_mangle]
pub extern "system" fn Java_org_partiql_jni_CompilationContext_nativeAddCatalog(
    env: JNIEnv,
    _class: JClass,
    handle: jlong,
    name: JString,
) -> jlong {
    // Add catalog and return catalogId
}

#[no_mangle]
pub extern "system" fn Java_org_partiql_jni_CompilationContext_nativeClose(
    env: JNIEnv,
    _class: JClass,
    handle: jlong,
) {
    // Clean up context
}

#[no_mangle]
pub extern "system" fn Java_org_partiql_jni_CompilationContext_registerCatalogCallback(
    env: JNIEnv,
    this: JObject,
    catalog_id: jlong,
    catalog: JObject,
) {
    // Store Java CompilationCatalog for callbacks
}

// Similar for ExecutionContext
```

#### 2. Catalog Bridge

**File:** `partiql-jni/src/catalog_bridge.rs` (does not exist yet)

Implement JNI callback mechanism to call Java catalog methods from Rust:

```rust
pub struct JavaCompilationCatalog {
    java_object: GlobalRef,
    jvm: JavaVM,
}

impl CompilationCatalog for JavaCompilationCatalog {
    fn get_table(&self, path: &[BindingsName]) -> Option<DataSourceHandle> {
        // 1. Attach to JVM
        // 2. Convert Rust BindingsName to Java BindingsName objects
        // 3. Call Java CompilationCatalog.getTable(List<BindingsName>)
        // 4. Convert result back to Rust DataSourceHandle
    }
}

// Similarly for JavaExecutionCatalog
```

#### 3. Update Existing Wrappers

**Update:** `partiql-jni/src/compiler.rs`
```rust
#[no_mangle]
pub extern "system" fn Java_org_partiql_jni_PlanCompiler_nativeCompile(
    env: JNIEnv,
    _class: JClass,
    sql: JString,
    context_handle: jlong,  // ✅ ADD THIS PARAMETER
) -> jlong {
    // Get CompilationContext from handle
    let context = get_compilation_context(context_handle)?;
    let plan_compiler = PlanCompiler::new(&context);
    // Use context during compilation
}
```

**Update:** `partiql-jni/src/vm.rs`
```rust
#[no_mangle]
pub extern "system" fn Java_org_partiql_jni_PartiQLVM_nativeNew(
    env: JNIEnv,
    _class: JClass,
    plan_handle: jlong,
    context_handle: jlong,  // ✅ REMOVE UNDERSCORE, USE THIS
) -> jlong {
    let plan = get_plan(plan_handle)?;
    let exec_context = get_execution_context(context_handle)?;  // ✅ GET FROM HANDLE
    let vm = PartiQLVM::new((*plan).clone(), &exec_context)?;
    Ok(create_vm_handle(vm) as jlong)
}
```

#### 4. Type Conversions

**File:** `partiql-jni/src/conversion.rs` (currently has stubs)

Add conversions between Rust and Java catalog types:
- `bindings_name_to_jobject()` - Rust → Java BindingsName
- `jobject_to_bindings_name_list()` - Java List → Rust Vec
- `scan_source_to_jobject()` - Rust → Java ScanSource
- `jobject_to_scan_layout()` - Java → Rust ScanLayout
- And more for complete catalog integration

#### 5. Testing

Create integration tests:
```java
@Test
void testCatalogIntegration() {
    // 1. Create CompilationContext
    CompilationContext compContext = new CompilationContext();
    
    // 2. Register catalog
    long catalogId = compContext.addCatalog("main", new MyCompilationCatalog());
    
    // 3. Compile query
    PlanCompiler compiler = new PlanCompiler();
    CompiledPlan plan = compiler.compile("SELECT * FROM users", compContext);
    
    // 4. Create ExecutionContext
    ExecutionContext execContext = new ExecutionContext();
    execContext.addCatalog(catalogId, new MyExecutionCatalog(userData));
    
    // 5. Execute
    try (PartiQLVM vm = new PartiQLVM(plan, execContext)) {
        ExecutionResult result = vm.execute();
        // Verify results from custom catalog
    }
}
```

---

## Architecture Summary

### Current Flow (Phase 1.5)
```
SQL → PlanCompiler.compile(sql, context)
      → nativeCompile(sql, contextHandle)  
      → Rust IGNORES contextHandle, uses default catalog
      → CompiledPlan

VM.execute() → nativeNew(planHandle, contextHandle)
             → Rust IGNORES contextHandle, uses default ExecutionContext
             → Executes with default catalog only
```

### Target Flow (Phase 2)
```
Compilation:
SQL → PlanCompiler.compile(sql, CompilationContext)
      → Rust gets context from handle
      → Calls JavaCompilationCatalog.getTable() via JNI callback
      → Returns DataSourceHandle with catalogId + config
      → CompiledPlan contains catalog references

Execution:
VM.execute() → Rust gets ExecutionContext from handle
             → Calls JavaExecutionCatalog.create(entryId, layout) via JNI callback
             → Returns JavaDataSource
             → Rust calls DataSource.next() via JNI to read rows
             → RegisterReader provides row data to query engine
```

---

## Implementation Roadmap

### Immediate Next Steps
1. ✅ Update this status document (current task)
2. Create `partiql-jni/src/context.rs` with context handle management
3. Add missing parameter to `compiler.rs::nativeCompile`
4. Remove underscore from `vm.rs::nativeNew` context parameter
5. Implement `catalog_bridge.rs` with JNI callback mechanism
6. Add type conversion helpers in `conversion.rs`
7. Create integration tests
8. Document usage examples

### Timeline Estimate
- Context handle management: 1 day
- Catalog bridge with callbacks: 3-5 days
- Type conversions: 2 days
- Integration tests: 2 days
- Documentation: 1 day

**Total: ~2 weeks for full catalog integration**

---

## Workarounds for Now

Until Phase 2 is complete, users can:

1. **Use default catalog only** - Works for simple queries on in-memory data
2. **Pre-process data** - Load data into formats the default catalog understands
3. **Fork and extend** - Modify `compiler.rs` to use custom Rust catalogs directly

---

## Design Validation

✅ **BindingsName** - Final class with `isDelimited()` as requested  
✅ **API Consistency** - Java API matches Rust types exactly  
✅ **Thread Safety** - Contexts can be per-thread for different datasets  
✅ **Memory Safety** - Handle-based approach, no JNI global refs for PartiQL objects  
✅ **Separation of Concerns** - Compilation vs Execution contexts clearly separated  
⚠️ **Implementation Gap** - Java API complete, Rust implementation uses stubs

---

**Status:** Documentation updated to reflect accurate implementation state (2026-02-03)
