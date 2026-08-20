#!/usr/bin/env bash
# Copy origin Compose, Caddy, islands Dockerfile, and prod scripts.
# Does not copy site.tgz or the islands binary.
set -eo pipefail

usage() {
    echo "usage: $0" >&2
    echo "env:" >&2
    echo "  DEPLOY_HOST          SSH host (required)" >&2
    echo "  DEPLOY_USER          SSH user (default deploy)" >&2
    echo "  ROCCI_BOOTSTRAP_DEST remote docker dir (default /srv/rocci/docker)" >&2
    echo "  RSYNC_RSH            ssh command (default ssh)" >&2
    exit 1
}

if [ "${1:-}" = "-h" ] || [ "${1:-}" = "--help" ]; then
    usage
fi

host="${DEPLOY_HOST:?set DEPLOY_HOST}"
if [ -z "${DEPLOY_USER:-}" ]; then
    user=deploy
else
    user="${DEPLOY_USER}"
fi
dest="${ROCCI_BOOTSTRAP_DEST:-/srv/rocci/docker}"
rsh="${RSYNC_RSH:-ssh}"
script_dir="$(cd "$(dirname "$0")" && pwd)"
repo_docker="$(cd "${script_dir}/.." && pwd)"

ssh_target="${user}@${host}"
${rsh} "${ssh_target}" "mkdir -p '${dest}'"

rsync -az -e "${rsh}" \
    --include='/' \
    --include='/compose.hybrid.yml' \
    --include='/cdn/' \
    --include='/cdn/***' \
    --include='/islands/' \
    --include='/islands/Dockerfile' \
    --exclude='/islands/***' \
    --include='/prod/' \
    --include='/prod/***' \
    --exclude='*' \
    "${repo_docker}/" \
    "${ssh_target}:${dest}/"

echo "bootstrapped ${ssh_target}:${dest}"
