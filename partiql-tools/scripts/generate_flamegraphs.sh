#!/bin/bash

# PartiQL Flamegraph Generation Script
# Generates flamegraphs for all engines using cargo-flamegraph

set -e

# Colors for output
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
NC='\033[0m' # No Color

# Default configuration
OUTPUT_DIR="flamegraphs"
DATA_SOURCE="mem:1000000"
ITERATIONS=100
ENGINES=("legacy" "vectorized-1" "vectorized-1024" "hybrid")

# Parse command line arguments
while [[ $# -gt 0 ]]; do
    case $1 in
        --output-dir)
            OUTPUT_DIR="$2"
            shift 2
            ;;
        --data)
            DATA_SOURCE="$2"
            shift 2
            ;;
        --iterations)
            ITERATIONS="$2"
            shift 2
            ;;
        --engines)
            IFS=',' read -ra ENGINES <<< "$2"
            shift 2
            ;;
        --help|-h)
            echo "Usage: $0 [OPTIONS]"
            echo ""
            echo "Options:"
            echo "  --output-dir <DIR>      Output directory for flamegraphs (default: flamegraphs)"
            echo "  --data <SOURCE>         Data source: mem:<rows>, *.ion, or *.10n (default: mem:1000000)"
            echo "  --iterations <N>        Number of iterations per profile (default: 100)"
            echo "  --engines <LIST>        Comma-separated engine list (default: legacy,vectorized-1,vectorized-1024,hybrid)"
            echo "  --help, -h              Show this help message"
            echo ""
            echo "Examples:"
            echo "  # Generate flamegraphs for all engines with 1M rows"
            echo "  $0"
            echo ""
            echo "  # Profile specific engines only"
            echo "  $0 --engines legacy,hybrid"
            echo ""
            echo "  # Use Ion file as data source"
            echo "  $0 --data test_data/data.ion"
            echo ""
            echo "  # Custom output directory"
            echo "  $0 --output-dir my_profiles"
            exit 0
            ;;
        *)
            echo -e "${RED}Unknown option: $1${NC}"
            exit 1
            ;;
    esac
done

# Check if cargo-flamegraph is installed
if ! command -v cargo-flamegraph &> /dev/null; then
    echo -e "${RED}Error: cargo-flamegraph is not installed${NC}"
    echo ""
    echo "Install it with:"
    echo "  cargo install flamegraph"
    echo ""
    exit 1
fi

# Check platform
if [[ "$OSTYPE" == "linux-gnu"* ]]; then
    if ! command -v perf &> /dev/null; then
        echo -e "${YELLOW}Warning: 'perf' not found. Flamegraph may not work.${NC}"
        echo "Install it with: sudo apt-get install linux-tools-common linux-tools-generic"
        echo ""
    fi
elif [[ "$OSTYPE" == "darwin"* ]]; then
    echo -e "${GREEN}Running on macOS - using DTrace${NC}"
else
    echo -e "${YELLOW}Warning: Unknown OS type: $OSTYPE${NC}"
    echo "Flamegraph may not work on this platform."
    echo ""
fi

# Create output directory
mkdir -p "$OUTPUT_DIR"

echo "╔═══════════════════════════════════════════════════════════════════════════╗"
echo "║                  PARTIQL FLAMEGRAPH GENERATION                            ║"
echo "╚═══════════════════════════════════════════════════════════════════════════╝"
echo ""
echo "Configuration:"
echo "  Output directory: $OUTPUT_DIR"
echo "  Data source:      $DATA_SOURCE"
echo "  Iterations:       $ITERATIONS"
echo "  Engines:          ${ENGINES[*]}"
echo ""

# Build the binary first
echo -e "${GREEN}Building partiql-profile in release mode...${NC}"
cd "$(dirname "$0")/.."
RUSTFLAGS="-A warnings" cargo build --release --bin partiql-profile

echo ""
echo "Generating flamegraphs..."
echo ""

# Generate flamegraph for each engine
for engine in "${ENGINES[@]}"; do
    echo -e "${GREEN}Profiling $engine engine...${NC}"
    
    output_file="$OUTPUT_DIR/${engine}.svg"
    
    # Run cargo-flamegraph
    if RUSTFLAGS="-A warnings" cargo flamegraph \
        --bin partiql-profile \
        --output "$output_file" \
        --freq 997 \
        -- \
        --engine "$engine" \
        --data "$DATA_SOURCE" \
        --iterations "$ITERATIONS" 2>&1 | grep -v "Finished\|Compiling\|Running"; then
        
        echo -e "${GREEN}✓ Generated: $output_file${NC}"
    else
        echo -e "${RED}✗ Failed to generate flamegraph for $engine${NC}"
    fi
    
    echo ""
done

echo "╔═══════════════════════════════════════════════════════════════════════════╗"
echo "║                          FLAMEGRAPHS GENERATED                            ║"
echo "╚═══════════════════════════════════════════════════════════════════════════╝"
echo ""
echo "Flamegraphs saved to: $OUTPUT_DIR/"
echo ""
echo "View flamegraphs:"
for engine in "${ENGINES[@]}"; do
    if [[ -f "$OUTPUT_DIR/${engine}.svg" ]]; then
        echo "  open $OUTPUT_DIR/${engine}.svg"
    fi
done
echo ""
echo "Or compare side-by-side in a browser:"
echo "  # Create an HTML file to view all flamegraphs"
echo "  cat > $OUTPUT_DIR/index.html << 'EOF'"
echo '<!DOCTYPE html>'
echo '<html><head><title>PartiQL Flamegraph Comparison</title></head>'
echo '<body style="font-family: Arial, sans-serif; margin: 20px;">'
echo '<h1>PartiQL Engine Flamegraph Comparison</h1>'
for engine in "${ENGINES[@]}"; do
    echo "<h2>${engine}</h2>"
    echo "<object data=\"${engine}.svg\" type=\"image/svg+xml\" width=\"100%\" height=\"600\"></object>"
done
echo '</body></html>'
echo 'EOF'
echo ""
echo "  open $OUTPUT_DIR/index.html"
