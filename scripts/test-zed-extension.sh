#!/bin/bash
set -euo pipefail

ROOT="$(cd "$(dirname "$0")/.." && pwd)"
cd "$ROOT"

echo "=== 1. Validating Zed Extension Manifest & Configuration ==="
test -f "$ROOT/editors/zed/extension.toml" || { echo "Missing extension.toml"; exit 1; }
test -f "$ROOT/editors/zed/languages/rocci/config.toml" || { echo "Missing languages/rocci/config.toml"; exit 1; }
test -f "$ROOT/editors/zed/languages/rocdown/config.toml" || { echo "Missing languages/rocdown/config.toml"; exit 1; }
grep -q 'languages = \["Rocci", "Rocdown"\]' "$ROOT/editors/zed/extension.toml" \
  || { echo "extension.toml must attach the language server to Rocci and Rocdown"; exit 1; }
echo "Manifest and language configs present; Rocci and Rocdown are attached."

echo "=== 2. Building rocci-language-server ==="
cargo build -p rocci-rocdown-lsp
test -f "$ROOT/target/debug/rocci-language-server" || { echo "Missing rocci-language-server binary"; exit 1; }
echo "rocci-language-server built: $ROOT/target/debug/rocci-language-server"

echo "=== 3. Building Zed WASM Extension ==="
cd "$ROOT/editors/zed"
cargo build --target wasm32-wasip2 --release
WASM="$ROOT/editors/zed/target/wasm32-wasip2/release/rocci.wasm"
test -f "$WASM" || { echo "WASM build failed; missing $WASM"; exit 1; }
echo "Zed extension compiled successfully: $WASM ($(du -h "$WASM" | cut -f1))"

echo "=== 4. Checking Zed CLI Integration ==="
if command -v zed >/dev/null 2>&1; then
  ZED_VER="$(zed --version 2>&1 || true)"
  echo "Found Zed CLI: $ZED_VER"
else
  echo "Zed CLI not found on PATH (skipping CLI launch test)."
fi

echo "Zed extension verification complete: SUCCESS"
