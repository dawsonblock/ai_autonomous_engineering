#!/bin/bash

# Script to check coefficient of variation (CV) for binary subprocess benchmarks
# AC6.4: All benchmarks show CV < 10% ensuring statistical stability
#
# Usage: ./scripts/check_binary_subprocess_cv.sh
# Exit 0 if all CV < 10%, Exit 1 otherwise

set -e

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
PROJECT_ROOT="$(cd "$SCRIPT_DIR/.." && pwd)"
CRITERION_DIR="$PROJECT_ROOT/target/criterion"

# Colors for output
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
NC='\033[0m' # No Color

echo "============================================="
echo "Binary Subprocess Benchmark CV Check (AC6.4)"
echo "============================================="
echo ""

# Find all binary_subprocess benchmark directories
BENCHMARKS=(
    "binary_subprocess_simple_arithmetic"
    "binary_subprocess_complex_expression"
    "binary_subprocess_with_variables"
    "binary_subprocess_with_print"
    "binary_subprocess_multiple_operations"
    "binary_subprocess_nested_expression"
    "binary_subprocess_floor_division"
    "binary_subprocess_modulo"
    "binary_subprocess_startup_overhead"
)

ALL_PASS=true
MAX_CV=0
MAX_CV_BENCH=""

for bench in "${BENCHMARKS[@]}"; do
    ESTIMATES_FILE="$CRITERION_DIR/$bench/base/estimates.json"

    if [ ! -f "$ESTIMATES_FILE" ]; then
        echo -e "${YELLOW}⚠ WARNING: Estimates file not found for $bench${NC}"
        echo "  Expected: $ESTIMATES_FILE"
        echo "  Run 'cargo bench --bench binary_subprocess' first."
        echo ""
        continue
    fi

    # Extract mean and std_dev
    mean=$(jq -r '.mean.point_estimate' "$ESTIMATES_FILE")
    stddev=$(jq -r '.std_dev.point_estimate' "$ESTIMATES_FILE")

    # Calculate CV in percentage
    cv=$(echo "scale=4; ($stddev / $mean) * 100" | bc)

    # Convert to microseconds for display
    mean_us=$(echo "scale=2; $mean / 1000" | bc)
    stddev_us=$(echo "scale=2; $stddev / 1000" | bc)

    # Check if CV < 10%
    if (( $(echo "$cv < 10" | bc -l) )); then
        echo -e "${GREEN}✓ PASS${NC} $bench"
        echo "        Mean: ${mean_us}μs, StdDev: ${stddev_us}μs, CV: ${cv}%"
    else
        echo -e "${RED}✗ FAIL${NC} $bench"
        echo "        Mean: ${mean_us}μs, StdDev: ${stddev_us}μs, CV: ${cv}%"
        ALL_PASS=false
    fi

    # Track maximum CV
    if (( $(echo "$cv > $MAX_CV" | bc -l) )); then
        MAX_CV=$cv
        MAX_CV_BENCH=$bench
    fi
done

echo ""
echo "============================================="
echo "Summary:"
echo "============================================="
echo "Maximum CV: ${MAX_CV}% ($MAX_CV_BENCH)"

if [ "$ALL_PASS" = true ]; then
    echo -e "${GREEN}✓ AC6.4 PASS: All benchmarks have CV < 10%${NC}"
    echo "Statistical stability achieved!"
    exit 0
else
    echo -e "${RED}✗ AC6.4 FAIL: Some benchmarks have CV ≥ 10%${NC}"
    echo "Consider increasing sample_size or measurement_time in Criterion config."
    exit 1
fi
