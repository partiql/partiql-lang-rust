# Benchmark Configuration Guide

The `engine_comparison.rs` benchmark uses a single, unified target that dynamically generates all benchmark combinations. Filtering is controlled entirely via environment variables, allowing you to run specific combinations of data sizes, engines, queries, and formats.

## Environment Variables

- **BENCH_SIZES**: Filter by data sizes (comma-separated, in rows)
- **BENCH_ENGINES**: Filter by engine types (comma-separated)
- **BENCH_QUERIES**: Filter by query names (comma-separated)
- **BENCH_FORMATS**: Filter by data format (comma-separated: mem, ion)

If no environment variables are set, all benchmarks run (default behavior).

## Available Options

### Data Sizes
All sizes are available and can be filtered via `BENCH_SIZES`:
- `100` - 100 rows
- `1000` - 1,000 rows
- `10000` - 10,000 rows
- `100000` - 100,000 rows
- `1000000` - 1,000,000 rows

### Engines
- `legacy` - Legacy PartiQL engine
- `vectorized_1` - Vectorized engine with batch size 1
- `vectorized_1024` - Vectorized engine with batch size 1024
- `hybrid` - Hybrid PartiQL VM engine

### Queries
- `proj` - Simple projection: `SELECT a, b FROM data`
- `every_other` - Filter every other row: `SELECT a, b FROM data WHERE a % 2 = 0`
- `every_other_complex` - Complex filter: `SELECT a, b FROM data WHERE ((a - a + b - b + a - a + b - b) + a % 2) = 0`

## Usage Examples

### Example 1: Size 100, Hybrid and Legacy engines only
```bash
BENCH_SIZES=100 BENCH_ENGINES=hybrid,legacy cargo bench --bench engine_comparison
```

### Example 2: Only the projection query across all sizes and engines
```bash
BENCH_QUERIES=proj cargo bench --bench engine_comparison
```

### Example 3: Large data with hybrid engine only
```bash
BENCH_SIZES=100000 BENCH_ENGINES=hybrid cargo bench --bench engine_comparison
```

### Example 4: Multiple sizes with specific engines
```bash
BENCH_SIZES=100,1000 BENCH_ENGINES=hybrid,vectorized_1024 cargo bench --bench engine_comparison
```

### Example 5: All filters combined
```bash
BENCH_SIZES=100 BENCH_ENGINES=hybrid,legacy BENCH_QUERIES=every_other cargo bench --bench engine_comparison
```

### Example 6: Compare all engines on small data
```bash
BENCH_SIZES=100,1000 cargo bench --bench engine_comparison
```

### Example 7: Test vectorized engines only
```bash
BENCH_ENGINES=vectorized_1,vectorized_1024 cargo bench --bench engine_comparison
```

### Example 8: Only Ion file benchmarks with size 100
```bash
BENCH_SIZES=100 BENCH_FORMATS=ion cargo bench --bench engine_comparison
```

### Example 9: Only in-memory benchmarks with hybrid and legacy
```bash
BENCH_FORMATS=mem BENCH_ENGINES=hybrid,legacy cargo bench --bench engine_comparison
```

### Example 10: Size 100 Ion files, hybrid and legacy engines only (your use case)
```bash
BENCH_SIZES=100 BENCH_FORMATS=ion BENCH_ENGINES=hybrid,legacy cargo bench --bench engine_comparison
```

## Notes

- Engine names are case-insensitive
- Whitespace around commas is automatically trimmed
- Invalid data sizes are ignored
- If a filter doesn't match anything, that benchmark group is skipped
- Both in-memory and Ion file data sources respect these filters
