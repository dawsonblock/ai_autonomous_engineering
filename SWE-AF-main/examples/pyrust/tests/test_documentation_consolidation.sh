#!/usr/bin/env bash
# Test script for documentation consolidation (Issue #13)
# Validates AC12 and AC13 from the PRD

set -e

# Color codes for output
GREEN='\033[0;32m'
RED='\033[0;31m'
YELLOW='\033[1;33m'
NC='\033[0m' # No Color

# Test counter
PASSED=0
FAILED=0
TOTAL=0

# Helper function to run test
run_test() {
    local test_name="$1"
    local command="$2"
    local expected="$3"
    local comparison="${4:-eq}" # eq, ge, le, ne

    TOTAL=$((TOTAL + 1))

    echo -n "Testing: $test_name ... "

    # Execute command and capture result
    set +e
    result=$(eval "$command" 2>&1)
    exit_code=$?
    set -e

    # Check result based on comparison type
    success=false
    case "$comparison" in
        "eq")
            [ "$result" = "$expected" ] && success=true
            ;;
        "ge")
            [ "$result" -ge "$expected" ] 2>/dev/null && success=true
            ;;
        "exit_code")
            [ "$exit_code" = "$expected" ] && success=true
            ;;
    esac

    if $success; then
        echo -e "${GREEN}PASS${NC}"
        PASSED=$((PASSED + 1))
    else
        echo -e "${RED}FAIL${NC}"
        echo "  Expected: $expected (comparison: $comparison)"
        echo "  Got: $result (exit code: $exit_code)"
        FAILED=$((FAILED + 1))
    fi
}

# Helper function for exit code tests
run_exit_code_test() {
    local test_name="$1"
    local command="$2"
    local expected_exit_code="$3"

    TOTAL=$((TOTAL + 1))

    echo -n "Testing: $test_name ... "

    set +e
    eval "$command" > /dev/null 2>&1
    exit_code=$?
    set -e

    if [ "$exit_code" = "$expected_exit_code" ]; then
        echo -e "${GREEN}PASS${NC}"
        PASSED=$((PASSED + 1))
    else
        echo -e "${RED}FAIL${NC}"
        echo "  Expected exit code: $expected_exit_code"
        echo "  Got exit code: $exit_code"
        FAILED=$((FAILED + 1))
    fi
}

echo "=========================================="
echo "Documentation Consolidation Tests"
echo "=========================================="
echo ""

# AC1: docs/ directory exists
run_exit_code_test "AC1: docs/ directory exists" "test -d docs" 0

# AC2: Count markdown files in docs/ (should be 6)
# Note: Using /usr/bin/find to avoid fd alias
run_test "AC2: docs/ has 6 markdown files" "/usr/bin/find docs -name '*.md' | wc -l | tr -d ' '" 6 eq

# AC3: No loose markdown in root (except README.md)
run_test "AC3: No loose markdown in root" "ls *.md 2>/dev/null | grep -v 'README.md' | wc -l | tr -d ' '" 0 eq

# AC4: validation.md exists
run_exit_code_test "AC4: docs/validation.md exists" "test -f docs/validation.md" 0

# AC5: performance.md exists
run_exit_code_test "AC5: docs/performance.md exists" "test -f docs/performance.md" 0

# AC6: implementation-notes.md exists
run_exit_code_test "AC6: docs/implementation-notes.md exists" "test -f docs/implementation-notes.md" 0

# AC7: integration-verification.md exists
run_exit_code_test "AC7: docs/integration-verification.md exists" "test -f docs/integration-verification.md" 0

# AC8: test-verification.md exists
run_exit_code_test "AC8: docs/test-verification.md exists" "test -f docs/test-verification.md" 0

# AC9: docs/README.md exists
run_exit_code_test "AC9: docs/README.md exists" "test -f docs/README.md" 0

echo ""
echo "=========================================="
echo "Content Verification Tests"
echo "=========================================="
echo ""

# Additional content verification tests
TOTAL=$((TOTAL + 1))
echo -n "Testing: docs/README.md has content ... "
if [ -s docs/README.md ] && [ $(wc -c < docs/README.md) -ge 100 ]; then
    echo -e "${GREEN}PASS${NC}"
    PASSED=$((PASSED + 1))
else
    echo -e "${RED}FAIL${NC}"
    echo "  docs/README.md is empty or too small"
    FAILED=$((FAILED + 1))
fi

# Verify docs/README.md is an index (should mention the other docs)
TOTAL=$((TOTAL + 1))
echo -n "Testing: docs/README.md references other docs ... "
if grep -q "validation.md" docs/README.md && \
   grep -q "performance.md" docs/README.md && \
   grep -q "implementation-notes.md" docs/README.md; then
    echo -e "${GREEN}PASS${NC}"
    PASSED=$((PASSED + 1))
else
    echo -e "${RED}FAIL${NC}"
    echo "  docs/README.md should reference validation.md, performance.md, and implementation-notes.md"
    FAILED=$((FAILED + 1))
fi

# Verify all moved files have content
TOTAL=$((TOTAL + 1))
echo -n "Testing: All moved files have content ... "
all_have_content=true
for file in docs/validation.md docs/performance.md docs/implementation-notes.md \
            docs/integration-verification.md docs/test-verification.md; do
    if [ ! -s "$file" ] || [ $(wc -c < "$file") -lt 100 ]; then
        all_have_content=false
        echo ""
        echo "  $file is empty or too small"
    fi
done

if $all_have_content; then
    echo -e "${GREEN}PASS${NC}"
    PASSED=$((PASSED + 1))
else
    echo -e "${RED}FAIL${NC}"
    FAILED=$((FAILED + 1))
fi

echo ""
echo "=========================================="
echo "Edge Case Tests"
echo "=========================================="
echo ""

# Test: Verify no old markdown files exist in root (that should have been moved)
TOTAL=$((TOTAL + 1))
echo -n "Testing: Old source files removed from root ... "
old_files_exist=false
for old_file in VALIDATION.md PERFORMANCE.md IMPLEMENTATION_NOTES.md \
                INTEGRATION_VERIFICATION_RESULTS.md TEST_VERIFICATION_EVIDENCE.md; do
    if [ -f "$old_file" ]; then
        old_files_exist=true
        echo ""
        echo "  Found old file: $old_file (should be removed)"
    fi
done

if ! $old_files_exist; then
    echo -e "${GREEN}PASS${NC}"
    PASSED=$((PASSED + 1))
else
    echo -e "${RED}FAIL${NC}"
    FAILED=$((FAILED + 1))
fi

# Test: Verify docs/ only contains markdown files and README
TOTAL=$((TOTAL + 1))
echo -n "Testing: docs/ only contains markdown files ... "
non_md_count=$(/usr/bin/find docs -type f ! -name "*.md" | wc -l | tr -d ' ')
if [ "$non_md_count" = "0" ]; then
    echo -e "${GREEN}PASS${NC}"
    PASSED=$((PASSED + 1))
else
    echo -e "${RED}FAIL${NC}"
    echo "  Found $non_md_count non-markdown files in docs/"
    /usr/bin/find docs -type f ! -name "*.md"
    FAILED=$((FAILED + 1))
fi

# Test: Verify file naming convention (lowercase with hyphens)
TOTAL=$((TOTAL + 1))
echo -n "Testing: Files use lowercase kebab-case naming ... "
bad_names=$(/usr/bin/find docs -name "*.md" | grep -v "^docs/README.md$" | grep -E "[A-Z_]" | wc -l | tr -d ' ')
if [ "$bad_names" = "0" ]; then
    echo -e "${GREEN}PASS${NC}"
    PASSED=$((PASSED + 1))
else
    echo -e "${RED}FAIL${NC}"
    echo "  Found $bad_names files not using lowercase kebab-case"
    /usr/bin/find docs -name "*.md" | grep -E "[A-Z_]"
    FAILED=$((FAILED + 1))
fi

# Test: README.md still exists in root
run_exit_code_test "Edge case: README.md exists in root" "test -f README.md" 0

echo ""
echo "=========================================="
echo "PRD Acceptance Criteria (AC12, AC13)"
echo "=========================================="
echo ""

# AC12: test -d docs && find docs -name "*.md" | wc -l outputs at least 3
TOTAL=$((TOTAL + 1))
echo -n "Testing: AC12 - docs/ with at least 3 markdown files ... "
if [ -d docs ]; then
    md_count=$(/usr/bin/find docs -name "*.md" | wc -l | tr -d ' ')
    if [ "$md_count" -ge 3 ]; then
        echo -e "${GREEN}PASS${NC} (found $md_count files)"
        PASSED=$((PASSED + 1))
    else
        echo -e "${RED}FAIL${NC}"
        echo "  Expected at least 3 markdown files in docs/, found $md_count"
        FAILED=$((FAILED + 1))
    fi
else
    echo -e "${RED}FAIL${NC}"
    echo "  docs/ directory does not exist"
    FAILED=$((FAILED + 1))
fi

# AC13: ls *.md 2>/dev/null | grep -v README.md | wc -l outputs 0
TOTAL=$((TOTAL + 1))
echo -n "Testing: AC13 - No loose markdown in root except README ... "
loose_md=$(ls *.md 2>/dev/null | grep -v 'README.md' | wc -l | tr -d ' ')
if [ "$loose_md" = "0" ]; then
    echo -e "${GREEN}PASS${NC}"
    PASSED=$((PASSED + 1))
else
    echo -e "${RED}FAIL${NC}"
    echo "  Found $loose_md loose markdown files in root:"
    ls *.md 2>/dev/null | grep -v 'README.md'
    FAILED=$((FAILED + 1))
fi

echo ""
echo "=========================================="
echo "Test Summary"
echo "=========================================="
echo "Total tests: $TOTAL"
echo -e "Passed: ${GREEN}$PASSED${NC}"
echo -e "Failed: ${RED}$FAILED${NC}"
echo ""

if [ $FAILED -eq 0 ]; then
    echo -e "${GREEN}✓ All tests passed!${NC}"
    exit 0
else
    echo -e "${RED}✗ Some tests failed${NC}"
    exit 1
fi
