# Engine Profiler Usage Guide

The `engine_profiler` binary is designed for deep performance profiling with tools like `cargo-flamegraph`.

## Quick Start

```bash
# Profile hybrid engine with in-memory data
ENGINE=hybrid FORMAT=mem SIZE=1000 cargo flamegraph --bin engine_profiler

# Profile legacy engine with Ion file
ENGINE=legacy FORMAT=ion SIZE=1000 \
  DATA_PATH=~/Desktop/test_data/data_b1_n1000.ion \
  cargo flamegraph --bin engine_profiler
```

## Configuration

All configuration is done via environment variables:

| Variable | Values | Default | Description |
|----------|--------|---------|-------------|
| `ENGINE` | `legacy`, `hybrid`, `vectorized-1`, `vectorized-1024` | `legacy` | Engine to profile |
| `QUERY` | `proj`, `filter_modulo`, `filter_complex` | `proj` | Query to execute |
| `FORMAT` | `mem`, `ion` | `mem` | Data source format |
| `SIZE` | Number | `1000` | Number of rows to process |
| `ITERATIONS` | Number | `1000` | How many times to execute (more = better flamegraph) |
| `DATA_PATH` | File path | - | Required for `FORMAT=ion` |

## Available Queries

- `proj`: `SELECT a, b FROM data` - Simple projection
- `filter_modulo`: `SELECT a, b FROM data WHERE a % 100 = 0` - Modulo filter
- `filter_complex`: `SELECT a, b FROM data WHERE ((a - a + b - b + a - a + b - b) + a % 100000) = 0` - Complex filter

## Example Use Cases

### Compare Hybrid vs Legacy with Ion

```bash
# Profile hybrid with Ion
ENGINE=hybrid FORMAT=ion SIZE=1000 \
  DATA_PATH=test_data/data.ion \
  cargo flamegraph --bin engine_profiler -- --output hybrid_ion.svg

# Profile legacy with Ion  
ENGINE=legacy FORMAT=ion SIZE=1000 \
  DATA_PATH=test_data/data.ion \
  cargo flamegraph --bin engine_profiler -- --output legacy_ion.svg

# Compare the two SVG files
```

### Compare In-Memory vs Ion for Same Engine

```bash
# Hybrid with in-memory
ENGINE=hybrid FORMAT=mem SIZE=10000 \
  cargo flamegraph --bin engine_profiler -- --output hybrid_mem.svg

# Hybrid with Ion
ENGINE=hybrid FORMAT=ion SIZE=10000 \
  DATA_PATH=test_data/data_b1_n10000.ion \
  cargo flamegraph --bin engine_profiler -- --output hybrid_ion.svg
```

### Profile Different Queries

```bash
# Simple projection
ENGINE=hybrid FORMAT=mem SIZE=1000 QUERY=proj \
  cargo flamegraph --bin engine_profiler

# Complex filter
ENGINE=hybrid FORMAT=mem SIZE=1000 QUERY=filter_complex \
  cargo flamegraph --bin engine_profiler
```

### Adjust Iteration Count

```bash
# Quick profile (fewer samples)
ENGINE=hybrid FORMAT=mem SIZE=1000 ITERATIONS=100 \
  cargo flamegraph --bin engine_profiler

# Deep profile (more samples, better quality)
ENGINE=hybrid FORMAT=mem SIZE=1000 ITERATIONS=5000 \
  cargo flamegraph --bin engine_profiler
```

## Tips for Good Flamegraphs

1. **Use enough iterations**: 1000+ gives good sampling, 5000+ for publication-quality
2. **Use realistic data sizes**: Too small and you'll see mostly setup overhead
3. **Run in release mode**: `cargo flamegraph` uses release by default
4. **Compare apples-to-apples**: Same SIZE and QUERY when comparing engines
5. **Look for wide bars**: These are your hot paths

## What to Look For

When analyzing flamegraphs:

- **Wide bars at the bottom** = hot functions (most time spent)
- **Tall stacks** = deep call chains (potential optimization target)
- **Compare relative widths** between engines to see where differences lie

## Building Without Profiling

```bash
cargo build --release --bin engine_profiler

# Run directly
ENGINE=hybrid FORMAT=mem SIZE=1000 ./target/release/engine_profiler
```

## Troubleshooting

**Error: "DATA_PATH environment variable required"**
- You're using `FORMAT=ion` but didn't provide a data file
- Solution: Set `DATA_PATH=/path/to/file.ion`

**Error: "Unknown engine"**
- Check spelling: `legacy`, `hybrid`, `vectorized-1`, `vectorized-1024`

**Flamegraph shows mostly setup**
- Increase `ITERATIONS` (try 5000)
- Increase `SIZE` (try 10000 or 100000)

**No flamegraph generated**
- Install cargo-flamegraph: `cargo install flamegraph`
- On macOS, you may need: `sudo cargo flamegraph ...`
