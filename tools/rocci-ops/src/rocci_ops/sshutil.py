from __future__ import annotations

import os
import re
import subprocess
from pathlib import Path

from rocci_ops.paths import repo_root

HEX_SHA = re.compile(r"^[0-9a-fA-F]+$")


def require_host() -> str:
    host = os.environ.get("DEPLOY_HOST")
    if not host:
        raise SystemExit("set DEPLOY_HOST")
    return host


def deploy_user() -> str:
    return os.environ.get("DEPLOY_USER") or "deploy"


def ssh_target() -> str:
    return f"{deploy_user()}@{require_host()}"


def identity_path() -> Path:
    return Path(os.environ.get("DEPLOY_SSH_IDENTITY") or Path.home() / ".ssh" / "deploy")


def ssh_opts() -> list[str]:
    opts = [
        "-o",
        "BatchMode=yes",
        "-o",
        "IdentitiesOnly=yes",
        "-o",
        "PreferredAuthentications=publickey",
        "-o",
        "StrictHostKeyChecking=accept-new",
        "-o",
        "ConnectTimeout=45",
        "-o",
        "ConnectionAttempts=1",
        "-o",
        "ServerAliveInterval=10",
        "-o",
        "ServerAliveCountMax=3",
    ]
    identity = identity_path()
    if identity.is_file():
        opts.extend(["-i", str(identity)])
    if os.environ.get("CF_ACCESS_CLIENT_ID"):
        proxy = repo_root() / "docker" / "prod" / "access-ssh-proxy.sh"
        opts.extend(["-o", f"ProxyCommand={proxy} %h"])
    if os.environ.get("ROCCI_SSH_VERBOSE"):
        opts.append("-vv")
    return opts


def scp_opts() -> list[str]:
    extra = ["-v"] if os.environ.get("ROCCI_SSH_VERBOSE") else []
    return ssh_opts() + extra


def rocci_ssh(args: list[str], *, runner=subprocess.run) -> subprocess.CompletedProcess:
    print(f"ssh: {' '.join(args)} (timeout 45s, BatchMode, IdentitiesOnly)", flush=True)
    return runner(["ssh", *ssh_opts(), *args], check=True)


def rocci_scp(args: list[str], *, runner=subprocess.run) -> subprocess.CompletedProcess:
    print(f"scp: {' '.join(args)}", flush=True)
    return runner(["scp", *scp_opts(), *args], check=True)


def validate_sha(sha: str) -> str:
    if not sha or not HEX_SHA.fullmatch(sha):
        raise SystemExit("error: SHA must be hex")
    return sha
