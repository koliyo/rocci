#!/usr/bin/env bash
# From CI or a laptop: rsync origin kit + artifacts, then run publish.sh on the VPS.
set -euo pipefail

usage() {
    echo "usage: $0 ARTIFACT_DIR SHA" >&2
    echo "  ARTIFACT_DIR  directory with site.tgz and islands" >&2
    echo "  SHA           git commit (hex)" >&2
    echo "env: DEPLOY_HOST (required), DEPLOY_USER (default deploy)," >&2
    echo "     ROCCI_ORIGIN_ROOT (default /srv/rocci), RSYNC_RSH" >&2
    exit 1
}

if [ $# -ne 2 ]; then
    usage
fi

artifact_dir="$1"
sha="$2"
host="${DEPLOY_HOST:?set DEPLOY_HOST}"
if [ -z "${DEPLOY_USER:-}" ]; then
    user=deploy
else
    user="${DEPLOY_USER}"
fi
origin_root="${ROCCI_ORIGIN_ROOT:-/srv/rocci}"
rsh="${RSYNC_RSH:-ssh}"
script_dir="$(cd "$(dirname "$0")" && pwd)"

if [ ! -d "${artifact_dir}" ]; then
    echo "error: not a directory: ${artifact_dir}" >&2
    exit 1
fi
if [ ! -f "${artifact_dir}/site.tgz" ] || [ ! -f "${artifact_dir}/islands" ]; then
    echo "error: ${artifact_dir} must contain site.tgz and islands" >&2
    exit 1
fi

ssh_target="${user}@${host}"
incoming="${origin_root}/incoming/${sha}"

"${script_dir}/bootstrap-rsync.sh"

${rsh} "${ssh_target}" "mkdir -p '${incoming}'"
rsync -az -e "${rsh}" \
    "${artifact_dir}/site.tgz" \
    "${artifact_dir}/islands" \
    "${ssh_target}:${incoming}/"

${rsh} "${ssh_target}" "${origin_root}/docker/prod/publish.sh '${sha}'"
