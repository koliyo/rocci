#!/bin/bash
set -euo pipefail

ROOT="$(cd "$(dirname "$0")/.." && pwd)"
cd "$ROOT"

mkdir -p examples/roc-datastar/assets
cp "$ROOT/assets/datastar.js" examples/roc-datastar/assets/

exec cargo run -q -p rocci-cli -- run examples/roc-datastar/main.roc "$@"
