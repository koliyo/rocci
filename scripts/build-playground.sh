#!/usr/bin/env bash
set -euo pipefail

ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
echo "Building Rocci Playground WASM & Web Bundle..."

# Ensure playground/dist exists with placeholders so include_bytes! succeeds on fresh checkouts
mkdir -p "$ROOT/playground/dist"
touch "$ROOT/playground/dist/app.js"
touch "$ROOT/playground/dist/compiler-worker.js"
touch "$ROOT/playground/dist/styles.css"
touch "$ROOT/playground/dist/compiler.wasm"

# 1. Build WASM package
cargo build -p rocci-playground-wasm --target wasm32-unknown-unknown --release

# 2. Build Web Bundle
cd "$ROOT/playground"
if [ ! -d "node_modules" ]; then
    npm install
fi
node build.js

echo "Playground build succeeded."
