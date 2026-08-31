import argparse
import os
import shutil
import subprocess
import tarfile
import time
import urllib.error
import urllib.request
from pathlib import Path

from rocci_ops.lanes import resolved_lane, should_publish_live
from rocci_ops.paths import repo_root
from rocci_ops.sshutil import validate_sha


def origin_root() -> Path:
    return Path(resolved_lane().origin_root)


def http_port() -> str:
    return resolved_lane().http_port


def keep_releases() -> int:
    return int(os.environ.get("ROCCI_KEEP_RELEASES", "5"))


def compose_file() -> Path:
    return repo_root() / "docker" / "compose.hybrid.yml"


def origin_examples_compose() -> Path:
    return repo_root() / "docker" / "compose.origin.yml"


def live_app_env_key(app_id: str) -> str:
    return "ROCCI_" + app_id.replace("-", "_").upper() + "_CONTEXT"


def live_app_ids(live_root: Path) -> list[str]:
    if not live_root.is_dir():
        return []
    ids = [
        path.name
        for path in sorted(live_root.iterdir())
        if path.is_dir() and (path / "server").is_file()
    ]
    return ids


def compose_project_dir() -> Path:
    return repo_root() / "docker"


def compose_env(root: Path, live_ids: list[str] | None = None) -> dict[str, str]:
    cfg = resolved_lane()
    env = os.environ.copy()
    env["COMPOSE_PROJECT_NAME"] = cfg.compose_project
    env["ROCCI_DIST"] = str(root / "dist")
    env["ROCCI_ISLANDS_CONTEXT"] = str(root / "islands-context")
    env["ROCCI_HTTP_PORT"] = cfg.http_port
    env["ROCCI_IMAGE_TAG"] = cfg.image_tag
    live_root = root / "examples-live"
    for app_id in live_ids if live_ids is not None else live_app_ids(live_root):
        env[live_app_env_key(app_id)] = str(live_root / app_id)
    return env


def compose_up(root: Path, *, runner=subprocess.run) -> None:
    live_ids = live_app_ids(root / "examples-live")
    if not should_publish_live(live_ids):
        live_ids = []
    env = compose_env(root, live_ids)
    argv = [
        "docker",
        "compose",
        "-f",
        str(compose_file()),
    ]
    if live_ids:
        extra = origin_examples_compose()
        if not extra.is_file():
            raise SystemExit(f"error: missing {extra}")
        argv.extend(["-f", str(extra)])
    argv.extend(
        [
            "--project-directory",
            str(compose_project_dir()),
            "up",
            "-d",
            "--build",
            "--remove-orphans",
        ]
    )
    result = runner(argv, env=env, check=False)
    if result.returncode != 0:
        raise SystemExit("error: docker compose failed")


def _direct_open(target: str | urllib.request.Request, timeout: float = 5):
    opener = urllib.request.build_opener(urllib.request.ProxyHandler({}))
    return opener.open(target, timeout=timeout)


def health_probe(
    url: str,
    *,
    headers: dict[str, str] | None = None,
    fetch=_direct_open,
) -> tuple[bool, str]:
    try:
        target: str | urllib.request.Request = url
        if headers:
            target = urllib.request.Request(url, headers=headers)
        with fetch(target, timeout=5) as response:
            status = getattr(response, "status", 200)
            if status == 200:
                return True, "200"
            return False, str(status)
    except urllib.error.HTTPError as exc:
        return False, str(exc.code)
    except (urllib.error.URLError, TimeoutError, OSError) as exc:
        reason = getattr(exc, "reason", exc)
        return False, f"{type(exc).__name__}:{reason}"


def health_ok(
    url: str,
    *,
    headers: dict[str, str] | None = None,
    fetch=_direct_open,
) -> bool:
    ok, _detail = health_probe(url, headers=headers, fetch=fetch)
    return ok


def example_public_hosts(app_id: str) -> tuple[str, ...]:
    cfg = resolved_lane()
    if cfg.name == "production" or cfg.publish_live is False:
        return ()
    hosts = (
        f"{app_id}-example-staging.rocci.dev",
        f"{app_id}.examples.localhost",
    )
    if cfg.name == "staging":
        return hosts
    return hosts + (f"{app_id}-example.rocci.dev",)


def health_checks(live_ids: list[str] | None = None) -> list[tuple[str, dict[str, str]]]:
    port = http_port()
    site = f"http://127.0.0.1:{port}/health"
    checks = [(site, {})]
    for app_id in live_ids or []:
        checks.append((f"http://127.0.0.1:{port}/play/{app_id}/health", {}))
    for app_id in live_ids or []:
        for host in example_public_hosts(app_id):
            checks.append((site, {"Host": host}))
    return checks


def health_urls() -> list[str]:
    return [url for url, _headers in health_checks()]


def wait_health(
    *,
    live_ids: list[str] | None = None,
    attempts: int = 36,
    delay: float = 5.0,
    fetch=_direct_open,
    sleeper=time.sleep,
) -> bool:
    checks = health_checks(live_ids)
    for index in range(1, attempts + 1):
        failures: list[str] = []
        for url, headers in checks:
            ok, detail = health_probe(url, headers=headers or None, fetch=fetch)
            if not ok:
                host = headers.get("Host", "")
                extra = f" Host={host}" if host else ""
                failures.append(f"{url}{extra} {detail}")
        if not failures:
            print(f"health {index}/{attempts} 200", flush=True)
            return True
        print(f"health {index}/{attempts} fail: {'; '.join(failures)}", flush=True)
        sleeper(delay)
    return False


def ensure_app_docker_context(app_dir: Path, docker_app: Path) -> None:
    if not (app_dir / "Dockerfile").is_file():
        shutil.copy2(docker_app / "Dockerfile", app_dir / "Dockerfile")
    if not (app_dir / "entrypoint.sh").is_file():
        shutil.copy2(docker_app / "entrypoint.sh", app_dir / "entrypoint.sh")
    assets = app_dir / "assets"
    if not assets.is_dir():
        assets.mkdir(parents=True)


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
    src_live = incoming / "examples-live"
    dest_live = release / "examples-live"
    if dest_live.exists():
        shutil.rmtree(dest_live)
    if src_live.is_dir():
        shutil.copytree(src_live, dest_live)
        docker_app = docker / "app"
        for app_dir in dest_live.iterdir():
            if app_dir.is_dir() and (app_dir / "server").is_file():
                ensure_app_docker_context(app_dir, docker_app)


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
    live_ids = live_app_ids(release / "examples-live")
    if not should_publish_live(live_ids):
        live_ids = []
    if not wait_health(live_ids=live_ids, fetch=fetch, sleeper=sleeper):
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
    project = resolved_lane().compose_project
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
