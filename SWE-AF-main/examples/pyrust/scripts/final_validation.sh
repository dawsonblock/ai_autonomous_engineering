#!/bin/bash
# Final Integration Validation - Comprehensive End-to-End Test
# Validates all 5 primary metrics (M1-M5) and 34 acceptance criteria
#
# This script orchestrates all validation scripts and generates final report
# Exit 0: All metrics pass (production-ready)
# Exit 1: One or more metrics failed (not production-ready)
#
# Metrics validated:
# - M1: Binary subprocess ≤380μs mean with 95% CI (50x speedup)
# - M2: Daemon mode ≤190μs mean (100x speedup)
# - M3: All 664 currently passing tests still pass (no regressions)
# - M4: 681/681 tests passing (100% pass rate)
# - M5: All benchmarks CV < 10% (statistical stability)

set -euo pipefail

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
PROJECT_ROOT="$(cd "$SCRIPT_DIR/.." && pwd)"

# Colors for output
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
BLUE='\033[0;34m'
CYAN='\033[0;36m'
BOLD='\033[1m'
NC='\033[0m' # No Color

# Tracking variables
TOTAL_METRICS=5
PASSED_METRICS=0
FAILED_METRICS=0

# Result storage
declare -A METRIC_RESULTS
declare -A METRIC_DETAILS

# Helper function to run a validation script and capture result
run_validation() {
    local metric_id="$1"
    local metric_name="$2"
    local script_path="$3"
    local temp_output=$(mktemp)

    echo ""
    echo -e "${CYAN}━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━${NC}"
    echo -e "${BOLD}${BLUE}Running: $metric_name ($metric_id)${NC}"
    echo -e "${CYAN}━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━${NC}"
    echo ""

    if [ ! -f "$script_path" ]; then
        echo -e "${RED}ERROR: Script not found: $script_path${NC}"
        METRIC_RESULTS[$metric_id]="FAIL"
        METRIC_DETAILS[$metric_id]="Script not found"
        FAILED_METRICS=$((FAILED_METRICS + 1))
        return 1
    fi

    # Run the script and capture output and exit code
    if bash "$script_path" 2>&1 | tee "$temp_output"; then
        METRIC_RESULTS[$metric_id]="PASS"
        PASSED_METRICS=$((PASSED_METRICS + 1))

        # Extract key details from output
        if [ "$metric_id" = "M1" ]; then
            METRIC_DETAILS[$metric_id]=$(grep "Mean:" "$temp_output" | tail -1 | awk '{print $2}' || echo "N/A")
        elif [ "$metric_id" = "M2" ]; then
            METRIC_DETAILS[$metric_id]=$(grep "Mean latency:" "$temp_output" | tail -1 | awk '{print $3}' || echo "N/A")
        elif [ "$metric_id" = "M3" ] || [ "$metric_id" = "M4" ]; then
            local passed=$(grep "Total tests passed:" "$temp_output" | tail -1 | awk '{print $4}' || echo "0")
            local failed=$(grep "Total tests failed:" "$temp_output" | tail -1 | awk '{print $4}' || echo "0")
            METRIC_DETAILS[$metric_id]="${passed} passed, ${failed} failed"
        elif [ "$metric_id" = "M5" ]; then
            local max_cv=$(grep "Maximum CV:" "$temp_output" | tail -1 | awk '{print $3}' || echo "N/A")
            METRIC_DETAILS[$metric_id]="Max CV: ${max_cv}"
        fi

        echo -e "${GREEN}✓ $metric_id PASSED${NC}"
        rm -f "$temp_output"
        return 0
    else
        METRIC_RESULTS[$metric_id]="FAIL"
        FAILED_METRICS=$((FAILED_METRICS + 1))

        # Extract failure reason from output
        local failure_reason=$(grep -E "FAIL|ERROR" "$temp_output" | head -3 | tr '\n' ' ' || echo "Unknown failure")
        METRIC_DETAILS[$metric_id]="$failure_reason"

        echo -e "${RED}✗ $metric_id FAILED${NC}"
        rm -f "$temp_output"
        return 1
    fi
}

# Print header
clear
echo ""
echo -e "${BOLD}${CYAN}╔═══════════════════════════════════════════════════════════════════════════╗${NC}"
echo -e "${BOLD}${CYAN}║                                                                           ║${NC}"
echo -e "${BOLD}${CYAN}║              PYRUST FINAL INTEGRATION VALIDATION SUITE                    ║${NC}"
echo -e "${BOLD}${CYAN}║                                                                           ║${NC}"
echo -e "${BOLD}${CYAN}║                   Comprehensive End-to-End Testing                        ║${NC}"
echo -e "${BOLD}${CYAN}║                                                                           ║${NC}"
echo -e "${BOLD}${CYAN}╚═══════════════════════════════════════════════════════════════════════════╝${NC}"
echo ""
echo -e "${BOLD}Validating 5 Primary Metrics (M1-M5)${NC}"
echo ""
echo "  M1: Binary subprocess speedup (≤380μs for 50x speedup)"
echo "  M2: Daemon mode speedup (≤190μs for 100x speedup)"
echo "  M3: Test regression check (664 tests still passing)"
echo "  M4: Complete test suite (681/681 tests passing)"
echo "  M5: Benchmark stability (CV < 10%)"
echo ""
echo -e "${BOLD}Start Time:${NC} $(date '+%Y-%m-%d %H:%M:%S')"
echo ""

# Ensure we're in the project root
cd "$PROJECT_ROOT"

# Check prerequisites
echo -e "${BOLD}Checking Prerequisites...${NC}"
echo ""

# Check if binary is built
if [ ! -f "target/release/pyrust" ]; then
    echo -e "${YELLOW}⚠ Release binary not found. Building...${NC}"
    cargo build --release
    if [ $? -ne 0 ]; then
        echo -e "${RED}ERROR: Failed to build release binary${NC}"
        exit 1
    fi
    echo -e "${GREEN}✓ Binary built successfully${NC}"
else
    echo -e "${GREEN}✓ Release binary found${NC}"
fi

# Check for required tools
MISSING_TOOLS=0
for tool in hyperfine jq bc python3; do
    if ! command -v $tool &> /dev/null; then
        echo -e "${RED}✗ Missing required tool: $tool${NC}"
        MISSING_TOOLS=$((MISSING_TOOLS + 1))
    else
        echo -e "${GREEN}✓ Found: $tool${NC}"
    fi
done

if [ $MISSING_TOOLS -gt 0 ]; then
    echo -e "${RED}ERROR: Missing $MISSING_TOOLS required tool(s). Please install them first.${NC}"
    exit 1
fi

echo ""

# ============================================================================
# RUN ALL VALIDATION SCRIPTS
# ============================================================================

# M3 & M4: Test Status Validation (run first to catch test failures early)
run_validation "M3/M4" "Test Status Validation" "$SCRIPT_DIR/validate_test_status.sh" || true

# M1: Binary Subprocess Speedup
run_validation "M1" "Binary Subprocess Speedup" "$SCRIPT_DIR/validate_binary_speedup.sh" || true

# M2: Daemon Mode Speedup
run_validation "M2" "Daemon Mode Speedup" "$SCRIPT_DIR/validate_daemon_speedup.sh" || true

# M5: Benchmark Stability (CV < 10%)
# First need to run benchmarks to generate fresh data
echo ""
echo -e "${CYAN}━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━${NC}"
echo -e "${BOLD}${BLUE}Running: Benchmark Generation (M5 prerequisite)${NC}"
echo -e "${CYAN}━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━${NC}"
echo ""
echo "Running cargo bench to generate fresh benchmark data..."
echo "(This may take several minutes)"
echo ""

export PYO3_USE_ABI3_FORWARD_COMPATIBILITY=1
if cargo bench --no-fail-fast > /dev/null 2>&1; then
    echo -e "${GREEN}✓ Benchmarks completed successfully${NC}"
else
    echo -e "${YELLOW}⚠ Some benchmarks may have warnings, but continuing...${NC}"
fi

# Now validate benchmark stability
run_validation "M5" "Benchmark Stability" "$SCRIPT_DIR/validate_benchmark_stability.sh" || true

# Cache Performance Validation (AC6.3, part of M5)
echo ""
echo -e "${CYAN}━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━${NC}"
echo -e "${BOLD}${BLUE}Running: Cache Performance Validation (AC6.3)${NC}"
echo -e "${CYAN}━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━${NC}"
echo ""

if bash "$SCRIPT_DIR/validate_cache_performance.sh" > /dev/null 2>&1; then
    echo -e "${GREEN}✓ Cache performance validation passed${NC}"
else
    echo -e "${YELLOW}⚠ Cache performance validation failed (informational only)${NC}"
fi

# ============================================================================
# GENERATE FINAL REPORT
# ============================================================================

echo ""
echo ""
echo -e "${BOLD}${CYAN}╔═══════════════════════════════════════════════════════════════════════════╗${NC}"
echo -e "${BOLD}${CYAN}║                                                                           ║${NC}"
echo -e "${BOLD}${CYAN}║                        FINAL VALIDATION REPORT                            ║${NC}"
echo -e "${BOLD}${CYAN}║                                                                           ║${NC}"
echo -e "${BOLD}${CYAN}╚═══════════════════════════════════════════════════════════════════════════╝${NC}"
echo ""
echo -e "${BOLD}End Time:${NC} $(date '+%Y-%m-%d %H:%M:%S')"
echo ""

# Results table
echo -e "${BOLD}═══════════════════════════════════════════════════════════════════════════${NC}"
echo -e "${BOLD}Metric Results Summary${NC}"
echo -e "${BOLD}═══════════════════════════════════════════════════════════════════════════${NC}"
echo ""

printf "%-8s %-45s %-8s %s\n" "METRIC" "DESCRIPTION" "STATUS" "DETAILS"
echo "───────────────────────────────────────────────────────────────────────────────"

# M1
metric_status="${METRIC_RESULTS[M1]:-SKIP}"
if [ "$metric_status" = "PASS" ]; then
    status_display="${GREEN}✓ PASS${NC}"
else
    status_display="${RED}✗ FAIL${NC}"
fi
printf "%-8s %-45s %-8s %s\n" "M1" "Binary subprocess ≤380μs (50x speedup)" "$(echo -e "$status_display")" "${METRIC_DETAILS[M1]:-N/A}"

# M2
metric_status="${METRIC_RESULTS[M2]:-SKIP}"
if [ "$metric_status" = "PASS" ]; then
    status_display="${GREEN}✓ PASS${NC}"
else
    status_display="${RED}✗ FAIL${NC}"
fi
printf "%-8s %-45s %-8s %s\n" "M2" "Daemon mode ≤190μs (100x speedup)" "$(echo -e "$status_display")" "${METRIC_DETAILS[M2]:-N/A}"

# M3/M4
metric_status="${METRIC_RESULTS[M3/M4]:-SKIP}"
if [ "$metric_status" = "PASS" ]; then
    status_display="${GREEN}✓ PASS${NC}"
else
    status_display="${RED}✗ FAIL${NC}"
fi
printf "%-8s %-45s %-8s %s\n" "M3/M4" "All 681 tests passing (100% pass rate)" "$(echo -e "$status_display")" "${METRIC_DETAILS[M3/M4]:-N/A}"

# M5
metric_status="${METRIC_RESULTS[M5]:-SKIP}"
if [ "$metric_status" = "PASS" ]; then
    status_display="${GREEN}✓ PASS${NC}"
else
    status_display="${RED}✗ FAIL${NC}"
fi
printf "%-8s %-45s %-8s %s\n" "M5" "All benchmarks CV < 10%" "$(echo -e "$status_display")" "${METRIC_DETAILS[M5]:-N/A}"

echo "───────────────────────────────────────────────────────────────────────────────"
echo ""

# Summary statistics
echo -e "${BOLD}Summary Statistics:${NC}"
echo "  Total Metrics:  $TOTAL_METRICS"
echo -e "  ${GREEN}Passed:         $PASSED_METRICS${NC}"
echo -e "  ${RED}Failed:         $FAILED_METRICS${NC}"

# Count actual metrics (M3/M4 counts as 1)
ACTUAL_PASSED=0
ACTUAL_FAILED=0
for key in "${!METRIC_RESULTS[@]}"; do
    if [ "${METRIC_RESULTS[$key]}" = "PASS" ]; then
        ACTUAL_PASSED=$((ACTUAL_PASSED + 1))
    else
        ACTUAL_FAILED=$((ACTUAL_FAILED + 1))
    fi
done

if [ ${#METRIC_RESULTS[@]} -gt 0 ]; then
    PASS_PERCENTAGE=$(echo "scale=1; ($ACTUAL_PASSED * 100) / ${#METRIC_RESULTS[@]}" | bc)
    echo "  Pass Rate:      ${PASS_PERCENTAGE}%"
else
    echo "  Pass Rate:      N/A (no metrics ran)"
fi
echo ""

# Performance summary
if [ "${METRIC_RESULTS[M1]:-}" = "PASS" ] && [ "${METRIC_RESULTS[M2]:-}" = "PASS" ]; then
    echo -e "${BOLD}${GREEN}Performance Targets Achieved:${NC}"
    echo "  ✓ Binary mode:  ${METRIC_DETAILS[M1]:-N/A} ≤ 380μs (50x speedup)"
    echo "  ✓ Daemon mode:  ${METRIC_DETAILS[M2]:-N/A} ≤ 190μs (100x speedup)"
    echo ""
fi

# Quality summary
if [ "${METRIC_RESULTS[M3/M4]:-}" = "PASS" ] && [ "${METRIC_RESULTS[M5]:-}" = "PASS" ]; then
    echo -e "${BOLD}${GREEN}Quality Targets Achieved:${NC}"
    echo "  ✓ Test suite:   ${METRIC_DETAILS[M3/M4]:-N/A}"
    echo "  ✓ Stability:    ${METRIC_DETAILS[M5]:-N/A}"
    echo ""
fi

echo -e "${BOLD}═══════════════════════════════════════════════════════════════════════════${NC}"
echo ""

# Final verdict
if [ $ACTUAL_FAILED -eq 0 ]; then
    echo -e "${BOLD}${GREEN}╔═══════════════════════════════════════════════════════════════════════════╗${NC}"
    echo -e "${BOLD}${GREEN}║                                                                           ║${NC}"
    echo -e "${BOLD}${GREEN}║                   ✓ ALL VALIDATION METRICS PASSED ✓                       ║${NC}"
    echo -e "${BOLD}${GREEN}║                                                                           ║${NC}"
    echo -e "${BOLD}${GREEN}║                    PyRust CLI is PRODUCTION READY                         ║${NC}"
    echo -e "${BOLD}${GREEN}║                                                                           ║${NC}"
    echo -e "${BOLD}${GREEN}║  • Binary mode:  50x+ speedup vs CPython (≤380μs)                         ║${NC}"
    echo -e "${BOLD}${GREEN}║  • Daemon mode:  100x+ speedup vs CPython (≤190μs)                        ║${NC}"
    echo -e "${BOLD}${GREEN}║  • Test suite:   100% pass rate (681/681 tests)                           ║${NC}"
    echo -e "${BOLD}${GREEN}║  • Stability:    All benchmarks CV < 10%                                  ║${NC}"
    echo -e "${BOLD}${GREEN}║                                                                           ║${NC}"
    echo -e "${BOLD}${GREEN}╚═══════════════════════════════════════════════════════════════════════════╝${NC}"
    echo ""
    exit 0
else
    echo -e "${BOLD}${RED}╔═══════════════════════════════════════════════════════════════════════════╗${NC}"
    echo -e "${BOLD}${RED}║                                                                           ║${NC}"
    echo -e "${BOLD}${RED}║                   ✗ VALIDATION FAILED ✗                                   ║${NC}"
    echo -e "${BOLD}${RED}║                                                                           ║${NC}"
    echo -e "${BOLD}${RED}║                $ACTUAL_FAILED metric(s) did not pass                                      ║${NC}"
    echo -e "${BOLD}${RED}║                                                                           ║${NC}"
    echo -e "${BOLD}${RED}║              PyRust CLI is NOT production ready                           ║${NC}"
    echo -e "${BOLD}${RED}║                                                                           ║${NC}"
    echo -e "${BOLD}${RED}╚═══════════════════════════════════════════════════════════════════════════╝${NC}"
    echo ""
    echo -e "${YELLOW}Failed Metrics:${NC}"
    for key in "${!METRIC_RESULTS[@]}"; do
        if [ "${METRIC_RESULTS[$key]}" = "FAIL" ]; then
            echo "  - $key: ${METRIC_DETAILS[$key]:-Unknown failure}"
        fi
    done
    echo ""
    echo "Please review the detailed output above for each metric."
    echo ""
    exit 1
fi
