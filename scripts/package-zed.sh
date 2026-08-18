#!/bin/bash
set -euo pipefail

ROOT="$(cd "$(dirname "$0")/.." && pwd)"
cd "$ROOT"

echo "Building Rocci language server release binary..."
cargo build -p rocci-rocdown-lsp --release

echo "Building Zed WASM extension..."
cd "$ROOT/editors/zed"
cargo build --target wasm32-wasip2 --release

cp -f "$ROOT/editors/zed/target/wasm32-wasip2/release/rocci.wasm" "$ROOT/editors/zed/extension.wasm"
echo "Packaged Zed extension to $ROOT/editors/zed/extension.wasm ($(du -h "$ROOT/editors/zed/extension.wasm" | cut -f1))"
