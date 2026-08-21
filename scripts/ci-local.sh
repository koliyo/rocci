#!/usr/bin/env bash
# Compatibility shim. Prefer: uv run rocci-ops ci
set -euo pipefail
ROOT="$(cd "$(dirname "$0")/.." && pwd)"
cd "$ROOT"
exec uv run rocci-ops ci "$@"
