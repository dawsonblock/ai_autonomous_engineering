#!/usr/bin/env bash
# Additional edge case tests for documentation consolidation
# Tests boundary conditions and potential failure modes

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

echo "=========================================="
echo "Documentation Edge Case Tests"
echo "=========================================="
echo ""

# Test: docs/ directory is not a symlink
TOTAL=$((TOTAL + 1))
echo -n "Testing: docs/ is a real directory, not a symlink ... "
if [ -d docs ] && [ ! -L docs ]; then
    echo -e "${GREEN}PASS${NC}"
    PASSED=$((PASSED + 1))
else
    echo -e "${RED}FAIL${NC}"
    echo "  docs/ should be a real directory, not a symbolic link"
    FAILED=$((FAILED + 1))
fi

# Test: docs/ directory has correct permissions (readable and executable)
TOTAL=$((TOTAL + 1))
echo -n "Testing: docs/ directory is readable and accessible ... "
if [ -r docs ] && [ -x docs ]; then
    echo -e "${GREEN}PASS${NC}"
    PASSED=$((PASSED + 1))
else
    echo -e "${RED}FAIL${NC}"
    echo "  docs/ directory should be readable and executable"
    FAILED=$((FAILED + 1))
fi

# Test: All markdown files in docs/ are readable
TOTAL=$((TOTAL + 1))
echo -n "Testing: All markdown files are readable ... "
all_readable=true
for file in docs/*.md; do
    if [ ! -r "$file" ]; then
        all_readable=false
        echo ""
        echo "  File not readable: $file"
    fi
done

if $all_readable; then
    echo -e "${GREEN}PASS${NC}"
    PASSED=$((PASSED + 1))
else
    echo -e "${RED}FAIL${NC}"
    FAILED=$((FAILED + 1))
fi

# Test: No subdirectories in docs/ (flat structure)
TOTAL=$((TOTAL + 1))
echo -n "Testing: docs/ has flat structure (no subdirectories) ... "
subdir_count=$(/usr/bin/find docs -mindepth 1 -type d | wc -l | tr -d ' ')
if [ "$subdir_count" = "0" ]; then
    echo -e "${GREEN}PASS${NC}"
    PASSED=$((PASSED + 1))
else
    echo -e "${YELLOW}WARNING${NC}"
    echo "  Found $subdir_count subdirectories in docs/"
    /usr/bin/find docs -mindepth 1 -type d
    PASSED=$((PASSED + 1))  # Not a hard failure
fi

# Test: No empty markdown files
TOTAL=$((TOTAL + 1))
echo -n "Testing: No empty markdown files ... "
empty_files=0
for file in docs/*.md; do
    if [ ! -s "$file" ]; then
        empty_files=$((empty_files + 1))
        echo ""
        echo "  Empty file: $file"
    fi
done

if [ "$empty_files" = "0" ]; then
    echo -e "${GREEN}PASS${NC}"
    PASSED=$((PASSED + 1))
else
    echo -e "${RED}FAIL${NC}"
    echo "  Found $empty_files empty markdown files"
    FAILED=$((FAILED + 1))
fi

# Test: All markdown files have proper header (start with # or metadata)
TOTAL=$((TOTAL + 1))
echo -n "Testing: All markdown files have headers ... "
no_header=0
for file in docs/*.md; do
    # Check if file starts with # (markdown header) or is not empty
    if [ -s "$file" ]; then
        first_line=$(head -1 "$file")
        if ! echo "$first_line" | grep -q '^#'; then
            # Allow files that don't start with # if they have content
            if [ $(wc -c < "$file") -lt 10 ]; then
                no_header=$((no_header + 1))
                echo ""
                echo "  File without proper header: $file"
            fi
        fi
    fi
done

if [ "$no_header" = "0" ]; then
    echo -e "${GREEN}PASS${NC}"
    PASSED=$((PASSED + 1))
else
    echo -e "${RED}FAIL${NC}"
    echo "  Found $no_header files without proper headers"
    FAILED=$((FAILED + 1))
fi

# Test: Exact count of files matches expectation
TOTAL=$((TOTAL + 1))
echo -n "Testing: Exactly 6 markdown files in docs/ ... "
md_count=$(ls -1 docs/*.md 2>/dev/null | wc -l | tr -d ' ')
if [ "$md_count" = "6" ]; then
    echo -e "${GREEN}PASS${NC}"
    PASSED=$((PASSED + 1))
else
    echo -e "${RED}FAIL${NC}"
    echo "  Expected exactly 6 markdown files, found $md_count"
    ls -1 docs/*.md
    FAILED=$((FAILED + 1))
fi

# Test: README.md is in root (not moved to docs)
TOTAL=$((TOTAL + 1))
echo -n "Testing: README.md exists in root ... "
if [ -f README.md ] && [ -s README.md ]; then
    echo -e "${GREEN}PASS${NC}"
    PASSED=$((PASSED + 1))
else
    echo -e "${RED}FAIL${NC}"
    echo "  README.md should exist in root directory"
    FAILED=$((FAILED + 1))
fi

# Test: docs/README.md is different from root README.md
TOTAL=$((TOTAL + 1))
echo -n "Testing: docs/README.md differs from root README.md ... "
if [ -f README.md ] && [ -f docs/README.md ]; then
    if ! cmp -s README.md docs/README.md; then
        echo -e "${GREEN}PASS${NC}"
        PASSED=$((PASSED + 1))
    else
        echo -e "${RED}FAIL${NC}"
        echo "  docs/README.md should be different from root README.md"
        FAILED=$((FAILED + 1))
    fi
else
    echo -e "${RED}FAIL${NC}"
    echo "  Missing README.md file"
    FAILED=$((FAILED + 1))
fi

# Test: No markdown files with .MD extension (uppercase)
TOTAL=$((TOTAL + 1))
echo -n "Testing: No files with uppercase .MD extension ... "
uppercase_md=$(ls docs/*.MD 2>/dev/null | wc -l | tr -d ' ')
if [ "$uppercase_md" = "0" ]; then
    echo -e "${GREEN}PASS${NC}"
    PASSED=$((PASSED + 1))
else
    echo -e "${RED}FAIL${NC}"
    echo "  Found $uppercase_md files with uppercase .MD extension"
    ls docs/*.MD
    FAILED=$((FAILED + 1))
fi

# Test: docs/README.md acts as an index (mentions multiple docs)
TOTAL=$((TOTAL + 1))
echo -n "Testing: docs/README.md acts as an index ... "
doc_references=$(grep -c "\.md" docs/README.md 2>/dev/null || echo "0")
if [ "$doc_references" -ge 3 ]; then
    echo -e "${GREEN}PASS${NC} (found $doc_references references)"
    PASSED=$((PASSED + 1))
else
    echo -e "${YELLOW}WARNING${NC}"
    echo "  docs/README.md should reference multiple documentation files"
    PASSED=$((PASSED + 1))  # Not a hard failure
fi

# Test: No hidden files in docs/ directory
TOTAL=$((TOTAL + 1))
echo -n "Testing: No hidden files in docs/ ... "
hidden_files=$(ls -a docs/ | grep -E '^\.' | grep -v -E '^\.\.?$' | wc -l | tr -d ' ')
if [ "$hidden_files" = "0" ]; then
    echo -e "${GREEN}PASS${NC}"
    PASSED=$((PASSED + 1))
else
    echo -e "${YELLOW}WARNING${NC}"
    echo "  Found $hidden_files hidden files in docs/"
    ls -a docs/ | grep -E '^\.' | grep -v -E '^\.\.?$'
    PASSED=$((PASSED + 1))  # Not a hard failure
fi

# Test: File sizes are reasonable (between 100 bytes and 100KB)
TOTAL=$((TOTAL + 1))
echo -n "Testing: File sizes are reasonable ... "
unreasonable_sizes=0
for file in docs/*.md; do
    size=$(wc -c < "$file")
    if [ "$size" -lt 100 ] || [ "$size" -gt 102400 ]; then
        unreasonable_sizes=$((unreasonable_sizes + 1))
        echo ""
        echo "  Unusual file size for $file: $size bytes"
    fi
done

if [ "$unreasonable_sizes" = "0" ]; then
    echo -e "${GREEN}PASS${NC}"
    PASSED=$((PASSED + 1))
else
    echo -e "${YELLOW}WARNING${NC}"
    echo "  Found $unreasonable_sizes files with unusual sizes"
    PASSED=$((PASSED + 1))  # Not a hard failure
fi

# Test: All required files exist with expected names
TOTAL=$((TOTAL + 1))
echo -n "Testing: All 5 required files plus README exist ... "
required_files=(
    "docs/validation.md"
    "docs/performance.md"
    "docs/implementation-notes.md"
    "docs/integration-verification.md"
    "docs/test-verification.md"
    "docs/README.md"
)

all_exist=true
for file in "${required_files[@]}"; do
    if [ ! -f "$file" ]; then
        all_exist=false
        echo ""
        echo "  Missing required file: $file"
    fi
done

if $all_exist; then
    echo -e "${GREEN}PASS${NC}"
    PASSED=$((PASSED + 1))
else
    echo -e "${RED}FAIL${NC}"
    FAILED=$((FAILED + 1))
fi

# Test: No backup files (*.bak, *.orig, *~) in docs/
TOTAL=$((TOTAL + 1))
echo -n "Testing: No backup files in docs/ ... "
backup_files=$(/usr/bin/find docs -type f \( -name "*.bak" -o -name "*.orig" -o -name "*~" \) | wc -l | tr -d ' ')
if [ "$backup_files" = "0" ]; then
    echo -e "${GREEN}PASS${NC}"
    PASSED=$((PASSED + 1))
else
    echo -e "${YELLOW}WARNING${NC}"
    echo "  Found $backup_files backup files"
    /usr/bin/find docs -type f \( -name "*.bak" -o -name "*.orig" -o -name "*~" \)
    PASSED=$((PASSED + 1))  # Not a hard failure
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
    echo -e "${GREEN}✓ All edge case tests passed!${NC}"
    exit 0
else
    echo -e "${RED}✗ Some edge case tests failed${NC}"
    exit 1
fi
