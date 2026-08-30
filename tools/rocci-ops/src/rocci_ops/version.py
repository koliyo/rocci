import re
from pathlib import Path

PACKAGE_VERSION_RE = re.compile(r'^version = "([^"]+)"', re.MULTILINE)
MEMBER_RE = re.compile(r'"crates/([^"]+)"')
SEMVER_RE = re.compile(r"^\d+\.\d+\.\d+(?:-[0-9A-Za-z.-]+)?$")
CORE_SEMVER_RE = re.compile(r"^(\d+)\.(\d+)\.(\d+)$")
BUMP_LEVELS = ("patch", "minor", "major")

CARGO_TOML = Path("Cargo.toml")
CARGO_LOCK = Path("Cargo.lock")
VERSION_PATHS = (CARGO_TOML, CARGO_LOCK)


def parse_release_version(tag: str) -> str:
    if not tag.startswith("v") or len(tag) < 2:
        raise SystemExit("release requires a v* name, bump level, or the movable dev tag")
    version = tag[1:]
    if not SEMVER_RE.fullmatch(version):
        raise SystemExit(f"release {tag} is not a vX.Y.Z version")
    return version


def next_release_version(current: str, level: str) -> str:
    if level not in BUMP_LEVELS:
        raise SystemExit(f"unknown bump level: {level}")
    match = CORE_SEMVER_RE.fullmatch(current)
    if not match:
        raise SystemExit(f"cannot bump {current}; expected X.Y.Z")
    major, minor, patch = (int(part) for part in match.groups())
    if level == "patch":
        return f"{major}.{minor}.{patch + 1}"
    if level == "minor":
        return f"{major}.{minor + 1}.0"
    return f"{major + 1}.0.0"


def workspace_crate_names(cargo_text: str) -> tuple[str, ...]:
    names = tuple(MEMBER_RE.findall(cargo_text))
    if not names:
        raise SystemExit("could not find workspace members")
    return names


def first_package_version(text: str) -> str | None:
    match = PACKAGE_VERSION_RE.search(text)
    return match.group(1) if match else None


def crate_versions(cargo_text: str, lock_text: str) -> str:
    cargo = first_package_version(cargo_text)
    if cargo is None:
        raise SystemExit("could not find package version")
    for name in workspace_crate_names(cargo_text):
        if f'name = "{name}"\nversion = "{cargo}"' not in lock_text:
            raise SystemExit(f"Cargo.lock does not match crate version {cargo} for {name}")
    return cargo


def replace_package_versions(text: str, version: str) -> str:
    updated, count = PACKAGE_VERSION_RE.subn(f'version = "{version}"', text)
    if count == 0:
        raise SystemExit("could not find package version")
    return updated


def replace_lock_crate_versions(text: str, version: str, names: tuple[str, ...]) -> str:
    pattern = re.compile(
        r'(^\[\[package\]\]\nname = "(?:'
        + "|".join(re.escape(name) for name in names)
        + r')"\n)version = "[^"]+"',
        re.MULTILINE,
    )
    updated, count = pattern.subn(rf'\1version = "{version}"', text)
    if count == 0:
        raise SystemExit("could not find workspace crate versions in Cargo.lock")
    return updated


def apply_release_version(root: Path, version: str) -> list[Path]:
    if not SEMVER_RE.fullmatch(version):
        raise SystemExit(f"invalid release version: {version}")
    cargo = root / CARGO_TOML
    cargo_text = cargo.read_text(encoding="utf-8")
    names = workspace_crate_names(cargo_text)
    cargo.write_text(replace_package_versions(cargo_text, version), encoding="utf-8")
    lock = root / CARGO_LOCK
    lock.write_text(
        replace_lock_crate_versions(lock.read_text(encoding="utf-8"), version, names),
        encoding="utf-8",
    )
    return [CARGO_TOML, CARGO_LOCK]


def release_files_match(root: Path, version: str) -> bool:
    cargo = root / CARGO_TOML
    lock = root / CARGO_LOCK
    if not cargo.is_file() or not lock.is_file():
        return False
    try:
        return crate_versions(cargo.read_text(encoding="utf-8"), lock.read_text(encoding="utf-8")) == version
    except SystemExit:
        return False
