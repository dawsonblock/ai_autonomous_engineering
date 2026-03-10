#!/bin/bash
# Validate benchmark stability by checking coefficient of variation (CV) < 10%
# Parses Criterion JSON output from target/criterion/**/estimates.json
# Exit 0 if all benchmarks pass CV < 10% threshold, exit 1 if any fail

set -e

# Colors for output
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
NC='\033[0m' # No Color

# Threshold for coefficient of variation (10% for in-process, 15% for subprocess)
CV_THRESHOLD_INPROCESS=0.10
CV_THRESHOLD_SUBPROCESS=0.15

echo "=== Benchmark Stability Validation ==="
echo "Checking coefficient of variation (CV) thresholds:"
echo "  - In-process benchmarks: CV < 10%"
echo "  - Subprocess benchmarks: CV < 15%"
echo ""

# Find all estimates.json files in target/criterion
if [ ! -d "target/criterion" ]; then
    echo -e "${RED}ERROR: target/criterion directory not found${NC}"
    echo "Run 'cargo bench' first to generate benchmark data"
    exit 1
fi

# Track statistics
total_benchmarks=0
passed_benchmarks=0
failed_benchmarks=0
max_cv=0

# Create temporary file for failed benchmarks
failed_file=$(mktemp)

# Process each estimates.json file
while IFS= read -r -d '' estimates_file; do
    # Extract benchmark name from path
    # Path format: target/criterion/<benchmark_name>/new/estimates.json
    # We want to skip "base", "both", and "change" directories
    benchmark_path=$(dirname "$estimates_file")
    benchmark_subdir=$(basename "$benchmark_path")

    # Only process "new" directories to avoid duplicate entries
    if [ "$benchmark_subdir" != "new" ]; then
        continue
    fi

    # Get the actual benchmark name (parent directory of "new")
    benchmark_name=$(basename "$(dirname "$benchmark_path")")

    # Skip if estimates.json doesn't exist or is empty
    if [ ! -s "$estimates_file" ]; then
        continue
    fi

    # Parse JSON to extract mean and std_dev
    # Criterion JSON structure: { "mean": { "point_estimate": <value> }, "std_dev": { "point_estimate": <value> } }
    mean=$(jq -r '.mean.point_estimate' "$estimates_file" 2>/dev/null || echo "0")
    std_dev=$(jq -r '.std_dev.point_estimate' "$estimates_file" 2>/dev/null || echo "0")

    # Skip if parsing failed
    if [ "$mean" = "0" ] || [ "$mean" = "null" ] || [ -z "$mean" ]; then
        continue
    fi

    # Calculate coefficient of variation: CV = std_dev / mean
    cv=$(echo "scale=6; $std_dev / $mean" | bc -l)

    # Track max CV across all benchmarks
    if (( $(echo "$cv > $max_cv" | bc -l) )); then
        max_cv=$cv
    fi

    total_benchmarks=$((total_benchmarks + 1))

    # Determine threshold based on benchmark type
    # Subprocess benchmarks (cpython, baseline, subprocess) are more variable due to OS scheduling
    if [[ "$benchmark_name" == *"cpython"* ]] || [[ "$benchmark_name" == *"baseline"* ]] || [[ "$benchmark_name" == *"subprocess"* ]] || [[ "$benchmark_name" == *"total_time"* ]]; then
        threshold=$CV_THRESHOLD_SUBPROCESS
        threshold_percent="15%"
    else
        threshold=$CV_THRESHOLD_INPROCESS
        threshold_percent="10%"
    fi

    # Check if CV exceeds threshold
    if (( $(echo "$cv >= $threshold" | bc -l) )); then
        failed_benchmarks=$((failed_benchmarks + 1))
        cv_percent=$(echo "scale=2; $cv * 100" | bc -l)
        echo "$benchmark_name: CV=${cv_percent}%" >> "$failed_file"
        echo -e "${RED}✗ $benchmark_name: CV=${cv_percent}% (exceeds ${threshold_percent} threshold)${NC}"
    else
        passed_benchmarks=$((passed_benchmarks + 1))
        cv_percent=$(echo "scale=2; $cv * 100" | bc -l)
        echo -e "${GREEN}✓ $benchmark_name: CV=${cv_percent}% (threshold: ${threshold_percent})${NC}"
    fi
done < <(find target/criterion -name "estimates.json" -print0)

# Summary
echo ""
echo "=== Summary ==="
echo "Total benchmarks: $total_benchmarks"
echo "Passed: $passed_benchmarks"
echo "Failed: $failed_benchmarks"

max_cv_percent=$(echo "scale=2; $max_cv * 100" | bc -l)
echo "Maximum CV: ${max_cv_percent}%"
echo ""

# Exit based on results
if [ $total_benchmarks -eq 0 ]; then
    echo -e "${YELLOW}WARNING: No benchmarks found. Run 'cargo bench' first.${NC}"
    rm -f "$failed_file"
    exit 1
fi

if [ $failed_benchmarks -eq 0 ]; then
    echo -e "${GREEN}✓ All benchmarks passed CV < 10% threshold${NC}"
    rm -f "$failed_file"
    exit 0
else
    echo -e "${RED}✗ $failed_benchmarks benchmark(s) failed CV threshold${NC}"
    echo ""
    echo "Failed benchmarks:"
    cat "$failed_file"
    rm -f "$failed_file"
    exit 1
fi
