#!/usr/bin/env bash
# Run KodeHold test suite via pytest
# Usage: bash tests/run.sh [pytest-args ...]
set -euo pipefail

echo "=========================================="
echo "  KodeHold Test Suite (pytest)"
echo "  $(date -u '+%Y-%m-%d %H:%M:%S UTC')"
echo "=========================================="
echo ""

cd "$(dirname "$0")/.."

exec python3 -m pytest tests/ -v "$@"
