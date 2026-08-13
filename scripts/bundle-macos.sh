#!/bin/sh
set -eu

if [ "$(uname -s)" != "Darwin" ]; then
  echo "The macOS app bundle can only be built on macOS." >&2
  exit 1
fi

project_dir=$(CDPATH= cd -- "$(dirname -- "$0")/.." && pwd)
bundle_dir="$project_dir/target/release/bundle/macos/Roc Datastar.app"
contents_dir="$bundle_dir/Contents"

cd "$project_dir"
cargo build --release

mkdir -p "$contents_dir/MacOS" "$contents_dir/Resources"
install -m 755 "$project_dir/target/release/roc-datastar" "$contents_dir/MacOS/roc-datastar"
install -m 644 "$project_dir/macos/Info.plist" "$contents_dir/Info.plist"
printf 'APPL????' > "$contents_dir/PkgInfo"

# Ad-hoc signing avoids a damaged-app warning for local builds. Distribution
# builds should replace this with a Developer ID signature and notarization.
codesign --force --deep --sign - "$bundle_dir"

echo "$bundle_dir"

