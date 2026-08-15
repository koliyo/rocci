#!/bin/bash
set -euo pipefail

ROOT="$(cd "$(dirname "$0")/.." && pwd)"
cd "$ROOT"

mkdir -p examples/counter/assets
cp "$ROOT/assets/datastar.js" "$ROOT/assets/app.css" examples/counter/assets/

exec cargo run -q -p rocci-cli -- run examples/counter/main.roc "$@"
