#!/usr/bin/env bash
# Copy origin Compose, Caddy, islands Dockerfile, and prod scripts.
# Does not copy site.tgz or the islands binary.
set -euo pipefail

usage() {
    echo "usage: $0" >&2
    echo "env:" >&2
    echo "  DEPLOY_HOST          SSH host (required)" >&2
    echo "  DEPLOY_USER          SSH user (default deploy)" >&2
    echo "  ROCCI_BOOTSTRAP_DEST remote docker dir (default /srv/rocci/docker)" >&2
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
script_dir="$(cd "$(dirname "$0")" && pwd)"
# shellcheck source=ssh-opts.sh
. "${script_dir}/ssh-opts.sh"
repo_docker="$(cd "${script_dir}/.." && pwd)"

ssh_target="${user}@${host}"
echo "=== bootstrap mkdir ${ssh_target}:${dest} ==="
rocci_ssh "${ssh_target}" "mkdir -p '${dest}/cdn' '${dest}/islands' '${dest}/prod'"
echo "=== bootstrap scp compose + cdn + islands Dockerfile + prod ==="

rocci_scp "${repo_docker}/compose.hybrid.yml" "${ssh_target}:${dest}/"
rocci_scp "${repo_docker}/cdn/Caddyfile" "${repo_docker}/cdn/Dockerfile" \
    "${repo_docker}/cdn/entrypoint.sh" "${ssh_target}:${dest}/cdn/"
rocci_scp "${repo_docker}/islands/Dockerfile" "${ssh_target}:${dest}/islands/"
rocci_scp \
    "${script_dir}/README.md" \
    "${script_dir}/access-ssh-proxy.sh" \
    "${script_dir}/backup-sqlite.sh" \
    "${script_dir}/bootstrap-scp.sh" \
    "${script_dir}/check-ssh.sh" \
    "${script_dir}/cloudflared-ingress.yml.example" \
    "${script_dir}/env.example" \
    "${script_dir}/publish.sh" \
    "${script_dir}/push-release.sh" \
    "${script_dir}/ssh-opts.sh" \
    "${script_dir}/up.sh" \
    "${ssh_target}:${dest}/prod/"

echo "bootstrapped ${ssh_target}:${dest}"
