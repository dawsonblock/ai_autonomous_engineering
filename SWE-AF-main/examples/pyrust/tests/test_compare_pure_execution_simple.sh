#!/bin/bash
# Simplified test suite for scripts/compare_pure_execution.sh
# Tests all acceptance criteria with clear, focused tests

set -euo pipefail

PROJECT_ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
SCRIPT_UNDER_TEST="$PROJECT_ROOT/scripts/compare_pure_execution.sh"
TEST_DIR="$PROJECT_ROOT/target/test_compare_pure"

TESTS_PASSED=0
TESTS_FAILED=0

# Cleanup function
cleanup() {
    rm -rf "$TEST_DIR"
}

# Trap to ensure cleanup
trap cleanup EXIT

# Helper to create mock JSON
create_mock_json() {
    local file="$1"
    local time_ns="$2"
    mkdir -p "$(dirname "$file")"
    cat > "$file" << EOF
{
  "mean": {
    "point_estimate": $time_ns
  }
}
EOF
}

# Test 1: Script exists and is executable (AC1)
echo "Test 1: Script exists with executable permissions (AC1)"
if [ -x "$SCRIPT_UNDER_TEST" ]; then
    echo "  ✓ PASS"
    ((TESTS_PASSED++))
else
    echo "  ✗ FAIL: Script not executable"
    ((TESTS_FAILED++))
fi

# Test 2: Script references correct input files (AC2)
echo "Test 2: Script reads correct input files (AC2)"
if grep -q "cold_start_simple/base/estimates.json" "$SCRIPT_UNDER_TEST" && \
   grep -q "cpython_pure_simple/base/estimates.json" "$SCRIPT_UNDER_TEST"; then
    echo "  ✓ PASS"
    ((TESTS_PASSED++))
else
    echo "  ✗ FAIL: Incorrect input paths"
    ((TESTS_FAILED++))
fi

# Test 3: Script uses bc for calculation (AC3)
echo "Test 3: Script uses bc for calculation (AC3)"
if grep -q "bc" "$SCRIPT_UNDER_TEST"; then
    echo "  ✓ PASS"
    ((TESTS_PASSED++))
else
    echo "  ✗ FAIL: bc not found in script"
    ((TESTS_FAILED++))
fi

# Test 4: Script writes to correct output file (AC4)
echo "Test 4: Script writes to target/speedup_validation.txt (AC4)"
if grep -q "target/speedup_validation.txt" "$SCRIPT_UNDER_TEST"; then
    echo "  ✓ PASS"
    ((TESTS_PASSED++))
else
    echo "  ✗ FAIL: Incorrect output path"
    ((TESTS_FAILED++))
fi

# Test 5: PASS case with exactly 50x speedup (AC4, AC5)
echo "Test 5: PASS case with 50x speedup (AC4, AC5)"
cleanup
create_mock_json "$TEST_DIR/pyrust.json" "300.0"
create_mock_json "$TEST_DIR/cpython.json" "15000.0"

# Temporarily modify script to use test files
output=$(cd "$PROJECT_ROOT" && sed "s|target/criterion/cold_start_simple/base/estimates.json|$TEST_DIR/pyrust.json|g; s|target/criterion/cpython_pure_simple/base/estimates.json|$TEST_DIR/cpython.json|g; s|target/speedup_validation.txt|$TEST_DIR/result.txt|g" "$SCRIPT_UNDER_TEST" | bash 2>&1) || true
exit_code=$?

if [ $exit_code -eq 0 ] && echo "$output" | grep -q "50.00x" && echo "$output" | grep -q "PASS" && [ -f "$TEST_DIR/result.txt" ] && grep -q "PASS" "$TEST_DIR/result.txt"; then
    echo "  ✓ PASS"
    ((TESTS_PASSED++))
else
    echo "  ✗ FAIL: Exit code=$exit_code, expected 0 and PASS output"
    ((TESTS_FAILED++))
fi

# Test 6: FAIL case with 25x speedup (AC4, AC5)
echo "Test 6: FAIL case with 25x speedup (AC4, AC5)"
cleanup
create_mock_json "$TEST_DIR/pyrust.json" "300.0"
create_mock_json "$TEST_DIR/cpython.json" "7500.0"

output=$(cd "$PROJECT_ROOT" && sed "s|target/criterion/cold_start_simple/base/estimates.json|$TEST_DIR/pyrust.json|g; s|target/criterion/cpython_pure_simple/base/estimates.json|$TEST_DIR/cpython.json|g; s|target/speedup_validation.txt|$TEST_DIR/result.txt|g" "$SCRIPT_UNDER_TEST" | bash 2>&1) || true
exit_code=$?

if [ $exit_code -eq 1 ] && echo "$output" | grep -q "25.00x" && echo "$output" | grep -q "FAIL" && [ -f "$TEST_DIR/result.txt" ] && grep -q "FAIL" "$TEST_DIR/result.txt"; then
    echo "  ✓ PASS"
    ((TESTS_PASSED++))
else
    echo "  ✗ FAIL: Exit code=$exit_code, expected 1 and FAIL output"
    ((TESTS_FAILED++))
fi

# Test 7: Edge case - 100x speedup (should PASS)
echo "Test 7: Edge case - 100x speedup should PASS"
cleanup
create_mock_json "$TEST_DIR/pyrust.json" "300.0"
create_mock_json "$TEST_DIR/cpython.json" "30000.0"

output=$(cd "$PROJECT_ROOT" && sed "s|target/criterion/cold_start_simple/base/estimates.json|$TEST_DIR/pyrust.json|g; s|target/criterion/cpython_pure_simple/base/estimates.json|$TEST_DIR/cpython.json|g; s|target/speedup_validation.txt|$TEST_DIR/result.txt|g" "$SCRIPT_UNDER_TEST" | bash 2>&1) || true
exit_code=$?

if [ $exit_code -eq 0 ] && echo "$output" | grep -q "100.00x"; then
    echo "  ✓ PASS"
    ((TESTS_PASSED++))
else
    echo "  ✗ FAIL"
    ((TESTS_FAILED++))
fi

# Test 8: Edge case - 49.99x speedup (should FAIL)
echo "Test 8: Edge case - 49.99x speedup should FAIL"
cleanup
create_mock_json "$TEST_DIR/pyrust.json" "300.0"
create_mock_json "$TEST_DIR/cpython.json" "14997.0"

sed "s|target/criterion/cold_start_simple/base/estimates.json|$TEST_DIR/pyrust.json|g; s|target/criterion/cpython_pure_simple/base/estimates.json|$TEST_DIR/cpython.json|g; s|target/speedup_validation.txt|$TEST_DIR/result.txt|g" "$SCRIPT_UNDER_TEST" | bash > /dev/null 2>&1 || true
exit_code=$?

if [ $exit_code -eq 1 ]; then
    echo "  ✓ PASS"
    ((TESTS_PASSED++))
else
    echo "  ✗ FAIL: Should have failed with exit code 1, got $exit_code"
    ((TESTS_FAILED++))
fi

# Test 9: Edge case - missing PyRust file
echo "Test 9: Edge case - missing PyRust file should error"
cleanup
create_mock_json "$TEST_DIR/cpython.json" "15000.0"

sed "s|target/criterion/cold_start_simple/base/estimates.json|$TEST_DIR/pyrust.json|g; s|target/criterion/cpython_pure_simple/base/estimates.json|$TEST_DIR/cpython.json|g; s|target/speedup_validation.txt|$TEST_DIR/result.txt|g" "$SCRIPT_UNDER_TEST" | bash > /dev/null 2>&1 || true
exit_code=$?

if [ $exit_code -ne 0 ]; then
    echo "  ✓ PASS"
    ((TESTS_PASSED++))
else
    echo "  ✗ FAIL: Should have failed"
    ((TESTS_FAILED++))
fi

# Test 10: Edge case - missing CPython file
echo "Test 10: Edge case - missing CPython file should error"
cleanup
create_mock_json "$TEST_DIR/pyrust.json" "300.0"

sed "s|target/criterion/cold_start_simple/base/estimates.json|$TEST_DIR/pyrust.json|g; s|target/criterion/cpython_pure_simple/base/estimates.json|$TEST_DIR/cpython.json|g; s|target/speedup_validation.txt|$TEST_DIR/result.txt|g" "$SCRIPT_UNDER_TEST" | bash > /dev/null 2>&1 || true
exit_code=$?

if [ $exit_code -ne 0 ]; then
    echo "  ✓ PASS"
    ((TESTS_PASSED++))
else
    echo "  ✗ FAIL: Should have failed"
    ((TESTS_FAILED++))
fi

# Summary
echo ""
echo "================================="
echo "Test Summary"
echo "================================="
echo "Passed: $TESTS_PASSED"
echo "Failed: $TESTS_FAILED"
echo ""

if [ $TESTS_FAILED -gt 0 ]; then
    echo "OVERALL: FAIL"
    exit 1
else
    echo "OVERALL: PASS"
    exit 0
fi
