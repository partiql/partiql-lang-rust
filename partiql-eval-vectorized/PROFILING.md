# PartiQL Profiling Guide

This guide explains how to profile PartiQL engines (Legacy, Vectorized, Hybrid) using flamegraphs to understand performance bottlenecks.

## Table of Contents

- [Quick Start](#quick-start)
- [Installation](#installation)
- [Manual Profiling](#manual-profiling)
- [Automated Profiling](#automated-profiling)
- [Interpreting Flamegraphs](#interpreting-flamegraphs)
- [Common Bottlenecks](#common-bottlenecks)
- [Advanced Usage](#advanced-usage)

## Quick Start

```bash
# 1. Install cargo-flamegraph
cargo install flamegraph

# 2. Generate flamegraphs for all engines (automated)
cd partiql-eval-vectorized
./scripts/generate_flamegraphs.sh

# 3. View results
open flamegraphs/legacy.svg
open flamegraphs/vectorized-1024.svg
open flamegraphs/hybrid.svg
```

## Installation

### Prerequisites

**On Linux:**
```bash
# Install perf (required for flamegraphs)
sudo apt-get install linux-tools-common linux-tools-generic linux-tools-$(uname -r)

# Allow perf to run without sudo
echo 0 | sudo tee /proc/sys/kernel/perf_event_paranoid
```

**On macOS:**

You have two options for profiling on macOS:

**Option 1: cargo-flamegraph (Recommended for this use case)**
```bash
# Requires full Xcode installation (~12GB)
# Install Xcode from App Store, then:
sudo xcode-select --switch /Applications/Xcode.app/Contents/Developer

# You may need to grant Terminal full disk access in System Preferences
```

**Option 2: cargo-instruments (If you don't want to install Xcode)**
```bash
# Works with Command Line Tools only
cargo install cargo-instruments

# Profile with:
cargo instruments -t time --bin partiql-profile --release -- \
  --engine legacy \
  --data mem:1000000

# Opens .trace file in Instruments app (not portable SVGs)
```

**Which to choose?**

Use **cargo-flamegraph** if you want:
- ✅ Portable SVG flamegraphs (works in any browser, easy to share)
- ✅ Side-by-side engine comparison
- ✅ Automation with scripts (like `generate_flamegraphs.sh`)
- ✅ Simple, intuitive visualization

Use **cargo-instruments** if you:
- ✅ Don't want to install 12GB of Xcode
- ✅ Need memory/allocation profiling (not just CPU)
- ✅ Only need local profiling (can't share .trace files easily)
- ✅ Prefer macOS native Instruments app

For comparing engine performance (the main goal), **cargo-flamegraph is recommended** despite the Xcode requirement.

### Install cargo-flamegraph

```bash
cargo install flamegraph
```

## Manual Profiling

The `partiql-profile` binary provides a controlled workload for profiling. It runs a query multiple times to generate enough samples for accurate profiling.

### Basic Usage

```bash
# Profile legacy engine with 1M in-memory rows
cargo flamegraph --bin partiql-profile -- \
  --engine legacy \
  --data mem:1000000

# Profile vectorized-1024 engine
cargo flamegraph --bin partiql-profile -- \
  --engine vectorized-1024 \
  --data mem:1000000

# Profile hybrid engine
cargo flamegraph --bin partiql-profile -- \
  --engine hybrid \
  --data mem:1000000
```

### Using File-Based Data

```bash
# Profile with Ion text file
cargo flamegraph --bin partiql-profile -- \
  --engine legacy \
  --data test_data/data.ion

# Profile with Ion binary file
cargo flamegraph --bin partiql-profile -- \
  --engine vectorized-1024 \
  --data test_data/data.10n
```

### Custom Queries

```bash
# Profile with custom query
cargo flamegraph --bin partiql-profile -- \
  --engine hybrid \
  --query "SELECT a FROM ~input~ WHERE a > 500000" \
  --data mem:1000000
```

### Adjusting Iterations

More iterations = more samples = better flamegraph, but takes longer:

```bash
cargo flamegraph --bin partiql-profile -- \
  --engine legacy \
  --iterations 200 \
  --data mem:1000000
```

### Output to Specific File

```bash
cargo flamegraph \
  --bin partiql-profile \
  --output my-profile.svg \
  -- \
  --engine hybrid \
  --data mem:1000000
```

## Automated Profiling

The `generate_flamegraphs.sh` script automates flamegraph generation for multiple engines.

### Basic Usage

```bash
# Generate flamegraphs for all engines (default: 1M in-memory rows)
./scripts/generate_flamegraphs.sh

# Output: flamegraphs/legacy.svg, flamegraphs/vectorized-*.svg, flamegraphs/hybrid.svg
```

### Custom Configuration

```bash
# Profile specific engines only
./scripts/generate_flamegraphs.sh --engines legacy,hybrid

# Use different data source
./scripts/generate_flamegraphs.sh --data test_data/large_data.ion

# Custom output directory
./scripts/generate_flamegraphs.sh --output-dir my_profiles

# More iterations for better accuracy
./scripts/generate_flamegraphs.sh --iterations 200
```

### View All Flamegraphs Together

The script creates an HTML file for side-by-side comparison:

```bash
# After generating flamegraphs
open flamegraphs/index.html
```

## Interpreting Flamegraphs

### What is a Flamegraph?

A flamegraph is a visualization of profiled software stack traces. Each rectangle represents a function call:

- **Width**: Time spent in that function (including calls it makes)
- **Height**: Stack depth (how deep in the call stack)
- **Color**: Random (no meaning, just for visual differentiation)

### Reading Flamegraphs

1. **Top-down view**: Bottom shows entry point, top shows leaf functions
2. **Wide rectangles**: Functions taking most time (potential bottlenecks)
3. **Tall stacks**: Deep call chains (may indicate recursion or abstraction overhead)
4. **Flat areas**: Functions that don't call others (pure computation)

### Interactive Features

Flamegraph SVGs are interactive:
- **Click** a rectangle to zoom in
- **Hover** to see exact function name and percentage
- **Search** (Ctrl+F in browser) to highlight specific functions
- **Reset** by clicking the title

## Common Bottlenecks

### Legacy Engine

Look for these hotspots:

1. **Value allocation**: `Value::Integer`, `Value::Tuple`, `Box::new`
   - Each row creates new Value objects
   - Look for: `partiql_value::value::Value`

2. **Ion parsing**: `ion_rs::`, `IonDecoder`
   - File I/O overhead
   - Look for: `ion_rs_old::binary_reader`

3. **Expression evaluation**: `eval::eval_expr`
   - Per-row interpretation
   - Look for: `partiql_eval::eval`

### Vectorized Engine

Look for these hotspots:

1. **Batch processing**: `VectorizedPlan::execute`
   - Should show batch operations, not per-row
   - Look for: `partiql_eval_vectorized::operators`

2. **Data readers**: `BatchReader::next_batch`
   - I/O and deserialization
   - Look for: `InMemoryGeneratedReader`, `PIonReader`

3. **Filter operations**: `FilterOperator::execute`
   - SIMD operations should be visible
   - Look for: `wide::` for SIMD operations

### Hybrid Engine

Look for these hotspots:

1. **Streaming overhead**: `RowStream::next_row`
   - Should be minimal per-row cost
   - Look for: `partiql_eval::engine::plan`

2. **Reader implementations**: `RowReader::next`
   - Ion parsing per row
   - Look for: `IonRowReader`

3. **Value conversion**: Iterator chains
   - Converting between formats
   - Look for: `core::iter`

## Advanced Usage

### Comparing Engines

Generate flamegraphs for each engine, then compare:

```bash
# Generate all
./scripts/generate_flamegraphs.sh

# Compare specific areas
# Open in browser and search for specific functions:
# - "ion" - Ion parsing overhead
# - "alloc" - Memory allocation
# - "filter" - Filter operation implementation
```

### Profiling Specific Workloads

```bash
# Large data, simple query (I/O bound)
cargo flamegraph --bin partiql-profile -- \
  --engine vectorized-1024 \
  --data test_data/10M_rows.ion \
  --query "SELECT a, b FROM ~input~"

# Small data, complex filter (CPU bound)
cargo flamegraph --bin partiql-profile -- \
  --engine hybrid \
  --data mem:100000 \
  --query "SELECT a FROM ~input~ WHERE a % 7 = 0 AND a % 11 = 0"
```

### Profiling Frequency

Higher sampling frequency = more accurate, but more overhead:

```bash
# Default frequency (997 Hz)
cargo flamegraph --bin partiql-profile -- --engine legacy

# Higher frequency (1999 Hz) - more accurate
cargo flamegraph --freq 1999 --bin partiql-profile -- --engine legacy
```

### Differential Flamegraphs

Compare two flamegraphs to see differences:

```bash
# Generate baseline
cargo flamegraph --output before.svg --bin partiql-profile -- --engine legacy

# Make changes to code, then generate new profile
cargo flamegraph --output after.svg --bin partiql-profile -- --engine legacy

# Compare visually or use flamegraph diff tools
```

## Performance Tuning Workflow

1. **Establish baseline**: Run benchmarks to identify slow cases
   ```bash
   cargo run --release --bin partiql-benchmarks -- test_data/*.ion
   ```

2. **Generate flamegraphs**: Profile the slow engine
   ```bash
   cargo flamegraph --bin partiql-profile -- --engine legacy --data test_data/slow.ion
   ```

3. **Identify hotspots**: Look for wide rectangles (>5% of time)
   - Focus on the widest first
   - Check if it's expected (I/O) or unexpected (allocation)

4. **Make targeted changes**: Optimize the identified hotspot

5. **Verify improvement**: Re-run benchmarks and compare flamegraphs
   ```bash
   cargo run --release --bin partiql-benchmarks -- test_data/*.ion
   ```

## Troubleshooting

### "perf not found" (Linux)

```bash
sudo apt-get install linux-tools-generic
```

### "Permission denied" when profiling (Linux)

```bash
# Temporarily allow profiling
echo 0 | sudo tee /proc/sys/kernel/perf_event_paranoid

# Or permanently (add to /etc/sysctl.conf)
kernel.perf_event_paranoid = 0
```

### "No samples" or empty flamegraph

- Increase `--iterations` (more samples needed)
- Check if binary has debug symbols (should be built with `debug = true` in release profile)
- On macOS, grant Terminal full disk access in System Preferences

### Flamegraph doesn't match expectations

- Ensure release build is used
- Check if compiler optimizations are inlining functions
- Use `--freq` to adjust sampling rate

## Tips

1. **Always use release builds**: Debug builds have misleading profiles
2. **Focus on wide bars**: These are where time is actually spent
3. **Ignore tiny bars**: <1% of time usually isn't worth optimizing
4. **Compare before/after**: Use git branches to A/B test optimizations
5. **Profile realistic workloads**: Use real data sizes and queries
6. **Multiple runs**: Run profiling several times to verify consistency

## Resources

- [Brendan Gregg's Flamegraph Site](http://www.brendangregg.com/flamegraphs.html)
- [cargo-flamegraph Documentation](https://github.com/flamegraph-rs/flamegraph)
- [Understanding Performance with Flamegraphs](https://www.brendangregg.com/FlameGraphs/cpuflamegraphs.html)
