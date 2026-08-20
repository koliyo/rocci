#!/usr/bin/env bash
set -euo pipefail

usage() {
    echo "usage: $0 SERVER_DIR [compose up args...]" >&2
    echo "  SERVER_DIR  output of \`rocci build --release\` (contains server + assets/)" >&2
    echo "  Build on the host first, e.g.:" >&2
    echo "    cargo run -q -p rocci-cli -- build --release examples/datastar --target arm64musl" >&2
    echo "    (use --target x64musl on Intel Mac / amd64 Docker; see docker/README.md)" >&2
    exit 1
}

if [ $# -lt 1 ]; then
    usage
fi

script_dir="$(cd "$(dirname "$0")" && pwd)"
repo_root="$(cd "${script_dir}/.." && pwd)"
dir_arg="$1"
shift

if [ ! -d "${dir_arg}" ]; then
    echo "error: not a directory: ${dir_arg}" >&2
    exit 1
fi

server_dir="$(cd "${dir_arg}" && pwd)"
if [ ! -f "${server_dir}/server" ]; then
    echo "error: no server binary in ${server_dir}; run \`rocci build --release\` first" >&2
    exit 1
fi

context="$(mktemp -d "${TMPDIR:-/tmp}/rocci-app-XXXXXX")"
cleanup() {
    rm -rf "${context}"
}
trap cleanup EXIT

cp "${script_dir}/app/Dockerfile" "${context}/Dockerfile"
cp "${script_dir}/app/entrypoint.sh" "${context}/entrypoint.sh"
cp "${server_dir}/server" "${context}/server"
chmod +x "${context}/server" "${context}/entrypoint.sh"
mkdir -p "${context}/assets"
if [ -d "${server_dir}/assets" ]; then
    cp -R "${server_dir}/assets/." "${context}/assets/"
fi
export ROCCI_APP_CONTEXT="${context}"

cd "${repo_root}"
docker compose -f "${script_dir}/compose.app.yml" up --build "$@"
