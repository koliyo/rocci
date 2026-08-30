from __future__ import annotations

import os
import shutil
from pathlib import Path

from rocci_ops.icons import render_brand_icons
from rocci_ops.paths import repo_root
from rocci_ops.site import package_site
from rocci_ops.util import require_darwin, run

PACKAGE_USAGE = "usage: rocci-ops package macos|vscode|zed|site|icons"


def package_vscode() -> int:
    root = repo_root()
    dist = root / "editors" / "vscode" / "dist"
    if dist.exists():
        shutil.rmtree(dist)
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


def package_macos() -> int:
    require_darwin("The macOS app bundle")
    run(["cargo", "run", "-p", "rocci-cli", "--", "bundle", "--config", "rocci.toml"])
    return 0


def package_command(argv: list[str]) -> int:
    if not argv or argv[0] in ("-h", "--help"):
        raise SystemExit(PACKAGE_USAGE)
    sub, rest = argv[0], argv[1:]
    if sub == "okf":
        raise SystemExit(
            "Rocci Knowledge.app is no longer built here; use https://github.com/koliyo/okmate"
        )
    if sub == "macos":
        if rest:
            raise SystemExit(PACKAGE_USAGE)
        return package_macos()
    if sub == "vscode":
        if rest:
            raise SystemExit(PACKAGE_USAGE)
        return package_vscode()
    if sub == "zed":
        if rest:
            raise SystemExit(PACKAGE_USAGE)
        return package_zed()
    if sub == "icons":
        if rest:
            raise SystemExit(PACKAGE_USAGE)
        return render_brand_icons()
    if sub == "site":
        target = "x64musl"
        extra = rest
        if extra[:1] == ["--target"] and len(extra) >= 2:
            target = extra[1]
            extra = extra[2:]
        if extra:
            raise SystemExit("usage: rocci-ops package site [--target x64musl]")
        return package_site(target=target)
    raise SystemExit(PACKAGE_USAGE)
