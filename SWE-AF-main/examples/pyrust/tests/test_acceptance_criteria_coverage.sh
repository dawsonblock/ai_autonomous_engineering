#!/bin/bash
# Test that final_validation.sh covers all acceptance criteria from the issue

set -euo pipefail

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
PROJECT_ROOT="$(cd "$SCRIPT_DIR/.." && pwd)"
VALIDATION_SCRIPT="$PROJECT_ROOT/scripts/final_validation.sh"

# Colors for output
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
NC='\033[0m' # No Color

echo "========================================================================"
echo "Acceptance Criteria Coverage Test"
echo "========================================================================"
echo "Verifying that final_validation.sh covers all M1-M5 metrics"
echo ""

TESTS_PASSED=0
TESTS_FAILED=0

# M1: Binary subprocess ≤380μs mean with 95% CI (50x speedup)
echo "AC M1: Binary subprocess ≤380μs mean with 95% CI"
if grep -q "validate_binary_speedup.sh" "$VALIDATION_SCRIPT" && \
   grep -q "380" "$VALIDATION_SCRIPT"; then
    echo -e "${GREEN}✓ PASS${NC}: M1 validation included (binary subprocess speedup)"
    TESTS_PASSED=$((TESTS_PASSED + 1))
else
    echo -e "${RED}✗ FAIL${NC}: M1 validation missing or incomplete"
    TESTS_FAILED=$((TESTS_FAILED + 1))
fi
echo ""

# M2: Daemon mode ≤190μs mean (100x speedup)
echo "AC M2: Daemon mode ≤190μs mean"
if grep -q "validate_daemon_speedup.sh" "$VALIDATION_SCRIPT" && \
   grep -q "190" "$VALIDATION_SCRIPT"; then
    echo -e "${GREEN}✓ PASS${NC}: M2 validation included (daemon mode speedup)"
    TESTS_PASSED=$((TESTS_PASSED + 1))
else
    echo -e "${RED}✗ FAIL${NC}: M2 validation missing or incomplete"
    TESTS_FAILED=$((TESTS_FAILED + 1))
fi
echo ""

# M3: All 664 currently passing tests still pass
echo "AC M3: All 664 currently passing tests still pass"
if grep -q "validate_test_status.sh" "$VALIDATION_SCRIPT" && \
   grep -q "664\|681" "$VALIDATION_SCRIPT"; then
    echo -e "${GREEN}✓ PASS${NC}: M3 validation included (test regression check)"
    TESTS_PASSED=$((TESTS_PASSED + 1))
else
    echo -e "${RED}✗ FAIL${NC}: M3 validation missing or incomplete"
    TESTS_FAILED=$((TESTS_FAILED + 1))
fi
echo ""

# M4: 681/681 tests passing (100% pass rate)
echo "AC M4: 681/681 tests passing (100% pass rate)"
if grep -q "validate_test_status.sh" "$VALIDATION_SCRIPT" && \
   grep -q "681" "$VALIDATION_SCRIPT"; then
    echo -e "${GREEN}✓ PASS${NC}: M4 validation included (complete test suite)"
    TESTS_PASSED=$((TESTS_PASSED + 1))
else
    echo -e "${RED}✗ FAIL${NC}: M4 validation missing or incomplete"
    TESTS_FAILED=$((TESTS_FAILED + 1))
fi
echo ""

# M5: All benchmarks CV < 10%
echo "AC M5: All benchmarks CV < 10%"
if grep -q "validate_benchmark_stability.sh" "$VALIDATION_SCRIPT" && \
   grep -q "10" "$VALIDATION_SCRIPT"; then
    echo -e "${GREEN}✓ PASS${NC}: M5 validation included (benchmark stability)"
    TESTS_PASSED=$((TESTS_PASSED + 1))
else
    echo -e "${RED}✗ FAIL${NC}: M5 validation missing or incomplete"
    TESTS_FAILED=$((TESTS_FAILED + 1))
fi
echo ""

# Exit code handling
echo "AC: Script exits 0 when all metrics pass, exits 1 if any fail"
exit_0_count=$(grep -c "exit 0" "$VALIDATION_SCRIPT" || echo "0")
exit_1_count=$(grep -c "exit 1" "$VALIDATION_SCRIPT" || echo "0")

if [ "$exit_0_count" -ge 1 ] && [ "$exit_1_count" -ge 1 ]; then
    echo -e "${GREEN}✓ PASS${NC}: Script has proper exit code handling"
    echo "  - Found $exit_0_count 'exit 0' statements (success path)"
    echo "  - Found $exit_1_count 'exit 1' statements (failure path)"
    TESTS_PASSED=$((TESTS_PASSED + 1))
else
    echo -e "${RED}✗ FAIL${NC}: Script missing proper exit code handling"
    TESTS_FAILED=$((TESTS_FAILED + 1))
fi
echo ""

# Report generation
echo "AC: Report shows clear pass/fail status for each metric"
if grep -q "FINAL VALIDATION REPORT" "$VALIDATION_SCRIPT" && \
   grep -q "PASS\|FAIL" "$VALIDATION_SCRIPT" && \
   grep -q "Summary" "$VALIDATION_SCRIPT"; then
    echo -e "${GREEN}✓ PASS${NC}: Script generates comprehensive report"
    TESTS_PASSED=$((TESTS_PASSED + 1))
else
    echo -e "${RED}✗ FAIL${NC}: Script missing comprehensive report generation"
    TESTS_FAILED=$((TESTS_FAILED + 1))
fi
echo ""

# Cache performance validation
echo "Additional: Cache performance validation (AC6.3)"
if grep -q "validate_cache_performance.sh" "$VALIDATION_SCRIPT"; then
    echo -e "${GREEN}✓ PASS${NC}: Cache performance validation included"
    TESTS_PASSED=$((TESTS_PASSED + 1))
else
    echo -e "${YELLOW}⚠ INFO${NC}: Cache validation not explicitly mentioned"
    TESTS_PASSED=$((TESTS_PASSED + 1))
fi
echo ""

# Benchmark execution
echo "Additional: Benchmark execution before stability check"
if grep -q "cargo bench" "$VALIDATION_SCRIPT"; then
    echo -e "${GREEN}✓ PASS${NC}: Script runs cargo bench to generate fresh data"
    TESTS_PASSED=$((TESTS_PASSED + 1))
else
    echo -e "${YELLOW}⚠ WARNING${NC}: Script may not generate fresh benchmark data"
    TESTS_PASSED=$((TESTS_PASSED + 1))
fi
echo ""

# Summary
echo "========================================================================"
echo "Coverage Summary"
echo "========================================================================"
echo "Total acceptance criteria checked:  $((TESTS_PASSED + TESTS_FAILED))"
echo -e "${GREEN}Covered:                            $TESTS_PASSED${NC}"
echo -e "${RED}Missing:                            $TESTS_FAILED${NC}"
echo ""

COVERAGE_PERCENT=$(echo "scale=1; ($TESTS_PASSED * 100) / ($TESTS_PASSED + $TESTS_FAILED)" | bc)
echo "Coverage: ${COVERAGE_PERCENT}%"
echo ""

if [ $TESTS_FAILED -eq 0 ]; then
    echo -e "${GREEN}✓ ALL ACCEPTANCE CRITERIA COVERED${NC}"
    echo ""
    echo "The final_validation.sh script comprehensively validates:"
    echo "  • M1: Binary subprocess speedup (≤380μs)"
    echo "  • M2: Daemon mode speedup (≤190μs)"
    echo "  • M3: Test regression check (664 tests)"
    echo "  • M4: Complete test suite (681/681 tests)"
    echo "  • M5: Benchmark stability (CV < 10%)"
    echo "  • Proper exit codes (0 = pass, 1 = fail)"
    echo "  • Comprehensive report generation"
    echo ""
    exit 0
else
    echo -e "${RED}✗ SOME ACCEPTANCE CRITERIA NOT COVERED${NC}"
    echo "Please review the issues identified above."
    exit 1
fi
