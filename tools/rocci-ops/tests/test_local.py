from rocci_ops.local import CLI_CRATES, parse_worktrees, require_darwin


def test_cli_crates() -> None:
    assert CLI_CRATES[0] == ("rocci-cli", "rocci")
    assert {binary for _, binary in CLI_CRATES} == {"rocci", "rocdown", "rocci-okf"}


def test_parse_worktrees() -> None:
    porcelain = """\
worktree /repo
HEAD abc
branch refs/heads/main

worktree /repo-feature
HEAD def
branch refs/heads/feature

worktree /repo-detach
HEAD ghi
detached
"""
    entries = parse_worktrees(porcelain)
    assert entries[0] == ("/repo", "refs/heads/main")
    assert entries[1] == ("/repo-feature", "refs/heads/feature")
    assert entries[2][0] == "/repo-detach"
    assert entries[2][1] is None


def test_require_darwin_rejects_other(monkeypatch) -> None:
    monkeypatch.setattr("rocci_ops.local.platform.system", lambda: "Linux")
    try:
        require_darwin("The macOS app bundle")
    except SystemExit as exc:
        assert "macOS" in str(exc)
    else:
        raise AssertionError("expected SystemExit")
