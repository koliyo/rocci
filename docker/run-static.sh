#!/usr/bin/env bash
set -euo pipefail

usage() {
    echo "usage: $0 DIST_DIR [compose up args...]" >&2
    echo "  DIST_DIR  pre-built Rocdown dist (relative to cwd or absolute)" >&2
    echo "  Build the site on the host first, e.g.:" >&2
    echo "    cargo run -q -p rocci-rocdown-cli -- build docs --cdn-only" >&2
    exit 1
}

if [ $# -lt 1 ]; then
    usage
fi

script_dir="$(cd "$(dirname "$0")" && pwd)"
repo_root="$(cd "${script_dir}/.." && pwd)"
dist_arg="$1"
shift

if [ ! -d "${dist_arg}" ]; then
    echo "error: not a directory: ${dist_arg}" >&2
    exit 1
fi

export ROCCI_DIST="$(cd "${dist_arg}" && pwd)"

if [ ! -f "${ROCCI_DIST}/index.html" ]; then
    echo "error: no index.html in ${ROCCI_DIST}; build the site on the host first" >&2
    exit 1
fi

cd "${repo_root}"
exec docker compose -f "${script_dir}/compose.static.yml" up "$@"
