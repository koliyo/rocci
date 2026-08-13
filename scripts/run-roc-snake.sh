#!/bin/bash
set -euo pipefail

ROOT="$(cd "$(dirname "$0")/.." && pwd)"
cd "$ROOT"

python3 "$ROOT/scripts/compile-rocci-module.py" \
    examples/roc-snake/Snake.rocci \
    -o examples/roc-snake/Snake.roc \
    --type-name Snake

mkdir -p examples/roc-snake/generated examples/roc-snake/assets
cp examples/roc-snake/Snake.roc examples/roc-snake/generated/Snake.roc
cp "$ROOT/assets/datastar.js" examples/roc-snake/assets/

cd examples/roc-snake
exec roc main.roc "$@"
