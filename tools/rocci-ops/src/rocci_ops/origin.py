from __future__ import annotations

import argparse
import os
import shutil
import subprocess
import tarfile
import time
import urllib.error
import urllib.request
from pathlib import Path

from rocci_ops.paths import repo_root
from rocci_ops.sshutil import validate_sha


def origin_root() -> Path:
    return Path(os.environ.get("ROCCI_ORIGIN_ROOT", "/srv/rocci"))


def http_port() -> str:
    return os.environ.get("ROCCI_HTTP_PORT", "8080")


def keep_releases() -> int:
    return int(os.environ.get("ROCCI_KEEP_RELEASES", "5"))


def compose_file() -> Path:
    return repo_root() / "docker" / "compose.hybrid.yml"


def compose_project_dir() -> Path:
    return repo_root() / "docker"


def compose_env(root: Path) -> dict[str, str]:
    env = os.environ.copy()
    env["COMPOSE_PROJECT_NAME"] = env.get("COMPOSE_PROJECT_NAME") or "rocci-prod"
    env["ROCCI_DIST"] = str(root / "dist")
    env["ROCCI_ISLANDS_CONTEXT"] = str(root / "islands-context")
    env["ROCCI_HTTP_PORT"] = http_port()
    return env


def compose_up(root: Path, *, runner=subprocess.run) -> None:
    env = compose_env(root)
    argv = [
        "docker",
        "compose",
        "-f",
        str(compose_file()),
        "--project-directory",
        str(compose_project_dir()),
    ]
    argv.extend(["up", "-d", "--build"])
    result = runner(argv, env=env, check=False)
    if result.returncode != 0:
        raise SystemExit("error: docker compose failed")


def health_ok(url: str, *, fetch=urllib.request.urlopen) -> bool:
    try:
        with fetch(url, timeout=5) as response:
            return getattr(response, "status", 200) == 200
    except (urllib.error.URLError, TimeoutError, OSError):
        return False


def health_urls() -> list[str]:
    port = http_port()
    return [
        f"http://127.0.0.1:{port}/health",
    ]


def wait_health(*, attempts: int = 36, delay: float = 5.0, fetch=urllib.request.urlopen, sleeper=time.sleep) -> bool:
    urls = health_urls()
    for index in range(1, attempts + 1):
        ok = all(health_ok(url, fetch=fetch) for url in urls)
        print(f"health {index}/{attempts} {'200' if ok else 'fail'}", flush=True)
        if ok:
            return True
        sleeper(delay)
    return False


def unpack_release(sha: str, incoming: Path, release: Path) -> None:
    tgz = incoming / "site.tgz"
    binary = incoming / "islands"
    if not tgz.is_file() or not binary.is_file():
        raise SystemExit(f"error: missing {tgz} or {binary}")
    dist = release / "dist"
    context = release / "islands-context"
    dist.mkdir(parents=True, exist_ok=True)
    context.mkdir(parents=True, exist_ok=True)
    with tarfile.open(tgz) as archive:
        archive.extractall(dist, filter="data")
    if not (dist / "index.html").is_file():
        raise SystemExit("error: site.tgz did not contain index.html")
    docker = repo_root() / "docker"
    shutil.copy2(docker / "islands" / "Dockerfile", context / "Dockerfile")
    shutil.copy2(binary, context / "islands")
    (context / "islands").chmod(0o755)


def prune_releases(releases: Path, keep_n: int) -> None:
    if not releases.is_dir():
        return
    dirs = sorted(
        [path for path in releases.iterdir() if path.is_dir()],
        key=lambda path: path.stat().st_mtime,
        reverse=True,
    )
    for stale in dirs[keep_n:]:
        shutil.rmtree(stale, ignore_errors=True)


def publish(sha: str, *, runner=subprocess.run, fetch=urllib.request.urlopen, sleeper=time.sleep) -> int:
    sha = validate_sha(sha)
    root = origin_root()
    incoming = root / "incoming" / sha
    release = root / "releases" / sha
    current = root / "current"
    previous = current.resolve() if current.is_symlink() else None
    print(f"=== publish {sha} ===", flush=True)
    unpack_release(sha, incoming, release)
    compose_up(release, runner=runner)
    if not wait_health(fetch=fetch, sleeper=sleeper):
        print(f"error: origin health failed for {sha}", flush=True)
        if previous is not None and previous.is_dir():
            print(f"=== rollback to {previous} ===", flush=True)
            compose_up(previous, runner=runner)
        raise SystemExit(1)
    current.parent.mkdir(parents=True, exist_ok=True)
    current.unlink(missing_ok=True)
    current.symlink_to(f"releases/{sha}")
    prune_releases(root / "releases", keep_releases())
    shutil.rmtree(incoming, ignore_errors=True)
    print(f"published {sha} at http://127.0.0.1:{http_port()}/", flush=True)
    return 0


def up(
    dist_arg: Path,
    bin_arg: Path,
    *,
    runner=subprocess.run,
) -> int:
    if not dist_arg.is_dir():
        raise SystemExit(f"error: not a directory: {dist_arg}")
    if not bin_arg.is_file():
        raise SystemExit(f"error: not a file: {bin_arg}")
    dist = dist_arg.resolve()
    if not (dist / "index.html").is_file():
        raise SystemExit(f"error: no index.html in {dist}")
    current = origin_root() / "current"
    context = current / "islands-context"
    current.mkdir(parents=True, exist_ok=True)
    context.mkdir(parents=True, exist_ok=True)
    dest_dist = current / "dist"
    if dist != dest_dist:
        if dest_dist.exists():
            shutil.rmtree(dest_dist)
        shutil.copytree(dist, dest_dist)
    docker = repo_root() / "docker"
    shutil.copy2(docker / "islands" / "Dockerfile", context / "Dockerfile")
    shutil.copy2(bin_arg, context / "islands")
    (context / "islands").chmod(0o755)
    compose_up(current, runner=runner)
    print(f"origin up: http://127.0.0.1:{http_port()}/", flush=True)
    return 0


def backup(dest_dir: Path, *, runner=subprocess.run) -> int:
    project = os.environ.get("COMPOSE_PROJECT_NAME") or "rocci-prod"
    volume = os.environ.get("ROCCI_ISLANDS_VOLUME") or f"{project}_islands-db"
    dest_dir.mkdir(parents=True, exist_ok=True)
    stamp = time.strftime("%Y%m%dT%H%M%SZ", time.gmtime())
    dest_name = f"site-{stamp}.db"
    runner(
        [
            "docker",
            "run",
            "--rm",
            "-v",
            f"{volume}:/data:ro",
            "-v",
            f"{dest_dir}:/backup",
            "debian:bookworm-slim",
            "cp",
            "/data/site.db",
            f"/backup/{dest_name}",
        ],
        check=True,
    )
    print(f"wrote {dest_dir / dest_name}", flush=True)
    return 0


def main(argv: list[str]) -> int:
    parser = argparse.ArgumentParser(prog="rocci-ops origin")
    sub = parser.add_subparsers(dest="command", required=True)
    pub = sub.add_parser("publish")
    pub.add_argument("sha")
    up_p = sub.add_parser("up")
    up_p.add_argument("dist_dir")
    up_p.add_argument("islands_bin")
    bak = sub.add_parser("backup")
    bak.add_argument("dest_dir", nargs="?", default="/var/backups/rocci")
    ns = parser.parse_args(argv)
    if ns.command == "publish":
        return publish(ns.sha)
    if ns.command == "up":
        return up(
            Path(ns.dist_dir),
            Path(ns.islands_bin),
        )
    if ns.command == "backup":
        return backup(Path(ns.dest_dir))
    raise SystemExit(2)
