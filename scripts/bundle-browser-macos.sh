#!/bin/sh
set -eu

if [ "$(uname -s)" != "Darwin" ]; then
  echo "Rocci Browser.app can only be assembled on macOS." >&2
  exit 1
fi

project_dir=$(CDPATH= cd -- "$(dirname -- "$0")/.." && pwd)
cd "$project_dir"
cargo build --release -p rocci-browser
cargo run --release -q -p rocci-browser -- package
