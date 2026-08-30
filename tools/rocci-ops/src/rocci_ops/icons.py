from __future__ import annotations

import shutil
import tempfile
from pathlib import Path

from rocci_ops.paths import repo_root
from rocci_ops.util import run

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
