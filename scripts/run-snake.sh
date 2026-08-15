#!/bin/bash
set -euo pipefail

ROOT="$(cd "$(dirname "$0")/.." && pwd)"
cd "$ROOT"

mkdir -p examples/snake/assets
cp "$ROOT/assets/datastar.js" examples/snake/assets/

exec cargo run -q -p rocci-cli -- run examples/snake/main.roc "$@"
