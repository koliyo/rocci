#!/bin/bash
set -euo pipefail

ROOT="$(cd "$(dirname "$0")/.." && pwd)"
cd "$ROOT"

mkdir -p examples/roc-counter/assets
cp "$ROOT/assets/datastar.js" "$ROOT/assets/app.css" examples/roc-counter/assets/

exec cargo run -q -p rocci-cli -- run examples/roc-counter/main.roc "$@"
