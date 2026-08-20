#!/usr/bin/env bash
set -euo pipefail

usage() {
    echo "usage: $0 DIST_DIR ISLANDS_BIN [compose up args...]" >&2
    echo "  DIST_DIR      pre-built Rocdown dist (CDN tree)" >&2
    echo "  ISLANDS_BIN   precompiled island process binary" >&2
    echo "  Build on the host first, e.g.:" >&2
    echo "    cargo run -q -p rocci-rocdown-cli -- package examples/rocdown-counter --target x64musl" >&2
    exit 1
}

if [ $# -lt 2 ]; then
    usage
fi

script_dir="$(cd "$(dirname "$0")" && pwd)"
repo_root="$(cd "${script_dir}/.." && pwd)"
dist_arg="$1"
bin_arg="$2"
shift 2

if [ ! -d "${dist_arg}" ]; then
    echo "error: not a directory: ${dist_arg}" >&2
    exit 1
fi
if [ ! -f "${bin_arg}" ]; then
    echo "error: not a file: ${bin_arg}" >&2
    exit 1
fi

export ROCCI_DIST="$(cd "${dist_arg}" && pwd)"
if [ ! -f "${ROCCI_DIST}/index.html" ]; then
    echo "error: no index.html in ${ROCCI_DIST}; package the site on the host first" >&2
    exit 1
fi

context="$(mktemp -d "${TMPDIR:-/tmp}/rocci-islands-XXXXXX")"
cleanup() {
    rm -rf "${context}"
}
trap cleanup EXIT

cp "${script_dir}/islands/Dockerfile" "${context}/Dockerfile"
cp "${bin_arg}" "${context}/islands"
chmod +x "${context}/islands"
export ROCCI_ISLANDS_CONTEXT="${context}"

cd "${repo_root}"
docker compose -f "${script_dir}/compose.hybrid.yml" up --build "$@"
