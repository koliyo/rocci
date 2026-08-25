from rocci_ops.release import (
    archive_stem,
    parse_check_line,
    release_params,
    version_from_ref,
    wait_for_check,
)


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
