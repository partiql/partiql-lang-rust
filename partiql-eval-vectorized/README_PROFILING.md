# PartiQL Performance Profiling Setup

This document provides a quick overview of the profiling infrastructure for comparing Legacy, Vectorized, and Hybrid PartiQL engines.

## What's Included

### 1. Profiling Binary (`partiql-profile`)

A dedicated binary for running controlled workloads suitable for profiling:

```bash
cargo run --release --bin partiql-profile -- \
  --engine legacy \
  --data mem:1000000 \
  --iterations 100
```

**Features:**
- Runs query multiple times for accurate sampling
- Supports all engines: `legacy`, `vectorized-1`, `vectorized-1024`, `hybrid`
- Multiple data sources: `mem:<rows>`, `*.ion`, `*.10n`
- Custom queries with `--query` flag

### 2. Automated Flamegraph Script (`generate_flamegraphs.sh`)

Generates flamegraphs for all engines automatically:

```bash
cd partiql-eval-vectorized
./scripts/generate_flamegraphs.sh
```

**Output:** Creates SVG flamegraphs in `flamegraphs/` directory
- `legacy.svg`
- `vectorized-1.svg`
- `vectorized-1024.svg`
- `hybrid.svg`

### 3. Comprehensive Documentation (`PROFILING.md`)

Complete guide covering:
- Installation and setup (perf on Linux, dtrace on macOS)
- Manual profiling with `cargo-flamegraph`
- Interpreting flamegraphs
- Common bottlenecks for each engine
- Performance tuning workflow

## Quick Start

### Step 1: Install cargo-flamegraph

```bash
cargo install flamegraph
```

**Linux only:** Install perf
```bash
sudo apt-get install linux-tools-common linux-tools-generic
```

### Step 2: Generate Flamegraphs

**Option A: Automated (all engines)**
```bash
cd partiql-eval-vectorized
./scripts/generate_flamegraphs.sh
open flamegraphs/*.svg
```

**Option B: Manual (single engine)**
```bash
cargo flamegraph --bin partiql-profile -- \
  --engine hybrid \
  --data mem:1000000
```

### Step 3: Compare and Analyze

Open the generated SVG files in your browser. Look for:
- **Wide rectangles** = functions consuming most time
- **Tall stacks** = deep call chains
- **Hot paths** = frequently executed code paths

## Use Cases

### Compare Engine Performance

Generate flamegraphs for each engine with the same workload:

```bash
./scripts/generate_flamegraphs.sh --data mem:1000000
```

Then compare:
- Legacy: Look for per-row Value allocation
- Vectorized: Look for batch operations and SIMD
- Hybrid: Look for streaming overhead

### Profile Specific Bottlenecks

```bash
# Profile I/O-heavy workload
cargo flamegraph --bin partiql-profile -- \
  --engine vectorized-1024 \
  --data large_file.ion

# Profile CPU-heavy filter
cargo flamegraph --bin partiql-profile -- \
  --engine hybrid \
  --query "SELECT * FROM ~input~ WHERE a % 7 = 0 AND a % 11 = 0"
```

### Before/After Optimization

```bash
# Generate baseline
cargo flamegraph --output before.svg --bin partiql-profile -- --engine legacy

# Make code changes...

# Generate after optimization
cargo flamegraph --output after.svg --bin partiql-profile -- --engine legacy

# Compare the two flamegraphs
```

## Files Created

```
partiql-eval-vectorized/
├── src/bin/
│   └── partiql_profile.rs          # Profiling workload binary
├── scripts/
│   └── generate_flamegraphs.sh     # Automation script
├── PROFILING.md                     # Comprehensive guide
└── README_PROFILING.md              # This file (quick overview)
```

## Integration with Cargo.toml

The `Cargo.toml` has been updated with:

```toml
[profile.release]
debug = true  # Enables debug symbols for profiling

[[bin]]
name = "partiql-profile"
path = "src/bin/partiql_profile.rs"
```

## Next Steps

1. **Read the full guide:** See [PROFILING.md](./PROFILING.md) for detailed instructions
2. **Generate your first flamegraph:** Start with `./scripts/generate_flamegraphs.sh`
3. **Identify bottlenecks:** Look for wide rectangles in the flamegraphs
4. **Make optimizations:** Focus on the hottest code paths first
5. **Verify improvements:** Re-run benchmarks and compare flamegraphs

## Tips

- **Always use release builds** for profiling (debug builds are misleading)
- **Run multiple iterations** (100+) for accurate sampling
- **Focus on >5% bars** - smaller functions rarely matter
- **Use realistic workloads** - profile with real data sizes
- **Compare before/after** - verify optimizations actually help

## Example Workflow

```bash
# 1. Establish baseline performance
cargo run --release --bin partiql-benchmarks -- test_data/*.ion

# 2. Identify slow engine/query
# (From benchmark results)

# 3. Generate flamegraph for that case
cargo flamegraph --bin partiql-profile -- \
  --engine legacy \
  --data test_data/slow_case.ion \
  --iterations 200

# 4. Analyze flamegraph
open flamegraph.svg
# Look for wide bars (>5% of total time)

# 5. Optimize the hotspot
# (Edit code)

# 6. Verify improvement
cargo run --release --bin partiql-benchmarks -- test_data/*.ion
cargo flamegraph --output after.svg --bin partiql-profile -- --engine legacy
```

## Troubleshooting

### macOS: "permission denied"
Grant Terminal full disk access in System Preferences → Security & Privacy

### Linux: "perf not found"
```bash
sudo apt-get install linux-tools-generic
```

### "No samples" or empty flamegraph
- Increase `--iterations` (need more samples)
- Check `debug = true` in `[profile.release]`
- Try higher `--freq` (e.g., `--freq 1999`)

## Resources

- [Full Profiling Guide](./PROFILING.md)
- [cargo-flamegraph](https://github.com/flamegraph-rs/flamegraph)
- [Brendan Gregg's Flamegraphs](http://www.brendangregg.com/flamegraphs.html)
