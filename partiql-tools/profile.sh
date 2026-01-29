#!/bin/bash

# Simple wrapper script for common profiling scenarios

set -e

# Colors for output
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
NC='\033[0m' # No Color

# Default values
ENGINE="hybrid"
FORMAT="ion"
SIZE="100"
ITERATIONS="1000"
QUERY="proj"
OUTPUT=""

# Help message
show_help() {
    echo "Usage: ./profile.sh [OPTIONS]"
    echo ""
    echo "Options:"
    echo "  --engine <ENGINE>       Engine: legacy, hybrid, vectorized-1, vectorized-1024 (default: hybrid)"
    echo "  --format <FORMAT>       Format: mem, ion (default: mem)"
    echo "  --size <SIZE>           Number of rows (default: 1000)"
    echo "  --iterations <N>        Number of iterations (default: 1000)"
    echo "  --query <QUERY>         Query: proj, filter_modulo, filter_complex (default: proj)"
    echo "  --data-path <PATH>      Path to data file (required for ion format)"
    echo "  --output <FILE>         Output SVG file (default: flamegraph.svg)"
    echo "  --help                  Show this help message"
    echo ""
    echo "Examples:"
    echo "  ./profile.sh --engine hybrid --format mem --size 10000"
    echo "  ./profile.sh --engine legacy --format ion --data-path test.ion --output legacy.svg"
    echo "  ./profile.sh --engine vectorized-1024 --query filter_complex --iterations 5000"
}

# Parse command line arguments
while [[ $# -gt 0 ]]; do
    case $1 in
        --engine)
            ENGINE="$2"
            shift 2
            ;;
        --format)
            FORMAT="$2"
            shift 2
            ;;
        --size)
            SIZE="$2"
            shift 2
            ;;
        --iterations)
            ITERATIONS="$2"
            shift 2
            ;;
        --query)
            QUERY="$2"
            shift 2
            ;;
        --data-path)
            DATA_PATH="$2"
            shift 2
            ;;
        --output)
            OUTPUT="$2"
            shift 2
            ;;
        --help)
            show_help
            exit 0
            ;;
        *)
            echo -e "${RED}Unknown option: $1${NC}"
            show_help
            exit 1
            ;;
    esac
done

# Validate format and data-path
if [ "$FORMAT" = "ion" ] && [ -z "$DATA_PATH" ]; then
    echo -e "${RED}Error: --data-path required when --format=ion${NC}"
    exit 1
fi

# Build output file name if not specified
if [ -z "$OUTPUT" ]; then
    OUTPUT="${ENGINE}_${FORMAT}_${SIZE}.svg"
fi

# Print configuration
echo -e "${GREEN}Profiling Configuration:${NC}"
echo "  Engine:     $ENGINE"
echo "  Format:     $FORMAT"
echo "  Size:       $SIZE rows"
echo "  Iterations: $ITERATIONS"
echo "  Query:      $QUERY"
if [ -n "$DATA_PATH" ]; then
    echo "  Data Path:  $DATA_PATH"
fi
echo "  Output:     $OUTPUT"
echo ""

# Build environment variables
ENV_VARS="ENGINE=$ENGINE FORMAT=$FORMAT SIZE=$SIZE ITERATIONS=$ITERATIONS QUERY=$QUERY"
if [ -n "$DATA_PATH" ]; then
    ENV_VARS="$ENV_VARS DATA_PATH=$DATA_PATH"
fi

# Run flamegraph
echo -e "${YELLOW}Running flamegraph... (this may take a while)${NC}"
echo ""

eval "$ENV_VARS cargo flamegraph --bin engine_profiler -- --output $OUTPUT"

echo ""
echo -e "${GREEN}Profiling complete!${NC}"
echo "  Flamegraph saved to: $OUTPUT"
echo "  Open in browser: open $OUTPUT"
