#!/bin/bash
# Generate mock data in all formats for different sizes

# Default to test_data directory in partiql-eval-vectorized
SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
PROJECT_ROOT="$(cd "$SCRIPT_DIR/.." && pwd)"
OUTPUT_DIR="${1:-$PROJECT_ROOT/test_data}"

echo "Generating mock data in all formats..."
echo "Output directory: $OUTPUT_DIR"
echo ""

# Create output directory
mkdir -p "$OUTPUT_DIR"

# Configuration 1: BATCH_SIZE=4096 NUM_BATCHES=244 (≈1M rows)
echo "=== Configuration 1: 4096 batch_size, 244 batches (≈1M rows) ==="
cargo run --release --bin generate-data -- \
  --format all \
  --output-dir "$OUTPUT_DIR" \
  --batch-size 4096 \
  --num-batches 244
echo ""

# Configuration 2: batch_size=1024, num_batches=10000 (10.24M rows - default)
echo "=== Configuration 2: 1024 batch_size, 10000 batches (10.24M rows) ==="
cargo run --release --bin generate-data -- \
  --format all \
  --output-dir "$OUTPUT_DIR" \
  --batch-size 1024 \
  --num-batches 10000
echo ""

# Configuration 3: BATCH_SIZE=1024 NUM_BATCHES=100000 (102.4M rows)
echo "=== Configuration 3: 1024 batch_size, 100000 batches (102.4M rows) ==="
cargo run --release --bin generate-data -- \
  --format all \
  --output-dir "$OUTPUT_DIR" \
  --batch-size 1024 \
  --num-batches 100000
echo ""

echo "All data generation complete!"
echo "Files created in: $OUTPUT_DIR"
ls -lh "$OUTPUT_DIR"/*.{arrow,parquet,ion} 2>/dev/null | head -20

