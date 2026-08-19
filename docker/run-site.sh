#!/usr/bin/env bash
set -euo pipefail

usage() {
    echo "usage: $0 SITE_DIR [compose up args...]" >&2
    echo "  SITE_DIR  hybrid Rocdown site (relative to cwd or absolute)" >&2
    exit 1
}

if [ $# -lt 1 ]; then
    usage
fi

script_dir="$(cd "$(dirname "$0")" && pwd)"
repo_root="$(cd "${script_dir}/.." && pwd)"
site_arg="$1"
shift

if [ ! -d "${site_arg}" ]; then
    echo "error: not a directory: ${site_arg}" >&2
    exit 1
fi

export ROCCI_SITE="$(cd "${site_arg}" && pwd)"

if [ ! -f "${ROCCI_SITE}/rocdown.toml" ]; then
    echo "error: no rocdown.toml in ${ROCCI_SITE}" >&2
    exit 1
fi

cd "${repo_root}"
exec docker compose -f "${script_dir}/compose.yml" up --build "$@"
