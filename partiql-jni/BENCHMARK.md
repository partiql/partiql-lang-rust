# PartiQL JNI Benchmarks

This directory contains JMH (Java Microbenchmark Harness) benchmarks for comparing the performance of partiql-jni against the reference partiql-eval implementation.

## Overview

The benchmarks measure query execution time for both implementations using the query:
```sql
SELECT a, b FROM data WHERE a % 2 = 0
```

This tests filtering performance on a simple dataset with two columns (an integer and a string).

## Benchmark Structure

- **PartiQLJniBenchmark**: Benchmarks the Rust-based partiql-jni implementation
- **PartiQLEvalBenchmark**: Benchmarks the Kotlin-based partiql-eval:1.3.3 implementation
- **BenchmarkDataGenerator**: Generates consistent test data for both implementations

## Running Benchmarks

### Prerequisites

1. Ensure you have Java 11 or later installed
2. The native library must be built (automatically handled by Gradle)

### Run All Benchmarks

```bash
./gradlew jmh
```

This will:
- Build the native library if needed
- Compile all benchmark classes
- Run all benchmarks with default parameters
- Generate results in `build/reports/jmh/results.json`

### Run Specific Benchmark

To run only the partiql-jni benchmark:
```bash
./gradlew jmh --includes='PartiQLJniBenchmark'
```

To run only the partiql-eval benchmark:
```bash
./gradlew jmh --includes='PartiQLEvalBenchmark'
```

### Customize Parameters

The benchmarks support different row counts (100, 1000, 10000 by default). To run with specific parameters:

```bash
./gradlew jmh -Pjmh.includes='.*Benchmark.*' -Pjmh.benchmarkParameters='rowCount=5000'
```

### Quick Run (Fewer Iterations)

For faster results during development:
```bash
./gradlew jmh -Pjmh.warmup=2 -Pjmh.iterations=3 -Pjmh.fork=1
```

## Benchmark Configuration

The benchmarks are configured with:
- **Warmup**: 5 iterations of 1 second each
- **Measurement**: 10 iterations of 1 second each
- **Forks**: 2 (to account for JVM variance)
- **Threads**: 1 (single-threaded execution)
- **Mode**: Average time per operation
- **Time Unit**: Microseconds

## Understanding Results

JMH will output results in this format:

```
Benchmark                            (rowCount)  Mode  Cnt     Score    Error  Units
PartiQLJniBenchmark.executeQuery            100  avgt   20   123.456 ±  5.678  us/op
PartiQLJniBenchmark.executeQuery           1000  avgt   20  1234.567 ± 56.789  us/op
PartiQLJniBenchmark.executeQuery          10000  avgt   20 12345.678 ± 567.89  us/op
PartiQLEvalBenchmark.executeQuery           100  avgt   20   234.567 ±  6.789  us/op
PartiQLEvalBenchmark.executeQuery          1000  avgt   20  2345.678 ± 67.890  us/op
PartiQLEvalBenchmark.executeQuery         10000  avgt   20 23456.789 ± 678.90  us/op
```

Where:
- **Score**: Average time per operation
- **Error**: Confidence interval (99.9%)
- **Units**: us/op (microseconds per operation)

Lower scores indicate better performance.

## Output Files

Benchmark results are saved to:
- **JSON format**: `build/reports/jmh/results.json`
- **Console output**: Displayed during benchmark run

## Analyzing Results

### Compare Implementations

To see which implementation is faster for each dataset size, look at the Score column. For example:
- At 100 rows: Compare PartiQLJniBenchmark vs PartiQLEvalBenchmark
- At 1000 rows: Compare PartiQLJniBenchmark vs PartiQLEvalBenchmark
- At 10000 rows: Compare PartiQLJniBenchmark vs PartiQLEvalBenchmark

### Scaling Analysis

Observe how execution time scales with data size:
- Linear scaling: Time doubles when data doubles (ideal)
- Super-linear: Time more than doubles (potential performance issues)
- Sub-linear: Time less than doubles (excellent optimization)

## Troubleshooting

### Native Library Not Found

If you see errors about the native library not being found:
```bash
./gradlew copyNativeLib
./gradlew jmh
```

### Out of Memory Errors

If benchmarks fail with OOM errors, increase heap size:
```bash
./gradlew jmh -Pjmh.jvmArgs='-Xmx4g'
```

### Compilation Errors

Clean and rebuild:
```bash
./gradlew clean jmhClasses
./gradlew jmh
```

## Adding New Benchmarks

To add a new benchmark:

1. Create a new class in `src/jmh/java/org/partiql/jni/benchmark/`
2. Annotate with JMH annotations (@Benchmark, @State, etc.)
3. Run `./gradlew jmh` to include it automatically

Example:
```java
@BenchmarkMode(Mode.AverageTime)
@OutputTimeUnit(TimeUnit.MICROSECONDS)
@State(Scope.Benchmark)
public class MyNewBenchmark {
    @Benchmark
    public void testMethod() {
        // benchmark code
    }
}
```

## References

- [JMH Documentation](https://openjdk.org/projects/code-tools/jmh/)
- [JMH Samples](https://github.com/openjdk/jmh/tree/master/jmh-samples/src/main/java/org/openjdk/jmh/samples)
- [PartiQL Specification](https://partiql.org/docs.html)
