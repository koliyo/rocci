#!/usr/bin/env bash
# Copy the islands SQLite file off the named volume.
set -euo pipefail

project="${COMPOSE_PROJECT_NAME:-rocci-prod}"
volume="${ROCCI_ISLANDS_VOLUME:-${project}_islands-db}"
dest_dir="${1:-/var/backups/rocci}"
stamp="$(date -u +%Y%m%dT%H%M%SZ)"
mkdir -p "${dest_dir}"
dest="${dest_dir}/site-${stamp}.db"

docker run --rm \
    -v "${volume}:/data:ro" \
    -v "${dest_dir}:/backup" \
    debian:bookworm-slim \
    cp /data/site.db "/backup/site-${stamp}.db"

echo "wrote ${dest}"
