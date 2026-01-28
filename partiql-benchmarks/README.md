# PartiQL Benchmarks

This package contains Criterion-based benchmarks for comparing PartiQL engine performance.

## Features

- **No binaries**: Fast compilation times since only benchmarks are built
- **Criterion integration**: Statistical analysis and HTML reports
- **Multiple engines**: Legacy, Vectorized (batch sizes 1 and 1024), and Hybrid
- **Clean separation**: Isolated from utility tools in `partiql-perf`

## Running Benchmarks

### Run all benchmarks
```bash
cargo bench --package partiql-benchmarks
```

### Run specific benchmark suite
```bash
# Simple comparison (10K rows)
cargo bench --package partiql-benchmarks --bench simple_comparison

# Full engine comparison (multiple data sizes and queries)
cargo bench --package partiql-benchmarks --bench engine_comparison
```

### Run specific test within a suite
```bash
# Only the 10K row cross-engine comparison
cargo bench --package partiql-benchmarks --bench engine_comparison -- engines_10k

# Only legacy engine scaling tests
cargo bench --package partiql-benchmarks --bench engine_comparison -- legacy_scaling
```

## Benchmark Suites

### simple_comparison
Basic comparison at 10K rows with a simple filtered projection query.
- Fast to run (~2-3 minutes)
- Good for quick performance checks

### engine_comparison
Comprehensive comparison across:
- **Data sizes**: 100, 1K, 10K, 100K, 1M rows
- **Queries**: Simple projection, modulo filter, complex filter, range filter
- **Engines**: Legacy, Vectorized-1, Vectorized-1024, Hybrid

Test groups:
- `legacy_scaling`: Legacy engine across data sizes
- `vectorized_1_scaling`: Vectorized with batch size 1
- `vectorized_1024_scaling`: Vectorized with batch size 1024
- `hybrid_scaling`: Hybrid engine across data sizes
- `engines_10k`: All engines at 10K rows
- `engines_100k`: All engines at 100K rows

## Results

Results are saved to `target/criterion/` with:
- HTML reports with plots
- Statistical analysis
- Comparison against previous runs

View results:
```bash
open target/criterion/report/index.html
```

## Adding New Benchmarks

1. Create a new file in `benches/`
2. Add it to `Cargo.toml`:
   ```toml
   [[bench]]
   name = "my_benchmark"
   harness = false
   ```
3. Use the common utilities from `partiql_benchmarks`:
   ```rust
   use partiql_benchmarks::{compile, create_catalog, lower, parse};
   ```

## Why a Separate Package?

Previously, benchmarks were in `partiql-perf` alongside binary tools. This caused:
- Slow `cargo bench` builds (compiled all binaries first)
- Mixed concerns (benchmarks vs utilities)

The new structure:
- **partiql-benchmarks**: Criterion benchmarks only (fast builds)
- **partiql-perf**: Binary utilities for manual testing and profiling
