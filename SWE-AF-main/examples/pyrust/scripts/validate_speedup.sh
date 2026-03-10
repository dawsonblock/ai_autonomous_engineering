#!/bin/bash
# Automated speedup validation comparing PyRust vs CPython using hyperfine
#
# This script validates:
# - AC6.5: scripts/validate_speedup.sh exits 0 indicating ≥50x speedup vs CPython baseline
# - AC6.4: All benchmarks show CV < 10% ensuring statistical stability
# - Uses hyperfine with 100 runs for statistical rigor
# - Outputs mean, stddev, min, max, and speedup ratio
#
# Exit 0 if ≥50x speedup achieved, non-zero otherwise

set -e

# Colors for output
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
BLUE='\033[0;34m'
NC='\033[0m' # No Color

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
PROJECT_ROOT="$(cd "$SCRIPT_DIR/.." && pwd)"
BINARY_PATH="$PROJECT_ROOT/target/release/pyrust"
CPYTHON_OUTPUT_JSON="/tmp/cpython_speedup_validation.json"
PYRUST_OUTPUT_JSON="/tmp/pyrust_speedup_validation.json"

# Target metrics
CPYTHON_BASELINE_MS=19  # CPython baseline from PRD
TARGET_SPEEDUP=50       # Minimum 50x speedup required
TARGET_CV_PERCENT=10    # Maximum 10% coefficient of variation

echo "================================================================"
echo "         PyRust vs CPython Speedup Validation"
echo "================================================================"
echo ""
echo "This script validates:"
echo "  • AC6.5: ≥50x speedup vs CPython baseline (19ms)"
echo "  • AC6.4: CV < 10% for statistical stability"
echo "  • Uses hyperfine with 100 runs for statistical rigor"
echo ""

# Check dependencies
echo "Checking dependencies..."
if ! command -v hyperfine &> /dev/null; then
    echo -e "${RED}ERROR: hyperfine not found${NC}"
    echo "Install with: brew install hyperfine (macOS) or cargo install hyperfine"
    exit 1
fi

if ! command -v python3 &> /dev/null; then
    echo -e "${RED}ERROR: python3 not found${NC}"
    exit 1
fi

if ! command -v jq &> /dev/null; then
    echo -e "${RED}ERROR: jq not found${NC}"
    echo "Install with: brew install jq (macOS) or apt-get install jq (Linux)"
    exit 1
fi

if ! command -v bc &> /dev/null; then
    echo -e "${RED}ERROR: bc not found${NC}"
    echo "Install with: brew install bc (macOS) or apt-get install bc (Linux)"
    exit 1
fi

echo -e "${GREEN}✓ All dependencies available${NC}"
echo ""

# Check if PyRust binary exists
if [ ! -f "$BINARY_PATH" ]; then
    echo -e "${RED}ERROR: PyRust binary not found at $BINARY_PATH${NC}"
    echo "Building release binary..."
    cd "$PROJECT_ROOT"
    cargo build --release
    if [ $? -ne 0 ]; then
        echo -e "${RED}ERROR: Failed to build release binary${NC}"
        exit 1
    fi
fi

echo -e "${GREEN}✓ PyRust binary found at $BINARY_PATH${NC}"
echo ""

# Step 1: Measure CPython baseline
echo "================================================================"
echo "Step 1: Measuring CPython Baseline (100 runs)"
echo "================================================================"
echo "Command: python3 -c '2+3'"
echo ""

hyperfine \
    --warmup 10 \
    --runs 100 \
    --export-json "$CPYTHON_OUTPUT_JSON" \
    "python3 -c '2+3'" \
    > /dev/null 2>&1

if [ $? -ne 0 ]; then
    echo -e "${RED}ERROR: CPython benchmark failed${NC}"
    exit 1
fi

# Extract CPython statistics
cpython_mean_seconds=$(jq -r '.results[0].mean' "$CPYTHON_OUTPUT_JSON")
cpython_mean_ms=$(echo "$cpython_mean_seconds * 1000" | bc)
cpython_mean_us=$(echo "$cpython_mean_seconds * 1000000" | bc)
cpython_mean_us_int=$(printf "%.0f" "$cpython_mean_us")

cpython_stddev_seconds=$(jq -r '.results[0].stddev' "$CPYTHON_OUTPUT_JSON")
cpython_stddev_ms=$(echo "$cpython_stddev_seconds * 1000" | bc)
cpython_stddev_us=$(echo "$cpython_stddev_seconds * 1000000" | bc)

cpython_min_seconds=$(jq -r '.results[0].min' "$CPYTHON_OUTPUT_JSON")
cpython_min_ms=$(echo "$cpython_min_seconds * 1000" | bc)
cpython_min_us=$(echo "$cpython_min_seconds * 1000000" | bc)

cpython_max_seconds=$(jq -r '.results[0].max' "$CPYTHON_OUTPUT_JSON")
cpython_max_ms=$(echo "$cpython_max_seconds * 1000" | bc)
cpython_max_us=$(echo "$cpython_max_seconds * 1000000" | bc)

cpython_cv=$(echo "scale=4; ($cpython_stddev_ms / $cpython_mean_ms) * 100" | bc)

echo "CPython Results:"
echo "  Mean:   $(printf "%.2f" "$cpython_mean_ms")ms ($(printf "%.0f" "$cpython_mean_us")μs)"
echo "  StdDev: $(printf "%.2f" "$cpython_stddev_ms")ms ($(printf "%.2f" "$cpython_stddev_us")μs)"
echo "  Min:    $(printf "%.2f" "$cpython_min_ms")ms ($(printf "%.2f" "$cpython_min_us")μs)"
echo "  Max:    $(printf "%.2f" "$cpython_max_ms")ms ($(printf "%.2f" "$cpython_max_us")μs)"
echo "  CV:     ${cpython_cv}%"
echo ""

# Step 2: Measure PyRust performance
echo "================================================================"
echo "Step 2: Measuring PyRust Performance (100 runs)"
echo "================================================================"
echo "Command: $BINARY_PATH -c '2+3'"
echo ""

hyperfine \
    --warmup 10 \
    --runs 100 \
    --export-json "$PYRUST_OUTPUT_JSON" \
    "$BINARY_PATH -c '2+3'" \
    > /dev/null 2>&1

if [ $? -ne 0 ]; then
    echo -e "${RED}ERROR: PyRust benchmark failed${NC}"
    exit 1
fi

# Extract PyRust statistics
pyrust_mean_seconds=$(jq -r '.results[0].mean' "$PYRUST_OUTPUT_JSON")
pyrust_mean_ms=$(echo "$pyrust_mean_seconds * 1000" | bc)
pyrust_mean_us=$(echo "$pyrust_mean_seconds * 1000000" | bc)
pyrust_mean_us_int=$(printf "%.0f" "$pyrust_mean_us")

pyrust_stddev_seconds=$(jq -r '.results[0].stddev' "$PYRUST_OUTPUT_JSON")
pyrust_stddev_ms=$(echo "$pyrust_stddev_seconds * 1000" | bc)
pyrust_stddev_us=$(echo "$pyrust_stddev_seconds * 1000000" | bc)

pyrust_min_seconds=$(jq -r '.results[0].min' "$PYRUST_OUTPUT_JSON")
pyrust_min_ms=$(echo "$pyrust_min_seconds * 1000" | bc)
pyrust_min_us=$(echo "$pyrust_min_seconds * 1000000" | bc)

pyrust_max_seconds=$(jq -r '.results[0].max' "$PYRUST_OUTPUT_JSON")
pyrust_max_ms=$(echo "$pyrust_max_seconds * 1000" | bc)
pyrust_max_us=$(echo "$pyrust_max_seconds * 1000000" | bc)

pyrust_cv=$(echo "scale=4; ($pyrust_stddev_us / $pyrust_mean_us) * 100" | bc)

echo "PyRust Results:"
echo "  Mean:   $(printf "%.2f" "$pyrust_mean_ms")ms ($(printf "%.0f" "$pyrust_mean_us")μs)"
echo "  StdDev: $(printf "%.2f" "$pyrust_stddev_ms")ms ($(printf "%.2f" "$pyrust_stddev_us")μs)"
echo "  Min:    $(printf "%.2f" "$pyrust_min_ms")ms ($(printf "%.2f" "$pyrust_min_us")μs)"
echo "  Max:    $(printf "%.2f" "$pyrust_max_ms")ms ($(printf "%.2f" "$pyrust_max_us")μs)"
echo "  CV:     ${pyrust_cv}%"
echo ""

# Step 3: Calculate speedup ratio
echo "================================================================"
echo "Step 3: Speedup Analysis"
echo "================================================================"

speedup=$(echo "scale=2; $cpython_mean_us / $pyrust_mean_us" | bc)
speedup_formatted=$(printf "%.1f" "$speedup")

echo "Speedup Ratio: ${speedup_formatted}x"
echo "  (CPython ${cpython_mean_us_int}μs ÷ PyRust ${pyrust_mean_us_int}μs)"
echo ""

# Step 4: Validation
echo "================================================================"
echo "Step 4: Acceptance Criteria Validation"
echo "================================================================"

PASS=true

# Validate AC6.4: CV < 10% for both CPython and PyRust
echo "AC6.4: Statistical Stability (CV < 10%)"
echo "--------------------------------------"

if (( $(echo "$cpython_cv < $TARGET_CV_PERCENT" | bc -l) )); then
    echo -e "${GREEN}✓ CPython CV: ${cpython_cv}% < 10%${NC}"
else
    echo -e "${RED}✗ FAIL: CPython CV: ${cpython_cv}% ≥ 10% (statistical instability)${NC}"
    PASS=false
fi

if (( $(echo "$pyrust_cv < $TARGET_CV_PERCENT" | bc -l) )); then
    echo -e "${GREEN}✓ PyRust CV: ${pyrust_cv}% < 10%${NC}"
else
    echo -e "${RED}✗ FAIL: PyRust CV: ${pyrust_cv}% ≥ 10% (statistical instability)${NC}"
    PASS=false
fi

echo ""

# Validate AC6.5: ≥50x speedup
echo "AC6.5: Speedup Target (≥50x vs CPython)"
echo "---------------------------------------"

if (( $(echo "$speedup >= $TARGET_SPEEDUP" | bc -l) )); then
    echo -e "${GREEN}✓ PASS: ${speedup_formatted}x speedup ≥ 50x target${NC}"
    echo -e "${GREEN}  PyRust achieves the required 50x performance improvement!${NC}"
else
    echo -e "${RED}✗ FAIL: ${speedup_formatted}x speedup < 50x target${NC}"
    deficit=$(echo "scale=1; $TARGET_SPEEDUP - $speedup" | bc)
    echo -e "${RED}  Deficit: ${deficit}x below target${NC}"
    PASS=false
fi

echo ""

# Step 5: Summary
echo "================================================================"
echo "Summary"
echo "================================================================"
echo ""
echo "Comparison Results:"
echo "  CPython:  $(printf "%.2f" "$cpython_mean_ms")ms (CV: ${cpython_cv}%)"
echo "  PyRust:   $(printf "%.2f" "$pyrust_mean_ms")ms (CV: ${pyrust_cv}%)"
echo "  Speedup:  ${speedup_formatted}x"
echo ""

# Clean up temporary files
rm -f "$CPYTHON_OUTPUT_JSON" "$PYRUST_OUTPUT_JSON"

if [ "$PASS" = true ]; then
    echo -e "${GREEN}================================================================${NC}"
    echo -e "${GREEN}             ✓ VALIDATION PASSED ✓${NC}"
    echo -e "${GREEN}================================================================${NC}"
    echo -e "${GREEN}PyRust achieves ≥50x speedup vs CPython baseline!${NC}"
    echo -e "${GREEN}All acceptance criteria (AC6.4, AC6.5) satisfied.${NC}"
    exit 0
else
    echo -e "${RED}================================================================${NC}"
    echo -e "${RED}             ✗ VALIDATION FAILED ✗${NC}"
    echo -e "${RED}================================================================${NC}"
    echo -e "${RED}PyRust does not meet acceptance criteria.${NC}"
    echo -e "${RED}Check above for specific failures (AC6.4: CV < 10%, AC6.5: ≥50x speedup).${NC}"
    exit 1
fi
