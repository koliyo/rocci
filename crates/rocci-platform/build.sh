#!/bin/bash
set -eo pipefail

# Get rust triple for a target name
get_rust_triple() {
    case "$1" in
        x64mac) echo "x86_64-apple-darwin" ;;
        arm64mac) echo "aarch64-apple-darwin" ;;
        x64musl) echo "x86_64-unknown-linux-musl" ;;
        arm64musl) echo "aarch64-unknown-linux-musl" ;;
        *) echo "Unknown target: $1" >&2; exit 1 ;;
    esac
}

detect_native_target() {
    local arch
    local os
    arch=$(uname -m)
    os=$(uname -s)

    if [ "$os" = "Darwin" ]; then
        if [ "$arch" = "arm64" ]; then
            echo "arm64mac"
        else
            echo "x64mac"
        fi
    elif [ "$os" = "Linux" ]; then
        if [ "$arch" = "aarch64" ]; then
            echo "arm64musl"
        else
            echo "x64musl"
        fi
    else
        echo "Unsupported OS: $os" >&2
        exit 1
    fi
}

CRATE_DIR="$(cd "$(dirname "$0")" && pwd)"
REPO_ROOT="$(cd "$CRATE_DIR/../.." && pwd)"
cd "$REPO_ROOT"
TARGET_DIR="${CARGO_TARGET_DIR:-$REPO_ROOT/target}"

if [ "${1:-}" = "--all" ]; then
    echo "crates/rocci-platform/build.sh --all is not proven yet; build the native triple only." >&2
    exit 1
fi

TARGET=$(detect_native_target)
RUST_TRIPLE=$(get_rust_triple "$TARGET")
echo "Building rocci-platform host for native target: $TARGET"

if [[ "$TARGET" == *"musl"* ]]; then
    cargo build -p rocci-platform --release --lib --target "$RUST_TRIPLE"
    HOST_LIB="$TARGET_DIR/$RUST_TRIPLE/release/libhost.a"
else
    cargo build -p rocci-platform --release --lib
    HOST_LIB="$TARGET_DIR/release/libhost.a"
fi

mkdir -p "$CRATE_DIR/platform/targets/$TARGET"
cp "$HOST_LIB" "$CRATE_DIR/platform/targets/$TARGET/libhost.a"
echo " -> crates/rocci-platform/platform/targets/$TARGET/libhost.a"
