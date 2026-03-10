#!/usr/bin/env bash
# ─────────────────────────────────────────────────────────────────
# Diagram Tool — Test Runner
# Runs all test DSL files against: validate, compile, preview
# Usage:  ./run_tests.sh
# ─────────────────────────────────────────────────────────────────
set -euo pipefail

SCRIPT_DIR="$(cd "$(dirname "$0")" && pwd)"
PROJECT_DIR="$(dirname "$SCRIPT_DIR")"
BIN="$PROJECT_DIR/target/release/diagrams"
OUT_DIR="$SCRIPT_DIR/output"

# Colors
GREEN='\033[0;32m'
RED='\033[0;31m'
YELLOW='\033[0;33m'
CYAN='\033[0;36m'
BOLD='\033[1m'
NC='\033[0m'

PASS=0
FAIL=0
SKIP=0

log_pass() { echo -e "  ${GREEN}✓ PASS${NC}  $1"; ((PASS++)); }
log_fail() { echo -e "  ${RED}✗ FAIL${NC}  $1"; ((FAIL++)); }
log_skip() { echo -e "  ${YELLOW}⊘ SKIP${NC}  $1"; ((SKIP++)); }
log_section() { echo -e "\n${CYAN}${BOLD}── $1 ──${NC}"; }

# ── Pre-flight ──────────────────────────────────────────────────
echo -e "${BOLD}Diagram Tool Test Suite${NC}"
echo "Binary: $BIN"

if [[ ! -x "$BIN" ]]; then
    echo -e "${RED}ERROR: Binary not found or not executable at $BIN${NC}"
    echo "Run 'cargo build --release' first."
    exit 1
fi

mkdir -p "$OUT_DIR"

# ── Helper: run command, capture exit code ──────────────────────
run_expect_ok() {
    local desc="$1"; shift
    if "$@" > /dev/null 2>&1; then
        log_pass "$desc"
    else
        log_fail "$desc (exit $?)"
    fi
}

run_expect_fail() {
    local desc="$1"
    local expected_code="$2"
    shift 2
    local actual_code=0
    "$@" > /dev/null 2>&1 || actual_code=$?
    if [[ "$actual_code" -ne 0 ]]; then
        if [[ "$expected_code" == "any" ]] || [[ "$actual_code" -eq "$expected_code" ]]; then
            log_pass "$desc (exit $actual_code as expected)"
        else
            log_pass "$desc (exit $actual_code — nonzero, expected $expected_code)"
        fi
    else
        log_fail "$desc (expected failure, got exit 0)"
    fi
}

# ═══════════════════════════════════════════════════════════════
#  TEST GROUP 1: validate (happy path)
# ═══════════════════════════════════════════════════════════════
log_section "validate — valid files"

for f in 01_basic.dsl 02_microservices.dsl 03_data_pipeline.dsl \
         04_comments_only.dsl 07_all_node_types.dsl; do
    run_expect_ok "validate $f" "$BIN" validate "$SCRIPT_DIR/$f"
done

# ═══════════════════════════════════════════════════════════════
#  TEST GROUP 2: validate (error cases)
# ═══════════════════════════════════════════════════════════════
log_section "validate — error cases"

run_expect_fail "validate 05_invalid_syntax.dsl (syntax error)" "any" \
    "$BIN" validate "$SCRIPT_DIR/05_invalid_syntax.dsl"

run_expect_fail "validate 06_undefined_ref.dsl (semantic error)" "any" \
    "$BIN" validate "$SCRIPT_DIR/06_undefined_ref.dsl"

# ═══════════════════════════════════════════════════════════════
#  TEST GROUP 3: compile → SVG
# ═══════════════════════════════════════════════════════════════
log_section "compile — SVG output"

for f in 01_basic.dsl 02_microservices.dsl 03_data_pipeline.dsl \
         04_comments_only.dsl 07_all_node_types.dsl; do
    base="${f%.dsl}"
    svg="$OUT_DIR/${base}.svg"
    run_expect_ok "compile $f → SVG" "$BIN" compile "$SCRIPT_DIR/$f" --output "$svg"

    # Check SVG file exists and contains valid SVG
    if [[ -f "$svg" ]]; then
        if grep -q '<svg' "$svg" 2>/dev/null; then
            log_pass "  $base.svg contains <svg> tag"
        else
            log_fail "  $base.svg missing <svg> tag"
        fi
        size=$(wc -c < "$svg" | tr -d ' ')
        if [[ "$size" -gt 100 ]]; then
            log_pass "  $base.svg size = ${size} bytes"
        else
            log_fail "  $base.svg too small (${size} bytes)"
        fi
    else
        log_fail "  $base.svg not created"
    fi
done

# Compile error cases should also fail
run_expect_fail "compile 05_invalid_syntax.dsl (should fail)" "any" \
    "$BIN" compile "$SCRIPT_DIR/05_invalid_syntax.dsl" --output "$OUT_DIR/should_not_exist.svg"

run_expect_fail "compile 06_undefined_ref.dsl (should fail)" "any" \
    "$BIN" compile "$SCRIPT_DIR/06_undefined_ref.dsl" --output "$OUT_DIR/should_not_exist2.svg"

# ═══════════════════════════════════════════════════════════════
#  TEST GROUP 4: preview (ASCII output)
# ═══════════════════════════════════════════════════════════════
log_section "preview — ASCII output"

for f in 01_basic.dsl 02_microservices.dsl 03_data_pipeline.dsl 07_all_node_types.dsl; do
    base="${f%.dsl}"
    output=$("$BIN" preview "$SCRIPT_DIR/$f" 2>&1) && rc=0 || rc=$?
    if [[ $rc -eq 0 ]] && [[ -n "$output" ]]; then
        log_pass "preview $f (${#output} chars)"
    else
        log_fail "preview $f (exit $rc, output length: ${#output})"
    fi
done

# ═══════════════════════════════════════════════════════════════
#  TEST GROUP 5: node type coverage in SVG
# ═══════════════════════════════════════════════════════════════
log_section "SVG content — node type rendering"

svg_all="$OUT_DIR/07_all_node_types.svg"
if [[ -f "$svg_all" ]]; then
    for label in "Web App" "MySQL" "SQS Queue" "Twilio SMS"; do
        if grep -q "$label" "$svg_all" 2>/dev/null; then
            log_pass "07_all_node_types.svg contains '$label'"
        else
            log_fail "07_all_node_types.svg missing '$label'"
        fi
    done
else
    log_skip "07_all_node_types.svg not found — skipping content checks"
fi

# ═══════════════════════════════════════════════════════════════
#  TEST GROUP 6: edge label coverage
# ═══════════════════════════════════════════════════════════════
log_section "SVG content — edge labels"

svg_basic="$OUT_DIR/01_basic.svg"
if [[ -f "$svg_basic" ]]; then
    if grep -q "sends data" "$svg_basic" 2>/dev/null; then
        log_pass "01_basic.svg contains 'sends data' edge label"
    else
        log_fail "01_basic.svg missing 'sends data' edge label"
    fi
else
    log_skip "01_basic.svg not found"
fi

# ═══════════════════════════════════════════════════════════════
#  SUMMARY
# ═══════════════════════════════════════════════════════════════
TOTAL=$((PASS + FAIL + SKIP))
echo ""
echo -e "${BOLD}═══════════════════════════════════════${NC}"
echo -e "${BOLD}Results: ${GREEN}$PASS passed${NC}, ${RED}$FAIL failed${NC}, ${YELLOW}$SKIP skipped${NC} / $TOTAL total"
echo -e "${BOLD}═══════════════════════════════════════${NC}"

if [[ "$FAIL" -gt 0 ]]; then
    echo -e "\nSVG outputs saved to: $OUT_DIR/"
    exit 1
else
    echo -e "\n${GREEN}All tests passed!${NC} SVG outputs: $OUT_DIR/"
    exit 0
fi
