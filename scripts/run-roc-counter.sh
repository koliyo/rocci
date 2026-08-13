#!/bin/bash
set -euo pipefail

ROOT="$(cd "$(dirname "$0")/.." && pwd)"
cd "$ROOT"

python3 "$ROOT/scripts/compile-rocci-module.py" \
    examples/roc-counter/Counter.rocci \
    -o examples/roc-counter/Counter.roc \
    --type-name Counter

mkdir -p examples/roc-counter/generated examples/roc-counter/assets
cp examples/roc-counter/Counter.roc examples/roc-counter/generated/Counter.roc
cp "$ROOT/assets/datastar.js" "$ROOT/assets/app.css" examples/roc-counter/assets/

cd examples/roc-counter
exec roc main.roc "$@"
