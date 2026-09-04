import os
import platform
import shutil
from pathlib import Path

from rocci_ops.paths import repo_root
from rocci_ops.util import run

CLI_CRATES = (
    ("rocci-cli", "rocci"),
    ("rocci-rocdown-cli", "rocdown"),
)

INSTALL_USAGE = "usage: rocci-ops install cli|vscode|cursor"


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
        installed = dest / binary
        installed.chmod(0o755)
        resign_copied_cli(installed)
    print("\nInstalled:")
    for _, binary in CLI_CRATES:
        print(f"  {dest / binary}")
    path = os.environ.get("PATH", "")
    if str(dest) not in path.split(os.pathsep):
        print(f"\n  Note: '{dest}' is not on your PATH.")
    return 0


def resign_copied_cli(path: Path) -> None:
    if platform.system() != "Darwin":
        return
    run(
        [
            "/usr/bin/codesign",
            "--sign",
            "-",
            "--force",
            "--timestamp=none",
            str(path),
        ]
    )


def latest_vsix(root: Path | None = None) -> Path:
    vsix = list(((root or repo_root()) / "editors" / "vscode").glob("rocci-*.vsix"))
    if not vsix:
        raise SystemExit("no rocci-*.vsix in editors/vscode; run `rocci-ops package vscode` first")
    return max(vsix, key=lambda path: path.stat().st_mtime)


def install_vscode_extension() -> int:
    vsix = latest_vsix()
    run(["code", "--install-extension", str(vsix)])
    return 0


def install_cursor_extension() -> int:
    vsix = latest_vsix()
    run(
        [
            "code",
            "--extensions-dir",
            str(Path.home() / ".cursor" / "extensions"),
            "--install-extension",
            str(vsix),
        ]
    )
    return 0


def install_command(argv: list[str]) -> int:
    if not argv or argv[0] in ("-h", "--help"):
        raise SystemExit(INSTALL_USAGE)
    sub, rest = argv[0], argv[1:]
    if rest:
        raise SystemExit(INSTALL_USAGE)
    if sub == "cli":
        return install_cli()
    if sub == "vscode":
        return install_vscode_extension()
    if sub == "cursor":
        return install_cursor_extension()
    raise SystemExit(INSTALL_USAGE)
