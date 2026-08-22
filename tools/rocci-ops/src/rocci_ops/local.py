from __future__ import annotations

import argparse
import os
import platform
import shlex
import shutil
import subprocess
import sys
import tempfile
import time
from pathlib import Path

from rocci_ops.paths import repo_root

CLI_CRATES = (
    ("rocci-cli", "rocci"),
    ("rocci-rocdown-cli", "rocdown"),
    ("rocci-okf", "rocci-okf"),
)

# Temporary site-packaging workaround for Roc's optimized backend recursion.
SITE_ROC_OPT = "dev"


def run(argv: list[str], *, cwd: Path | None = None, env: dict[str, str] | None = None) -> None:
    started = time.monotonic()
    print(
        f"[rocci-ops] phase=command status=start command={shlex.join(argv)}",
        flush=True,
    )
    try:
        subprocess.run(argv, cwd=cwd or repo_root(), env=env, check=True)
    except subprocess.CalledProcessError:
        elapsed_ms = int((time.monotonic() - started) * 1000)
        print(f"[rocci-ops] phase=command status=failed elapsed_ms={elapsed_ms}", flush=True)
        raise
    elapsed_ms = int((time.monotonic() - started) * 1000)
    print(f"[rocci-ops] phase=command status=done elapsed_ms={elapsed_ms}", flush=True)


def require_darwin(kind: str) -> None:
    if platform.system() != "Darwin":
        raise SystemExit(f"{kind} can only be built on macOS.")


def install_cli(*, dest: Path | None = None) -> int:
    root = repo_root()
    dest = dest or Path.home() / ".local" / "bin"
    print(f"Rocci CLI installer\n  Source: {root}\n  Destination: {dest}\n")
    if not dest.is_dir():
        answer = input(f"  '{dest}' does not exist. Create it? [y/N] ")
        if answer.strip().lower() not in {"y", "yes"}:
            print("  Aborted.")
            return 1
        dest.mkdir(parents=True)
    if not os.access(dest, os.W_OK):
        raise SystemExit(f"  Error: '{dest}' is not writable.")
    for crate, binary in CLI_CRATES:
        run(["cargo", "build", "--release", "-p", crate], cwd=root)
        src = root / "target" / "release" / binary
        if not src.is_file():
            raise SystemExit(f"  Error: expected binary not found at '{src}'")
        shutil.copy2(src, dest / binary)
        (dest / binary).chmod(0o755)
    print("\nInstalled:")
    for _, binary in CLI_CRATES:
        print(f"  {dest / binary}")
    path = os.environ.get("PATH", "")
    if str(dest) not in path.split(os.pathsep):
        print(f"\n  Note: '{dest}' is not on your PATH.")
    return 0


def package_vscode() -> int:
    root = repo_root()
    run(["cargo", "build", "-p", "rocci-rocdown-lsp", "--release"], cwd=root)
    dist = root / "editors" / "vscode" / "dist"
    if dist.exists():
        shutil.rmtree(dist)
    bin_dir = dist / "bin"
    bin_dir.mkdir(parents=True)
    name = "rocci-language-server.exe" if os.name == "nt" else "rocci-language-server"
    shutil.copy2(root / "target" / "release" / name, bin_dir / name)
    run(["npm", "install"], cwd=root / "editors" / "vscode")
    run(["npm", "run", "vscode:package"], cwd=root / "editors" / "vscode")
    return 0


def package_zed() -> int:
    root = repo_root()
    run(["cargo", "build", "-p", "rocci-rocdown-lsp", "--release"], cwd=root)
    run(["cargo", "build", "--target", "wasm32-wasip2", "--release"], cwd=root / "editors" / "zed")
    wasm = root / "editors" / "zed" / "target" / "wasm32-wasip2" / "release" / "rocci.wasm"
    if os.environ.get("CARGO_TARGET_DIR"):
        wasm = Path(os.environ["CARGO_TARGET_DIR"]) / "wasm32-wasip2" / "release" / "rocci.wasm"
    dest = root / "editors" / "zed" / "extension.wasm"
    shutil.copy2(wasm, dest)
    print(f"Packaged Zed extension to {dest}")
    return 0


def verify_zed() -> int:
    root = repo_root()
    required = [
        root / "editors" / "zed" / "extension.toml",
        root / "editors" / "zed" / "languages" / "rocci" / "config.toml",
        root / "editors" / "zed" / "languages" / "rocdown" / "config.toml",
        root / "editors" / "zed" / "icons" / "rocci-file.svg",
        root / "editors" / "zed" / "icons" / "rocci-file-light.svg",
        root / "editors" / "zed" / "icon_themes" / "rocci.json",
    ]
    for path in required:
        if not path.is_file():
            raise SystemExit(f"Missing {path.name}")
    text = (root / "editors" / "zed" / "extension.toml").read_text(encoding="utf-8")
    if 'languages = ["Rocci", "Rocdown"]' not in text:
        raise SystemExit("extension.toml must attach the language server to Rocci and Rocdown")
    theme = (root / "editors" / "zed" / "icon_themes" / "rocci.json").read_text(
        encoding="utf-8"
    )
    if '"rocci"' not in theme or '"rocdown"' not in theme:
        raise SystemExit("icon theme must map the rocci and rocdown suffixes")
    run(["cargo", "build", "-p", "rocci-rocdown-lsp"], cwd=root)
    ls_dir = Path(os.environ.get("CARGO_TARGET_DIR") or root / "target")
    if not (ls_dir / "debug" / "rocci-language-server").is_file():
        raise SystemExit("Missing rocci-language-server binary")
    run(["cargo", "build", "--target", "wasm32-wasip2", "--release"], cwd=root / "editors" / "zed")
    return 0


def bundle_macos() -> int:
    require_darwin("The macOS app bundle")
    run(["cargo", "run", "-p", "rocci-cli", "--", "bundle", "--config", "rocci.toml"])
    return 0


def install_cursor_extension() -> int:
    root = repo_root()
    vsix = list((root / "editors" / "vscode").glob("rocci-*.vsix"))
    if not vsix:
        raise SystemExit("no rocci-*.vsix in editors/vscode")
    run(
        [
            "code",
            "--extensions-dir",
            str(Path.home() / ".cursor" / "extensions"),
            "--install-extension",
            str(vsix[0]),
        ]
    )
    return 0


def ensure_wasm32_unknown_unknown() -> None:
    listed = subprocess.run(
        ["rustup", "target", "list", "--installed"],
        check=True,
        capture_output=True,
        text=True,
    )
    if "wasm32-unknown-unknown" not in listed.stdout.splitlines():
        run(["rustup", "target", "add", "wasm32-unknown-unknown"])


def build_playground() -> int:
    root = repo_root()
    dist = root / "playground" / "dist"
    dist.mkdir(parents=True, exist_ok=True)
    for name in ("app.js", "compiler-worker.js", "styles.css", "compiler.wasm"):
        (dist / name).touch()
    ensure_wasm32_unknown_unknown()
    run(
        [
            "cargo",
            "build",
            "-p",
            "rocci-playground-wasm",
            "--target",
            "wasm32-unknown-unknown",
            "--release",
        ],
        cwd=root,
    )
    playground = root / "playground"
    if not (playground / "node_modules").is_dir():
        run(["npm", "install"], cwd=playground)
    run(["node", "build.js"], cwd=playground)
    print("Playground build succeeded.")
    return 0


def render_brand_icons() -> int:
    root = repo_root()
    if shutil.which("rsvg-convert") is None:
        raise SystemExit("rsvg-convert is required (librsvg).")
    brand = root / "brand"
    run(
        [
            "rsvg-convert",
            "-w",
            "1024",
            "-h",
            "1024",
            str(brand / "rocci-app.svg"),
            "-o",
            str(root / "crates/rocci-desktop/assets/rocci-icon.png"),
        ],
        cwd=root,
    )
    run(
        [
            "rsvg-convert",
            "-w",
            "1024",
            "-h",
            "1024",
            str(brand / "rocci-file.svg"),
            "-o",
            str(brand / "rocci-file.png"),
        ],
        cwd=root,
    )
    run(
        [
            "rsvg-convert",
            "-w",
            "180",
            "-h",
            "180",
            str(brand / "rocci-app.svg"),
            "-o",
            str(root / "site/assets/apple-touch-icon.png"),
        ],
        cwd=root,
    )
    shutil.copy2(brand / "rocci-mark.svg", root / "site/assets/favicon.svg")
    (root / "editors/vscode/icons").mkdir(parents=True, exist_ok=True)
    (root / "editors/zed/icons").mkdir(parents=True, exist_ok=True)
    linux_mime = root / "packaging/linux/icons/hicolor/scalable/mimetypes"
    linux_mime.mkdir(parents=True, exist_ok=True)
    shutil.copy2(brand / "rocci-file.svg", root / "editors/vscode/icons/rocci-file.svg")
    shutil.copy2(brand / "rocci-file-light.svg", root / "editors/vscode/icons/rocci-file-light.svg")
    shutil.copy2(brand / "rocci-file.svg", root / "editors/zed/icons/rocci-file.svg")
    shutil.copy2(brand / "rocci-file-light.svg", root / "editors/zed/icons/rocci-file-light.svg")
    shutil.copy2(brand / "rocci-file.svg", linux_mime / "text-x-rocci.svg")
    shutil.copy2(brand / "rocci-file.svg", linux_mime / "text-x-rocdown.svg")
    _render_macos_icns(root, brand)
    print(f"Rendered brand icons from {brand}")
    return 0


ICONSET_SIZES = (
    ("icon_16x16.png", 16),
    ("icon_16x16@2x.png", 32),
    ("icon_32x32.png", 32),
    ("icon_32x32@2x.png", 64),
    ("icon_128x128.png", 128),
    ("icon_128x128@2x.png", 256),
    ("icon_256x256.png", 256),
    ("icon_256x256@2x.png", 512),
    ("icon_512x512.png", 512),
    ("icon_512x512@2x.png", 1024),
)


def _render_macos_icns(root: Path, brand: Path) -> None:
    if shutil.which("iconutil") is None:
        print("Skipped .icns generation: iconutil not found.")
        return
    with tempfile.TemporaryDirectory(prefix="rocci-iconset-") as tmp:
        iconset = Path(tmp) / "rocci.iconset"
        iconset.mkdir()
        svg = brand / "rocci-app.svg"
        for name, px in ICONSET_SIZES:
            run(
                [
                    "rsvg-convert",
                    "-w",
                    str(px),
                    "-h",
                    str(px),
                    str(svg),
                    "-o",
                    str(iconset / name),
                ],
                cwd=root,
            )
        run(
            [
                "iconutil",
                "-c",
                "icns",
                "-o",
                str(brand / "rocci-app.icns"),
                str(iconset),
            ],
            cwd=root,
        )


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


def _git(root: Path, *args: str, check: bool = True) -> subprocess.CompletedProcess:
    return subprocess.run(["git", "-C", str(root), *args], check=check, capture_output=True, text=True)


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
        # Use the dev backend for every live site artifact until the pinned Roc
        # nightly's optimized backend is safe for the site applications.
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
        print(f"[rocci-ops] phase=live-app status=done app={app_id}", flush=True)
    for docs_only in ("counter", "styling", "blocks", "snake"):
        if (live_root / docs_only).exists():
            raise SystemExit(f"error: docs-only id `{docs_only}` must not be in {live_root}")
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


def parse_worktrees(porcelain: str) -> list[tuple[str, str | None]]:
    entries: list[tuple[str, str | None]] = []
    path = ""
    branch: str | None = None
    for line in porcelain.splitlines():
        if line.startswith("worktree "):
            if path:
                entries.append((path, branch))
            path = line[len("worktree ") :]
            branch = None
        elif line.startswith("branch "):
            branch = line[len("branch ") :]
    if path:
        entries.append((path, branch))
    return entries


def push_worktrees(*, remote: str | None, dry_run: bool) -> int:
    root = repo_root()
    if remote is None:
        up = _git(root, "rev-parse", "--abbrev-ref", "--symbolic-full-name", "@{upstream}", check=False)
        remote = up.stdout.strip().split("/", 1)[0] if up.returncode == 0 and up.stdout.strip() else "origin"
    if _git(root, "remote", "get-url", remote, check=False).returncode != 0:
        raise SystemExit(f"Remote '{remote}' is not configured in {root}")
    listed = _git(root, "worktree", "list", "--porcelain")
    pushed = skipped = 0
    for path, branch_ref in parse_worktrees(listed.stdout):
        if not branch_ref:
            print(f"Skipping {path} (detached HEAD)")
            skipped += 1
            continue
        branch_name = branch_ref.removeprefix("refs/heads/")
        worktree = Path(path)
        if _git(worktree, "rev-parse", "--verify", "HEAD", check=False).returncode != 0:
            print(f"Skipping {path} (no HEAD)")
            skipped += 1
            continue
        up = _git(
            worktree,
            "rev-parse",
            "--abbrev-ref",
            "--symbolic-full-name",
            "@{upstream}",
            check=False,
        )
        if up.returncode == 0 and up.stdout.strip():
            ahead = _git(worktree, "rev-list", "--count", f"{up.stdout.strip()}..HEAD")
            if ahead.stdout.strip() == "0":
                print(f"Skipping {branch_name} ({path}): no commits ahead of {up.stdout.strip()}")
                skipped += 1
                continue
            argv = ["git", "-C", path, "push", remote, f"HEAD:{branch_name}"]
        else:
            argv = ["git", "-C", path, "push", "-u", remote, f"HEAD:{branch_name}"]
        if dry_run:
            print("  " + " ".join(argv))
        else:
            subprocess.run(argv, check=True)
        pushed += 1
    print(f"\nSummary: pushed {pushed}, skipped {skipped}")
    return 0


def promote_staging() -> int:
    """Rebase staging onto main, push it, and restore the starting branch."""
    original = subprocess.run(
        ["git", "branch", "--show-current"],
        cwd=repo_root(),
        capture_output=True,
        text=True,
        check=True,
    ).stdout.strip()
    if not original:
        raise SystemExit("promote-staging requires a named starting branch")

    try:
        if original != "staging":
            run(["git", "switch", "staging"])
        run(["git", "rebase", "main"])
        run(["git", "push", "origin", "staging"])
    finally:
        if original != "staging":
            run(["git", "switch", original])
    return 0


def main(argv: list[str]) -> int:
    parser = argparse.ArgumentParser(prog="rocci-ops")
    # invoked as subcommand via cli.py with remaining args only
    if not argv:
        raise SystemExit("missing local command")
    command = argv[0]
    rest = argv[1:]
    if command == "install-cli":
        return install_cli()
    if command == "site":
        if rest:
            raise SystemExit("usage: rocci-ops site")
        return build_site()
    if command == "package":
        if rest == ["vscode"]:
            return package_vscode()
        if rest == ["zed"]:
            return package_zed()
        if rest and rest[0] == "site":
            target = "x64musl"
            extra = rest[1:]
            if extra[:1] == ["--target"] and len(extra) >= 2:
                target = extra[1]
            elif extra:
                raise SystemExit("usage: rocci-ops package site [--target x64musl]")
            return package_site(target=target)
        raise SystemExit("usage: rocci-ops package vscode|zed|site")
    if command == "verify-zed":
        return verify_zed()
    if command == "bundle":
        if rest == ["macos"]:
            return bundle_macos()
        raise SystemExit("usage: rocci-ops bundle macos")
    if command == "install-cursor-extension":
        return install_cursor_extension()
    if command == "build-playground":
        return build_playground()
    if command == "render-brand-icons":
        return render_brand_icons()
    if command == "serve":
        if len(rest) < 2:
            raise SystemExit("usage: rocci-ops serve hybrid|static|site|app ...")
        kind = rest[0]
        if kind == "hybrid":
            if len(rest) < 3:
                raise SystemExit("usage: rocci-ops serve hybrid DIST_DIR ISLANDS_BIN [compose args...]")
            return serve_hybrid(Path(rest[1]), Path(rest[2]), rest[3:])
        if kind == "static":
            return serve_static(Path(rest[1]), rest[2:])
        if kind == "site":
            return serve_site(Path(rest[1]), rest[2:])
        if kind == "app":
            return serve_app(Path(rest[1]), rest[2:])
        raise SystemExit("usage: rocci-ops serve hybrid|static|site|app ...")
    if command == "push-worktrees":
        p = argparse.ArgumentParser(prog="rocci-ops push-worktrees")
        p.add_argument("-n", "--dry-run", action="store_true")
        p.add_argument("-r", "--remote")
        ns = p.parse_args(rest)
        return push_worktrees(remote=ns.remote, dry_run=ns.dry_run)
    if command == "promote-staging":
        if rest:
            raise SystemExit("usage: rocci-ops promote-staging")
        return promote_staging()
    raise SystemExit(f"unknown local command: {command}")
