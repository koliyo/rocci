#!/usr/bin/env bash
# First origin publish: unpack hybrid artifacts and compose up Caddy + islands.
# Does not SSH, does not read GitHub secrets, does not install rocci/roc.
set -euo pipefail

usage() {
    echo "usage: $0 DIST_DIR ISLANDS_BIN" >&2
    echo "  DIST_DIR     unpacked CDN tree (contains index.html)" >&2
    echo "  ISLANDS_BIN  linux/amd64 musl islands binary" >&2
    echo "env:" >&2
    echo "  ROCCI_ORIGIN_ROOT  persistent origin dir (default /srv/rocci)" >&2
    echo "  ROCCI_HTTP_PORT    published Caddy port (default 8080)" >&2
    exit 1
}

if [ $# -ne 2 ]; then
    usage
fi

dist_arg="$1"
bin_arg="$2"
origin_root="${ROCCI_ORIGIN_ROOT:-/srv/rocci}"
http_port="${ROCCI_HTTP_PORT:-8080}"
script_dir="$(cd "$(dirname "$0")" && pwd)"
repo_docker="$(cd "${script_dir}/.." && pwd)"

if [ ! -d "${dist_arg}" ]; then
    echo "error: not a directory: ${dist_arg}" >&2
    exit 1
fi
if [ ! -f "${bin_arg}" ]; then
    echo "error: not a file: ${bin_arg}" >&2
    exit 1
fi

dist="$(cd "${dist_arg}" && pwd)"
if [ ! -f "${dist}/index.html" ]; then
    echo "error: no index.html in ${dist}" >&2
    exit 1
fi

current="${origin_root}/current"
context="${current}/islands-context"
mkdir -p "${current}" "${context}"
if [ "${dist}" != "${current}/dist" ]; then
    rm -rf "${current}/dist"
    mkdir -p "${current}/dist"
    cp -a "${dist}/." "${current}/dist/"
fi
cp "${repo_docker}/islands/Dockerfile" "${context}/Dockerfile"
cp "${bin_arg}" "${context}/islands"
chmod +x "${context}/islands"

export COMPOSE_PROJECT_NAME="${COMPOSE_PROJECT_NAME:-rocci-prod}"
export ROCCI_DIST="${current}/dist"
export ROCCI_ISLANDS_CONTEXT="${context}"
export ROCCI_HTTP_PORT="${http_port}"

cd "${repo_docker}"
docker compose -f "${repo_docker}/compose.hybrid.yml" up -d --build
echo "origin up: http://127.0.0.1:${http_port}/ (SQLite volume ${COMPOSE_PROJECT_NAME}_islands-db)"
