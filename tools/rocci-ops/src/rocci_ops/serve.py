from __future__ import annotations

import os
import shutil
import tempfile
from pathlib import Path

from rocci_ops.paths import repo_root
from rocci_ops.util import run

SERVE_USAGE = "usage: rocci-ops serve hybrid|static|site|app ..."


def _compose(file_name: str, extra: list[str], env: dict[str, str]) -> int:
    root = repo_root()
    merged = os.environ.copy()
    merged.update(env)
    argv = ["docker", "compose", "-f", str(root / "docker" / file_name), "up", *extra]
    run(argv, cwd=root, env=merged)
    return 0


def serve_hybrid(dist_arg: Path, bin_arg: Path, extra: list[str]) -> int:
    if not dist_arg.is_dir():
        raise SystemExit(f"error: not a directory: {dist_arg}")
    if not bin_arg.is_file():
        raise SystemExit(f"error: not a file: {bin_arg}")
    dist = dist_arg.resolve()
    if not (dist / "index.html").is_file():
        raise SystemExit(f"error: no index.html in {dist}; package the site on the host first")
    docker = repo_root() / "docker"
    context = Path(tempfile.mkdtemp(prefix="rocci-islands-"))
    try:
        shutil.copy2(docker / "islands" / "Dockerfile", context / "Dockerfile")
        shutil.copy2(bin_arg, context / "islands")
        (context / "islands").chmod(0o755)
        return _compose(
            "compose.hybrid.yml",
            ["--build", *extra],
            {"ROCCI_DIST": str(dist), "ROCCI_ISLANDS_CONTEXT": str(context)},
        )
    finally:
        shutil.rmtree(context, ignore_errors=True)


def serve_static(dist_arg: Path, extra: list[str]) -> int:
    if not dist_arg.is_dir():
        raise SystemExit(f"error: not a directory: {dist_arg}")
    dist = dist_arg.resolve()
    if not (dist / "index.html").is_file():
        raise SystemExit(f"error: no index.html in {dist}; build the site on the host first")
    return _compose("compose.static.yml", extra, {"ROCCI_DIST": str(dist)})


def serve_site(site_arg: Path, extra: list[str]) -> int:
    if not site_arg.is_dir():
        raise SystemExit(f"error: not a directory: {site_arg}")
    site = site_arg.resolve()
    if not (site / "rocdown.toml").is_file():
        raise SystemExit(f"error: no rocdown.toml in {site}")
    return _compose("compose.yml", ["--build", *extra], {"ROCCI_SITE": str(site)})


def serve_app(dir_arg: Path, extra: list[str]) -> int:
    if not dir_arg.is_dir():
        raise SystemExit(f"error: not a directory: {dir_arg}")
    server_dir = dir_arg.resolve()
    if not (server_dir / "server").is_file():
        raise SystemExit(f"error: no server binary in {server_dir}; run `rocci build --release` first")
    docker = repo_root() / "docker"
    context = Path(tempfile.mkdtemp(prefix="rocci-app-"))
    try:
        shutil.copy2(docker / "app" / "Dockerfile", context / "Dockerfile")
        shutil.copy2(docker / "app" / "entrypoint.sh", context / "entrypoint.sh")
        shutil.copy2(server_dir / "server", context / "server")
        (context / "server").chmod(0o755)
        (context / "entrypoint.sh").chmod(0o755)
        assets = context / "assets"
        assets.mkdir()
        src_assets = server_dir / "assets"
        if src_assets.is_dir():
            shutil.copytree(src_assets, assets, dirs_exist_ok=True)
        return _compose("compose.app.yml", ["--build", *extra], {"ROCCI_APP_CONTEXT": str(context)})
    finally:
        shutil.rmtree(context, ignore_errors=True)


def serve_command(argv: list[str]) -> int:
    if len(argv) < 2:
        raise SystemExit(SERVE_USAGE)
    kind = argv[0]
    if kind == "hybrid":
        if len(argv) < 3:
            raise SystemExit("usage: rocci-ops serve hybrid DIST_DIR ISLANDS_BIN [compose args...]")
        return serve_hybrid(Path(argv[1]), Path(argv[2]), argv[3:])
    if kind == "static":
        return serve_static(Path(argv[1]), argv[2:])
    if kind == "site":
        return serve_site(Path(argv[1]), argv[2:])
    if kind == "app":
        return serve_app(Path(argv[1]), argv[2:])
    raise SystemExit(SERVE_USAGE)
