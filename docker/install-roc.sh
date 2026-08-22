#!/usr/bin/env bash
set -euo pipefail

# Pinned to the basic-webserver 0.16.0 platform night (see examples/rocci/standalone/counter).
ROC_NIGHTLY_DATE="${ROC_NIGHTLY_DATE:-2026-08-10}"
ROC_NIGHTLY_SHA="${ROC_NIGHTLY_SHA:-7df8509}"
ROC_NIGHTLY_TAG="${ROC_NIGHTLY_TAG:-nightly-${ROC_NIGHTLY_DATE}-${ROC_NIGHTLY_SHA}}"
TARGETARCH="${TARGETARCH:-amd64}"

case "$TARGETARCH" in
    amd64 | x86_64) roc_arch="x86_64" ;;
    arm64 | aarch64) roc_arch="arm64" ;;
    *)
        echo "unsupported TARGETARCH=$TARGETARCH" >&2
        exit 1
        ;;
esac

archive="roc_nightly-linux_${roc_arch}-${ROC_NIGHTLY_DATE}-${ROC_NIGHTLY_SHA}.tar.gz"
url="https://github.com/roc-lang/nightlies/releases/download/${ROC_NIGHTLY_TAG}/${archive}"
prefix="${ROC_PREFIX:-/opt/roc}"
link_dir="${ROC_LINK_DIR:-/usr/local/bin}"
marker="$prefix/.rocci-roc-${ROC_NIGHTLY_TAG}"

if [ -x "$prefix/roc" ] && [ -f "$marker" ]; then
    mkdir -p "$link_dir"
    ln -sf "$prefix/roc" "$link_dir/roc"
    "$prefix/roc" version
    exit 0
fi

tmp="$(mktemp -d)"
trap 'rm -rf "$tmp"' EXIT
curl -fsSL "$url" -o "$tmp/roc.tar.gz"
mkdir -p "$prefix" "$link_dir"
tar -xzf "$tmp/roc.tar.gz" -C "$tmp"
roc_bin="$(find "$tmp" -type f -name roc | head -n 1)"
if [ -z "$roc_bin" ]; then
    echo "roc binary not found in $archive" >&2
    exit 1
fi
roc_root="$(cd "$(dirname "$roc_bin")" && pwd)"
cp -a "$roc_root/." "$prefix/"
ln -sf "$prefix/roc" "$link_dir/roc"
mkdir -p "$prefix"
touch "$marker"
export PATH="$link_dir:$PATH"
roc version
