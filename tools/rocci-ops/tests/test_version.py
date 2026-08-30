from pathlib import Path

from rocci_ops.paths import repo_root
from rocci_ops.version import (
    apply_release_version,
    crate_versions,
    first_package_version,
    next_release_version,
    parse_release_version,
    release_files_match,
    replace_lock_crate_versions,
    replace_package_versions,
    workspace_crate_names,
)

MINIMAL_CARGO = """\
[workspace]
members = [
    "crates/rocci-cli",
    "crates/rocci-core",
]

[workspace.package]
version = "0.1.0"
"""

MINIMAL_LOCK = """\
[[package]]
name = "rocci-cli"
version = "0.1.0"

[[package]]
name = "rocci-core"
version = "0.1.0"
"""


def test_parse_release_version() -> None:
    assert parse_release_version("v1.2.3") == "1.2.3"
    try:
        parse_release_version("1.2.3")
    except SystemExit as exc:
        assert "v*" in str(exc)
    else:
        raise AssertionError("expected SystemExit")
    try:
        parse_release_version("vnext")
    except SystemExit as exc:
        assert "vX.Y.Z" in str(exc)
    else:
        raise AssertionError("expected SystemExit")


def test_next_release_version() -> None:
    assert next_release_version("0.1.2", "patch") == "0.1.3"
    assert next_release_version("0.1.2", "minor") == "0.2.0"
    assert next_release_version("0.1.2", "major") == "1.0.0"
    try:
        next_release_version("1.2", "patch")
    except SystemExit as exc:
        assert "X.Y.Z" in str(exc)
    else:
        raise AssertionError("expected SystemExit")


def test_workspace_crate_names() -> None:
    assert workspace_crate_names(MINIMAL_CARGO) == ("rocci-cli", "rocci-core")


def test_crate_versions_require_lock_agreement() -> None:
    assert crate_versions(MINIMAL_CARGO, MINIMAL_LOCK) == "0.1.0"
    try:
        crate_versions(MINIMAL_CARGO, '[[package]]\nname = "rocci-cli"\nversion = "0.1.0"\n')
    except SystemExit as exc:
        assert "rocci-core" in str(exc)
    else:
        raise AssertionError("expected SystemExit")


def test_replace_package_versions_skips_dependency_tables() -> None:
    text = '[workspace.package]\nversion = "0.1.0"\n\nclap = { version = "4.5" }\n'
    updated = replace_package_versions(text, "2.0.0")
    assert first_package_version(updated) == "2.0.0"
    assert 'clap = { version = "4.5" }' in updated


def test_replace_lock_crate_versions() -> None:
    updated = replace_lock_crate_versions(MINIMAL_LOCK, "3.1.4", ("rocci-cli", "rocci-core"))
    assert updated.count('version = "3.1.4"') == 2


def test_apply_release_version(tmp_path: Path) -> None:
    (tmp_path / "Cargo.toml").write_text(MINIMAL_CARGO, encoding="utf-8")
    (tmp_path / "Cargo.lock").write_text(MINIMAL_LOCK, encoding="utf-8")
    apply_release_version(tmp_path, "9.8.7")
    assert release_files_match(tmp_path, "9.8.7")
    assert not release_files_match(tmp_path, "0.1.0")


def test_checked_in_crate_versions_match() -> None:
    root = repo_root()
    version = first_package_version((root / "Cargo.toml").read_text(encoding="utf-8"))
    assert version
    assert release_files_match(root, version)
