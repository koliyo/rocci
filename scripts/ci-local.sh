#!/usr/bin/env bash
# Compatibility shim. Prefer: uv run --project tools/rocci-ops rocci-ops ci
set -euo pipefail
ROOT="$(cd "$(dirname "$0")/.." && pwd)"
exec uv run --project "${ROOT}/tools/rocci-ops" rocci-ops ci "$@"
