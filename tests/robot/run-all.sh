#!/bin/bash
set -euo pipefail

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
cd "$SCRIPT_DIR"

RESULTS_DIR="${RESULTS_DIR:-results}"
mkdir -p "$RESULTS_DIR"

echo "=== Proof-of-Contact Network: Full Test Suite ==="
echo "Results: $RESULTS_DIR"
echo ""

robot \
    --variablefile variables.py \
    --pythonpath libraries/ \
    --outputdir "$RESULTS_DIR" \
    --timestampoutputs \
    --loglevel INFO \
    --exclude security \
    test-suites/

echo ""
echo "=== All tests completed ==="
echo "Report: $RESULTS_DIR/report.html"
