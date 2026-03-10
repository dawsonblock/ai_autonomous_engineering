#!/usr/bin/env bash
# Build from clean state and run all tests — run this in a REAL terminal
set -euo pipefail

PROJECT="/Users/santoshkumarradha/Documents/agentfield/code/int-agentfield-examples/af-swe/example-diagrams"
RESULTS="/tmp/diagram_test_results.txt"

echo "=== Building diagrams (debug mode) ===" | tee "$RESULTS"
cargo build --manifest-path "$PROJECT/Cargo.toml" 2>&1 | tee -a "$RESULTS"
echo "" | tee -a "$RESULTS"

BIN="$PROJECT/target/debug/diagrams"
TESTS="$PROJECT/test_examples"
OUT="$TESTS/output"
mkdir -p "$OUT"

echo "=== Quick smoke test ===" | tee -a "$RESULTS"
"$BIN" --help 2>&1 | head -5 | tee -a "$RESULTS"
echo "" | tee -a "$RESULTS"

# Run the full test suite
echo "=== Running test suite ===" | tee -a "$RESULTS"

PASS=0; FAIL=0

run_ok() {
    local desc="$1"; shift
    if "$@" > /dev/null 2>&1; then
        echo "  PASS  $desc" | tee -a "$RESULTS"; ((PASS++))
    else
        echo "  FAIL  $desc (exit $?)" | tee -a "$RESULTS"; ((FAIL++))
    fi
}

run_fail() {
    local desc="$1"; shift
    if "$@" > /dev/null 2>&1; then
        echo "  FAIL  $desc (expected error, got 0)" | tee -a "$RESULTS"; ((FAIL++))
    else
        echo "  PASS  $desc (exit $? as expected)" | tee -a "$RESULTS"; ((PASS++))
    fi
}

echo "-- validate (valid files) --" | tee -a "$RESULTS"
for f in 01_basic.dsl 02_microservices.dsl 03_data_pipeline.dsl 04_comments_only.dsl 07_all_node_types.dsl; do
    run_ok "validate $f" "$BIN" validate "$TESTS/$f"
done

echo "-- validate (error cases) --" | tee -a "$RESULTS"
run_fail "validate 05_invalid_syntax.dsl" "$BIN" validate "$TESTS/05_invalid_syntax.dsl"
run_fail "validate 06_undefined_ref.dsl" "$BIN" validate "$TESTS/06_undefined_ref.dsl"

echo "-- compile → SVG --" | tee -a "$RESULTS"
for f in 01_basic.dsl 02_microservices.dsl 03_data_pipeline.dsl 04_comments_only.dsl 07_all_node_types.dsl; do
    base="${f%.dsl}"
    run_ok "compile $f" "$BIN" compile "$TESTS/$f" --output "$OUT/${base}.svg"
done

echo "-- compile (error cases) --" | tee -a "$RESULTS"
run_fail "compile 05_invalid_syntax.dsl" "$BIN" compile "$TESTS/05_invalid_syntax.dsl" --output "$OUT/bad.svg"
run_fail "compile 06_undefined_ref.dsl" "$BIN" compile "$TESTS/06_undefined_ref.dsl" --output "$OUT/bad2.svg"

echo "-- preview --" | tee -a "$RESULTS"
for f in 01_basic.dsl 02_microservices.dsl 03_data_pipeline.dsl 07_all_node_types.dsl; do
    run_ok "preview $f" "$BIN" preview "$TESTS/$f"
done

echo "-- SVG content checks --" | tee -a "$RESULTS"
if grep -q '<svg' "$OUT/01_basic.svg" 2>/dev/null; then
    echo "  PASS  01_basic.svg has <svg> tag" | tee -a "$RESULTS"; ((PASS++))
else
    echo "  FAIL  01_basic.svg missing <svg> tag" | tee -a "$RESULTS"; ((FAIL++))
fi
if grep -q 'Web App' "$OUT/07_all_node_types.svg" 2>/dev/null; then
    echo "  PASS  07_all_node_types.svg has node labels" | tee -a "$RESULTS"; ((PASS++))
else
    echo "  FAIL  07_all_node_types.svg missing node labels" | tee -a "$RESULTS"; ((FAIL++))
fi

# Convert SVGs to PNGs using sips
echo "" | tee -a "$RESULTS"
echo "=== Converting SVGs to PNGs ===" | tee -a "$RESULTS"
for svg in "$OUT"/*.svg; do
    [ -f "$svg" ] || continue
    png="${svg%.svg}.png"
    if sips -s format png "$svg" --out "$png" > /dev/null 2>&1; then
        echo "  OK  $(basename "$png")" | tee -a "$RESULTS"
    else
        echo "  ERR $(basename "$svg") → PNG failed" | tee -a "$RESULTS"
    fi
done

TOTAL=$((PASS + FAIL))
echo "" | tee -a "$RESULTS"
echo "===============================" | tee -a "$RESULTS"
echo "Results: $PASS passed, $FAIL failed / $TOTAL total" | tee -a "$RESULTS"
echo "SVGs: $OUT/*.svg" | tee -a "$RESULTS"
echo "PNGs: $OUT/*.png" | tee -a "$RESULTS"
echo "===============================" | tee -a "$RESULTS"

# Show a preview
echo "" | tee -a "$RESULTS"
echo "=== Preview of 01_basic.dsl ===" | tee -a "$RESULTS"
"$BIN" preview "$TESTS/01_basic.dsl" 2>&1 | tee -a "$RESULTS"

echo "" | tee -a "$RESULTS"
echo "=== SVG file sizes ===" | tee -a "$RESULTS"
ls -lh "$OUT"/*.svg "$OUT"/*.png 2>/dev/null | tee -a "$RESULTS"

echo "" | tee -a "$RESULTS"
echo "Done! Results saved to $RESULTS" | tee -a "$RESULTS"
