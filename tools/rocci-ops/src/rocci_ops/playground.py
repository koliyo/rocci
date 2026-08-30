from __future__ import annotations

import os
import shutil
import subprocess
from pathlib import Path

from rocci_ops.paths import repo_root
from rocci_ops.util import run


def playground_wasm_artifact(root: Path) -> Path:
    rel = Path("wasm32-unknown-unknown") / "release" / "rocci_playground_wasm.wasm"
    candidates = []
    env_dir = os.environ.get("CARGO_TARGET_DIR")
    if env_dir:
        candidates.append(Path(env_dir) / rel)
    candidates.append(root / "target" / rel)
    for path in candidates:
        if path.is_file() and path.stat().st_size > 0:
            return path
    looked = ", ".join(str(path) for path in candidates)
    raise SystemExit(f"error: playground WASM artifact not found; looked in: {looked}")


def ensure_wasm32_unknown_unknown() -> None:
    listed = subprocess.run(
        ["rustup", "target", "list", "--installed"],
        check=True,
        capture_output=True,
        text=True,
    )
    if "wasm32-unknown-unknown" not in listed.stdout.splitlines():
        run(["rustup", "target", "add", "wasm32-unknown-unknown"])


def _require_playground_dist(dist: Path) -> None:
    missing: list[str] = []
    for name in ("app.js", "compiler-worker.js", "styles.css", "compiler.wasm"):
        path = dist / name
        if not path.is_file() or path.stat().st_size == 0:
            missing.append(name)
    if missing:
        raise SystemExit(
            "error: playground dist missing or empty after build: " + ", ".join(missing)
        )
    wasm = dist / "compiler.wasm"
    if wasm.read_bytes()[:4] != b"\0asm":
        raise SystemExit("error: playground/dist/compiler.wasm is not a WebAssembly module")


def build_playground() -> int:
    root = repo_root()
    dist = root / "playground" / "dist"
    dist.mkdir(parents=True, exist_ok=True)
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
    shutil.copy2(playground_wasm_artifact(root), dist / "compiler.wasm")
    playground = root / "playground"
    if not (playground / "node_modules").is_dir():
        run(["npm", "install"], cwd=playground)
    run(["node", "build.js"], cwd=playground)
    _require_playground_dist(dist)
    print("Playground build succeeded.")
    return 0
