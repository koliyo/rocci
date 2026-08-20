#!/usr/bin/env bash
# Unpack CI artifacts into releases/<sha>, compose up, flip current on health.
set -euo pipefail

usage() {
    echo "usage: $0 SHA" >&2
    exit 1
}

if [ $# -ne 1 ]; then
    usage
fi

sha="$1"
case "$sha" in
    *[!0-9a-fA-F]* | "")
        echo "error: SHA must be hex" >&2
        exit 1
        ;;
esac

origin_root="${ROCCI_ORIGIN_ROOT:-/srv/rocci}"
http_port="${ROCCI_HTTP_PORT:-8080}"
keep_n="${ROCCI_KEEP_RELEASES:-5}"
script_dir="$(cd "$(dirname "$0")" && pwd)"
repo_docker="$(cd "${script_dir}/.." && pwd)"
incoming="${origin_root}/incoming/${sha}"
release="${origin_root}/releases/${sha}"
current="${origin_root}/current"
compose_file="${repo_docker}/compose.hybrid.yml"

tgz="${incoming}/site.tgz"
bin="${incoming}/islands"
if [ ! -f "${tgz}" ] || [ ! -f "${bin}" ]; then
    echo "error: missing ${tgz} or ${bin}" >&2
    exit 1
fi

previous=""
if [ -L "${current}" ]; then
    previous="$(readlink -f "${current}" || true)"
fi

mkdir -p "${release}/dist" "${release}/islands-context"
tar -xzf "${tgz}" -C "${release}/dist"
if [ ! -f "${release}/dist/index.html" ]; then
    echo "error: site.tgz did not contain index.html" >&2
    exit 1
fi
cp "${repo_docker}/islands/Dockerfile" "${release}/islands-context/Dockerfile"
cp "${bin}" "${release}/islands-context/islands"
chmod +x "${release}/islands-context/islands"

compose_up() {
    local root="$1"
    export COMPOSE_PROJECT_NAME="${COMPOSE_PROJECT_NAME:-rocci-prod}"
    export ROCCI_DIST="${root}/dist"
    export ROCCI_ISLANDS_CONTEXT="${root}/islands-context"
    export ROCCI_HTTP_PORT="${http_port}"
    docker compose -f "${compose_file}" --project-directory "${repo_docker}" up -d --build
}

wait_health() {
    local i
    for i in $(seq 1 36); do
        if curl -sf "http://127.0.0.1:${http_port}/health" >/dev/null; then
            return 0
        fi
        sleep 5
    done
    return 1
}

compose_up "${release}"
if ! wait_health; then
    echo "error: origin health failed for ${sha}" >&2
    if [ -n "${previous}" ] && [ -d "${previous}" ]; then
        compose_up "${previous}"
    fi
    exit 1
fi

ln -sfn "releases/${sha}" "${current}"

if [ -d "${origin_root}/releases" ]; then
    # shellcheck disable=SC2012
    ls -1dt "${origin_root}/releases"/*/ 2>/dev/null | tail -n "+$((keep_n + 1))" | xargs -r rm -rf
fi
rm -rf "${incoming}"

echo "published ${sha} at http://127.0.0.1:${http_port}/"
