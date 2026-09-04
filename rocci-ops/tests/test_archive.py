from hashlib import sha256

from rocci_ops.archive import (
    archive_stem,
    collect_release_artifacts,
    package_platform_bundle,
    release_params,
    version_from_ref,
)
from rocci_ops.ghutil import DEFAULT_CHECKS, parse_check_line, wait_for_check


def test_tag_version() -> None:
    version, prerelease = version_from_ref("tag", "v1.2.3", "abcdef012345")
    assert version == "v1.2.3"
    assert prerelease is False


def test_dev_tag_is_prerelease() -> None:
    version, prerelease = version_from_ref("tag", "dev", "abcdef012345")
    assert version == "dev-abcdef0"
    assert prerelease is True
    tag, name, pre = release_params("tag", "dev", "abcdef012345")
    assert tag == "dev"
    assert name == "Development Build (abcdef0)"
    assert pre is True


def test_dev_version_uses_short_sha() -> None:
    version, prerelease = version_from_ref("branch", "main", "abcdef012345")
    assert version == "dev-abcdef0"
    assert prerelease is True


def test_archive_stem() -> None:
    assert archive_stem("dev-abcdef0", "x86_64-unknown-linux-gnu") == (
        "rocci-dev-abcdef0-x86_64-unknown-linux-gnu"
    )


def test_release_params_dev() -> None:
    tag, name, prerelease = release_params("branch", "main", "abcdef012345")
    assert tag == "dev"
    assert name == "Development Build (abcdef0)"
    assert prerelease is True


def test_parse_check_line() -> None:
    assert parse_check_line("completed success") == ("completed", "success")
    assert parse_check_line("") is None


def test_default_checks_include_lint_and_workspace_tests() -> None:
    assert DEFAULT_CHECKS == (
        "Code Formatting & Lints",
        "Test Workspace (macos-latest)",
        "Test Workspace (ubuntu-latest)",
    )


def test_wait_for_check_success() -> None:
    calls = {"n": 0}

    def gh(_args: list[str]) -> str:
        calls["n"] += 1
        if calls["n"] == 1:
            return ""
        return "completed success\n"

    sleeps: list[float] = []
    wait_for_check(
        repo="owner/rocci",
        sha="abc",
        check="Test Workspace (ubuntu-latest)",
        gh=gh,
        sleep=sleeps.append,
    )
    assert sleeps == [30]
    assert calls["n"] == 2


def test_package_platform_bundle_copies_and_checksums(tmp_path) -> None:
    crate = tmp_path / "crates" / "rocci-platform"
    crate.mkdir(parents=True)
    hashed = crate / "rocci-hash.tar.zst"
    hashed.write_bytes(b"platform-bytes")
    dest = package_platform_bundle(tmp_path)
    assert dest.name == "rocci-platform.tar.zst"
    assert dest.read_bytes() == b"platform-bytes"
    checksum = dest.with_name(dest.name + ".sha256")
    digest = checksum.read_text(encoding="utf-8").split()[0]
    assert digest == sha256(b"platform-bytes").hexdigest()


def test_package_platform_bundle_fails_without_archive(tmp_path) -> None:
    (tmp_path / "crates" / "rocci-platform").mkdir(parents=True)
    try:
        package_platform_bundle(tmp_path)
    except SystemExit as exc:
        assert "bundle.sh" in str(exc)
    else:
        raise AssertionError("expected SystemExit")


def test_package_platform_bundle_rejects_empty_archive(tmp_path) -> None:
    crate = tmp_path / "crates" / "rocci-platform"
    crate.mkdir(parents=True)
    (crate / "empty.tar.zst").write_bytes(b"")
    try:
        package_platform_bundle(tmp_path)
    except SystemExit as exc:
        assert "empty" in str(exc)
    else:
        raise AssertionError("expected SystemExit")


def test_collect_release_artifacts_includes_platform_tar_zst(tmp_path) -> None:
    (tmp_path / "rocci-dev-linux.tar.gz").write_bytes(b"cli")
    (tmp_path / "rocci-dev-linux.tar.gz.sha256").write_text("abc  rocci-dev-linux.tar.gz\n")
    (tmp_path / "rocci-platform.tar.zst").write_bytes(b"pf")
    (tmp_path / "rocci-platform.tar.zst.sha256").write_text("def  rocci-platform.tar.zst\n")
    names = [path.name for path in collect_release_artifacts(tmp_path)]
    assert "rocci-platform.tar.zst" in names
    assert "rocci-dev-linux.tar.gz" in names
    assert "rocci-platform.tar.zst.sha256" in names


def test_collect_release_artifacts_requires_platform_tar_zst(tmp_path) -> None:
    (tmp_path / "rocci-dev-linux.tar.gz").write_bytes(b"cli")
    (tmp_path / "rocci-dev-linux.tar.gz.sha256").write_text("abc  rocci-dev-linux.tar.gz\n")
    try:
        collect_release_artifacts(tmp_path)
    except SystemExit as exc:
        assert "tar.zst" in str(exc)
    else:
        raise AssertionError("expected SystemExit")
