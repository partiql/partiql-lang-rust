# PartiQL JNI Bindings

JNI (Java Native Interface) bindings for the PartiQL Rust engine, providing a high-performance Java API for executing PartiQL queries.

## Overview

This crate provides Java bindings to the PartiQL Rust engine, enabling Java applications to execute PartiQL queries with near-native performance. The implementation uses JNI to bridge between Java and Rust, providing an idiomatic Java API with proper resource management.

## Features

- ✅ **Complete SQL Compilation Pipeline**: SQL → AST → Logical Plan → Compiled Plan
- ✅ **Streaming Query Results**: Iterator-based result access with zero-copy RegisterReader
- ✅ **Thread-Safe Plans**: CompiledPlan can be shared across threads
- ✅ **RAII Resource Management**: All resources implement AutoCloseable
- ✅ **Direct Register Access**: RegisterReader provides efficient column access
- ✅ **Idiomatic Java API**: Standard Iterator pattern, try-with-resources support

## Requirements

### Build Requirements
- **Rust**: 1.70 or later (with Cargo)
- **Java**: JDK 17 or later
- **Gradle**: 8.5 or later (optional - wrapper included)

### Runtime Requirements
- Java 17 or later
- Native library for your platform (automatically packaged in JAR)

## Building

### Option 1: Using Gradle (Recommended)

If you don't have Gradle installed, first install it:
```bash
# macOS
brew install gradle

# Linux (Ubuntu/Debian)
sudo apt-get install gradle

# Or download from https://gradle.org/install/
```

Then build the project:
```bash
# Build everything (Rust + Java)
cd partiql-jni
gradle build

# Or use specific tasks
gradle buildRustLib    # Build only Rust library
gradle compileJava     # Compile only Java code
gradle test            # Run tests
gradle jar             # Create JAR file
```

## Quick Start

### Basic Usage

```java
import org.partiql.jni.*;

public class Example {
    public static void main(String[] args) {
        // Create compiler
        PlanCompiler compiler = new PlanCompiler();
        
        // Compile and execute query
        try (CompiledPlan plan = compiler.compile("SELECT 42 AS answer")) {
            try (PartiQLVM vm = new PartiQLVM(plan)) {
                try (ExecutionResult result = vm.execute()) {
                    if (result.isQuery()) {
                        try (QueryIterator iter = result.asQueryIterator()) {
                            while (iter.hasNext()) {
                                RegisterReader row = iter.next();
                                Long answer = row.getI64(0);
                                System.out.println("Answer: " + answer);
                            }
                        }
                    }
                }
            }
        }
    }
}
```

### Working with Multiple Rows

```java
try (CompiledPlan plan = compiler.compile(
    "SELECT id, name, age FROM users WHERE age > 21"
)) {
    try (PartiQLVM vm = new PartiQLVM(plan)) {
        try (ExecutionResult result = vm.execute()) {
            try (QueryIterator iter = result.asQueryIterator()) {
                while (iter.hasNext()) {
                    RegisterReader row = iter.next();
                    
                    Long id = row.getI64(0);
                    String name = row.getStr(1);
                    Long age = row.getI64(2);
                    
                    System.out.printf("User %d: %s (age %d)%n", 
                        id, name, age);
                }
            }
        }
    }
}
```

### Reusing Compiled Plans

```java
PlanCompiler compiler = new PlanCompiler();

// Compile once
try (CompiledPlan plan = compiler.compile("SELECT * FROM users")) {
    
    // Execute multiple times with different VMs (even concurrently)
    try (PartiQLVM vm1 = new PartiQLVM(plan)) {
        processResults(vm1.execute());
    }
    
    try (PartiQLVM vm2 = new PartiQLVM(plan)) {
        processResults(vm2.execute());
    }
}
```

### Reusing VM Across Queries

```java
try (PartiQLVM vm = new PartiQLVM(plan1)) {
    // Execute first plan
    processResults(vm.execute());
    
    // Load and execute different plan
    vm.loadPlan(plan2);
    processResults(vm.execute());
}
```

## API Reference

### Core Classes

#### `PlanCompiler`
Compiles PartiQL SQL strings into executable plans.

```java
PlanCompiler compiler = new PlanCompiler();
CompiledPlan plan = compiler.compile("SELECT * FROM data");
```

#### `CompiledPlan`
Thread-safe compiled query plan. Can be shared across threads.

```java
try (CompiledPlan plan = compiler.compile(sql)) {
    // Use plan...
}
```

#### `PartiQLVM`
Single-threaded virtual machine for executing plans.

```java
try (PartiQLVM vm = new PartiQLVM(plan)) {
    ExecutionResult result = vm.execute();
    vm.loadPlan(anotherPlan);  // Reuse VM
}
```

#### `ExecutionResult`
Result of query execution. Currently supports Query results only.

```java
try (ExecutionResult result = vm.execute()) {
    if (result.isQuery()) {
        QueryIterator iter = result.asQueryIterator();
        // Process rows...
    }
}
```

#### `QueryIterator`
Streaming iterator over query result rows. Implements standard Java `Iterator<RegisterReader>`.

```java
try (QueryIterator iter = result.asQueryIterator()) {
    while (iter.hasNext()) {
        RegisterReader row = iter.next();
        // Access row data...
    }
}
```

#### `RegisterReader`
Direct access to row column values. Each RegisterReader is valid only until the next `next()` call.

```java
RegisterReader row = iter.next();
Long id = row.getI64(0);        // Column 0 as i64
String name = row.getStr(1);    // Column 1 as string
Value val = row.getValue(2);    // Column 2 as generic Value
```

### Exception Hierarchy

All exceptions extend `PartiQLException` (a RuntimeException):

- `PartiQLException` - Base exception
- `TypeException` - Type mismatch errors
- `IllegalStateException` - Invalid operation state
- `NotImplementedException` - Feature not yet implemented
- `PlanningException` - Query planning errors

## Architecture

### Memory Model

```
┌─────────────────────────────────────┐
│ Java Heap                            │
│  CompiledPlan (handle)              │
│  PartiQLVM (handle)                 │
│  QueryIterator (handle)             │
└─────────────────────────────────────┘
              │
              │ JNI calls with handles
              ▼
┌─────────────────────────────────────┐
│ Rust Heap                            │
│  HandleMap<CompiledPlan>            │
│  HandleMap<PartiQLVM>               │
│  HandleMap<QueryIterator>           │
│  Arena (per-row memory)             │
│  Register array (reused)            │
└─────────────────────────────────────┘
```

### Thread Safety

- **CompiledPlan**: Thread-safe, can be shared across threads
- **PartiQLVM**: Single-threaded, one VM per thread
- **QueryIterator**: Single-threaded, tied to its VM
- **RegisterReader**: Lifetime tied to iterator, invalidated on next()

### Resource Management

All resources implement AutoCloseable and follow RAII principles:

1. Use try-with-resources for automatic cleanup
2. Resources are freed when close() is called
3. Handles are validated on every JNI call
4. Arena memory is reset per row

## Performance Considerations

### Best Practices

1. **Reuse CompiledPlans**: Compile once, execute many times
2. **Pool VMs**: For concurrent execution, create VM pool
3. **Copy Values Early**: Extract values from RegisterReader before next()
4. **Batch Operations**: Group related queries when possible

### Known Limitations

- `QueryIterator.hasNext()` cannot peek without consuming (Rust iterator limitation)
- RegisterReader values only valid until next `next()` call
- No custom UDF support yet
- ExecutionResult only supports Query variant currently

## Development

### Project Structure

```
partiql-jni/
├── Cargo.toml              # Rust crate configuration
├── build.rs                # Rust build script
├── build.gradle.kts        # Gradle build configuration
├── settings.gradle.kts     # Gradle settings
├── src/                    # Rust source code
│   ├── lib.rs
│   ├── handles.rs          # Handle management
│   ├── error.rs            # Error conversion
│   ├── compiler.rs         # Compiler JNI bindings
│   ├── plan.rs             # Plan JNI bindings
│   ├── vm.rs               # VM JNI bindings
│   ├── result.rs           # Result/Iterator JNI bindings
│   ├── register_reader.rs  # RegisterReader JNI bindings
│   └── conversion.rs       # Value conversion (stub)
└── java/                   # Java source code
    └── org/partiql/jni/
        ├── PlanCompiler.java
        ├── CompiledPlan.java
        ├── PartiQLVM.java
        ├── ExecutionResult.java
        ├── QueryIterator.java
        ├── RegisterReader.java
        ├── Value.java
        ├── NativeLibrary.java
        └── exceptions/
            ├── PartiQLException.java
            ├── TypeException.java
            ├── NotImplementedException.java
            ├── IllegalStateException.java
            └── PlanningException.java
```

### Running Tests

```bash
# Run all tests
gradle test

# Run with verbose output
gradle test --info

# Run specific test
gradle test --tests "org.partiql.jni.PartiQLVMTest"
```

### Building Documentation

```bash
# Generate JavaDoc
gradle javadoc

# View docs
open build/docs/javadoc/index.html
```

## Troubleshooting

### Native Library Not Found

If you see `UnsatisfiedLinkError`, ensure:
1. Native library is in JAR resources
2. Library name matches your platform
3. Java library path is set correctly

### Compilation Errors

```bash
# Clean and rebuild
gradle clean build

# Or manually
cargo clean && cargo build --release
```

### JNI Errors

Enable JNI checks for debugging:
```bash
java -Xcheck:jni -cp partiql-jni.jar YourMainClass
```

## Contributing

See the main PartiQL Rust repository for contribution guidelines.

## License

Apache License 2.0 - See LICENSE file for details.

## Status

**Current Status**: Alpha - Core functionality working, some features pending

### ✅ Implemented
- Full compilation pipeline
- Query execution with streaming results
- RegisterReader with i64/string access
- Proper resource management
- Exception handling

### ⚠️ Pending
- Full Value type hierarchy
- UDF support
- Custom data source readers
- Mutation/Definition result types
- Comprehensive test suite

## Benchmarks

JMH (Java Microbenchmark Harness) benchmarks are available to compare partiql-jni performance against the reference partiql-eval implementation.

### Quick Start

```bash
# Run all benchmarks
./gradlew jmh

# Run specific benchmark
./gradlew jmh --includes='PartiQLJniBenchmark'
```

### Benchmark Details

The benchmarks test query execution time for:
```sql
SELECT a, b FROM data WHERE a % 2 = 0
```

With dataset sizes: 100, 1000, and 10000 rows.

For complete documentation, see [BENCHMARK.md](BENCHMARK.md).

## Related Documentation

- [PartiQL Specification](https://partiql.org/docs.html)
- [PartiQL Rust Engine Design](../docs/final/design.md)
- [JNI Implementation Plan](../docs/scratch/jni_plan.md)
- [Benchmark Guide](BENCHMARK.md)
