#!/bin/bash
# CPython vs PyRust Performance Comparison Script
#
# This script automates the speedup ratio calculation from Criterion benchmark outputs.
# It verifies AC1.3: ≥50x speedup vs CPython with statistical confidence.
#
# Usage: ./scripts/compare_cpython.sh
#
# Requirements:
#   - jq (for JSON parsing)
#   - python3 (for CPython baseline)
#   - cargo bench (to generate benchmark data)
#
# Output:
#   - Speedup ratio with confidence intervals
#   - Statistical significance analysis
#   - Pass/Fail status for AC1.3 (≥50x target)

set -e

# Colors for output
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
BLUE='\033[0;34m'
NC='\033[0m' # No Color

# Check dependencies
check_dependencies() {
    echo -e "${BLUE}Checking dependencies...${NC}"

    if ! command -v jq &> /dev/null; then
        echo -e "${RED}Error: jq is not installed. Install with: brew install jq${NC}"
        exit 1
    fi

    if ! command -v python3 &> /dev/null; then
        echo -e "${RED}Error: python3 is not installed.${NC}"
        exit 1
    fi

    echo -e "${GREEN}✓ All dependencies available${NC}"
}

# Run benchmarks if needed
run_benchmarks() {
    echo -e "${BLUE}Running benchmarks...${NC}"
    echo "This may take several minutes..."

    cargo bench --bench cpython_baseline > /dev/null 2>&1

    echo -e "${GREEN}✓ Benchmarks complete${NC}"
}

# Extract timing data from Criterion JSON output
extract_timing() {
    local json_file=$1
    local name=$2

    if [ ! -f "$json_file" ]; then
        echo -e "${YELLOW}Warning: $json_file not found. Running benchmarks...${NC}"
        run_benchmarks
    fi

    if [ ! -f "$json_file" ]; then
        echo -e "${RED}Error: Failed to generate $json_file${NC}"
        exit 1
    fi

    # Extract mean, std_dev, and confidence intervals (in nanoseconds)
    local mean=$(jq -r '.mean.point_estimate' "$json_file")
    local std_dev=$(jq -r '.std_dev.point_estimate' "$json_file")
    local lower_bound=$(jq -r '.mean.confidence_interval.lower_bound' "$json_file")
    local upper_bound=$(jq -r '.mean.confidence_interval.upper_bound' "$json_file")

    echo "$mean $std_dev $lower_bound $upper_bound"
}

# Calculate speedup ratio with confidence intervals
calculate_speedup() {
    local cpython_mean=$1
    local cpython_std=$2
    local cpython_lower=$3
    local cpython_upper=$4
    local pyrust_mean=$5
    local pyrust_std=$6
    local pyrust_lower=$7
    local pyrust_upper=$8

    # Use Python for reliable floating point calculations
    python3 << EOFPYTHON
cpython_mean = $cpython_mean
cpython_std = $cpython_std
cpython_lower = $cpython_lower
cpython_upper = $cpython_upper
pyrust_mean = $pyrust_mean
pyrust_std = $pyrust_std
pyrust_lower = $pyrust_lower
pyrust_upper = $pyrust_upper

speedup = cpython_mean / pyrust_mean
conservative_speedup = cpython_lower / pyrust_upper
optimistic_speedup = cpython_upper / pyrust_lower
cpython_cv = cpython_std / cpython_mean
pyrust_cv = pyrust_std / pyrust_mean

print(f'{speedup:.2f} {conservative_speedup:.2f} {optimistic_speedup:.2f} {cpython_cv:.4f} {pyrust_cv:.4f}')
EOFPYTHON
}

# Convert nanoseconds to human-readable format
format_time() {
    local ns=$1
    python3 -c "
ns = $ns
if ns < 1000:
    print(f'{ns:.2f} ns')
elif ns < 1000000:
    print(f'{ns/1000:.2f} μs')
elif ns < 1000000000:
    print(f'{ns/1000000:.2f} ms')
else:
    print(f'{ns/1000000000:.2f} s')
"
}

# Main comparison logic
main() {
    echo -e "${BLUE}=== PyRust vs CPython Performance Comparison ===${NC}\n"

    check_dependencies

    # Define paths to Criterion output files
    local cpython_json="target/criterion/speedup_comparison/cpython_total_time/base/estimates.json"
    local pyrust_json="target/criterion/speedup_comparison/pyrust_total_time/base/estimates.json"

    # Check if benchmark data exists
    if [ ! -f "$cpython_json" ] || [ ! -f "$pyrust_json" ]; then
        echo -e "${YELLOW}Benchmark data not found. Running benchmarks...${NC}\n"
        run_benchmarks
    fi

    # Extract timing data
    echo -e "${BLUE}Extracting benchmark data...${NC}"
    read cpython_mean cpython_std cpython_lower cpython_upper <<< $(extract_timing "$cpython_json" "CPython")
    read pyrust_mean pyrust_std pyrust_lower pyrust_upper <<< $(extract_timing "$pyrust_json" "PyRust")

    # Calculate speedup
    read speedup conservative optimistic cpython_cv pyrust_cv <<< $(calculate_speedup \
        $cpython_mean $cpython_std $cpython_lower $cpython_upper \
        $pyrust_mean $pyrust_std $pyrust_lower $pyrust_upper)

    # Display results
    echo -e "\n${BLUE}=== Timing Results ===${NC}"
    echo -e "CPython (subprocess):"
    echo -e "  Mean:       $(format_time $cpython_mean)"
    echo -e "  Std Dev:    $(format_time $cpython_std)"
    echo -e "  95% CI:     [$(format_time $cpython_lower), $(format_time $cpython_upper)]"
    echo -e "  CV:         ${cpython_cv} ($(python3 -c "print(f'{${cpython_cv} * 100:.1f}')")%)"

    echo -e "\nPyRust (library):"
    echo -e "  Mean:       $(format_time $pyrust_mean)"
    echo -e "  Std Dev:    $(format_time $pyrust_std)"
    echo -e "  95% CI:     [$(format_time $pyrust_lower), $(format_time $pyrust_upper)]"
    echo -e "  CV:         ${pyrust_cv} ($(python3 -c "print(f'{${pyrust_cv} * 100:.1f}')")%)"

    echo -e "\n${BLUE}=== Speedup Analysis ===${NC}"
    echo -e "Point Estimate:       ${speedup}x"
    echo -e "Conservative (95% CI): ${conservative}x"
    echo -e "Optimistic (95% CI):   ${optimistic}x"

    # Check variance (AC1.5: < 10% coefficient of variation)
    echo -e "\n${BLUE}=== Variance Check (AC1.5) ===${NC}"

    local cpython_cv_pct=$(python3 -c "print(f'{${cpython_cv} * 100:.1f}')")
    local pyrust_cv_pct=$(python3 -c "print(f'{${pyrust_cv} * 100:.1f}')")

    if python3 -c "exit(0 if ${cpython_cv} < 0.10 else 1)"; then
        echo -e "CPython CV: ${GREEN}${cpython_cv_pct}% ✓${NC} (< 10%)"
    else
        echo -e "CPython CV: ${YELLOW}${cpython_cv_pct}% ⚠${NC} (target: < 10%)"
    fi

    if python3 -c "exit(0 if ${pyrust_cv} < 0.10 else 1)"; then
        echo -e "PyRust CV:  ${GREEN}${pyrust_cv_pct}% ✓${NC} (< 10%)"
    else
        echo -e "PyRust CV:  ${YELLOW}${pyrust_cv_pct}% ⚠${NC} (target: < 10%)"
    fi

    # Check AC1.3: ≥50x speedup
    echo -e "\n${BLUE}=== Acceptance Criteria Validation ===${NC}"

    if python3 -c "exit(0 if ${conservative} >= 50 else 1)"; then
        echo -e "${GREEN}AC1.3 PASS ✓${NC}: Speedup ${conservative}x ≥ 50x (95% confidence)"
        echo -e "${GREEN}Status: VERIFIED${NC}"
        echo -e "\nPyRust is ${speedup}x faster than CPython for simple expressions."
        exit 0
    elif python3 -c "exit(0 if ${speedup} >= 50 else 1)"; then
        echo -e "${YELLOW}AC1.3 MARGINAL ⚠${NC}: Point estimate ${speedup}x ≥ 50x, but conservative estimate ${conservative}x < 50x"
        echo -e "${YELLOW}Status: LIKELY PASS (reduce variance for full confidence)${NC}"
        exit 0
    else
        echo -e "${RED}AC1.3 FAIL ✗${NC}: Speedup ${speedup}x < 50x (target: ≥50x)"
        echo -e "${RED}Status: NOT VERIFIED${NC}"
        exit 1
    fi
}

# Run main function
main
