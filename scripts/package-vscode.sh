#!/bin/bash
set -euo pipefail

ROOT="$(cd "$(dirname "$0")/.." && pwd)"
cd "$ROOT"

cargo build -p rocci-lsp --release

DIST="$ROOT/editors/vscode/dist"
rm -rf "$DIST"
mkdir -p "$DIST/bin"

BIN="rocci-language-server"
if [[ "${OS:-}" == "Windows_NT" ]]; then
  BIN="rocci-language-server.exe"
fi
cp -f "$ROOT/target/release/$BIN" "$DIST/bin/"

cd "$ROOT/editors/vscode"
npm install
npm run vscode:package
