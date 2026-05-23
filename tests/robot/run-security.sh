#!/bin/bash
set -euo pipefail

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
cd "$SCRIPT_DIR"

RESULTS_DIR="${RESULTS_DIR:-results-security}"
mkdir -p "$RESULTS_DIR"

echo "=== Proof-of-Contact Network: Security Tests ==="
echo "Results: $RESULTS_DIR"
echo ""

robot \
    --variablefile variables.py \
    --pythonpath libraries/ \
    --outputdir "$RESULTS_DIR" \
    --timestampoutputs \
    --loglevel DEBUG \
    --include security \
    test-suites/99-security/

echo ""
echo "=== Security tests completed ==="
echo "Report: $RESULTS_DIR/report.html"
