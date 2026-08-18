#!/usr/bin/env bash
set -euo pipefail

ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
echo "Building Rocci Playground WASM & Web Bundle..."

# 1. Build WASM package
cargo build -p rocci-playground-wasm --target wasm32-unknown-unknown --release

# 2. Build Web Bundle
cd "$ROOT/playground"
if [ ! -d "node_modules" ]; then
    npm install
fi
node build.js

echo "Playground build succeeded."
