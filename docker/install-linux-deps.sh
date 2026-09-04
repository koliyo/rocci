#!/usr/bin/env bash
set -euo pipefail

mode="${1:-runtime}"
export DEBIAN_FRONTEND=noninteractive

apt-get update

runtime_pkgs=(
    ca-certificates
    curl
    libgtk-3-0
    libwebkit2gtk-4.1-0
    libayatana-appindicator3-1
    librsvg2-2
    libsqlite3-0
)

builder_pkgs=(
    build-essential
    pkg-config
    libssl-dev
    libgtk-3-dev
    libwebkit2gtk-4.1-dev
    libayatana-appindicator3-dev
    librsvg2-dev
    patchelf
    libsqlite3-dev
    musl-tools
)

case "$mode" in
    runtime)
        apt-get install -y --no-install-recommends "${runtime_pkgs[@]}"
        ;;
    builder)
        apt-get install -y --no-install-recommends "${runtime_pkgs[@]}" "${builder_pkgs[@]}"
        ;;
    *)
        echo "usage: $0 runtime|builder" >&2
        exit 1
        ;;
esac

rm -rf /var/lib/apt/lists/*
