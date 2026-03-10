#!/bin/bash
# Integration test for daemon fallback (AC2.4)
# Tests that execution works correctly when daemon is not running

set -e

# Colors for output
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
NC='\033[0m' # No Color

BINARY="./target/release/pyrust"
SOCKET_PATH="/tmp/pyrust.sock"
PID_FILE="/tmp/pyrust.pid"

# Cleanup function
cleanup() {
    echo "Cleaning up..."
    # Try to stop daemon if running
    if [ -f "$PID_FILE" ]; then
        PID=$(cat "$PID_FILE" 2>/dev/null || echo "")
        if [ -n "$PID" ]; then
            kill -TERM "$PID" 2>/dev/null || true
            sleep 0.2
        fi
    fi
    # Remove socket and PID file
    rm -f "$SOCKET_PATH" "$PID_FILE"
}

# Trap to ensure cleanup on exit
trap cleanup EXIT

echo "=== Daemon Fallback Test ==="
echo

# Step 1: Ensure daemon is NOT running
echo -e "${YELLOW}Step 1: Ensuring daemon is not running...${NC}"
cleanup
sleep 0.1

if [ -f "$SOCKET_PATH" ]; then
    echo -e "${RED}FAIL: Socket file exists when it shouldn't${NC}"
    exit 1
fi

if [ -f "$PID_FILE" ]; then
    echo -e "${RED}FAIL: PID file exists when it shouldn't${NC}"
    exit 1
fi

echo -e "${GREEN}✓ Daemon is not running${NC}"
echo

# Step 2: Test fallback execution (AC2.4)
echo -e "${YELLOW}Step 2: Testing fallback execution (AC2.4: Fallback works when daemon not running)...${NC}"

test_fallback_expression() {
    local code="$1"
    local expected="$2"
    local result=$("$BINARY" -c "$code" 2>&1)
    local exit_code=$?

    if [ $exit_code -ne 0 ]; then
        echo -e "${RED}FAIL: Code '$code' failed with exit code $exit_code${NC}"
        echo "Output: $result"
        return 1
    fi

    if [ "$result" = "$expected" ]; then
        echo -e "${GREEN}✓ '$code' = '$result' (correct)${NC}"
        return 0
    else
        echo -e "${RED}FAIL: '$code' returned '$result', expected '$expected'${NC}"
        return 1
    fi
}

# Test various expressions without daemon
test_fallback_expression "2+3" "5"
test_fallback_expression "10 * 5" "50"
test_fallback_expression "100 - 25" "75"
test_fallback_expression "50 / 10" "5"

# Test print statement (with newline preserved)
"$BINARY" -c "print(42)" > /tmp/pyrust_fallback_test.txt 2>&1
PRINT_BYTES=$(od -An -tx1 < /tmp/pyrust_fallback_test.txt | tr -d ' ')
EXPECTED_BYTES="34320a"  # "42\n" in hex

if [ "$PRINT_BYTES" = "$EXPECTED_BYTES" ]; then
    echo -e "${GREEN}✓ 'print(42)' = '42\\n' (correct)${NC}"
else
    echo -e "${RED}FAIL: 'print(42)' via fallback returned unexpected result${NC}"
    echo "Expected bytes: $EXPECTED_BYTES"
    echo "Got bytes: $PRINT_BYTES"
    rm -f /tmp/pyrust_fallback_test.txt
    exit 1
fi
rm -f /tmp/pyrust_fallback_test.txt

# Test complex program (write to file to preserve newlines)
COMPLEX_CODE="x = 10
y = 20
z = x + y
print(z)
z"

"$BINARY" -c "$COMPLEX_CODE" > /tmp/pyrust_complex_test.txt 2>&1
EXIT_CODE=$?

if [ $EXIT_CODE -ne 0 ]; then
    echo -e "${RED}FAIL: Complex program failed with exit code $EXIT_CODE${NC}"
    cat /tmp/pyrust_complex_test.txt
    rm -f /tmp/pyrust_complex_test.txt
    exit 1
fi

RESULT_BYTES=$(od -An -tx1 < /tmp/pyrust_complex_test.txt | tr -d ' ')
EXPECTED_BYTES="33300a3330"  # "30\n30" in hex

if [ "$RESULT_BYTES" = "$EXPECTED_BYTES" ]; then
    echo -e "${GREEN}✓ Complex program executed correctly via fallback${NC}"
else
    echo -e "${RED}FAIL: Complex program returned incorrect result${NC}"
    echo "Expected bytes: $EXPECTED_BYTES"
    echo "Got bytes: $RESULT_BYTES"
    cat /tmp/pyrust_complex_test.txt | od -An -tx1
    rm -f /tmp/pyrust_complex_test.txt
    exit 1
fi
rm -f /tmp/pyrust_complex_test.txt

echo

# Step 3: Verify fallback still works after daemon was running
echo -e "${YELLOW}Step 3: Testing fallback after daemon lifecycle...${NC}"

# Start daemon
echo "Starting daemon..."
"$BINARY" --daemon > /dev/null 2>&1
sleep 0.3

# Execute through daemon
RESULT=$("$BINARY" -c "5+5" 2>&1)
if [ "$RESULT" != "10" ]; then
    echo -e "${RED}FAIL: Execution through daemon failed${NC}"
    exit 1
fi
echo -e "${GREEN}✓ Execution through daemon successful${NC}"

# Stop daemon
echo "Stopping daemon..."
"$BINARY" --stop-daemon > /dev/null 2>&1
sleep 0.2

# Verify daemon stopped
if [ -f "$SOCKET_PATH" ]; then
    echo -e "${RED}FAIL: Socket file not removed after daemon stop${NC}"
    exit 1
fi

# Now test fallback again
RESULT=$("$BINARY" -c "7+8" 2>&1)
EXIT_CODE=$?

if [ $EXIT_CODE -ne 0 ]; then
    echo -e "${RED}FAIL: Fallback after daemon stop failed with exit code $EXIT_CODE${NC}"
    echo "Output: $RESULT"
    exit 1
fi

if [ "$RESULT" = "15" ]; then
    echo -e "${GREEN}✓ Fallback works after daemon lifecycle${NC}"
else
    echo -e "${RED}FAIL: Incorrect fallback result after daemon lifecycle${NC}"
    echo "Expected: 15"
    echo "Got: $RESULT"
    exit 1
fi

echo

echo -e "${GREEN}=== ALL TESTS PASSED ===${NC}"
echo "✓ AC2.4: Fallback execution works when daemon is not running"
echo "✓ Fallback works correctly after daemon lifecycle"
echo "✓ Direct execution produces correct results"
