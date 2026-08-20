#!/bin/sh
set -eu
port="${ROCCI_HTTP_PORT:-8080}"
printf '\n  Open http://127.0.0.1:%s/\n\n' "$port"
exec server "$@"
