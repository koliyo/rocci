from __future__ import annotations

import shutil
import subprocess
from pathlib import Path

from rocci_ops.paths import repo_root
from rocci_ops.playground import build_playground
from rocci_ops.util import run

SITE_ROC_OPT = "dev"


def stage_example_docs() -> None:
    root = repo_root()
    run(
        [
            "cargo",
            "run",
            "-q",
            "-p",
            "rocci-docs",
            "--",
            "--catalog",
            str(root / "examples/rocci/apps.toml"),
            "--output",
            str(root / "dist/example-docs"),
        ],
        cwd=root,
    )


def build_site() -> int:
    build_playground()
    root = repo_root()
    stage_example_docs()
    for action in ("check", "test", "build"):
        run(
            ["cargo", "run", "-q", "-p", "rocci-rocdown-cli", "--", action, "site"],
            cwd=root,
        )
    return 0


def package_site(*, target: str) -> int:
    build_playground()
    root = repo_root()
    live_root = root / "dist/examples-live"
    stage_example_docs()
    catalog = root / "examples/rocci/apps.toml"
    print(f"[rocci-ops] phase=list-live status=start catalog={catalog}", flush=True)
    listed = subprocess.run(
        [
            "cargo",
            "run",
            "-q",
            "-p",
            "rocci-docs",
            "--",
            "--catalog",
            str(catalog),
            "--print-live",
        ],
        cwd=root,
        check=True,
        capture_output=True,
        text=True,
    )
    live_entries = [line for line in listed.stdout.splitlines() if line.strip()]
    print(
        f"[rocci-ops] phase=list-live status=done count={len(live_entries)}",
        flush=True,
    )
    if live_root.exists():
        shutil.rmtree(live_root)
    live_root.mkdir(parents=True)
    for raw in live_entries:
        line = raw.strip()
        if not line:
            continue
        app_id, rel, entry = line.split("\t")
        src = root / "examples/rocci" / rel
        if entry != ".":
            src = src / entry
        dest = live_root / app_id
        opt = SITE_ROC_OPT
        print(
            f"[rocci-ops] phase=live-app status=start app={app_id} source={src} target={target}"
            f" opt={opt or 'speed'}",
            flush=True,
        )
        build_args = [
            "cargo",
            "run",
            "-q",
            "-p",
            "rocci-cli",
            "--",
            "build",
            "--release",
            str(src),
            "--target",
            target,
            "--verbose",
        ]
        if opt:
            build_args.extend(["--opt", opt])
        build_args.extend(["--output", str(dest)])
        run(build_args, cwd=root)
        if not (dest / "server").is_file():
            raise SystemExit(f"error: live app `{app_id}` did not write {dest / 'server'}")
        docker_app = root / "docker" / "app"
        shutil.copy2(docker_app / "Dockerfile", dest / "Dockerfile")
        shutil.copy2(docker_app / "entrypoint.sh", dest / "entrypoint.sh")
        if not (dest / "assets").is_dir():
            (dest / "assets").mkdir()
        print(f"[rocci-ops] phase=live-app status=done app={app_id}", flush=True)
    expected = {line.split("\t", 1)[0] for line in live_entries if line.strip()}
    found = {path.name for path in live_root.iterdir() if path.is_dir()}
    if found != expected:
        raise SystemExit(
            f"error: live root {live_root} has {sorted(found)}, expected {sorted(expected)}"
        )
    run(
        [
            "cargo",
            "run",
            "-q",
            "-p",
            "rocci-rocdown-cli",
            "--",
            "package",
            "site",
            "--target",
            target,
        ],
        cwd=root,
    )
    return 0
