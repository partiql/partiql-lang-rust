# PartiQL JNI Bindings Implementation Plan

>Author: Cline AI Assistant
>Date: 2026-01-29
>Status: Design Phase

## 1. Executive Summary

This document outlines the plan for creating JNI (Java Native Interface) bindings for the PartiQL Rust engine, specifically exposing the `PartiQLVM` to Java applications. The implementation will maintain API consistency with the Rust implementation while providing idiomatic Java interfaces.

## 2. Design Goals

1. **API Consistency**: Keep Java class/interface names matching Rust (PartiQLVM, CompiledPlan, etc.)
2. **Memory Safety**: Leverage Rust ownership through opaque handle system
3. **Zero JNI Global References**: Use handle maps instead of JNI global refs for PartiQL objects
4. **Idiomatic Java**: Provide AutoCloseable resources and builder patterns
5. **Performance**: Minimize JNI boundary crossings and data conversions
6. **Service Integration**: Design for long-running service environments

## 3. Architecture Overview

### 3.1 Memory Management Model

```
┌─────────────────────────────────────────────────┐
│ Java Heap                                        │
│  PartiQLVM (nativeHandle=0x12345)              │
│  CompiledPlan (nativeHandle=0xABCDE)           │
└─────────────────────────────────────────────────┘
                     │
                     │ JNI call with handle (long)
                     ▼
┌─────────────────────────────────────────────────┐
│ Rust Heap                                        │
│  HandleMap<u64, Box<PartiQLVM>>                 │
│    0x12345 → PartiQLVM { ... }                  │
│  HandleMap<u64, Arc<CompiledPlan>>              │
│    0xABCDE → Arc<CompiledPlan> { ... }          │
└─────────────────────────────────────────────────┘
```

**Key Principle**: Rust owns all PartiQL objects. Java holds opaque handles (long values) that index into concurrent hash maps.

### 3.2 Thread Safety

- `CompiledPlan`: Wrapped in `Arc<>`, thread-safe, shareable across Java threads
- `PartiQLVM`: Single-threaded, one VM per Java thread
- Handle maps: Use `DashMap` or `RwLock<HashMap>` for concurrent access

## 4. Project Structure

```
partiql-jni/
├── Cargo.toml                          # Rust JNI crate config
├── build.rs                            # Build script for JNI setup
├── src/
│   ├── lib.rs                          # JNI exports and handle management
│   ├── vm.rs                           # PartiQLVM JNI wrapper
│   ├── compiler.rs                     # PlanCompiler JNI wrapper
│   ├── result.rs                       # ExecutionResult JNI wrapper
│   ├── error.rs                        # Error conversion (Rust ↔ Java)
│   ├── conversion.rs                   # Value conversion layer
│   └── handles.rs                      # Global handle management
├── java/
│   └── org/partiql/jni/
│       ├── PartiQLVM.java
│       ├── CompiledPlan.java
│       ├── PlanCompiler.java
│       ├── ExecutionResult.java
│       ├── QueryIterator.java
│       ├── RowView.java
│       ├── ValueView.java
│       ├── Schema.java
│       ├── NativeLibrary.java          # Native lib loading
│       └── exceptions/
│           ├── PartiQLException.java
│           ├── TypeException.java
│           ├── NotImplementedException.java
│           └── IllegalStateException.java
└── build.gradle.kts                    # Gradle build config
```

## 5. Core JNI Implementation

### 5.1 Handle Management System

**Rust Side** (`handles.rs`):

```rust
use dashmap::DashMap;
use once_cell::sync::Lazy;
use std::sync::Arc;
use std::sync::atomic::{AtomicU64, Ordering};

// Global handle storage
static VM_HANDLES: Lazy<DashMap<u64, Box<PartiQLVM>>> = 
    Lazy::new(|| DashMap::new());

static PLAN_HANDLES: Lazy<DashMap<u64, Arc<CompiledPlan>>> = 
    Lazy::new(|| DashMap::new());

static NEXT_HANDLE: AtomicU64 = AtomicU64::new(1);

// Handle operations
pub fn create_vm_handle(vm: PartiQLVM) -> u64 {
    let handle = NEXT_HANDLE.fetch_add(1, Ordering::Relaxed);
    VM_HANDLES.insert(handle, Box::new(vm));
    handle
}

pub fn get_vm(handle: u64) -> Result<&mut PartiQLVM, JniError> {
    VM_HANDLES.get_mut(&handle)
        .map(|mut r| r.value_mut() as *mut PartiQLVM)
        .map(|ptr| unsafe { &mut *ptr })
        .ok_or(JniError::InvalidHandle)
}

pub fn remove_vm_handle(handle: u64) -> Option<Box<PartiQLVM>> {
    VM_HANDLES.remove(&handle).map(|(_, vm)| vm)
}

pub fn create_plan_handle(plan: CompiledPlan) -> u64 {
    let handle = NEXT_HANDLE.fetch_add(1, Ordering::Relaxed);
    PLAN_HANDLES.insert(handle, Arc::new(plan));
    handle
}

pub fn get_plan(handle: u64) -> Result<Arc<CompiledPlan>, JniError> {
    PLAN_HANDLES.get(&handle)
        .map(|r| r.value().clone())
        .ok_or(JniError::InvalidHandle)
}

pub fn remove_plan_handle(handle: u64) -> Option<Arc<CompiledPlan>> {
    PLAN_HANDLES.remove(&handle).map(|(_, plan)| plan)
}
```

### 5.2 PartiQLVM JNI Wrapper

**Rust Side** (`vm.rs`):

```rust
use jni::JNIEnv;
use jni::objects::{JClass, JObject};
use jni::sys::{jlong, jboolean};

#[no_mangle]
pub extern "system" fn Java_org_partiql_jni_PartiQLVM_nativeNew(
    env: JNIEnv,
    _class: JClass,
    plan_handle: jlong,
) -> jlong {
    jni_guard!(env, {
        let plan = get_plan(plan_handle as u64)?;
        let vm = PartiQLVM::new((*plan).clone(), None)?;
        Ok(create_vm_handle(vm) as jlong)
    })
}

#[no_mangle]
pub extern "system" fn Java_org_partiql_jni_PartiQLVM_nativeExecute(
    env: JNIEnv,
    _class: JClass,
    vm_handle: jlong,
) -> jlong {
    jni_guard!(env, {
        let vm = get_vm(vm_handle as u64)?;
        let result = vm.execute()?;
        Ok(create_result_handle(result) as jlong)
    })
}

#[no_mangle]
pub extern "system" fn Java_org_partiql_jni_PartiQLVM_nativeLoadPlan(
    env: JNIEnv,
    _class: JClass,
    vm_handle: jlong,
    plan_handle: jlong,
) {
    jni_guard_void!(env, {
        let vm = get_vm(vm_handle as u64)?;
        let plan = get_plan(plan_handle as u64)?;
        vm.load_plan((*plan).clone(), None)?;
        Ok(())
    })
}

#[no_mangle]
pub extern "system" fn Java_org_partiql_jni_PartiQLVM_nativeClose(
    env: JNIEnv,
    _class: JClass,
    vm_handle: jlong,
) {
    jni_guard_void!(env, {
        remove_vm_handle(vm_handle as u64);
        Ok(())
    })
}
```

**Java Side** (`PartiQLVM.java`):

```java
package org.partiql.jni;

public final class PartiQLVM implements AutoCloseable {
    private long nativeHandle;
    private boolean closed = false;
    
    static {
        NativeLibrary.ensureLoaded();
    }
    
    private static native long nativeNew(long compiledPlanHandle);
    private static native long nativeExecute(long handle);
    private static native void nativeLoadPlan(long handle, long planHandle);
    private static native void nativeClose(long handle);
    
    /**
     * Creates a new PartiQLVM from a compiled plan.
     */
    public PartiQLVM(CompiledPlan compiledPlan) {
        this.nativeHandle = nativeNew(compiledPlan.getNativeHandle());
    }
    
    /**
     * Executes the currently loaded plan.
     * @return ExecutionResult containing query results, mutation summary, or definition summary
     */
    public ExecutionResult execute() {
        checkNotClosed();
        long resultHandle = nativeExecute(nativeHandle);
        return new ExecutionResult(resultHandle);
    }
    
    /**
     * Loads a new plan for execution. The VM must not have an active iterator.
     */
    public void loadPlan(CompiledPlan compiledPlan) {
        checkNotClosed();
        nativeLoadPlan(nativeHandle, compiledPlan.getNativeHandle());
    }
    
    @Override
    public void close() {
        if (!closed) {
            nativeClose(nativeHandle);
            closed = true;
        }
    }
    
    private void checkNotClosed() {
        if (closed) {
            throw new IllegalStateException("VM already closed");
        }
    }
}
```

### 5.3 Value Conversion Layer

**Design Principle**: Based on design.md's ValueWriter concept, provide type-safe conversion without exposing internal ValueRef representation.

**Rust Side** (`conversion.rs`):

```rust
use jni::JNIEnv;
use jni::objects::{JObject, JValue};
use partiql_eval::engine::{ValueView, RowView};

pub fn value_view_to_jobject(
    env: &JNIEnv,
    value: &ValueView,
) -> Result<JObject, JniError> {
    match value {
        ValueView::Null => Ok(JObject::null()),
        ValueView::Missing => {
            // Create special Missing sentinel or use null
            Ok(JObject::null())
        }
        
        ValueView::Bool(b) => {
            let bool_class = env.find_class("java/lang/Boolean")?;
            env.new_object(
                bool_class,
                "(Z)V",
                &[JValue::Bool(*b as u8)]
            ).map_err(Into::into)
        }
        
        ValueView::I64(v) => {
            let long_class = env.find_class("java/lang/Long")?;
            env.new_object(
                long_class,
                "(J)V",
                &[JValue::Long(*v)]
            ).map_err(Into::into)
        }
        
        ValueView::F64(v) => {
            let double_class = env.find_class("java/lang/Double")?;
            env.new_object(
                double_class,
                "(D)V",
                &[JValue::Double(*v)]
            ).map_err(Into::into)
        }
        
        ValueView::String(s) => {
            env.new_string(s)
                .map(|s| s.into())
                .map_err(Into::into)
        }
        
        ValueView::Tuple(tuple) => {
            // Create HashMap<String, Object>
            let map_class = env.find_class("java/util/HashMap")?;
            let map = env.new_object(map_class, "()V", &[])?;
            
            // Populate map with tuple fields
            for (key, value) in tuple.iter() {
                let key_str = env.new_string(key)?;
                let value_obj = value_view_to_jobject(env, &value)?;
                
                env.call_method(
                    map,
                    "put",
                    "(Ljava/lang/Object;Ljava/lang/Object;)Ljava/lang/Object;",
                    &[JValue::Object(key_str.into()), JValue::Object(value_obj)]
                )?;
            }
            
            Ok(map)
        }
        
        ValueView::List(list) => {
            // Create ArrayList<Object>
            let list_class = env.find_class("java/util/ArrayList")?;
            let list_obj = env.new_object(list_class, "()V", &[])?;
            
            for item in list.iter() {
                let item_obj = value_view_to_jobject(env, &item)?;
                env.call_method(
                    list_obj,
                    "add",
                    "(Ljava/lang/Object;)Z",
                    &[JValue::Object(item_obj)]
                )?;
            }
            
            Ok(list_obj)
        }
    }
}

pub fn row_view_to_jobject(
    env: &JNIEnv,
    row: &RowView,
    schema: &Schema,
) -> Result<JObject, JniError> {
    // Create RowView Java object
    let row_class = env.find_class("org/partiql/jni/RowView")?;
    let row_obj = env.new_object(row_class, "()V", &[])?;
    
    // Populate with values
    for (idx, value) in row.iter().enumerate() {
        let java_value = value_view_to_jobject(env, value)?;
        // Call setter or store in internal array
        env.call_method(
            row_obj,
            "setValue",
            "(ILjava/lang/Object;)V",
            &[JValue::Int(idx as i32), JValue::Object(java_value)]
        )?;
    }
    
    Ok(row_obj)
}
```

**Java Side** (`ValueView.java`):

```java
package org.partiql.jni;

import org.partiql.jni.exceptions.TypeException;
import java.util.List;
import java.util.Map;

/**
 * View of a PartiQL value. Provides type-safe accessors.
 */
public interface ValueView {
    /**
     * Returns true if this value is SQL NULL.
     */
    boolean isNull();
    
    /**
     * Returns true if this value is PartiQL MISSING.
     */
    boolean isMissing();
    
    /**
     * Returns the value as a long.
     * @throws TypeException if the value is not an integer
     */
    long asLong() throws TypeException;
    
    /**
     * Returns the value as a double.
     * @throws TypeException if the value is not a number
     */
    double asDouble() throws TypeException;
    
    /**
     * Returns the value as a string.
     * @throws TypeException if the value is not a string
     */
    String asString() throws TypeException;
    
    /**
     * Returns the value as a boolean.
     * @throws TypeException if the value is not a boolean
     */
    boolean asBoolean() throws TypeException;
    
    /**
     * Returns the value as a map.
     * @throws TypeException if the value is not a tuple/struct
     */
    Map<String, ValueView> asMap() throws TypeException;
    
    /**
     * Returns the value as a list.
     * @throws TypeException if the value is not a list/array
     */
    List<ValueView> asList() throws TypeException;
    
    /**
     * Converts the value to a Java object.
     * Primitives are boxed, collections are converted to Java collections.
     */
    Object asJavaObject();
}
```

## 6. Iterator Pattern for Streaming

### 6.1 Challenge

Rust's `QueryIterator` uses lending iterator pattern (borrow checker), but Java needs owned objects per iteration.

### 6.2 Solution: Materialize Per Row

**Rust Side** (`result.rs`):

```rust
use jni::JNIEnv;
use jni::objects::{JClass, JObject};
use jni::sys::{jlong, jboolean, jobject};

#[no_mangle]
pub extern "system" fn Java_org_partiql_jni_QueryIterator_nativeHasNext(
    env: JNIEnv,
    _class: JClass,
    iter_handle: jlong,
) -> jboolean {
    jni_guard!(env, {
        let iter = get_iterator(iter_handle as u64)?;
        Ok(iter.peek().is_some() as jboolean)
    })
}

#[no_mangle]
pub extern "system" fn Java_org_partiql_jni_QueryIterator_nativeNext(
    env: JNIEnv,
    _class: JClass,
    iter_handle: jlong,
) -> jobject {
    jni_guard!(env, {
        let iter = get_iterator_mut(iter_handle as u64)?;
        let schema = iter.schema();
        
        if let Some(row) = iter.next()? {
            // Materialize row into Java RowView object
            let row_obj = row_view_to_jobject(&env, &row, schema)?;
            Ok(row_obj.into_inner())
        } else {
            Ok(JObject::null().into_inner())
        }
    })
}

#[no_mangle]
pub extern "system" fn Java_org_partiql_jni_QueryIterator_nativeClose(
    env: JNIEnv,
    _class: JClass,
    iter_handle: jlong,
) {
    jni_guard_void!(env, {
        remove_iterator_handle(iter_handle as u64);
        Ok(())
    })
}
```

**Java Side** (`QueryIterator.java`):

```java
package org.partiql.jni;

import java.util.Iterator;
import java.util.NoSuchElementException;

/**
 * Iterator over query results. Implements AutoCloseable for resource management.
 */
public final class QueryIterator implements Iterator<RowView>, AutoCloseable {
    private long nativeHandle;
    private boolean closed = false;
    
    static {
        NativeLibrary.ensureLoaded();
    }
    
    private static native boolean nativeHasNext(long handle);
    private static native RowView nativeNext(long handle);
    private static native void nativeClose(long handle);
    
    QueryIterator(long nativeHandle) {
        this.nativeHandle = nativeHandle;
    }
    
    @Override
    public boolean hasNext() {
        checkNotClosed();
        return nativeHasNext(nativeHandle);
    }
    
    @Override
    public RowView next() {
        checkNotClosed();
        RowView row = nativeNext(nativeHandle);
        if (row == null) {
            throw new NoSuchElementException();
        }
        return row;
    }
    
    @Override
    public void close() {
        if (!closed) {
            nativeClose(nativeHandle);
            closed = true;
        }
    }
    
    private void checkNotClosed() {
        if (closed) {
            throw new IllegalStateException("Iterator already closed");
        }
    }
}
```

## 7. Error Handling

### 7.1 Exception Hierarchy

**Java Side**:

```java
package org.partiql.jni.exceptions;

/**
 * Base exception for all PartiQL errors.
 */
public class PartiQLException extends RuntimeException {
    public PartiQLException(String message) {
        super(message);
    }
    
    public PartiQLException(String message, Throwable cause) {
        super(message, cause);
    }
}

/**
 * Thrown when a type mismatch occurs.
 */
public class TypeException extends PartiQLException {
    public TypeException(String message) {
        super(message);
    }
}

/**
 * Thrown when a feature is not yet implemented.
 */
public class NotImplementedException extends PartiQLException {
    public NotImplementedException(String message) {
        super(message);
    }
}

/**
 * Thrown when an illegal state is detected.
 */
public class IllegalStateException extends PartiQLException {
    public IllegalStateException(String message) {
        super(message);
    }
}

/**
 * Thrown when query planning fails.
 */
public class PlanningException extends PartiQLException {
    public PlanningException(String message) {
        super(message);
    }
}
```

### 7.2 Rust → Java Exception Mapping

**Rust Side** (`error.rs`):

```rust
use jni::JNIEnv;
use partiql_eval::engine::EngineError;

pub enum JniError {
    InvalidHandle,
    EngineError(EngineError),
    JniError(jni::errors::Error),
}

impl From<EngineError> for JniError {
    fn from(err: EngineError) -> Self {
        JniError::EngineError(err)
    }
}

impl From<jni::errors::Error> for JniError {
    fn from(err: jni::errors::Error) -> Self {
        JniError::JniError(err)
    }
}

pub fn throw_engine_error(env: &JNIEnv, err: JniError) {
    let (exception_class, message) = match err {
        JniError::InvalidHandle => (
            "org/partiql/jni/exceptions/IllegalStateException",
            "Invalid native handle".to_string(),
        ),
        JniError::EngineError(engine_err) => match engine_err {
            EngineError::TypeError(msg) => (
                "org/partiql/jni/exceptions/TypeException",
                msg,
            ),
            EngineError::NotImplemented => (
                "org/partiql/jni/exceptions/NotImplementedException",
                "Feature not implemented".to_string(),
            ),
            EngineError::IllegalState(msg) => (
                "org/partiql/jni/exceptions/IllegalStateException",
                msg,
            ),
            _ => (
                "org/partiql/jni/exceptions/PartiQLException",
                engine_err.to_string(),
            ),
        },
        JniError::JniError(jni_err) => (
            "org/partiql/jni/exceptions/PartiQLException",
            format!("JNI error: {}", jni_err),
        ),
    };
    
    let _ = env.throw_new(exception_class, message);
}

// Macro for exception handling
macro_rules! jni_guard {
    ($env:expr, $body:expr) => {
        match (|| -> Result<_, JniError> { $body })() {
            Ok(v) => v,
            Err(e) => {
                throw_engine_error(&$env, e);
                return Default::default();
            }
        }
    };
}

macro_rules! jni_guard_void {
    ($env:expr, $body:expr) => {
        match (|| -> Result<_, JniError> { $body })() {
            Ok(_) => {}
            Err(e) => {
                throw_engine_error(&$env, e);
            }
        }
    };
}
```

## 8. Build System Integration

### 8.1 Cargo Configuration

**Cargo.toml**:

```toml
[package]
name = "partiql-jni"
version = "0.14.0"
edition = "2021"
authors = ["PartiQL Team <partiql-team@amazon.com>"]

[lib]
crate-type = ["cdylib"]  # Dynamic library for JNI

[dependencies]
partiql-eval = { path = "../partiql-eval", version = "0.14.*" }
jni = "0.21"
dashmap = "5"
once_cell = "1"

[build-dependencies]
cc = "1.0"
```

**build.rs**:

```rust
fn main() {
    // Set up JNI include paths if needed
    println!("cargo:rerun-if-changed=build.rs");
    
    // Platform-specific configuration
    if cfg!(target_os = "macos") {
        println!("cargo:rustc-link-arg=-Wl,-rpath,@loader_path");
    } else if cfg!(target_os = "linux") {
        println!("cargo:rustc-link-arg=-Wl,-rpath,$ORIGIN");
    }
}
```

### 8.2 Gradle Configuration

**build.gradle.kts**:

```kotlin
plugins {
    `java-library`
    `maven-publish`
}

group = "org.partiql"
version = "0.14.0"

java {
    sourceCompatibility = JavaVersion.VERSION_17
    targetCompatibility = JavaVersion.VERSION_17
    withSourcesJar()
    withJavadocJar()
}

sourceSets {
    main {
        java {
            srcDir("java")
        }
        resources {
            srcDir("src/main/resources")
        }
    }
}

// Task to build Rust library
tasks.register<Exec>("buildRustLib") {
    workingDir = file(".")
    commandLine = listOf("cargo", "build", "--release")
}

// Copy native library to resources
tasks.register<Copy>("copyNativeLib") {
    dependsOn("buildRustLib")
    
    val targetDir = file("../target/release")
    val resourceDir = file("src/main/resources/native")
    
    from(targetDir) {
        include("*.so", "*.dylib", "*.dll")
    }
    into(resourceDir)
}

tasks.named("compileJava") {
    dependsOn("copyNativeLib")
}

// Clean Rust artifacts
tasks.named("clean") {
    doLast {
        exec {
            workingDir = file(".")
            commandLine = listOf("cargo", "clean")
        }
    }
}

// Test configuration
tasks.test {
    useJUnitPlatform()
    testLogging {
        events("passed", "skipped", "failed")
        exceptionFormat = org.gradle.api.tasks.testing.logging.TestExceptionFormat.FULL
    }
}

dependencies {
    testImplementation("org.junit.jupiter:junit-jupiter:5.9.2")
    testImplementation("org.assertj:assertj-core:3.24.2")
}

// Maven publishing
publishing {
    publications {
        create<MavenPublication>("maven") {
            from(components["java"])
            
            pom {
                name.set("PartiQL JNI")
                description.set("JNI bindings for PartiQL Rust engine")
                url.set("https://github.com/partiql/partiql-lang-rust")
                
                licenses {
                    license {
                        name.set("Apache License 2.0")
                        url.set("https://www.apache.org/licenses/LICENSE-2.0")
                    }
                }
            }
        }
    }
}
```

### 8.3 Native Library Loading

**NativeLibrary.java**:

```java
package org.partiql.jni;

import java.io.IOException;
import java.io.InputStream;
import java.nio.file.Files;
import java.nio.file.Path;
import java.nio.file.StandardCopyOption;

/**
 * Handles loading of the native PartiQL library.
 */
final class NativeLibrary {
    private static boolean loaded = false;
    
    /**
     * Ensures the native library is loaded. This is called automatically
     * by static initializers in classes that need native methods.
     */
    static synchronized void ensureLoaded() {
        if (loaded) {
            return;
        }
        
        String os = System.getProperty("os.name").toLowerCase();
        String libName;
        
        if (os.contains("mac")) {
            libName = "libpartiql_jni.dylib";
        } else if (os.contains("win")) {
            libName = "partiql_jni.dll";
        } else {
            libName = "libpartiql_jni.so";
        }
        
        try {
            // Try to extract from JAR and load
            extractAndLoad(libName);
            loaded = true;
        } catch (Exception e) {
            throw new RuntimeException("Failed to load native library: " + libName, e);
        }
    }
    
    private static void extractAndLoad(String libName) throws IOException {
        InputStream in = NativeLibrary.class
            .getResourceAsStream("/native/" + libName);
        
        if (in == null) {
            throw new IOException("Native library not found in JAR: " + libName);
        }
        
        // Extract to temp file
        Path tempFile = Files.createTempFile("partiql_jni", 
            libName.substring(libName.lastIndexOf('.')));
        tempFile.toFile().deleteOnExit();
        
        Files.copy(in, tempFile, StandardCopyOption.REPLACE_EXISTING);
        
        // Load from temp location
        System.load(tempFile.toAbsolutePath().toString());
    }
}
```

## 9. Usage Examples

### 9.1 Basic Query Execution

```java
import org.partiql.jni.*;

public class BasicExample {
    public static void main(String[] args) {
        // Compile query
        PlanCompiler compiler = new PlanCompiler();
        CompiledPlan plan = compiler.compile(logicalPlan);
        
        // Create VM and execute
        try (PartiQLVM vm = new PartiQLVM(plan)) {
            ExecutionResult result = vm.execute();
            
            if (result.isQuery()) {
                try (QueryIterator iter = result.asQueryIterator()) {
                    while (iter.hasNext()) {
                        RowView row = iter.next();
                        System.out.println("Value: " + row.getLong(0));
                    }
                }
            }
        }
    }
}
```

### 9.2 Multi-Statement Batch

```java
public class BatchExample {
    public void processBatch(List<CompiledPlan> compiledStatements) {
        try (PartiQLVM vm = new PartiQLVM(compiledStatements.get(0))) {
            for (CompiledPlan plan : compiledStatements) {
                vm.loadPlan(plan);
                
                ExecutionResult result = vm.execute();
                switch (result.getType()) {
                    case QUERY:
                        processQuery(result.asQueryIterator());
                        break;
                    case MUTATION:
                        System.out.println("Rows affected: " + 
                            result.asMutationSummary().getRowsAffected());
                        break;
                    case DEFINITION:
                        System.out.println("Objects created: " + 
                            result.asDefinitionSummary().getObjectsCreated());
                        break;
                }
            }
        }
    }
    
    private void processQuery(QueryIterator iter) {
        try (iter) {
            while (iter.hasNext()) {
                RowView row = iter.next();
                // Process row...
            }
        }
    }
}
```

### 9.3 Concurrent Execution

```java
public class ConcurrentExample {
    public void processConcurrently(CompiledPlan plan, int numThreads) 
            throws Exception {
        ExecutorService executor = Executors.newFixedThreadPool(numThreads);
        List<Future<Integer>> futures = new ArrayList<>();
        
        for (int i = 0; i < 100; i++) {
            futures.add(executor.submit(() -> {
                try (PartiQLVM vm = new PartiQLVM(plan);
                     QueryIterator iter = vm.execute().asQueryIterator()) {
                    int count = 0;
                    while (iter.hasNext()) {
                        iter.next();
                        count++;
                    }
                    return count;
                }
            }));
        }
        
        for (Future<Integer> future : futures) {
            System.out.println("Rows: " + future.get());
        }
        
        executor.shutdown();
    }
}
```

## 10. Testing Strategy

### 10.1 Unit Tests (Java)

```java
import org.junit.jupiter.api.Test;
import static org.junit.jupiter.api.Assertions.*;

class PartiQLVMTest {
    @Test
    void testBasicQuery() {
        CompiledPlan plan = compileTestQuery();
        
        try (PartiQLVM vm = new PartiQLVM(plan);
             QueryIterator iter = vm.execute().asQueryIterator()) {
            
            assertTrue(iter.hasNext());
            RowView row = iter.next();
            assertEquals(42, row.getLong(0));
            assertFalse(iter.hasNext());
        }
    }
    
    @Test
    void testResourceCleanup() {
        CompiledPlan plan = compileTestQuery();
        PartiQLVM vm = new PartiQLVM(plan);
        vm.close();
        
        // Should throw after close
        assertThrows(IllegalStateException.class, () -> vm.execute());
    }
    
    @Test
    void testLoadPlan() {
        CompiledPlan plan1 = compileTestQuery("SELECT 1");
        CompiledPlan plan2 = compileTestQuery("SELECT 2");
        
        try (PartiQLVM vm = new PartiQLVM(plan1)) {
            // Execute first plan
            try (QueryIterator iter = vm.execute().asQueryIterator()) {
                RowView row = iter.next();
                assertEquals(1, row.getLong(0));
            }
            
            // Load and execute second plan
            vm.loadPlan(plan2);
            try (QueryIterator iter = vm.execute().asQueryIterator()) {
                RowView row = iter.next();
                assertEquals(2, row.getLong(0));
            }
        }
    }
}
```

### 10.2 Integration Tests

```java
@Test
void testConcurrentExecution() throws Exception {
    CompiledPlan plan = compileTestQuery();
    
    ExecutorService executor = Executors.newFixedThreadPool(4);
    List<Future<Integer>> futures = new ArrayList<>();
    
    for (int i = 0; i < 100; i++) {
        futures.add(executor.submit(() -> {
            try (PartiQLVM vm = new PartiQLVM(plan);
                 QueryIterator iter = vm.execute().asQueryIterator()) {
                int count = 0;
                while (iter.hasNext()) {
                    iter.next();
                    count++;
                }
                return count;
            }
        }));
    }
    
    for (Future<Integer> future : futures) {
        assertEquals(1, future.get());
    }
    
    executor.shutdown();
}
```

## 11. Performance Considerations

### 11.1 JNI Boundary Optimization

1. **Batch Operations**: Group operations to minimize crossings
2. **Direct Buffer Access**: Use ByteBuffer for bulk data transfer when appropriate
3. **Object Pooling**: Reuse RowView objects where possible
4. **Lazy Materialization**: Only convert values when accessed

### 11.2 Memory Management

1. **Handle Cleanup**: Ensure all handles are released via close()
2. **Arena Reuse**: VM arena persists across queries
3. **Register Array**: Pre-allocated, reused per execution
4. **No Global Refs**: Avoid JNI global reference overhead

## 12. Implementation Roadmap

### Phase 1: Foundation (Week 1)
- [x] Create comprehensive design document
- [ ] Create crate skeleton (`partiql-jni/`)
- [ ] Implement handle management system
- [ ] Add basic JNI exports (hello world)
- [ ] Set up Gradle build integration
- [ ] Implement native library loading

### Phase 2: Core API (Week 2)
- [ ] Implement CompiledPlan wrapper
- [ ] Implement PartiQLVM wrapper
- [ ] Implement ExecutionResult wrapper
- [ ] Add basic error handling

### Phase 3: Value Conversion (Week 3)
- [ ] Implement ValueView interface
- [ ] Implement RowView class
- [ ] Add type-safe conversion methods
- [ ] Handle primitive types efficiently

### Phase 4: Iterator Support (Week 4)
- [ ] Implement QueryIterator
- [ ] Add streaming support
- [ ] Implement RAII patterns
- [ ] Add row materialization

### Phase 5: Testing & Polish (Week 5)
- [ ] Write unit tests (Java)
- [ ] Write integration tests
- [ ] Add benchmarks
- [ ] Write documentation
- [ ] Create usage examples

### Phase 6: Advanced Features (Week 6+)
- [ ] Add UDF support (if needed)
- [ ] Implement custom readers (if needed)
- [ ] Add batch processing hints
- [ ] Performance tuning

## 13. Open Questions

1. **UDF Support**: How should Java UDFs be registered and called?
2. **Custom Readers**: Should Java implement ScanProvider interface?
3. **Batch Hints**: How to expose execution hints to Java?
4. **Logging**: Bridge Rust logging to Java logging frameworks?
5. **Metrics**: Expose VM metrics (arena size, register count, etc.)?

## 14. Success Criteria

- [ ] All PartiQL Rust API surface exposed to Java
- [ ] Zero-copy where semantically correct
- [ ] Performance within 10% of pure Rust
- [ ] Memory-safe under concurrent usage
- [ ] Comprehensive test coverage (>80%)
- [ ] Complete usage documentation

## 15. References

- PartiQL Design Doc: `docs/final/design.md`
- JNI Specification: https://docs.oracle.com/javase/8/docs/technotes/guides/jni/
- jni-rs Documentation: https://docs.rs/jni/latest/jni/
- DashMap: https://docs.rs/dashmap/latest/dashmap/

---

**Document Status**: Ready for implementation. Review and approve before proceeding to Phase 1.
