#!/bin/bash
# Integration test for error propagation (AC2.5)
# Tests that error messages are identical between daemon and direct execution

# Don't use set -e because we're intentionally running commands that fail
set +e

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

echo "=== Error Propagation Test ==="
echo

# Step 1: Test errors without daemon (direct execution)
echo -e "${YELLOW}Step 1: Testing errors via direct execution (no daemon)...${NC}"
cleanup
sleep 0.1

# Test division by zero
DIRECT_DIV_BY_ZERO=$("$BINARY" -c "10 / 0" 2>&1 || true)
echo "Direct division by zero error: $DIRECT_DIV_BY_ZERO"

# Test undefined variable
DIRECT_UNDEFINED_VAR=$("$BINARY" -c "undefined_var" 2>&1 || true)
echo "Direct undefined variable error: $DIRECT_UNDEFINED_VAR"

# Test syntax error
DIRECT_SYNTAX_ERROR=$("$BINARY" -c "x = @" 2>&1 || true)
echo "Direct syntax error: $DIRECT_SYNTAX_ERROR"

echo -e "${GREEN}✓ Direct execution errors captured${NC}"
echo

# Step 2: Start daemon
echo -e "${YELLOW}Step 2: Starting daemon...${NC}"
"$BINARY" --daemon > /dev/null 2>&1
sleep 0.3

if [ ! -S "$SOCKET_PATH" ]; then
    echo -e "${RED}FAIL: Daemon failed to start${NC}"
    exit 1
fi
echo -e "${GREEN}✓ Daemon started${NC}"
echo

# Step 3: Test errors through daemon (AC2.5)
echo -e "${YELLOW}Step 3: Testing errors through daemon (AC2.5: Error messages identical)...${NC}"

# Test division by zero through daemon
DAEMON_DIV_BY_ZERO=$("$BINARY" -c "10 / 0" 2>&1 || true)
echo "Daemon division by zero error: $DAEMON_DIV_BY_ZERO"

# Test undefined variable through daemon
DAEMON_UNDEFINED_VAR=$("$BINARY" -c "undefined_var" 2>&1 || true)
echo "Daemon undefined variable error: $DAEMON_UNDEFINED_VAR"

# Test syntax error through daemon
DAEMON_SYNTAX_ERROR=$("$BINARY" -c "x = @" 2>&1 || true)
echo "Daemon syntax error: $DAEMON_SYNTAX_ERROR"

echo

# Step 4: Compare error messages
echo -e "${YELLOW}Step 4: Comparing error messages...${NC}"

compare_errors() {
    local error_type="$1"
    local direct="$2"
    local daemon="$3"

    if [ "$direct" = "$daemon" ]; then
        echo -e "${GREEN}✓ $error_type: Error messages are identical${NC}"
        return 0
    else
        echo -e "${RED}FAIL: $error_type: Error messages differ${NC}"
        echo "Direct:  $direct"
        echo "Daemon:  $daemon"
        return 1
    fi
}

ALL_PASSED=true

compare_errors "Division by zero" "$DIRECT_DIV_BY_ZERO" "$DAEMON_DIV_BY_ZERO" || ALL_PASSED=false
compare_errors "Undefined variable" "$DIRECT_UNDEFINED_VAR" "$DAEMON_UNDEFINED_VAR" || ALL_PASSED=false
compare_errors "Syntax error" "$DIRECT_SYNTAX_ERROR" "$DAEMON_SYNTAX_ERROR" || ALL_PASSED=false

echo

# Step 5: Test error exit codes
echo -e "${YELLOW}Step 5: Testing error exit codes...${NC}"

# Both daemon and direct execution should return exit code 1 for errors
# We already have a running daemon from step 2, so test daemon first

# Test exit code for error (daemon)
"$BINARY" -c "10 / 0" > /dev/null 2>&1
DAEMON_EXIT_CODE=$?

# Stop daemon
"$BINARY" --stop-daemon > /dev/null 2>&1
sleep 0.2

# Ensure cleanup
rm -f "$SOCKET_PATH" "$PID_FILE"

# Test exit code for error (direct)
"$BINARY" -c "10 / 0" > /dev/null 2>&1
DIRECT_EXIT_CODE=$?

if [ $DIRECT_EXIT_CODE -eq $DAEMON_EXIT_CODE ] && [ $DIRECT_EXIT_CODE -ne 0 ]; then
    echo -e "${GREEN}✓ Error exit codes match (both non-zero: $DIRECT_EXIT_CODE)${NC}"
else
    echo -e "${RED}FAIL: Error exit codes differ${NC}"
    echo "Direct exit code: $DIRECT_EXIT_CODE"
    echo "Daemon exit code: $DAEMON_EXIT_CODE"
    ALL_PASSED=false
fi

echo

# Step 6: Clean up
echo -e "${YELLOW}Step 6: Cleaning up...${NC}"
"$BINARY" --stop-daemon > /dev/null 2>&1 || true
sleep 0.2
echo -e "${GREEN}✓ Cleanup complete${NC}"
echo

if [ "$ALL_PASSED" = true ]; then
    echo -e "${GREEN}=== ALL TESTS PASSED ===${NC}"
    echo "✓ AC2.5: Error messages are identical for daemon vs direct execution"
    echo "✓ Division by zero errors match"
    echo "✓ Undefined variable errors match"
    echo "✓ Syntax errors match"
    echo "✓ Exit codes match for errors"
    exit 0
else
    echo -e "${RED}=== SOME TESTS FAILED ===${NC}"
    exit 1
fi
