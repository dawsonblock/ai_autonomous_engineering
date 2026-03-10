#!/bin/bash

# Script to validate binary subprocess speedup meets M1 acceptance criterion
# M1: Binary subprocess execution ≤380μs mean measured via hyperfine 100 runs
#
# Usage: ./scripts/validate_binary_speedup.sh
# Exit 0 if pass, Exit 1 if fail

set -e

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
PROJECT_ROOT="$(cd "$SCRIPT_DIR/.." && pwd)"
BINARY_PATH="$PROJECT_ROOT/target/release/pyrust"
OUTPUT_JSON="/tmp/binary_speedup_validation.json"

# Colors for output
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
NC='\033[0m' # No Color

echo "=================================="
echo "Binary Speedup Validation (M1)"
echo "=================================="
echo ""

# Check if binary exists
if [ ! -f "$BINARY_PATH" ]; then
    echo -e "${RED}ERROR: Binary not found at $BINARY_PATH${NC}"
    echo "Please run 'cargo build --release' first."
    exit 1
fi

echo "Binary: $BINARY_PATH"
echo "Target: Mean ≤380μs (50x speedup vs 19ms CPython baseline)"
echo ""

# Run hyperfine with 100 runs and export JSON
echo "Running hyperfine benchmark with 100 runs..."
echo "Command: $BINARY_PATH -c '2+3'"
echo ""

hyperfine \
    --warmup 10 \
    --runs 100 \
    --export-json "$OUTPUT_JSON" \
    "$BINARY_PATH -c '2+3'" \
    > /dev/null 2>&1

# Check if hyperfine succeeded
if [ $? -ne 0 ]; then
    echo -e "${RED}ERROR: hyperfine benchmark failed${NC}"
    exit 1
fi

# Extract mean time in microseconds using jq
mean_seconds=$(jq -r '.results[0].mean' "$OUTPUT_JSON")
mean_us=$(echo "$mean_seconds * 1000000" | bc)
mean_us_int=$(printf "%.0f" "$mean_us")

# Extract standard deviation
stddev_seconds=$(jq -r '.results[0].stddev' "$OUTPUT_JSON")
stddev_us=$(echo "$stddev_seconds * 1000000" | bc)

# Extract min and max
min_seconds=$(jq -r '.results[0].min' "$OUTPUT_JSON")
min_us=$(echo "$min_seconds * 1000000" | bc)

max_seconds=$(jq -r '.results[0].max' "$OUTPUT_JSON")
max_us=$(echo "$max_seconds * 1000000" | bc)

# Calculate coefficient of variation (CV)
cv=$(echo "scale=4; ($stddev_us / $mean_us) * 100" | bc)

echo "=================================="
echo "Results:"
echo "=================================="
echo "Mean:   ${mean_us_int}μs"
echo "StdDev: $(printf "%.2f" "$stddev_us")μs"
echo "Min:    $(printf "%.2f" "$min_us")μs"
echo "Max:    $(printf "%.2f" "$max_us")μs"
echo "CV:     ${cv}%"
echo ""

# Validation checks
PASS=true

# Check M1: Mean ≤380μs
if (( $(echo "$mean_us_int <= 380" | bc -l) )); then
    echo -e "${GREEN}✓ M1 PASS: Mean ${mean_us_int}μs ≤ 380μs${NC}"
else
    echo -e "${RED}✗ M1 FAIL: Mean ${mean_us_int}μs > 380μs${NC}"
    PASS=false
fi

# Check CV < 10% for statistical stability (AC6.4)
if (( $(echo "$cv < 10" | bc -l) )); then
    echo -e "${GREEN}✓ AC6.4 PASS: CV ${cv}% < 10%${NC}"
else
    echo -e "${YELLOW}⚠ AC6.4 WARNING: CV ${cv}% ≥ 10% (lower stability)${NC}"
    # Note: We don't fail on CV, just warn
fi

echo ""
echo "=================================="

if [ "$PASS" = true ]; then
    echo -e "${GREEN}VALIDATION PASSED${NC}"
    echo "Binary subprocess achieves 50x speedup target!"
    exit 0
else
    echo -e "${RED}VALIDATION FAILED${NC}"
    echo "Binary subprocess does not meet 50x speedup target."
    exit 1
fi
