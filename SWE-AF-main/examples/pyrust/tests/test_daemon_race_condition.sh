#!/bin/bash
# Test for daemon startup race condition fix
# Verifies that parent process waits for child to be ready before exiting
# This test validates the pipe-based synchronization mechanism

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

echo "=== Daemon Race Condition Test ==="
echo "Testing that parent waits for child readiness (AC2.1 robustness)"
echo

# Test: Rapid daemon start and immediate execution
# This tests the race condition fix where parent exits before child is ready
echo -e "${YELLOW}Test 1: Rapid start and execute (race condition test)...${NC}"
cleanup
sleep 0.1

# Start daemon and immediately try to execute
"$BINARY" --daemon > /dev/null 2>&1
EXIT_CODE=$?

if [ $EXIT_CODE -ne 0 ]; then
    echo -e "${RED}FAIL: Daemon startup failed${NC}"
    exit 1
fi

# Immediately try to execute (with very short delay)
sleep 0.1

# Try to execute - if race condition exists, socket might not be ready
RESULT=$("$BINARY" -c "42" 2>&1)
EXEC_EXIT_CODE=$?

if [ $EXEC_EXIT_CODE -eq 0 ] && [ "$RESULT" = "42" ]; then
    echo -e "${GREEN}✓ Execution succeeded immediately after daemon start${NC}"
    echo -e "${GREEN}✓ No race condition detected (parent waited for child)${NC}"
else
    echo -e "${RED}FAIL: Execution failed after daemon start${NC}"
    echo "Exit code: $EXEC_EXIT_CODE"
    echo "Result: $RESULT"
    exit 1
fi

cleanup
sleep 0.2
echo

# Test: Multiple rapid start/stop cycles
echo -e "${YELLOW}Test 2: Multiple rapid start/stop cycles...${NC}"

SUCCESS_COUNT=0
for i in {1..5}; do
    cleanup
    sleep 0.05

    # Start daemon
    "$BINARY" --daemon > /dev/null 2>&1
    START_EXIT=$?

    # Very brief wait
    sleep 0.1

    # Execute immediately
    RESULT=$("$BINARY" -c "$i + $i" 2>&1)
    EXEC_EXIT=$?
    EXPECTED=$((i + i))

    # Stop daemon
    "$BINARY" --stop-daemon > /dev/null 2>&1

    if [ $START_EXIT -eq 0 ] && [ $EXEC_EXIT -eq 0 ] && [ "$RESULT" = "$EXPECTED" ]; then
        SUCCESS_COUNT=$((SUCCESS_COUNT + 1))
    else
        echo -e "${RED}FAIL: Cycle $i failed${NC}"
        echo "Start exit: $START_EXIT, Exec exit: $EXEC_EXIT"
        echo "Result: $RESULT, Expected: $EXPECTED"
    fi

    sleep 0.1
done

if [ $SUCCESS_COUNT -eq 5 ]; then
    echo -e "${GREEN}✓ All 5 rapid start/stop cycles succeeded${NC}"
    echo -e "${GREEN}✓ Parent-child synchronization is reliable${NC}"
else
    echo -e "${RED}FAIL: Only $SUCCESS_COUNT/5 cycles succeeded${NC}"
    exit 1
fi

cleanup
sleep 0.2
echo

# Test: Verify socket exists before parent exits
echo -e "${YELLOW}Test 3: Verify socket creation timing...${NC}"
cleanup
sleep 0.1

# Start daemon and capture timing
START_TIME=$(date +%s%N)
"$BINARY" --daemon > /dev/null 2>&1
EXIT_TIME=$(date +%s%N)

# Parent has exited at this point, socket should exist
if [ -S "$SOCKET_PATH" ]; then
    echo -e "${GREEN}✓ Socket file exists immediately after parent exits${NC}"
    echo -e "${GREEN}✓ Child signaled readiness before parent exit${NC}"
else
    echo -e "${RED}FAIL: Socket file doesn't exist after parent exit${NC}"
    echo -e "${RED}Race condition: parent exited before child was ready${NC}"
    exit 1
fi

# Verify daemon is actually running
if [ -f "$PID_FILE" ]; then
    DAEMON_PID=$(cat "$PID_FILE")
    if ps -p "$DAEMON_PID" > /dev/null 2>&1; then
        echo -e "${GREEN}✓ Daemon process is running (PID $DAEMON_PID)${NC}"
    else
        echo -e "${RED}FAIL: PID file exists but process not running${NC}"
        exit 1
    fi
else
    echo -e "${RED}FAIL: PID file not created${NC}"
    exit 1
fi

cleanup
echo

echo -e "${GREEN}=== ALL RACE CONDITION TESTS PASSED ===${NC}"
echo "✓ Parent waits for child to be ready before exiting"
echo "✓ Socket is available immediately after daemon start"
echo "✓ Multiple rapid cycles work reliably"
echo "✓ Pipe-based synchronization prevents race conditions"
