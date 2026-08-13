#!/bin/sh
set -eu

if [ "$(uname -s)" != "Darwin" ]; then
  echo "The macOS app bundle can only be built on macOS." >&2
  exit 1
fi

project_dir=$(CDPATH= cd -- "$(dirname -- "$0")/.." && pwd)
cd "$project_dir"
cargo run -p roc-cli -- bundle --config roc.toml
