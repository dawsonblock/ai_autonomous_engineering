#!/usr/bin/env bash
# Test script for broken internal documentation links
# Validates that all references to moved documentation files have been updated

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
echo "Internal Documentation Link Tests"
echo "=========================================="
echo ""

# Test: No references to old uppercase markdown file names
TOTAL=$((TOTAL + 1))
echo -n "Testing: No references to old file names in README.md ... "

# Search for references to old file names (case-insensitive for paths, but we want exact matches)
old_refs_count=$(grep -c -E "(IMPLEMENTATION_NOTES\.md|PERFORMANCE\.md|VALIDATION\.md|INTEGRATION_VERIFICATION_RESULTS\.md|TEST_VERIFICATION_EVIDENCE\.md)" README.md 2>/dev/null || true)
# grep -c returns 0 when no matches found, but exits with 1
if [ -z "$old_refs_count" ]; then
    old_refs_count="0"
fi

if [ "$old_refs_count" = "0" ]; then
    echo -e "${GREEN}PASS${NC}"
    PASSED=$((PASSED + 1))
else
    echo -e "${RED}FAIL${NC}"
    echo "  Found $old_refs_count references to old file names in README.md:"
    grep -n -E "(IMPLEMENTATION_NOTES\.md|PERFORMANCE\.md|VALIDATION\.md|INTEGRATION_VERIFICATION_RESULTS\.md|TEST_VERIFICATION_EVIDENCE\.md)" README.md
    FAILED=$((FAILED + 1))
fi

# Test: README.md should reference new docs/ paths
TOTAL=$((TOTAL + 1))
echo -n "Testing: README.md references docs/ directory ... "

if grep -q "docs/" README.md; then
    echo -e "${GREEN}PASS${NC}"
    PASSED=$((PASSED + 1))
else
    echo -e "${YELLOW}WARNING${NC}"
    echo "  README.md should ideally reference the docs/ directory"
    PASSED=$((PASSED + 1))  # Not a hard failure
fi

# Test: Check all markdown files for broken links to old names
TOTAL=$((TOTAL + 1))
echo -n "Testing: No broken links in docs/ markdown files ... "

broken_links=0
for file in docs/*.md; do
    # Check for references to old uppercase names that should now be lowercase
    if grep -q -E "(IMPLEMENTATION_NOTES\.md|PERFORMANCE\.md|VALIDATION\.md|INTEGRATION_VERIFICATION_RESULTS\.md|TEST_VERIFICATION_EVIDENCE\.md)" "$file" 2>/dev/null; then
        broken_links=$((broken_links + 1))
        echo ""
        echo "  Found broken link in $file:"
        grep -n -E "(IMPLEMENTATION_NOTES\.md|PERFORMANCE\.md|VALIDATION\.md|INTEGRATION_VERIFICATION_RESULTS\.md|TEST_VERIFICATION_EVIDENCE\.md)" "$file"
    fi
done

if [ "$broken_links" = "0" ]; then
    echo -e "${GREEN}PASS${NC}"
    PASSED=$((PASSED + 1))
else
    echo -e "${RED}FAIL${NC}"
    echo "  Found broken links in $broken_links files"
    FAILED=$((FAILED + 1))
fi

# Test: Verify relative links in docs/README.md work
TOTAL=$((TOTAL + 1))
echo -n "Testing: All links in docs/README.md point to existing files ... "

all_links_valid=true
while IFS= read -r line; do
    # Extract markdown links [text](path) - simpler regex
    if echo "$line" | grep -q '\[.*\](.*\.md)'; then
        # Extract the link path using sed
        link_path=$(echo "$line" | sed -n 's/.*\[\([^]]*\)\](\([^)]*\.md\)).*/\2/p' | head -1)

        # Skip if no link extracted
        if [ -z "$link_path" ]; then
            continue
        fi

        # Skip external links (http/https)
        if echo "$link_path" | grep -q '^https\?://'; then
            continue
        fi

        # Skip anchors
        if echo "$link_path" | grep -q '^#'; then
            continue
        fi

        # Resolve relative path from docs/ directory
        if echo "$link_path" | grep -q '^\.\.\/'; then
            # Link goes up to root
            resolved_path=$(echo "$link_path" | sed 's|^\.\./||')
        else
            # Link is relative to docs/
            resolved_path="docs/$link_path"
        fi

        # Check if file exists
        if [ ! -f "$resolved_path" ]; then
            all_links_valid=false
            echo ""
            echo "  Broken link in docs/README.md: $link_path (resolved to: $resolved_path)"
        fi
    fi
done < docs/README.md

if $all_links_valid; then
    echo -e "${GREEN}PASS${NC}"
    PASSED=$((PASSED + 1))
else
    echo -e "${RED}FAIL${NC}"
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
    echo -e "${GREEN}✓ All link tests passed!${NC}"
    exit 0
else
    echo -e "${RED}✗ Some link tests failed${NC}"
    exit 1
fi
