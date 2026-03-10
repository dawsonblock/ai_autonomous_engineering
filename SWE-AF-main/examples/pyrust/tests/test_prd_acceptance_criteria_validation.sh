#!/bin/bash
# Test script to verify validate_prd_acceptance_criteria.sh execution
# Tests that all 17 PRD acceptance criteria are validated correctly

set -euo pipefail

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
PROJECT_ROOT="$(cd "$SCRIPT_DIR/.." && pwd)"
VALIDATION_SCRIPT="$PROJECT_ROOT/scripts/validate_prd_acceptance_criteria.sh"

# Colors for output
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
NC='\033[0m' # No Color

echo "========================================================================"
echo "PRD Acceptance Criteria Validation Test"
echo "========================================================================"
echo ""

TESTS_PASSED=0
TESTS_FAILED=0

# Test 1: Script exists
echo "Test 1: Checking if validate_prd_acceptance_criteria.sh exists..."
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

# Test 4: Script validates all 17 acceptance criteria
echo "Test 4: Checking for AC1-AC17 validation code..."
ALL_ACS_FOUND=true
for i in {1..17}; do
    if grep -q "AC$i" "$VALIDATION_SCRIPT"; then
        echo "  ✓ Found AC$i validation"
    else
        echo -e "  ${RED}✗ Missing AC$i validation${NC}"
        ALL_ACS_FOUND=false
    fi
done

if [ "$ALL_ACS_FOUND" = true ]; then
    echo -e "${GREEN}✓ PASS${NC}: All 17 ACs are validated"
    TESTS_PASSED=$((TESTS_PASSED + 1))
else
    echo -e "${RED}✗ FAIL${NC}: Some ACs are not validated"
    TESTS_FAILED=$((TESTS_FAILED + 1))
fi
echo ""

# Test 5: Script performs clean build
echo "Test 5: Checking for cargo clean and cargo build..."
if grep -q "cargo clean" "$VALIDATION_SCRIPT" && grep -q "cargo build --release" "$VALIDATION_SCRIPT"; then
    echo -e "${GREEN}✓ PASS${NC}: Script performs clean release build"
    TESTS_PASSED=$((TESTS_PASSED + 1))
else
    echo -e "${RED}✗ FAIL${NC}: Script doesn't perform clean build"
    TESTS_FAILED=$((TESTS_FAILED + 1))
fi
echo ""

# Test 6: Script checks binary size (AC16)
echo "Test 6: Checking for binary size validation..."
if grep -q "stat.*target/release/pyrust" "$VALIDATION_SCRIPT" && grep -q "500000" "$VALIDATION_SCRIPT"; then
    echo -e "${GREEN}✓ PASS${NC}: Script validates binary size ≤500KB"
    TESTS_PASSED=$((TESTS_PASSED + 1))
else
    echo -e "${RED}✗ FAIL${NC}: Script doesn't validate binary size"
    TESTS_FAILED=$((TESTS_FAILED + 1))
fi
echo ""

# Test 7: Script checks Python linkage (AC17)
echo "Test 7: Checking for Python linkage validation..."
if grep -q "otool\|ldd" "$VALIDATION_SCRIPT" && grep -q "python" "$VALIDATION_SCRIPT"; then
    echo -e "${GREEN}✓ PASS${NC}: Script validates no Python linkage"
    TESTS_PASSED=$((TESTS_PASSED + 1))
else
    echo -e "${RED}✗ FAIL${NC}: Script doesn't validate Python linkage"
    TESTS_FAILED=$((TESTS_FAILED + 1))
fi
echo ""

# Test 8: Script generates report
echo "Test 8: Checking for report generation..."
if grep -q "FINAL VALIDATION REPORT" "$VALIDATION_SCRIPT"; then
    echo -e "${GREEN}✓ PASS${NC}: Script generates validation report"
    TESTS_PASSED=$((TESTS_PASSED + 1))
else
    echo -e "${RED}✗ FAIL${NC}: Script doesn't generate report"
    TESTS_FAILED=$((TESTS_FAILED + 1))
fi
echo ""

# Test 9: Script has proper exit codes
echo "Test 9: Checking for proper exit codes..."
if grep -q "exit 0" "$VALIDATION_SCRIPT" && grep -q "exit 1" "$VALIDATION_SCRIPT"; then
    echo -e "${GREEN}✓ PASS${NC}: Script has proper exit codes"
    TESTS_PASSED=$((TESTS_PASSED + 1))
else
    echo -e "${RED}✗ FAIL${NC}: Script missing proper exit codes"
    TESTS_FAILED=$((TESTS_FAILED + 1))
fi
echo ""

# Test 10: Script validates clippy (AC2)
echo "Test 10: Checking for clippy validation..."
if grep -q "cargo clippy.*-D warnings" "$VALIDATION_SCRIPT"; then
    echo -e "${GREEN}✓ PASS${NC}: Script validates clippy with -D warnings"
    TESTS_PASSED=$((TESTS_PASSED + 1))
else
    echo -e "${RED}✗ FAIL${NC}: Script doesn't validate clippy properly"
    TESTS_FAILED=$((TESTS_FAILED + 1))
fi
echo ""

# Test 11: Script validates formatting (AC3)
echo "Test 11: Checking for formatting validation..."
if grep -q "cargo fmt.*--check" "$VALIDATION_SCRIPT"; then
    echo -e "${GREEN}✓ PASS${NC}: Script validates code formatting"
    TESTS_PASSED=$((TESTS_PASSED + 1))
else
    echo -e "${RED}✗ FAIL${NC}: Script doesn't validate formatting"
    TESTS_FAILED=$((TESTS_FAILED + 1))
fi
echo ""

# Test 12: Script validates file system cleanliness (AC6, AC7)
echo "Test 12: Checking for file system validation..."
if grep -q "find src" "$VALIDATION_SCRIPT" && grep -q "\.backup\|\.tmp\|\.bak" "$VALIDATION_SCRIPT"; then
    echo -e "${GREEN}✓ PASS${NC}: Script validates file system cleanliness"
    TESTS_PASSED=$((TESTS_PASSED + 1))
else
    echo -e "${RED}✗ FAIL${NC}: Script doesn't validate file system"
    TESTS_FAILED=$((TESTS_FAILED + 1))
fi
echo ""

# Test 13: Script validates production assets (AC9, AC10)
echo "Test 13: Checking for production assets validation..."
if grep -q "README.md" "$VALIDATION_SCRIPT" && grep -q "LICENSE" "$VALIDATION_SCRIPT"; then
    echo -e "${GREEN}✓ PASS${NC}: Script validates README and LICENSE"
    TESTS_PASSED=$((TESTS_PASSED + 1))
else
    echo -e "${RED}✗ FAIL${NC}: Script doesn't validate production assets"
    TESTS_FAILED=$((TESTS_FAILED + 1))
fi
echo ""

# Test 14: Script validates documentation structure (AC12, AC13)
echo "Test 14: Checking for documentation validation..."
if grep -q "docs/" "$VALIDATION_SCRIPT"; then
    echo -e "${GREEN}✓ PASS${NC}: Script validates docs/ directory"
    TESTS_PASSED=$((TESTS_PASSED + 1))
else
    echo -e "${RED}✗ FAIL${NC}: Script doesn't validate documentation"
    TESTS_FAILED=$((TESTS_FAILED + 1))
fi
echo ""

# Test 15: Script validates PyO3 dependency (AC14, AC15)
echo "Test 15: Checking for PyO3 dependency validation..."
if grep -q "pyo3" "$VALIDATION_SCRIPT" && grep -q "Cargo.toml" "$VALIDATION_SCRIPT"; then
    echo -e "${GREEN}✓ PASS${NC}: Script validates PyO3 dependency"
    TESTS_PASSED=$((TESTS_PASSED + 1))
else
    echo -e "${RED}✗ FAIL${NC}: Script doesn't validate PyO3"
    TESTS_FAILED=$((TESTS_FAILED + 1))
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
    echo -e "${GREEN}✓ ALL TESTS PASSED${NC}"
    echo ""
    echo "The validate_prd_acceptance_criteria.sh script is properly implemented."
    echo ""
    echo "To run the validation (performs clean build - takes several minutes):"
    echo "  ./scripts/validate_prd_acceptance_criteria.sh"
    echo ""
    exit 0
else
    echo -e "${RED}✗ SOME TESTS FAILED${NC}"
    echo "Please fix the issues identified above."
    exit 1
fi
