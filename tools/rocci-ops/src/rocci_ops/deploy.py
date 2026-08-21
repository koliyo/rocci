from __future__ import annotations

import argparse
import os
import shutil
import subprocess
from pathlib import Path

from rocci_ops.paths import repo_root
from rocci_ops.sshutil import (
    deploy_user,
    identity_path,
    require_host,
    rocci_scp,
    rocci_ssh,
    ssh_target,
    validate_sha,
)

ORIGIN_ROOT_DEFAULT = "/srv/rocci"
BOOTSTRAP_DOCKER_DEFAULT = "/srv/rocci/docker"


def origin_publish_cmd(sha: str, origin_root: str) -> str:
    return (
        f"cd '{origin_root}' && uv run --project tools/rocci-ops --no-dev "
        f"rocci-ops origin publish '{sha}'"
    )


def probe(*, runner=subprocess.run) -> int:
    host = require_host()
    identity = identity_path()
    target = ssh_target()
    print("=== preflight ===", flush=True)
    print(f"DEPLOY_HOST set: yes chars={len(host)}", flush=True)
    print(f"DEPLOY_USER: {deploy_user()} chars={len(deploy_user())}", flush=True)
    print(f"CF_SSH_HOSTNAME: {os.environ.get('CF_SSH_HOSTNAME', 'unset')}", flush=True)
    cid = os.environ.get("CF_ACCESS_CLIENT_ID", "")
    csec = os.environ.get("CF_ACCESS_CLIENT_SECRET", "")
    print(f"CF_ACCESS_CLIENT_ID set: {'yes' if cid else 'no'} chars={len(cid)}", flush=True)
    print(
        f"CF_ACCESS_CLIENT_SECRET set: {'yes' if csec else 'no'} chars={len(csec)}",
        flush=True,
    )
    print(f"identity: {identity}", flush=True)
    if not identity.is_file():
        raise SystemExit(f"error: missing SSH identity file {identity}")
    text = identity.read_text(encoding="utf-8", errors="replace")
    if "BEGIN" not in text or "PRIVATE KEY" not in text:
        raise SystemExit(
            "error: identity does not look like a private key (public key pasted? missing newlines?)"
        )
    if "BEGIN" in text and "END" not in text:
        raise SystemExit("error: private key BEGIN without END (truncated secret)")
    if not shutil.which("cloudflared"):
        print("cloudflared: MISSING", flush=True)
    rocci_ssh(
        [
            target,
            "echo PROBE_OK; docker compose version",
        ],
        runner=runner,
    )
    print("=== probe succeeded ===", flush=True)
    return 0


def bootstrap(*, runner=subprocess.run) -> int:
    dest = os.environ.get("ROCCI_BOOTSTRAP_DEST", BOOTSTRAP_DOCKER_DEFAULT)
    origin_root = os.environ.get("ROCCI_ORIGIN_ROOT", ORIGIN_ROOT_DEFAULT)
    root = repo_root()
    docker = root / "docker"
    prod = docker / "prod"
    target = ssh_target()
    ops_dest = f"{origin_root}/tools/rocci-ops"
    rocci_ssh(
        [
            target,
            f"mkdir -p '{dest}/cdn' '{dest}/islands' '{dest}/prod' '{ops_dest}/src'",
        ],
        runner=runner,
    )
    rocci_scp([str(docker / "compose.hybrid.yml"), f"{target}:{dest}/"], runner=runner)
    rocci_scp(
        [
            str(docker / "cdn" / "Caddyfile"),
            str(docker / "cdn" / "Dockerfile"),
            str(docker / "cdn" / "entrypoint.sh"),
            f"{target}:{dest}/cdn/",
        ],
        runner=runner,
    )
    rocci_scp(
        [str(docker / "islands" / "Dockerfile"), f"{target}:{dest}/islands/"],
        runner=runner,
    )
    rocci_scp(
        [
            str(prod / "README.md"),
            str(prod / "access-ssh-proxy.sh"),
            str(prod / "cloudflared-ingress.yml.example"),
            str(prod / "env.example"),
            f"{target}:{dest}/prod/",
        ],
        runner=runner,
    )
    ops = root / "tools" / "rocci-ops"
    rocci_scp(
        [
            str(ops / "pyproject.toml"),
            str(ops / "uv.lock"),
            str(ops / ".python-version"),
            f"{target}:{ops_dest}/",
        ],
        runner=runner,
    )
    rocci_scp(
        ["-r", str(ops / "src" / "rocci_ops"), f"{target}:{ops_dest}/src/"],
        runner=runner,
    )
    rocci_ssh([target, f"chmod +x '{dest}/prod/access-ssh-proxy.sh'"], runner=runner)
    print(f"bootstrapped {target}:{dest}", flush=True)
    return 0


def push(artifact_dir: Path, sha: str, *, runner=subprocess.run) -> int:
    sha = validate_sha(sha)
    if not artifact_dir.is_dir():
        raise SystemExit(f"error: not a directory: {artifact_dir}")
    tgz = artifact_dir / "site.tgz"
    islands = artifact_dir / "islands"
    if not tgz.is_file() or not islands.is_file():
        raise SystemExit(f"error: {artifact_dir} must contain site.tgz and islands")
    origin_root = os.environ.get("ROCCI_ORIGIN_ROOT", ORIGIN_ROOT_DEFAULT)
    target = ssh_target()
    incoming = f"{origin_root}/incoming/{sha}"
    bootstrap(runner=runner)
    rocci_ssh([target, f"mkdir -p '{incoming}'"], runner=runner)
    rocci_scp(
        [str(tgz), str(islands), f"{target}:{incoming}/"],
        runner=runner,
    )
    rocci_ssh([target, origin_publish_cmd(sha, origin_root)], runner=runner)
    return 0


def main(argv: list[str]) -> int:
    parser = argparse.ArgumentParser(prog="rocci-ops deploy")
    sub = parser.add_subparsers(dest="command", required=True)
    sub.add_parser("probe")
    sub.add_parser("bootstrap")
    push_p = sub.add_parser("push")
    push_p.add_argument("artifact_dir")
    push_p.add_argument("sha")
    ns = parser.parse_args(argv)
    if ns.command == "probe":
        return probe()
    if ns.command == "bootstrap":
        return bootstrap()
    if ns.command == "push":
        return push(Path(ns.artifact_dir), ns.sha)
    raise SystemExit(2)
