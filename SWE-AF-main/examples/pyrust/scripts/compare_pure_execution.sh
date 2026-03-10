#!/bin/bash
# CPython Pure Execution vs PyRust Cold Start Comparison Script
#
# This script compares PyRust cold_start_simple vs CPython pure execution,
# calculates speedup ratio, and validates ≥50x speedup for AC6.
#
# Usage: ./scripts/compare_pure_execution.sh
#
# Requirements:
#   - jq (for JSON parsing)
#   - bc (for floating point arithmetic)
#
# Input files:
#   - target/criterion/cold_start_simple/base/estimates.json (PyRust)
#   - target/criterion/cpython_pure_simple/base/estimates.json (CPython)
#
# Output:
#   - Speedup calculation and PASS/FAIL verdict to stdout
#   - Result written to target/speedup_validation.txt
#   - Exit code 0 on PASS (≥50x), 1 on FAIL (<50x)

set -euo pipefail

# Paths to Criterion JSON output files
PYRUST_JSON="target/criterion/cold_start_simple/base/estimates.json"
CPYTHON_JSON="target/criterion/cpython_pure_simple/base/estimates.json"
OUTPUT_FILE="target/speedup_validation.txt"

# Check dependencies
if ! command -v jq &> /dev/null; then
    echo "Error: jq is not installed. Install with: brew install jq" >&2
    exit 1
fi

if ! command -v bc &> /dev/null; then
    echo "Error: bc is not installed. Install with: brew install bc" >&2
    exit 1
fi

# Check if input files exist
if [ ! -f "$PYRUST_JSON" ]; then
    echo "Error: PyRust benchmark file not found: $PYRUST_JSON" >&2
    echo "Run: cargo bench --bench startup_benchmarks" >&2
    exit 1
fi

if [ ! -f "$CPYTHON_JSON" ]; then
    echo "Error: CPython benchmark file not found: $CPYTHON_JSON" >&2
    echo "Run: cargo bench --bench cpython_pure_execution" >&2
    exit 1
fi

# Extract timing data from Criterion JSON output (in nanoseconds)
PYRUST_TIME_NS=$(jq -r '.mean.point_estimate' "$PYRUST_JSON")
CPYTHON_TIME_NS=$(jq -r '.mean.point_estimate' "$CPYTHON_JSON")

# Validate extracted values are numeric
if ! [[ "$PYRUST_TIME_NS" =~ ^[0-9.]+$ ]]; then
    echo "Error: Invalid PyRust time value: $PYRUST_TIME_NS" >&2
    exit 1
fi

if ! [[ "$CPYTHON_TIME_NS" =~ ^[0-9.]+$ ]]; then
    echo "Error: Invalid CPython time value: $CPYTHON_TIME_NS" >&2
    exit 1
fi

# Calculate speedup = cpython_time_ns / pyrust_time_ns using bc
SPEEDUP=$(echo "scale=2; $CPYTHON_TIME_NS / $PYRUST_TIME_NS" | bc)

# Display results
echo "=== CPython Pure Execution vs PyRust Cold Start Comparison ==="
echo ""
echo "PyRust (cold_start_simple):    ${PYRUST_TIME_NS} ns"
echo "CPython (cpython_pure_simple): ${CPYTHON_TIME_NS} ns"
echo ""
echo "Speedup: ${SPEEDUP}x"
echo ""

# Determine PASS/FAIL based on ≥50.0 threshold
# bc returns 1 for true, 0 for false
PASS_CHECK=$(echo "$SPEEDUP >= 50.0" | bc)

if [ "$PASS_CHECK" -eq 1 ]; then
    VERDICT="PASS"
    echo "Result: PASS (speedup ${SPEEDUP}x ≥ 50.0x)"
    echo "AC6 validation: PyRust achieves ≥50x speedup vs CPython pure execution"
    echo ""
    echo "$VERDICT" > "$OUTPUT_FILE"
    exit 0
else
    VERDICT="FAIL"
    echo "Result: FAIL (speedup ${SPEEDUP}x < 50.0x)"
    echo "AC6 validation: PyRust does NOT achieve ≥50x speedup vs CPython pure execution"
    echo ""
    echo "$VERDICT" > "$OUTPUT_FILE"
    exit 1
fi
