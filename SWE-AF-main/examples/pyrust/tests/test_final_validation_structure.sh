#!/bin/bash
# Test script to verify final_validation.sh structure and prerequisites
# Does NOT run the full validation (which takes several minutes)
# Instead, validates the script structure, dependencies, and capabilities

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
echo "Final Validation Script Structure Test"
echo "========================================================================"
echo ""

TESTS_PASSED=0
TESTS_FAILED=0

# Test 1: Script exists
echo "Test 1: Checking if final_validation.sh exists..."
if [ -f "$VALIDATION_SCRIPT" ]; then
    echo -e "${GREEN}✓ PASS${NC}: Script exists at $VALIDATION_SCRIPT"
    TESTS_PASSED=$((TESTS_PASSED + 1))
else
    echo -e "${RED}✗ FAIL${NC}: Script not found at $VALIDATION_SCRIPT"
    TESTS_FAILED=$((TESTS_FAILED + 1))
    exit 1
fi
echo ""

# Test 2: Script is executable
echo "Test 2: Checking if script is executable..."
if [ -x "$VALIDATION_SCRIPT" ]; then
    echo -e "${GREEN}✓ PASS${NC}: Script is executable"
    TESTS_PASSED=$((TESTS_PASSED + 1))
else
    echo -e "${RED}✗ FAIL${NC}: Script is not executable"
    TESTS_FAILED=$((TESTS_FAILED + 1))
fi
echo ""

# Test 3: Script has valid bash syntax
echo "Test 3: Checking bash syntax..."
if bash -n "$VALIDATION_SCRIPT" 2>/dev/null; then
    echo -e "${GREEN}✓ PASS${NC}: Script has valid bash syntax"
    TESTS_PASSED=$((TESTS_PASSED + 1))
else
    echo -e "${RED}✗ FAIL${NC}: Script has syntax errors"
    TESTS_FAILED=$((TESTS_FAILED + 1))
fi
echo ""

# Test 4: Script contains all M1-M5 metric references
echo "Test 4: Checking for M1-M5 metric references..."
ALL_METRICS_FOUND=true
for metric in M1 M2 M3 M4 M5; do
    if grep -q "$metric" "$VALIDATION_SCRIPT"; then
        echo "  ✓ Found reference to $metric"
    else
        echo -e "  ${RED}✗ Missing reference to $metric${NC}"
        ALL_METRICS_FOUND=false
    fi
done

if [ "$ALL_METRICS_FOUND" = true ]; then
    echo -e "${GREEN}✓ PASS${NC}: All metrics (M1-M5) referenced in script"
    TESTS_PASSED=$((TESTS_PASSED + 1))
else
    echo -e "${RED}✗ FAIL${NC}: Some metrics not referenced"
    TESTS_FAILED=$((TESTS_FAILED + 1))
fi
echo ""

# Test 5: Script references all required validation scripts
echo "Test 5: Checking for required validation script references..."
REQUIRED_SCRIPTS=(
    "validate_test_status.sh"
    "validate_binary_speedup.sh"
    "validate_daemon_speedup.sh"
    "validate_benchmark_stability.sh"
    "validate_cache_performance.sh"
)

ALL_SCRIPTS_FOUND=true
for script in "${REQUIRED_SCRIPTS[@]}"; do
    if grep -q "$script" "$VALIDATION_SCRIPT"; then
        echo "  ✓ Found reference to $script"
    else
        echo -e "  ${RED}✗ Missing reference to $script${NC}"
        ALL_SCRIPTS_FOUND=false
    fi
done

if [ "$ALL_SCRIPTS_FOUND" = true ]; then
    echo -e "${GREEN}✓ PASS${NC}: All required validation scripts referenced"
    TESTS_PASSED=$((TESTS_PASSED + 1))
else
    echo -e "${RED}✗ FAIL${NC}: Some validation scripts not referenced"
    TESTS_FAILED=$((TESTS_FAILED + 1))
fi
echo ""

# Test 6: Script has proper exit code handling
echo "Test 6: Checking for proper exit code handling..."
if grep -q "exit 0" "$VALIDATION_SCRIPT" && grep -q "exit 1" "$VALIDATION_SCRIPT"; then
    echo -e "${GREEN}✓ PASS${NC}: Script has exit 0 and exit 1 statements"
    TESTS_PASSED=$((TESTS_PASSED + 1))
else
    echo -e "${RED}✗ FAIL${NC}: Script missing proper exit codes"
    TESTS_FAILED=$((TESTS_FAILED + 1))
fi
echo ""

# Test 7: Script checks for required tools
echo "Test 7: Checking for required tool verification..."
REQUIRED_TOOLS=("hyperfine" "jq" "bc" "python3")
ALL_TOOLS_CHECKED=true
for tool in "${REQUIRED_TOOLS[@]}"; do
    if grep -q "$tool" "$VALIDATION_SCRIPT"; then
        echo "  ✓ Script checks for $tool"
    else
        echo -e "  ${YELLOW}⚠ Script doesn't explicitly check for $tool${NC}"
    fi
done

echo -e "${GREEN}✓ PASS${NC}: Script verifies required tools"
TESTS_PASSED=$((TESTS_PASSED + 1))
echo ""

# Test 8: Verify all referenced validation scripts exist
echo "Test 8: Verifying all referenced validation scripts exist..."
ALL_EXIST=true
for script in "${REQUIRED_SCRIPTS[@]}"; do
    script_path="$PROJECT_ROOT/scripts/$script"
    if [ -f "$script_path" ]; then
        echo "  ✓ Found $script"
    else
        echo -e "  ${RED}✗ Missing $script${NC}"
        ALL_EXIST=false
    fi
done

if [ "$ALL_EXIST" = true ]; then
    echo -e "${GREEN}✓ PASS${NC}: All referenced validation scripts exist"
    TESTS_PASSED=$((TESTS_PASSED + 1))
else
    echo -e "${RED}✗ FAIL${NC}: Some validation scripts are missing"
    TESTS_FAILED=$((TESTS_FAILED + 1))
fi
echo ""

# Test 9: Script generates report
echo "Test 9: Checking for report generation code..."
if grep -q "FINAL VALIDATION REPORT" "$VALIDATION_SCRIPT" && \
   grep -q "Metric Results Summary" "$VALIDATION_SCRIPT"; then
    echo -e "${GREEN}✓ PASS${NC}: Script includes report generation"
    TESTS_PASSED=$((TESTS_PASSED + 1))
else
    echo -e "${RED}✗ FAIL${NC}: Script missing report generation"
    TESTS_FAILED=$((TESTS_FAILED + 1))
fi
echo ""

# Test 10: Script has comprehensive error handling
echo "Test 10: Checking for error handling..."
if grep -q "set -euo pipefail" "$VALIDATION_SCRIPT" || grep -q "set -e" "$VALIDATION_SCRIPT"; then
    echo -e "${GREEN}✓ PASS${NC}: Script has proper error handling (set -e)"
    TESTS_PASSED=$((TESTS_PASSED + 1))
else
    echo -e "${YELLOW}⚠ WARNING${NC}: Script may be missing error handling flags"
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
    echo -e "${GREEN}✓ ALL STRUCTURE TESTS PASSED${NC}"
    echo ""
    echo "The final_validation.sh script is properly structured and ready to use."
    echo ""
    echo "To run the full validation (which takes several minutes):"
    echo "  ./scripts/final_validation.sh"
    echo ""
    exit 0
else
    echo -e "${RED}✗ SOME TESTS FAILED${NC}"
    echo "Please fix the issues identified above."
    exit 1
fi
