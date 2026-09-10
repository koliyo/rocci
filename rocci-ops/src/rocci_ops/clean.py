import argparse
import os
import shutil
import tempfile
from dataclasses import dataclass
from datetime import datetime, timezone
from pathlib import Path

CACHE_DIR_NAME = "cursor-sandbox-cache"

CLEAN_USAGE = """\
usage: rocci-ops clean cursor-sandbox-cache [--apply] [--keep HASH] [--keep-newest N]

List Cursor Agent sandbox cache trees (hashed dirs under the platform temp
directory). Default is a dry run. --apply deletes entries that are not kept.

Keeps, in order:
  hashes named by --keep
  hashes referenced by CARGO_TARGET_DIR, CARGO_HOME, RUSTUP_HOME, or any
    environment value containing cursor-sandbox-cache
  the --keep-newest most recently modified hashes (default 1)
"""


@dataclass(frozen=True)
class CacheEntry:
    root: Path
    name: str
    path: Path
    size: int
    mtime: float


def sandbox_cache_roots(*, tmp: Path | None = None) -> list[Path]:
    seen: set[Path] = set()
    roots: list[Path] = []
    candidates = [
        (tmp or Path(os.environ.get("TMPDIR") or tempfile.gettempdir()))
        / CACHE_DIR_NAME,
        Path("/tmp") / CACHE_DIR_NAME,
    ]
    for path in candidates:
        resolved = path.resolve() if path.exists() else path
        if resolved in seen or not path.is_dir():
            continue
        seen.add(resolved)
        roots.append(path)
    return roots


def dir_size(path: Path) -> int:
    total = 0
    for dirpath, _dirnames, filenames in os.walk(path, followlinks=False):
        for name in filenames:
            file_path = Path(dirpath) / name
            try:
                total += file_path.stat().st_size
            except OSError:
                continue
    return total


def list_entries(roots: list[Path]) -> list[CacheEntry]:
    entries: list[CacheEntry] = []
    for root in roots:
        try:
            children = sorted(root.iterdir(), key=lambda p: p.name)
        except OSError:
            continue
        for child in children:
            if not child.is_dir():
                continue
            try:
                st = child.stat()
            except OSError:
                continue
            entries.append(
                CacheEntry(
                    root=root,
                    name=child.name,
                    path=child,
                    size=dir_size(child),
                    mtime=st.st_mtime,
                )
            )
    return entries


def hashes_referenced_by_env(environ: dict[str, str] | None = None) -> set[str]:
    env = os.environ if environ is None else environ
    found: set[str] = set()
    marker = f"/{CACHE_DIR_NAME}/"
    for value in env.values():
        if CACHE_DIR_NAME not in value:
            continue
        normalized = value.replace("\\", "/")
        idx = normalized.find(marker)
        if idx < 0:
            continue
        rest = normalized[idx + len(marker) :]
        name = rest.split("/", 1)[0]
        if name:
            found.add(name)
    return found


def keep_names(
    entries: list[CacheEntry],
    *,
    keep: list[str],
    keep_newest: int,
    referenced: set[str],
) -> dict[str, str]:
    reasons: dict[str, str] = {}
    for name in keep:
        reasons[name] = "keep"
    for name in referenced:
        reasons.setdefault(name, "env")
    newest = sorted(entries, key=lambda e: e.mtime, reverse=True)
    for entry in newest[: max(keep_newest, 0)]:
        reasons.setdefault(entry.name, "newest")
    return reasons


def format_bytes(n: int) -> str:
    value = float(n)
    for unit in ("B", "KiB", "MiB", "GiB", "TiB"):
        if value < 1024.0 or unit == "TiB":
            if unit == "B":
                return f"{int(value)} {unit}"
            return f"{value:.1f} {unit}"
        value /= 1024.0
    return f"{n} B"


def format_mtime(mtime: float) -> str:
    return datetime.fromtimestamp(mtime, tz=timezone.utc).strftime("%Y-%m-%d %H:%M UTC")


def parse_args(argv: list[str]) -> argparse.Namespace:
    parser = argparse.ArgumentParser(
        prog="rocci-ops clean",
        usage=CLEAN_USAGE.splitlines()[0].removeprefix("usage: "),
        description=CLEAN_USAGE,
        formatter_class=argparse.RawDescriptionHelpFormatter,
    )
    parser.add_argument(
        "target",
        nargs="?",
        help="cleanup target (only cursor-sandbox-cache)",
    )
    parser.add_argument(
        "--apply",
        action="store_true",
        help="delete dropped cache directories (default: dry run)",
    )
    parser.add_argument(
        "--keep",
        action="append",
        default=[],
        metavar="HASH",
        help="keep this hashed cache directory (repeatable)",
    )
    parser.add_argument(
        "--keep-newest",
        type=int,
        default=1,
        metavar="N",
        help="keep the N most recently modified hashes (default: 1)",
    )
    return parser.parse_args(argv)


def clean_cursor_sandbox_cache(
    *,
    apply: bool,
    keep: list[str],
    keep_newest: int,
    tmp: Path | None = None,
    environ: dict[str, str] | None = None,
) -> int:
    if keep_newest < 0:
        raise SystemExit("--keep-newest must be >= 0")
    roots = sandbox_cache_roots(tmp=tmp)
    if not roots:
        print(f"no {CACHE_DIR_NAME} directory under TMPDIR or /tmp")
        return 0
    entries = list_entries(roots)
    referenced = hashes_referenced_by_env(environ)
    reasons = keep_names(
        entries, keep=keep, keep_newest=keep_newest, referenced=referenced
    )
    drop_bytes = 0
    drop_count = 0
    for root in roots:
        print(f"root: {root}")
    if not entries:
        print("  (empty)")
        return 0
    width = max(len(e.name) for e in entries)
    for entry in sorted(entries, key=lambda e: (-e.size, e.name)):
        reason = reasons.get(entry.name)
        action = f"keep ({reason})" if reason else "drop"
        print(
            f"  {entry.name:<{width}}  {format_bytes(entry.size):>10}  "
            f"{format_mtime(entry.mtime)}  {action}"
        )
        if reason is None:
            drop_bytes += entry.size
            drop_count += 1
            if apply:
                shutil.rmtree(entry.path)
    if apply:
        print(f"deleted {format_bytes(drop_bytes)} in {drop_count} director{'y' if drop_count == 1 else 'ies'}")
    else:
        print(
            f"dry-run: would delete {format_bytes(drop_bytes)} in {drop_count} "
            f"director{'y' if drop_count == 1 else 'ies'}; pass --apply to remove"
        )
    return 0


def main(argv: list[str]) -> int:
    if not argv or argv[0] in ("-h", "--help"):
        print(CLEAN_USAGE, end="" if CLEAN_USAGE.endswith("\n") else "\n")
        return 0 if argv else 2
    ns = parse_args(argv)
    if ns.target != "cursor-sandbox-cache":
        print(CLEAN_USAGE, end="" if CLEAN_USAGE.endswith("\n") else "\n")
        return 2
    return clean_cursor_sandbox_cache(
        apply=ns.apply,
        keep=ns.keep,
        keep_newest=ns.keep_newest,
    )
