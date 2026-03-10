#!/bin/bash
# Validate daemon mode speedup meets M2 acceptance criteria (≤190μs mean)
#
# This script:
# 1. Builds the release binary
# 2. Starts the daemon server
# 3. Warms up daemon cache
# 4. Measures 1000 daemon requests via Unix socket (pure daemon latency)
# 5. Validates mean ≤190μs with CV < 10%
# 6. Stops the daemon
# 7. Exits 0 if pass, non-zero if fail
#
# Note: Uses direct Unix socket communication to measure pure daemon server
# latency without CLI process spawn overhead (matches Criterion benchmark)

set -e  # Exit on error

# Colors for output
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
NC='\033[0m' # No Color

# Configuration
SOCKET_PATH="/tmp/pyrust.sock"
PID_FILE_PATH="/tmp/pyrust.pid"
BINARY_PATH="target/release/pyrust"
TARGET_MEAN_US=190
NUM_RUNS=5000
WARMUP_RUNS=1000

echo "=== Daemon Mode Speedup Validation ==="
echo "Target: ≤${TARGET_MEAN_US}μs mean latency (M2, AC6.2)"
echo "Method: Unix socket client with ${NUM_RUNS} runs after ${WARMUP_RUNS} warmup"
echo ""

# Step 1: Build release binary
echo "Building release binary..."
cargo build --release
if [ $? -ne 0 ]; then
    echo -e "${RED}ERROR: Failed to build release binary${NC}"
    exit 1
fi

if [ ! -f "$BINARY_PATH" ]; then
    echo -e "${RED}ERROR: Binary not found at $BINARY_PATH${NC}"
    exit 1
fi

echo -e "${GREEN}✓ Binary built successfully${NC}"
echo ""

# Step 2: Clean up any existing daemon
echo "Cleaning up any existing daemon..."
if [ -f "$SOCKET_PATH" ]; then
    $BINARY_PATH --stop-daemon 2>/dev/null || true
    sleep 0.2
fi
rm -f "$SOCKET_PATH" "$PID_FILE_PATH"

# Step 3: Start daemon
echo "Starting daemon..."
$BINARY_PATH --daemon &
DAEMON_PID=$!

# Wait for daemon to start (socket file appears)
MAX_WAIT=10
WAITED=0
while [ ! -S "$SOCKET_PATH" ]; do
    if [ $WAITED -ge $MAX_WAIT ]; then
        echo -e "${RED}ERROR: Daemon failed to start within ${MAX_WAIT}s${NC}"
        kill $DAEMON_PID 2>/dev/null || true
        exit 1
    fi
    sleep 0.1
    WAITED=$((WAITED + 1))
done

echo -e "${GREEN}✓ Daemon started successfully${NC}"
echo ""

# Step 4: Warm up daemon (using socket directly to avoid CLI spawn overhead)
echo "Warming up daemon with ${WARMUP_RUNS} requests..."
# Create a simple Python script to send socket requests for warmup
# IMPORTANT: Reuse socket connection to eliminate handshake overhead
python3 << PYTHON_EOF > /dev/null 2>&1
import socket
import struct
SOCKET_PATH = "/tmp/pyrust.sock"
WARMUP_RUNS = ${WARMUP_RUNS}

def send_request(sock, code):
    code_bytes = code.encode('utf-8')
    length = len(code_bytes)
    request = struct.pack('>I', length) + code_bytes
    sock.sendall(request)
    header = sock.recv(5)
    if len(header) < 5:
        return None
    status, output_len = struct.unpack('>BI', header)
    output_bytes = sock.recv(output_len)
    return output_bytes.decode('utf-8') if status == 0 else None

# Connect once and reuse the connection for all warmup requests
sock = socket.socket(socket.AF_UNIX, socket.SOCK_STREAM)
sock.connect(SOCKET_PATH)
for i in range(WARMUP_RUNS):
    result = send_request(sock, "2+3")
    if result is None:
        raise Exception(f"Warmup request {i} failed")
sock.close()
PYTHON_EOF
echo -e "${GREEN}✓ Warmup complete${NC}"
echo ""

# Step 5: Measure daemon socket latency using Python client
# This measures pure daemon server latency via Unix socket communication
# without process spawn overhead (matches Criterion benchmark methodology)
echo "Measuring ${NUM_RUNS} daemon requests via Unix socket..."
STATS_OUTPUT=$(python3 2>&1 << PYTHON_EOF
import socket
import struct
import time
import statistics

SOCKET_PATH = "/tmp/pyrust.sock"
NUM_RUNS = ${NUM_RUNS}

def send_request(sock, code):
    code_bytes = code.encode('utf-8')
    length = len(code_bytes)
    request = struct.pack('>I', length) + code_bytes
    sock.sendall(request)
    header = sock.recv(5)
    if len(header) < 5:
        return None
    status, output_len = struct.unpack('>BI', header)
    output_bytes = sock.recv(output_len)
    return output_bytes.decode('utf-8') if status == 0 else None

# Connect once and reuse the connection for all measurement requests
# This eliminates socket handshake overhead and measures pure daemon latency
sock = socket.socket(socket.AF_UNIX, socket.SOCK_STREAM)
sock.connect(SOCKET_PATH)

# Measure individual request latencies
latencies = []
for i in range(NUM_RUNS):
    start_time = time.time()
    result = send_request(sock, "2+3")
    end_time = time.time()
    if result is None:
        raise Exception(f"Measurement request {i} failed")
    latency_us = (end_time - start_time) * 1_000_000
    latencies.append(latency_us)

sock.close()

# Calculate statistics
mean_us = int(statistics.mean(latencies))
stddev_us = int(statistics.stdev(latencies))
cv_percent = (stddev_us / mean_us) * 100

print(f"MEAN:{mean_us}")
print(f"STDDEV:{stddev_us}")
print(f"CV:{cv_percent:.2f}")
PYTHON_EOF
)

if [ $? -ne 0 ]; then
    echo -e "${RED}ERROR: Socket measurement failed${NC}"
    $BINARY_PATH --stop-daemon
    exit 1
fi

# Parse output
MEAN_US=$(echo "$STATS_OUTPUT" | grep "MEAN:" | cut -d: -f2)
STDDEV_US=$(echo "$STATS_OUTPUT" | grep "STDDEV:" | cut -d: -f2)
CV_PERCENT=$(echo "$STATS_OUTPUT" | grep "CV:" | cut -d: -f2)

echo "Mean latency: ${MEAN_US}μs"
echo "Std deviation: ${STDDEV_US}μs"
echo "Coefficient of variation: ${CV_PERCENT}%"
echo ""

# Step 6: Stop daemon
echo "Stopping daemon..."
$BINARY_PATH --stop-daemon
sleep 0.2

# Verify cleanup
if [ -S "$SOCKET_PATH" ]; then
    echo -e "${YELLOW}WARNING: Socket file still exists, forcing cleanup${NC}"
    rm -f "$SOCKET_PATH"
fi
if [ -f "$PID_FILE_PATH" ]; then
    rm -f "$PID_FILE_PATH"
fi

echo -e "${GREEN}✓ Daemon stopped${NC}"
echo ""

# Step 7: Validate against target
echo "=== VALIDATION RESULTS ==="
echo "Mean latency: ${MEAN_US}μs"
echo "Target:       ≤${TARGET_MEAN_US}μs"
echo "CV:           ${CV_PERCENT}%"
echo "CV Target:    <10%"
echo ""

# Check if CV is acceptable (< 10%)
CV_OK=0
if [ -n "$CV_PERCENT" ]; then
    CV_CHECK=$(python3 -c "print('1' if float('$CV_PERCENT') < 10.0 else '0')")
    if [ "$CV_CHECK" = "1" ]; then
        CV_OK=1
    fi
fi

if [ $MEAN_US -le $TARGET_MEAN_US ] && [ $CV_OK -eq 1 ]; then
    SPEEDUP=$(python3 -c "print(f'{19000 / $MEAN_US:.1f}x')")
    echo -e "${GREEN}✓ PASS: Daemon mode achieves ≤${TARGET_MEAN_US}μs mean latency${NC}"
    echo -e "${GREEN}  Speedup vs CPython baseline (19ms): ${SPEEDUP}${NC}"
    echo -e "${GREEN}  Coefficient of variation: ${CV_PERCENT}% (< 10% required) ✓${NC}"
    echo -e "${GREEN}  M2 acceptance criteria satisfied ✓${NC}"
    echo -e "${GREEN}  AC6.2 acceptance criteria satisfied ✓${NC}"
    exit 0
elif [ $MEAN_US -gt $TARGET_MEAN_US ]; then
    DEFICIT=$((MEAN_US - TARGET_MEAN_US))
    echo -e "${RED}✗ FAIL: Mean latency ${MEAN_US}μs exceeds target ${TARGET_MEAN_US}μs${NC}"
    echo -e "${RED}  Deficit: ${DEFICIT}μs${NC}"
    echo -e "${RED}  M2 acceptance criteria NOT satisfied${NC}"
    echo -e "${RED}  AC6.2 acceptance criteria NOT satisfied${NC}"
    exit 1
else
    echo -e "${RED}✗ FAIL: Coefficient of variation ${CV_PERCENT}% exceeds 10% threshold${NC}"
    echo -e "${RED}  Statistical stability requirement NOT satisfied${NC}"
    echo -e "${RED}  M2 acceptance criteria NOT satisfied${NC}"
    exit 1
fi
