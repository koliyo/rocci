import argparse
import os
import shutil
import subprocess
import tarfile
import tempfile
from pathlib import Path

from rocci_ops.lanes import resolved_lane
from rocci_ops.paths import repo_root
from rocci_ops.sshutil import (
    deploy_user,
    identity_path,
    require_host,
    rocci_ssh,
    rocci_ssh_stdin,
    ssh_target,
    validate_sha,
)


def origin_publish_cmd(sha: str, origin_root: str | None = None) -> str:
    cfg = resolved_lane()
    root = origin_root if origin_root is not None else cfg.origin_root
    live = "1" if cfg.publish_live is not False else "0"
    exports = [
        f"ROCCI_ORIGIN_ROOT='{root}'",
        f"ROCCI_HTTP_PORT='{cfg.http_port}'",
        f"COMPOSE_PROJECT_NAME='{cfg.compose_project}'",
        f"ROCCI_PUBLISH_LIVE='{live}'",
        f"ROCCI_IMAGE_TAG='{cfg.image_tag}'",
    ]
    if cfg.name:
        exports.insert(0, f"ROCCI_LANE='{cfg.name}'")
    return f"cd '{root}' && {' '.join(exports)} uv run --no-dev rocci-ops origin publish '{sha}'"


def stage_origin_kit(dest: Path) -> None:
    root = repo_root()
    docker = root / "docker"
    dest_docker = dest / "docker"
    for sub in ("cdn", "islands", "app", "prod"):
        (dest_docker / sub).mkdir(parents=True, exist_ok=True)
    shutil.copy2(docker / "compose.hybrid.yml", dest_docker / "compose.hybrid.yml")
    shutil.copy2(docker / "compose.origin.yml", dest_docker / "compose.origin.yml")
    for name in (
        "Caddyfile",
        "examples.caddy",
        "examples.stub.caddy",
        "Dockerfile",
        "entrypoint.sh",
    ):
        shutil.copy2(docker / "cdn" / name, dest_docker / "cdn" / name)
    shutil.copy2(docker / "islands" / "Dockerfile", dest_docker / "islands" / "Dockerfile")
    shutil.copy2(docker / "app" / "Dockerfile", dest_docker / "app" / "Dockerfile")
    shutil.copy2(docker / "app" / "entrypoint.sh", dest_docker / "app" / "entrypoint.sh")
    for name in (
        "README.md",
        "access-ssh-proxy.sh",
        "cloudflared-ingress.yml.example",
        "env.example",
    ):
        shutil.copy2(docker / "prod" / name, dest_docker / "prod" / name)
    shutil.copy2(root / "pyproject.toml", dest / "pyproject.toml")
    shutil.copy2(root / "uv.lock", dest / "uv.lock")
    shutil.copy2(root / ".python-version", dest / ".python-version")
    ops = root / "rocci-ops"
    ops_dest = dest / "rocci-ops"
    (ops_dest / "src").mkdir(parents=True, exist_ok=True)
    shutil.copy2(ops / "pyproject.toml", ops_dest / "pyproject.toml")
    shutil.copy2(ops / ".python-version", ops_dest / ".python-version")
    dest_pkg = ops_dest / "src" / "rocci_ops"
    if dest_pkg.exists():
        shutil.rmtree(dest_pkg)
    shutil.copytree(
        ops / "src" / "rocci_ops",
        dest_pkg,
        ignore=shutil.ignore_patterns("__pycache__", "*.pyc"),
    )


def stage_incoming(dest: Path, artifact_dir: Path, sha: str) -> None:
    incoming = dest / "incoming" / sha
    incoming.mkdir(parents=True, exist_ok=True)
    shutil.copy2(artifact_dir / "site.tgz", incoming / "site.tgz")
    shutil.copy2(artifact_dir / "islands", incoming / "islands")
    live = artifact_dir / "examples-live"
    if live.is_dir():
        shutil.copytree(live, incoming / "examples-live")


def write_origin_tar(tree: Path, tar_path: Path) -> None:
    with tarfile.open(tar_path, mode="w:gz") as archive:
        for path in sorted(tree.rglob("*")):
            if path.is_file():
                archive.add(path, arcname=path.relative_to(tree).as_posix())


def provision_remote(
    tree: Path,
    *,
    publish_sha: str | None = None,
    runner=subprocess.run,
) -> None:
    cfg = resolved_lane()
    origin_root = cfg.origin_root
    proxy = f"{origin_root}/docker/prod/access-ssh-proxy.sh"
    remote = (
        f"mkdir -p '{origin_root}' && tar -xzf - -C '{origin_root}' && chmod +x '{proxy}'"
    )
    if publish_sha is not None:
        remote = f"{remote} && {origin_publish_cmd(publish_sha, origin_root)}"
    with tempfile.TemporaryDirectory() as tmp:
        tar_path = Path(tmp) / "origin.tar.gz"
        write_origin_tar(tree, tar_path)
        rocci_ssh_stdin(remote, tar_path, runner=runner)


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
    cfg = resolved_lane()
    with tempfile.TemporaryDirectory() as tmp:
        tree = Path(tmp) / "tree"
        tree.mkdir()
        stage_origin_kit(tree)
        provision_remote(tree, runner=runner)
    print(f"bootstrapped {ssh_target()}:{cfg.bootstrap_dest}", flush=True)
    return 0


def push(artifact_dir: Path, sha: str, *, runner=subprocess.run) -> int:
    sha = validate_sha(sha)
    if not artifact_dir.is_dir():
        raise SystemExit(f"error: not a directory: {artifact_dir}")
    tgz = artifact_dir / "site.tgz"
    islands = artifact_dir / "islands"
    if not tgz.is_file() or not islands.is_file():
        raise SystemExit(f"error: {artifact_dir} must contain site.tgz and islands")
    with tempfile.TemporaryDirectory() as tmp:
        tree = Path(tmp) / "tree"
        tree.mkdir()
        stage_origin_kit(tree)
        stage_incoming(tree, artifact_dir, sha)
        provision_remote(tree, publish_sha=sha, runner=runner)
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
