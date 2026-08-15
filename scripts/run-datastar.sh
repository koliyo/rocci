#!/bin/bash
set -euo pipefail

ROOT="$(cd "$(dirname "$0")/.." && pwd)"
cd "$ROOT"

mkdir -p examples/datastar/assets
cp "$ROOT/assets/datastar.js" examples/datastar/assets/

exec cargo run -q -p rocci-cli -- run examples/datastar/main.roc "$@"
