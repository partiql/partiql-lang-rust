# PartiQL JNI Catalog Implementation Status

## ✅ Completed: Java Catalog API (Phase 1)

All Java interfaces and classes for the catalog system have been successfully implemented.

### Core Types (14 files created)

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
  - Manages catalog name → CompilationCatalog mappings
  
- **ExecutionContext.java** - Execution catalog registry
  - `addCatalog(catalogId, catalog)` - maps catalogId to ExecutionCatalog
  - Enables different datasets for same compiled plan

### Updated Core Classes

#### PlanCompiler.java
```java
public CompiledPlan compile(String sql, CompilationContext context)
```
- Now requires CompilationContext parameter
- Context provides catalog resolution during compilation

#### PartiQLVM.java
```java
public PartiQLVM(CompiledPlan plan, ExecutionContext context)
```
- Now requires ExecutionContext parameter
- Context provides data access during execution

## 📋 Remaining Work: Rust JNI Implementation (Phase 2)

The following Rust implementation work is needed to complete the catalog integration:

### 1. Context Handle Management

**File:** `partiql-jni/src/context.rs` (new file)

```rust
// Add handles for CompilationContext and ExecutionContext
static COMPILATION_CONTEXT_HANDLES: Lazy<DashMap<u64, CompilationContext>> = ...;
static EXECUTION_CONTEXT_HANDLES: Lazy<DashMap<u64, ExecutionContext>> = ...;

pub fn create_compilation_context_handle() -> u64 { ... }
pub fn create_execution_context_handle() -> u64 { ... }
```

### 2. Java → Rust Catalog Bridge

**File:** `partiql-jni/src/catalog_bridge.rs` (new file)

Implement JNI callback mechanism to call Java catalog methods from Rust:

```rust
// Bridge CompilationCatalog::getTable() calls from Rust to Java
pub struct JavaCompilationCatalog {
    java_object: GlobalRef,
    jvm: JavaVM,
}

impl CompilationCatalog for JavaCompilationCatalog {
    fn get_table(&self, path: &[BindingsName]) -> Option<DataSourceHandle> {
        // Call Java CompilationCatalog.getTable(List<BindingsName>)
        // Convert Rust BindingsName to Java BindingsName objects
        // Call method and convert result back
    }
}

// Similarly for ExecutionCatalog
pub struct JavaExecutionCatalog { ... }
```

### 3. Native Method Implementations

**Update:** `partiql-jni/src/compiler.rs`

```rust
#[no_mangle]
pub extern "system" fn Java_org_partiql_jni_PlanCompiler_nativeCompile(
    env: JNIEnv,
    _class: JClass,
    sql: JString,
    context_handle: jlong,  // NEW PARAMETER
) -> jlong {
    // Get CompilationContext from handle
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
    context_handle: jlong,  // NEW PARAMETER
) -> jlong {
    // Get ExecutionContext from handle
    // Pass context to VM construction
}
```

**New file:** `partiql-jni/src/context.rs`

```rust
#[no_mangle]
pub extern "system" fn Java_org_partiql_jni_catalog_CompilationContext_nativeNew(
    env: JNIEnv,
    _class: JClass,
) -> jlong {
    // Create CompilationContext and return handle
}

#[no_mangle]
pub extern "system" fn Java_org_partiql_jni_catalog_CompilationContext_nativeAddCatalog(
    env: JNIEnv,
    _class: JClass,
    handle: jlong,
    name: JString,
) -> jlong {
    // Add catalog to context and return catalogId
}

#[no_mangle]
pub extern "system" fn Java_org_partiql_jni_catalog_CompilationContext_registerCatalogCallback(
    env: JNIEnv,
    this: JObject,
    catalog_id: jlong,
    catalog: JObject,
) {
    // Store Java CompilationCatalog reference for callbacks
    // Create JavaCompilationCatalog wrapper
}

// Similar methods for ExecutionContext
```

### 4. Type Conversions

**File:** `partiql-jni/src/conversion.rs` (update)

Add conversions between Rust and Java catalog types:

```rust
// Convert Rust BindingsName to Java BindingsName
pub fn bindings_name_to_jobject(env: &JNIEnv, name: &BindingsName) -> Result<JObject> {
    match name {
        BindingsName::CaseSensitive(s) => {
            // Create Java BindingsName with delimited=true
        }
        BindingsName::CaseInsensitive(s) => {
            // Create Java BindingsName with delimited=false
        }
    }
}

// Convert Java List<BindingsName> to Rust Vec<BindingsName>
pub fn jobject_to_bindings_name_list(env: &JNIEnv, list: JObject) -> Result<Vec<BindingsName>> {
    // Iterate Java List, convert each element
}

// Similar for ScanSource, ScanLayout, etc.
```

### 5. Testing

Create integration tests demonstrating the full catalog flow:

```java
// Test file: partiql-jni/src/test/java/CatalogIntegrationTest.java

@Test
void testCatalogIntegration() {
    // 1. Create CompilationContext
    CompilationContext compContext = new CompilationContext();
    
    // 2. Implement and register CompilationCatalog
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
        // Verify results
    }
}
```

## Architecture Summary

```
Compilation Flow:
SQL → PlanCompiler.compile(sql, CompilationContext)
      → Rust compiler uses CompilationContext
      → Calls Java CompilationCatalog.getTable() via JNI callback
      → Returns DataSourceHandle with catalogId + config
      → CompiledPlan contains catalog references

Execution Flow:
VM.execute() → Rust VM uses ExecutionContext
             → Calls Java ExecutionCatalog.create(entryId, layout) via JNI callback
             → Returns Java DataSource
             → Rust calls DataSource.next() via JNI to read rows
             → RegisterReader provides row data to query engine
```

## Next Steps

1. Implement `partiql-jni/src/catalog_bridge.rs` with JNI callback mechanism
2. Update `partiql-jni/src/compiler.rs` to use CompilationContext
3. Update `partiql-jni/src/vm.rs` to use ExecutionContext
4. Add type conversion helpers in `partiql-jni/src/conversion.rs`
5. Create integration tests
6. Document usage examples

## Design Validation

✅ **BindingsName** - Now a final class with `isDelimited()` method as requested
✅ **API Consistency** - Matches Rust types exactly
✅ **Thread Safety** - Contexts can be per-thread for different datasets
✅ **Memory Safety** - Handle-based approach, no JNI global refs for PartiQL objects
✅ **Separation of Concerns** - Compilation vs Execution contexts clearly separated
