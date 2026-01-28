# PartiQL Performance Benchmarks

This directory contains Criterion-based benchmarks for comparing PartiQL engine performance.

## Quick Start

### Recommended: Simple Comparison (Fast)
```bash
# Run lightweight benchmark comparing all 4 engines at 10K rows
cargo bench --package partiql-perf --bench simple_comparison
```

### Comprehensive: Full Analysis (Slow)
```bash
# WARNING: This takes a long time due to many combinations
cargo bench --package partiql-perf --bench engine_comparison
```

## Available Benchmarks

### 1. Simple Comparison (`simple_comparison.rs`) - RECOMMENDED

**Fast, focused benchmark** comparing all engines with a single query at 10K rows:
- Query: `SELECT a, b FROM data WHERE a % 100 = 0`
- All 4 engines: Legacy, Vectorized-1, Vectorized-1024, Hybrid
- ~30 samples per engine for statistical significance
- **Completes in 2-3 minutes**

```bash
# Run the simple benchmark
cargo bench --bench simple_comparison
```

### 2. Engine Comparison (`engine_comparison.rs`) - COMPREHENSIVE

**Full benchmark suite** with extensive coverage:
- 4 different queries (projection, filters, complex expressions)
- 5 data sizes (100, 1K, 10K, 100K, 1M rows)
- 4 engines with all combinations
- **WARNING: Takes 30+ minutes and may hang during compilation**

```bash
# Run specific groups to avoid the full suite
cargo bench --bench engine_comparison -- legacy_scaling
cargo bench --bench engine_comparison -- vectorized_1_scaling
cargo bench --bench engine_comparison -- engines_10k
```

## Troubleshooting

### Benchmark Hangs During Compilation

The `engine_comparison` benchmark can stall while compiling due to the large number of combinations. Solutions:

1. **Use `simple_comparison` instead** (recommended for quick results)
2. **Run specific benchmark groups** instead of all at once
3. **Reduce data sizes** in the QUERIES or DATA_SIZES constants
4. **Increase system resources** (more RAM/CPU for parallel compilation)

### Running Specific Benchmark Groups

```bash
# Fast comparisons at fixed sizes
cargo bench --bench engine_comparison -- engines_10k
cargo bench --bench engine_comparison -- engines_100k

# Individual engine scaling tests
cargo bench --bench engine_comparison -- legacy_scaling
cargo bench --bench engine_comparison -- hybrid_scaling
```

## Benchmark Structure

### Engines Tested
- **Legacy**: Row-at-a-time iterator-based engine
- **Vectorized-1**: Batch processing with batch_size=1 (row-by-row vectorized)
- **Vectorized-1024**: Batch processing with batch_size=1024 (true batching)
- **Hybrid**: Hybrid row/batch engine

### Query Examples
- Simple projection: `SELECT a, b FROM data`
- Filter with modulo: `SELECT a, b FROM data WHERE a % 100 = 0`
- Complex filter: `SELECT a, b FROM data WHERE ((a - a + b - b + a - a + b - b) + a % 100000) = 0`
- Range filter: `SELECT a, b FROM data WHERE a > 1000 AND a < 9000`

## Understanding Results

Criterion generates detailed HTML reports in `target/criterion/` including:
- Performance graphs with confidence intervals
- Statistical analysis of results
- Comparison charts between engines
- Regression detection

### Key Metrics

1. **Throughput (rows/sec)**: Higher is better
2. **Latency (ms)**: Lower is better
3. **Scaling efficiency**: How performance scales with data size
4. **Batch size impact**: Compare Vectorized-1 vs Vectorized-1024

### Expected Patterns

- **Small datasets (< 1K rows)**: Legacy and Hybrid may be faster due to lower overhead
- **Large datasets (> 10K rows)**: Vectorized-1024 should show better performance
- **Vectorized-1 vs Legacy**: Similar performance (vectorized overhead vs iterator overhead)
- **Hybrid**: Should be fastest for in-memory data due to minimal abstraction

## Methodology

The benchmarks use Criterion's best practices:
- **Compilation happens once** per configuration (not in timing loop)
- **Only execution time is measured** (setup excluded)
- **Automatic warmup** and statistical analysis
- **Outlier detection** and removal
- **Confidence intervals** for all measurements

This addresses issues in the original `partiql_benchmarks.rs` where:
- Compilation was inside the timing loop for vectorized
- No statistical significance testing
- Insufficient warmup iterations
- No variance reporting

## Baseline Comparisons

```bash
# Save current results as baseline
cargo bench --bench simple_comparison -- --save-baseline main

# Compare against baseline after changes
cargo bench --bench simple_comparison -- --baseline main
```

## Profiling

For detailed profiling with flamegraphs:

```bash
cargo install flamegraph

# Profile specific engine (use partiql-profile binary)
cargo flamegraph --bin partiql-profile -- --engine legacy --data mem:10000
cargo flamegraph --bin partiql-profile -- --engine vectorized-1024 --data mem:10000
```

## Alternative: Manual Benchmarks

For quick ad-hoc testing without Criterion:

```bash
# Run manual benchmark suite with your own data
cargo run --release --bin partiql-benchmarks -- ~/Desktop/test_data/data_b1_n1.ion

# Or with in-memory data
cargo run --release --bin partiql-benchmarks -- --include-mem 10000
```

See `../src/bin/partiql_benchmarks.rs` for the manual benchmark tool.
