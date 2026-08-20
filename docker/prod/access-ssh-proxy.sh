#!/usr/bin/env bash
# ProxyCommand for `cloudflared access ssh`. Tokens stay in this process env.
set -euo pipefail

hostname="${CF_SSH_HOSTNAME:-${1:-${DEPLOY_HOST:?set CF_SSH_HOSTNAME or DEPLOY_HOST}}}"
echo "access-ssh-proxy: cloudflared access ssh --hostname ${hostname}" >&2
loglevel=info
if [ -n "${ROCCI_SSH_VERBOSE:-}" ]; then
    loglevel=debug
fi
exec cloudflared access ssh \
    --loglevel "${loglevel}" \
    --hostname "${hostname}" \
    --service-token-id "${CF_ACCESS_CLIENT_ID:?set CF_ACCESS_CLIENT_ID}" \
    --service-token-secret "${CF_ACCESS_CLIENT_SECRET:?set CF_ACCESS_CLIENT_SECRET}"
