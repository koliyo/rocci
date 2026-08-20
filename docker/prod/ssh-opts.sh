# Sourced by bootstrap-scp.sh, check-ssh.sh, and push-release.sh.
# When CF_ACCESS_CLIENT_ID is set, OpenSSH uses access-ssh-proxy.sh as ProxyCommand.

_rocci_prod_dir="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
_identity="${DEPLOY_SSH_IDENTITY:-$HOME/.ssh/deploy}"

ROCCI_SSH_OPTS=(
    -o BatchMode=yes
    -o IdentitiesOnly=yes
    # -o IdentityFile=${HOME}/.ssh/id_ed25519_hetzner_deploy
    -o PreferredAuthentications=publickey
    -o StrictHostKeyChecking=accept-new
    -o ConnectTimeout=45
    -o ConnectionAttempts=1
    -o ServerAliveInterval=10
    -o ServerAliveCountMax=3
)
if [ -f "${_identity}" ]; then
    ROCCI_SSH_OPTS+=(-i "${_identity}")
fi
if [ -n "${CF_ACCESS_CLIENT_ID:-}" ]; then
    ROCCI_SSH_OPTS+=(-o "ProxyCommand=${_rocci_prod_dir}/access-ssh-proxy.sh %h")
fi
if [ -n "${ROCCI_SSH_VERBOSE:-}" ]; then
    ROCCI_SSH_OPTS+=(-vv)
    ROCCI_SCP_OPTS=(-v)
else
    ROCCI_SCP_OPTS=()
fi

rocci_ssh() {
    echo "ssh: $* (timeout 45s, BatchMode, IdentitiesOnly)" >&2
    echo "ROCCI_SSH_OPTS: ${ROCCI_SSH_OPTS[@]}" >&2
    ssh "${ROCCI_SSH_OPTS[@]}" "$@"
}

rocci_scp() {
    echo "scp: $*" >&2
    scp "${ROCCI_SSH_OPTS[@]}" "${ROCCI_SCP_OPTS[@]}" "$@"
}
