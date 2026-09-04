#!/usr/bin/env bash
set -euo pipefail

CRATE_DIR="$(cd "$(dirname "$0")" && pwd)"
if [[ "${1:-}" != "--skip-build" ]]; then
    "$CRATE_DIR/build.sh"
fi

cd "$CRATE_DIR/platform"
shopt -s nullglob
roc_files=(*.roc)
lib_files=()
for lib in targets/*/*.a targets/*/*.o; do
    if [[ -f "$lib" ]]; then
        lib_files+=("$lib")
    fi
done
shopt -u nullglob

if [[ ${#roc_files[@]} -eq 0 ]]; then
    echo "no platform/*.roc files to bundle" >&2
    exit 1
fi
if [[ ${#lib_files[@]} -eq 0 ]]; then
    echo "no native libhost under platform/targets/*/ to bundle" >&2
    exit 1
fi

echo "Bundling ${#roc_files[@]} .roc files and ${#lib_files[@]} library files..."
roc bundle "${roc_files[@]}" "${lib_files[@]}" --output-dir "$CRATE_DIR"
echo " -> crates/rocci-platform/*.tar.zst"
