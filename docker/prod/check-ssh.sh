#!/usr/bin/env bash
# Print deploy preflight (no secret values) and run a verbose SSH probe.
set -euo pipefail

script_dir="$(cd "$(dirname "$0")" && pwd)"
# shellcheck source=ssh-opts.sh
. "${script_dir}/ssh-opts.sh"

host="${DEPLOY_HOST:?set DEPLOY_HOST}"
if [ -z "${DEPLOY_USER:-}" ]; then
    user=deploy
else
    user="${DEPLOY_USER}"
fi
identity="${DEPLOY_SSH_IDENTITY:-$HOME/.ssh/deploy}"
ssh_target="${user}@${host}"

echo "=== preflight ==="
echo "date: $(date -u +%Y-%m-%dT%H:%M:%SZ)"
echo "runner: $(uname -a)"
echo "DEPLOY_HOST set: $([ -n "${DEPLOY_HOST}" ] && echo yes || echo no) chars=${#DEPLOY_HOST}"
echo "DEPLOY_USER: ${user} chars=${#user}"
echo "CF_SSH_HOSTNAME: ${CF_SSH_HOSTNAME:-unset}"
cid="${CF_ACCESS_CLIENT_ID-}"
csec="${CF_ACCESS_CLIENT_SECRET-}"
echo "CF_ACCESS_CLIENT_ID set: $([ -n "${cid}" ] && echo yes || echo no) chars=${#cid}"
echo "CF_ACCESS_CLIENT_SECRET set: $([ -n "${csec}" ] && echo yes || echo no) chars=${#csec}"
echo "identity: ${identity}"
if [ ! -f "${identity}" ]; then
    echo "error: missing SSH identity file ${identity}" >&2
    exit 1
fi
echo "identity bytes=$(wc -c < "${identity}") lines=$(wc -l < "${identity}") mode=$(stat -c '%a' "${identity}" 2>/dev/null || stat -f '%OLp' "${identity}")"
if grep -q "BEGIN OPENSSH PRIVATE KEY\|BEGIN .* PRIVATE KEY" "${identity}"; then
    echo "identity: looks like a private key (BEGIN line present)"
else
    echo "error: identity does not look like a private key (public key pasted? missing newlines?)" >&2
    exit 1
fi
if grep -q "BEGIN OPENSSH PRIVATE KEY\|BEGIN .* PRIVATE KEY" "${identity}" \
    && ! grep -q "END OPENSSH PRIVATE KEY\|END .* PRIVATE KEY" "${identity}"; then
    echo "error: private key BEGIN without END (truncated secret)" >&2
    exit 1
fi
ssh-keygen -lf "${identity}" || true
echo "cloudflared: $(command -v cloudflared || echo MISSING)"
cloudflared version || cloudflared --version || true
echo "=== DNS ${CF_SSH_HOSTNAME:-${host}} ==="
dns_name="${CF_SSH_HOSTNAME:-${host}}"
getent ahosts "${dns_name}" || true
getent hosts "${dns_name}" || true

echo "=== SSH probe ${ssh_target} ==="
rocci_ssh "${ssh_target}" "set -e; echo PROBE_OK; date -u +%Y-%m-%dT%H:%M:%SZ; hostname; uname -a; id; command -v docker; docker compose version; ls -ld /srv/rocci /srv/rocci/docker /srv/rocci/incoming /srv/rocci/releases 2>&1 || true"
echo "=== probe succeeded ==="
