#!/bin/bash
# Integration test for daemon lifecycle (AC2.1, AC2.2, AC2.3)
# Tests daemon startup, code execution through daemon, and shutdown

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

echo "=== Daemon Lifecycle Test ==="
echo

# Step 1: Ensure no daemon is running
echo -e "${YELLOW}Step 1: Ensuring clean state...${NC}"
cleanup
sleep 0.1

if [ -f "$SOCKET_PATH" ]; then
    echo -e "${RED}FAIL: Socket file still exists after cleanup${NC}"
    exit 1
fi

if [ -f "$PID_FILE" ]; then
    echo -e "${RED}FAIL: PID file still exists after cleanup${NC}"
    exit 1
fi

echo -e "${GREEN}✓ Clean state verified${NC}"
echo

# Step 2: Start daemon (AC2.1)
echo -e "${YELLOW}Step 2: Starting daemon (AC2.1: pyrust --daemon forks background process)...${NC}"
OUTPUT=$("$BINARY" --daemon 2>&1)
EXIT_CODE=$?

if [ $EXIT_CODE -ne 0 ]; then
    echo -e "${RED}FAIL: Daemon startup failed with exit code $EXIT_CODE${NC}"
    echo "Output: $OUTPUT"
    exit 1
fi

# Check that parent process exited
if echo "$OUTPUT" | grep -q "Daemon started with PID"; then
    echo -e "${GREEN}✓ Parent process exited successfully${NC}"
else
    echo -e "${RED}FAIL: Unexpected daemon startup output: $OUTPUT${NC}"
    exit 1
fi

# Give daemon time to initialize
sleep 0.3

# Verify socket file exists
if [ ! -S "$SOCKET_PATH" ]; then
    echo -e "${RED}FAIL: Socket file not created at $SOCKET_PATH${NC}"
    ls -la /tmp/pyrust* || true
    exit 1
fi
echo -e "${GREEN}✓ Socket file created at $SOCKET_PATH${NC}"

# Verify PID file exists
if [ ! -f "$PID_FILE" ]; then
    echo -e "${RED}FAIL: PID file not created at $PID_FILE${NC}"
    exit 1
fi
echo -e "${GREEN}✓ PID file created at $PID_FILE${NC}"

# Verify daemon process is running
DAEMON_PID=$(cat "$PID_FILE")
if ! ps -p "$DAEMON_PID" > /dev/null 2>&1; then
    echo -e "${RED}FAIL: Daemon process (PID $DAEMON_PID) is not running${NC}"
    exit 1
fi
echo -e "${GREEN}✓ Daemon process is running (PID $DAEMON_PID)${NC}"
echo

# Step 3: Execute code through daemon (AC2.2)
echo -e "${YELLOW}Step 3: Executing code through daemon (AC2.2: pyrust -c '2+3' returns correct output)...${NC}"
RESULT=$("$BINARY" -c '2+3' 2>&1)
EXIT_CODE=$?

if [ $EXIT_CODE -ne 0 ]; then
    echo -e "${RED}FAIL: Execution through daemon failed with exit code $EXIT_CODE${NC}"
    echo "Output: $RESULT"
    exit 1
fi

if [ "$RESULT" = "5" ]; then
    echo -e "${GREEN}✓ Correct output received: $RESULT${NC}"
else
    echo -e "${RED}FAIL: Incorrect output. Expected '5', got '$RESULT'${NC}"
    exit 1
fi
echo

# Test more complex expressions
echo -e "${YELLOW}Step 4: Testing additional expressions through daemon...${NC}"

test_expression() {
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

test_expression "10 * 5" "50"
test_expression "100 / 4" "25"

# Test print statement (which includes newline)
# Note: We need to preserve the newline, so write to file instead of command substitution
"$BINARY" -c "print(42)" > /tmp/pyrust_test_output.txt 2>&1
PRINT_RESULT=$(cat /tmp/pyrust_test_output.txt)
PRINT_BYTES=$(od -An -tx1 < /tmp/pyrust_test_output.txt | tr -d ' ')
EXPECTED_BYTES="34320a"  # "42\n" in hex

if [ "$PRINT_BYTES" = "$EXPECTED_BYTES" ]; then
    echo -e "${GREEN}✓ 'print(42)' = '42\\n' (correct)${NC}"
else
    echo -e "${RED}FAIL: 'print(42)' returned unexpected result${NC}"
    echo "Expected bytes: $EXPECTED_BYTES"
    echo "Got bytes: $PRINT_BYTES"
    rm -f /tmp/pyrust_test_output.txt
    exit 1
fi
rm -f /tmp/pyrust_test_output.txt

# Test multiline code
MULTILINE_RESULT=$("$BINARY" -c "x = 10
y = 20
x + y" 2>&1)
if [ "$MULTILINE_RESULT" = "30" ]; then
    echo -e "${GREEN}✓ Multiline code = '30' (correct)${NC}"
else
    echo -e "${RED}FAIL: Multiline code returned '$MULTILINE_RESULT', expected '30'${NC}"
    exit 1
fi

echo

# Step 5: Stop daemon (AC2.3)
echo -e "${YELLOW}Step 5: Stopping daemon (AC2.3: pyrust --stop-daemon shuts down cleanly)...${NC}"
OUTPUT=$("$BINARY" --stop-daemon 2>&1)
EXIT_CODE=$?

if [ $EXIT_CODE -ne 0 ]; then
    echo -e "${RED}FAIL: Daemon shutdown failed with exit code $EXIT_CODE${NC}"
    echo "Output: $OUTPUT"
    exit 1
fi

if echo "$OUTPUT" | grep -q "Daemon stopped successfully"; then
    echo -e "${GREEN}✓ Daemon stop command succeeded${NC}"
else
    echo -e "${RED}FAIL: Unexpected shutdown output: $OUTPUT${NC}"
    exit 1
fi

# Wait for cleanup
sleep 0.2

# Verify socket file removed
if [ -e "$SOCKET_PATH" ]; then
    echo -e "${RED}FAIL: Socket file not removed after shutdown${NC}"
    exit 1
fi
echo -e "${GREEN}✓ Socket file removed${NC}"

# Verify PID file removed
if [ -e "$PID_FILE" ]; then
    echo -e "${RED}FAIL: PID file not removed after shutdown${NC}"
    exit 1
fi
echo -e "${GREEN}✓ PID file removed${NC}"

# Verify daemon process stopped
if ps -p "$DAEMON_PID" > /dev/null 2>&1; then
    echo -e "${RED}FAIL: Daemon process still running after shutdown${NC}"
    kill -9 "$DAEMON_PID" 2>/dev/null || true
    exit 1
fi
echo -e "${GREEN}✓ Daemon process stopped${NC}"
echo

echo -e "${GREEN}=== ALL TESTS PASSED ===${NC}"
echo "✓ AC2.1: Daemon starts and forks to background"
echo "✓ AC2.2: Code execution through daemon returns correct output"
echo "✓ AC2.3: Daemon shuts down cleanly"
