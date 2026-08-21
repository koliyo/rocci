#!/bin/sh
set -eu

if ! command -v rsvg-convert >/dev/null 2>&1; then
  echo "rsvg-convert is required (librsvg)." >&2
  exit 1
fi

project_dir=$(CDPATH= cd -- "$(dirname -- "$0")/.." && pwd)
brand="$project_dir/brand"

rsvg-convert -w 1024 -h 1024 "$brand/rocci-app.svg" \
  -o "$project_dir/crates/rocci-desktop/assets/rocci-icon.png"
rsvg-convert -w 1024 -h 1024 "$brand/rocci-file.svg" \
  -o "$brand/rocci-file.png"
rsvg-convert -w 180 -h 180 "$brand/rocci-app.svg" \
  -o "$project_dir/site/assets/apple-touch-icon.png"
cp "$brand/rocci-mark.svg" "$project_dir/site/assets/favicon.svg"

echo "Rendered brand icons from $brand"
