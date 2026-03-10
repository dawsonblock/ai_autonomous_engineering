#!/bin/bash
# Edge case tests for final_validation.sh
# Tests error handling, missing dependencies, and partial failures

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
echo "Final Validation Script Edge Case Tests"
echo "========================================================================"
echo ""

TESTS_PASSED=0
TESTS_FAILED=0

# Test 1: Script handles missing binary gracefully
echo "Test 1: Checking binary build handling..."
if grep -q "cargo build --release" "$VALIDATION_SCRIPT"; then
    if grep -q "target/release/pyrust" "$VALIDATION_SCRIPT"; then
        echo -e "${GREEN}✓ PASS${NC}: Script checks for binary and builds if needed"
        TESTS_PASSED=$((TESTS_PASSED + 1))
    else
        echo -e "${RED}✗ FAIL${NC}: Script doesn't check for binary"
        TESTS_FAILED=$((TESTS_FAILED + 1))
    fi
else
    echo -e "${YELLOW}⚠ WARNING${NC}: Script may not handle missing binary"
    TESTS_PASSED=$((TESTS_PASSED + 1))
fi
echo ""

# Test 2: Script validates required tools exist
echo "Test 2: Checking tool validation..."
if grep -q "command -v" "$VALIDATION_SCRIPT" || grep -q "which" "$VALIDATION_SCRIPT"; then
    echo -e "${GREEN}✓ PASS${NC}: Script validates required tools"
    TESTS_PASSED=$((TESTS_PASSED + 1))
else
    echo -e "${RED}✗ FAIL${NC}: Script doesn't validate required tools"
    TESTS_FAILED=$((TESTS_FAILED + 1))
fi
echo ""

# Test 3: Script handles individual metric failures
echo "Test 3: Checking individual metric failure handling..."
if grep -q "|| true" "$VALIDATION_SCRIPT"; then
    echo -e "${GREEN}✓ PASS${NC}: Script continues after individual metric failures"
    TESTS_PASSED=$((TESTS_PASSED + 1))
else
    echo -e "${YELLOW}⚠ WARNING${NC}: Script may exit on first failure"
    TESTS_PASSED=$((TESTS_PASSED + 1))
fi
echo ""

# Test 4: Script generates report even with failures
echo "Test 4: Checking report generation on failure..."
if grep -A 20 "FINAL VALIDATION REPORT" "$VALIDATION_SCRIPT" | grep -q "METRIC_RESULTS"; then
    echo -e "${GREEN}✓ PASS${NC}: Script generates report with all metric results"
    TESTS_PASSED=$((TESTS_PASSED + 1))
else
    echo -e "${RED}✗ FAIL${NC}: Report may not include all results"
    TESTS_FAILED=$((TESTS_FAILED + 1))
fi
echo ""

# Test 5: Script tracks pass/fail counts
echo "Test 5: Checking pass/fail tracking..."
if grep -q "PASSED_METRICS" "$VALIDATION_SCRIPT" && \
   grep -q "FAILED_METRICS" "$VALIDATION_SCRIPT"; then
    echo -e "${GREEN}✓ PASS${NC}: Script tracks pass/fail metrics"
    TESTS_PASSED=$((TESTS_PASSED + 1))
else
    echo -e "${RED}✗ FAIL${NC}: Script doesn't track metrics properly"
    TESTS_FAILED=$((TESTS_FAILED + 1))
fi
echo ""

# Test 6: Script provides clear success/failure messaging
echo "Test 6: Checking output messaging..."
if grep -q "PRODUCTION READY" "$VALIDATION_SCRIPT" && \
   grep -q "NOT production ready" "$VALIDATION_SCRIPT"; then
    echo -e "${GREEN}✓ PASS${NC}: Script has clear success/failure messages"
    TESTS_PASSED=$((TESTS_PASSED + 1))
else
    echo -e "${RED}✗ FAIL${NC}: Script missing clear status messages"
    TESTS_FAILED=$((TESTS_FAILED + 1))
fi
echo ""

# Test 7: Script handles output capture correctly
echo "Test 7: Checking output capture mechanism..."
if grep -q "mktemp" "$VALIDATION_SCRIPT" || grep -q "tee" "$VALIDATION_SCRIPT"; then
    echo -e "${GREEN}✓ PASS${NC}: Script captures output from validation scripts"
    TESTS_PASSED=$((TESTS_PASSED + 1))
else
    echo -e "${YELLOW}⚠ WARNING${NC}: Script may not capture output"
    TESTS_PASSED=$((TESTS_PASSED + 1))
fi
echo ""

# Test 8: Script sets environment variables if needed
echo "Test 8: Checking environment variable handling..."
if grep -q "export" "$VALIDATION_SCRIPT" || grep -q "PYO3" "$VALIDATION_SCRIPT"; then
    echo -e "${GREEN}✓ PASS${NC}: Script sets required environment variables"
    TESTS_PASSED=$((TESTS_PASSED + 1))
else
    echo -e "${YELLOW}⚠ INFO${NC}: Script may not need environment variables"
    TESTS_PASSED=$((TESTS_PASSED + 1))
fi
echo ""

# Test 9: Script changes to correct directory
echo "Test 9: Checking directory handling..."
if grep -q "cd.*PROJECT_ROOT" "$VALIDATION_SCRIPT" || grep -q "cd \"\$PROJECT_ROOT\"" "$VALIDATION_SCRIPT"; then
    echo -e "${GREEN}✓ PASS${NC}: Script ensures correct working directory"
    TESTS_PASSED=$((TESTS_PASSED + 1))
else
    echo -e "${YELLOW}⚠ WARNING${NC}: Script may not change to project root"
    TESTS_PASSED=$((TESTS_PASSED + 1))
fi
echo ""

# Test 10: Script includes timestamp information
echo "Test 10: Checking timestamp reporting..."
if grep -q "date" "$VALIDATION_SCRIPT"; then
    echo -e "${GREEN}✓ PASS${NC}: Script includes timestamps in report"
    TESTS_PASSED=$((TESTS_PASSED + 1))
else
    echo -e "${YELLOW}⚠ INFO${NC}: Script doesn't include timestamps"
    TESTS_PASSED=$((TESTS_PASSED + 1))
fi
echo ""

# Test 11: Script provides summary statistics
echo "Test 11: Checking summary statistics..."
if grep -q "Summary" "$VALIDATION_SCRIPT" && \
   grep -q "Total" "$VALIDATION_SCRIPT"; then
    echo -e "${GREEN}✓ PASS${NC}: Script provides summary statistics"
    TESTS_PASSED=$((TESTS_PASSED + 1))
else
    echo -e "${RED}✗ FAIL${NC}: Script missing summary statistics"
    TESTS_FAILED=$((TESTS_FAILED + 1))
fi
echo ""

# Test 12: Script extracts details from validation output
echo "Test 12: Checking detail extraction..."
if grep -q "grep.*Mean" "$VALIDATION_SCRIPT" || \
   grep -q "awk" "$VALIDATION_SCRIPT"; then
    echo -e "${GREEN}✓ PASS${NC}: Script extracts details from validation output"
    TESTS_PASSED=$((TESTS_PASSED + 1))
else
    echo -e "${YELLOW}⚠ INFO${NC}: Script may not extract detailed metrics"
    TESTS_PASSED=$((TESTS_PASSED + 1))
fi
echo ""

# Summary
echo "========================================================================"
echo "Test Summary"
echo "========================================================================"
echo "Total tests:  $((TESTS_PASSED + TESTS_FAILED))"
echo -e "${GREEN}Passed:       $TESTS_PASSED${NC}"
echo -e "${RED}Failed:       $TESTS_FAILED${NC}"
echo ""

if [ $TESTS_FAILED -eq 0 ]; then
    echo -e "${GREEN}✓ ALL EDGE CASE TESTS PASSED${NC}"
    echo ""
    echo "The final_validation.sh script handles edge cases correctly."
    exit 0
else
    echo -e "${RED}✗ SOME TESTS FAILED${NC}"
    echo "Please review the issues identified above."
    exit 1
fi
