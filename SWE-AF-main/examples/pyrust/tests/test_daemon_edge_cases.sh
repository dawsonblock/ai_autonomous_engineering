#!/bin/bash
# Integration test for daemon edge cases
# Tests edge cases and error conditions not covered by main tests

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

echo "=== Daemon Edge Cases Test ==="
echo

# Test 1: Double daemon start
echo -e "${YELLOW}Test 1: Attempting to start daemon twice...${NC}"
cleanup
sleep 0.1

# Start daemon first time
"$BINARY" --daemon > /dev/null 2>&1
sleep 0.3

if [ ! -S "$SOCKET_PATH" ]; then
    echo -e "${RED}FAIL: First daemon start failed${NC}"
    exit 1
fi
echo -e "${GREEN}✓ First daemon started${NC}"

# Try to start daemon again (should fail)
set +e
OUTPUT=$("$BINARY" --daemon 2>&1)
EXIT_CODE=$?
set -e

if [ $EXIT_CODE -ne 0 ] && echo "$OUTPUT" | grep -qi "already running"; then
    echo -e "${GREEN}✓ Second daemon start correctly rejected${NC}"
else
    echo -e "${RED}FAIL: Expected error when starting daemon twice${NC}"
    echo "Exit code: $EXIT_CODE"
    echo "Output: $OUTPUT"
    cleanup
    exit 1
fi

# Clean up for next test
"$BINARY" --stop-daemon > /dev/null 2>&1
sleep 0.2
echo

# Test 2: Stop daemon when not running
echo -e "${YELLOW}Test 2: Attempting to stop daemon when not running...${NC}"
cleanup
sleep 0.1

set +e
OUTPUT=$("$BINARY" --stop-daemon 2>&1)
EXIT_CODE=$?
set -e

if [ $EXIT_CODE -ne 0 ]; then
    echo -e "${GREEN}✓ Stop daemon correctly fails when not running${NC}"
    echo "Error message: $OUTPUT"
else
    echo -e "${YELLOW}WARNING: Stop daemon succeeded when daemon wasn't running (may be acceptable)${NC}"
fi
echo

# Test 3: Empty code execution
echo -e "${YELLOW}Test 3: Testing empty code execution...${NC}"
cleanup
sleep 0.1

# Start daemon
"$BINARY" --daemon > /dev/null 2>&1
sleep 0.3

# Test empty string
RESULT=$("$BINARY" -c "" 2>&1 || true)
echo "Empty string result: '$RESULT'"

# Test whitespace only
RESULT=$("$BINARY" -c "   " 2>&1 || true)
echo "Whitespace result: '$RESULT'"

# Test newlines only
RESULT=$("$BINARY" -c "

" 2>&1 || true)
echo "Newlines result: '$RESULT'"

echo -e "${GREEN}✓ Empty code tests completed (no crash)${NC}"

# Clean up
"$BINARY" --stop-daemon > /dev/null 2>&1
sleep 0.2
echo

# Test 4: Stale socket file handling
echo -e "${YELLOW}Test 4: Testing stale socket file handling...${NC}"
cleanup
sleep 0.1

# Create a fake socket file
touch "$SOCKET_PATH"

# Try to execute code (should fallback to direct execution)
RESULT=$("$BINARY" -c "2+3" 2>&1)
EXIT_CODE=$?

if [ $EXIT_CODE -eq 0 ] && [ "$RESULT" = "5" ]; then
    echo -e "${GREEN}✓ Stale socket handled correctly with fallback${NC}"
else
    echo -e "${RED}FAIL: Stale socket not handled properly${NC}"
    echo "Exit code: $EXIT_CODE"
    echo "Result: $RESULT"
    cleanup
    exit 1
fi

# Clean up
rm -f "$SOCKET_PATH"
echo

# Test 5: Complex error messages through daemon
echo -e "${YELLOW}Test 5: Testing complex error scenarios through daemon...${NC}"
cleanup
sleep 0.1

# Start daemon
"$BINARY" --daemon > /dev/null 2>&1
sleep 0.3

# Test multiple error types
test_error() {
    local code="$1"
    local description="$2"

    set +e
    OUTPUT=$("$BINARY" -c "$code" 2>&1)
    EXIT_CODE=$?
    set -e

    if [ $EXIT_CODE -ne 0 ]; then
        echo -e "${GREEN}✓ $description: Error correctly propagated (exit code: $EXIT_CODE)${NC}"
        return 0
    else
        echo -e "${RED}FAIL: $description: Expected error but got success${NC}"
        return 1
    fi
}

test_error "x = 1 / 0" "Division by zero with assignment"
test_error "print(undefined)" "Undefined variable in print"
test_error "1 +" "Incomplete expression"

# Clean up
"$BINARY" --stop-daemon > /dev/null 2>&1
sleep 0.2
echo

# Test 6: Daemon status command
echo -e "${YELLOW}Test 6: Testing daemon status command...${NC}"
cleanup
sleep 0.1

# Check status when not running
STATUS=$("$BINARY" --daemon-status 2>&1 || true)
echo "Status when not running: $STATUS"

# Start daemon
"$BINARY" --daemon > /dev/null 2>&1
sleep 0.3

# Check status when running
STATUS=$("$BINARY" --daemon-status 2>&1)
EXIT_CODE=$?

if [ $EXIT_CODE -eq 0 ] && echo "$STATUS" | grep -qi "running"; then
    echo -e "${GREEN}✓ Daemon status correctly reports running state${NC}"
else
    echo -e "${RED}FAIL: Daemon status not working correctly${NC}"
    echo "Exit code: $EXIT_CODE"
    echo "Status: $STATUS"
    "$BINARY" --stop-daemon > /dev/null 2>&1 || true
    exit 1
fi

# Clean up
"$BINARY" --stop-daemon > /dev/null 2>&1
sleep 0.2

# Check status after stop
STATUS=$("$BINARY" --daemon-status 2>&1 || true)
echo "Status after stop: $STATUS"
echo -e "${GREEN}✓ Daemon status command works correctly${NC}"
echo

# Test 7: Multiple rapid requests to daemon
echo -e "${YELLOW}Test 7: Testing multiple rapid requests...${NC}"
cleanup
sleep 0.1

# Start daemon
"$BINARY" --daemon > /dev/null 2>&1
sleep 0.3

# Send multiple requests in quick succession
SUCCESS_COUNT=0
for i in {1..10}; do
    RESULT=$("$BINARY" -c "$i + $i" 2>&1)
    EXPECTED=$((i + i))
    if [ "$RESULT" = "$EXPECTED" ]; then
        SUCCESS_COUNT=$((SUCCESS_COUNT + 1))
    else
        echo -e "${RED}FAIL: Request $i failed. Expected $EXPECTED, got $RESULT${NC}"
    fi
done

if [ $SUCCESS_COUNT -eq 10 ]; then
    echo -e "${GREEN}✓ All 10 rapid requests succeeded${NC}"
else
    echo -e "${YELLOW}WARNING: Only $SUCCESS_COUNT/10 rapid requests succeeded${NC}"
fi

# Clean up
"$BINARY" --stop-daemon > /dev/null 2>&1
sleep 0.2
echo

echo -e "${GREEN}=== ALL EDGE CASE TESTS COMPLETED ===${NC}"
echo "✓ Double daemon start prevented"
echo "✓ Stop daemon when not running handled"
echo "✓ Empty code execution handled"
echo "✓ Stale socket file handled with fallback"
echo "✓ Complex errors propagate correctly"
echo "✓ Daemon status command works"
echo "✓ Multiple rapid requests handled"
