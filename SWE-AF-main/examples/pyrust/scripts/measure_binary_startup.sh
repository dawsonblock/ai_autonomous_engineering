#!/usr/bin/env bash
# Script to measure binary startup overhead using hyperfine
# Tests AC1.3: Binary startup overhead ≤500μs mean measured via hyperfine 100 runs

set -e

# Ensure binary is built
if [ ! -f "target/release/pyrust" ]; then
    echo "Error: Release binary not found. Run 'cargo build --release' first."
    exit 1
fi

echo "Measuring binary startup overhead with hyperfine..."
echo "Running 100 warmup runs followed by 100 measurement runs"
echo ""

# Run hyperfine with 100 runs to measure startup overhead
# We use a minimal command to isolate startup time
# Note: This measures the total time including shell spawn, process execution, and result return
hyperfine \
    --warmup 100 \
    --runs 100 \
    --export-json /tmp/pyrust_startup_results.json \
    './target/release/pyrust -c "2+3"'

# Parse results and check if mean is ≤500μs
if command -v python3 &> /dev/null; then
    python3 -c '
import json
import sys

with open("/tmp/pyrust_startup_results.json", "r") as f:
    data = json.load(f)

results = data["results"][0]
mean_seconds = results["mean"]
mean_us = mean_seconds * 1_000_000
stddev_seconds = results["stddev"]
stddev_us = stddev_seconds * 1_000_000

# Calculate 95% confidence interval (assuming normal distribution)
# 95% CI ≈ mean ± 1.96 * stddev
ci_lower = mean_us - 1.96 * stddev_us
ci_upper = mean_us + 1.96 * stddev_us

print(f"\n=== Binary Startup Performance ===")
print(f"Mean:   {mean_us:.2f} μs")
print(f"StdDev: {stddev_us:.2f} μs")
print(f"95% CI: [{ci_lower:.2f}, {ci_upper:.2f}] μs")
print(f"")

# Check acceptance criteria
target_us = 500.0
if mean_us <= target_us:
    print(f"✓ PASS: Mean startup time ({mean_us:.2f} μs) ≤ {target_us} μs")
    sys.exit(0)
else:
    print(f"✗ FAIL: Mean startup time ({mean_us:.2f} μs) > {target_us} μs")
    print(f"  Exceeds target by {mean_us - target_us:.2f} μs")
    sys.exit(1)
'
else
    echo ""
    echo "Note: Install python3 to automatically check acceptance criteria"
    echo "Manually verify that mean time ≤ 500μs from hyperfine output above"
fi
