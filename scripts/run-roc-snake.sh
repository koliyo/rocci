#!/bin/bash
set -euo pipefail

ROOT="$(cd "$(dirname "$0")/.." && pwd)"
cd "$ROOT"

mkdir -p examples/roc-snake/assets
cp "$ROOT/assets/datastar.js" examples/roc-snake/assets/

exec cargo run -q -p rocci-cli -- run examples/roc-snake/main.roc "$@"
