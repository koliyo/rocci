from pathlib import Path
from types import SimpleNamespace

import pytest

from rocci_ops.pr_checkout import checkout_pr, list_open_prs, local_pr_branch, main, parse_pr_ref


def test_parse_pr_number_and_hash() -> None:
    assert parse_pr_ref("39").number == 39
    assert parse_pr_ref("#39").number == 39
    assert parse_pr_ref("  #39  ").branch is None


def test_parse_github_pr_url() -> None:
    url = "https://github.com/koliyo/rocci/pull/39"
    assert parse_pr_ref(url).number == 39
    assert parse_pr_ref(f"{url}/files").number == 39
    assert parse_pr_ref("http://github.com/koliyo/rocci/pull/7/commits/abc").number == 7


def test_parse_branch_path() -> None:
    ref = parse_pr_ref("feat/example-source-sidebar")
    assert ref.number is None
    assert ref.branch == "feat/example-source-sidebar"
    assert parse_pr_ref("refs/heads/feat/foo").branch == "feat/foo"


def test_local_pr_branch_prefixes_once() -> None:
    assert local_pr_branch("feat/example-source-sidebar") == "pr/feat/example-source-sidebar"
    assert local_pr_branch("pr/feat/example-source-sidebar") == "pr/feat/example-source-sidebar"
    assert local_pr_branch("refs/heads/fix/typo") == "pr/fix/typo"


def test_checkout_switches_prefixed_branch(monkeypatch, tmp_path: Path, capsys) -> None:
    repo = tmp_path / "rocci"
    calls: list[tuple[str, ...]] = []

    def git(root: Path, *args: str, check: bool = True):
        calls.append(args)
        if args[:2] == ("status", "--porcelain"):
            return SimpleNamespace(returncode=0, stdout="", stderr="")
        if args[:2] == ("fetch", "origin"):
            return SimpleNamespace(returncode=0, stdout="", stderr="")
        if args[:2] == ("rev-parse", "--verify"):
            return SimpleNamespace(returncode=0, stdout="abc123\n", stderr="")
        if args[:2] == ("switch", "-C"):
            return SimpleNamespace(returncode=0, stdout="", stderr="")
        if args[0] == "branch":
            return SimpleNamespace(returncode=0, stdout="", stderr="")
        if check:
            raise AssertionError(args)
        return SimpleNamespace(returncode=1, stdout="", stderr="")

    monkeypatch.setattr("rocci_ops.pr_checkout._git", git)
    monkeypatch.setattr(
        "rocci_ops.pr_checkout._gh_head_ref",
        lambda _root, number: "feat/example-source-sidebar",
    )

    branch = checkout_pr(parse_pr_ref("#39"), root=repo)
    assert branch == "pr/feat/example-source-sidebar"
    assert ("fetch", "origin", "pull/39/head") in calls
    assert ("switch", "-C", "pr/feat/example-source-sidebar", "abc123") in calls
    assert not any(args[:2] == ("worktree", "add") for args in calls)
    assert "pr/feat/example-source-sidebar" in capsys.readouterr().out


def test_checkout_fetches_named_branch(monkeypatch, tmp_path: Path) -> None:
    calls: list[tuple[str, ...]] = []

    def git(root: Path, *args: str, check: bool = True):
        calls.append(args)
        if args[:2] == ("status", "--porcelain"):
            return SimpleNamespace(returncode=0, stdout="", stderr="")
        if args[:2] == ("fetch", "origin"):
            return SimpleNamespace(returncode=0, stdout="", stderr="")
        if args[:2] == ("rev-parse", "--verify"):
            return SimpleNamespace(returncode=0, stdout="def456\n", stderr="")
        if args[:2] == ("switch", "-C"):
            return SimpleNamespace(returncode=0, stdout="", stderr="")
        if args[0] == "branch":
            return SimpleNamespace(returncode=0, stdout="", stderr="")
        if check:
            raise AssertionError(args)
        return SimpleNamespace(returncode=1, stdout="", stderr="")

    monkeypatch.setattr("rocci_ops.pr_checkout._git", git)

    branch = checkout_pr(parse_pr_ref("feat/example-source-sidebar"), root=tmp_path)
    assert branch == "pr/feat/example-source-sidebar"
    assert ("fetch", "origin", "feat/example-source-sidebar") in calls
    assert ("switch", "-C", "pr/feat/example-source-sidebar", "def456") in calls


def test_checkout_refuses_dirty_worktree(monkeypatch, tmp_path: Path) -> None:
    def git(root: Path, *args: str, check: bool = True):
        if args[:2] == ("status", "--porcelain"):
            return SimpleNamespace(returncode=0, stdout=" M README.md\n", stderr="")
        raise AssertionError(args)

    monkeypatch.setattr("rocci_ops.pr_checkout._git", git)
    monkeypatch.setattr(
        "rocci_ops.pr_checkout._gh_head_ref",
        lambda *_: "feat/example-source-sidebar",
    )

    with pytest.raises(SystemExit, match="uncommitted"):
        checkout_pr(parse_pr_ref("39"), root=tmp_path)


def test_list_open_prs_runs_gh(monkeypatch, tmp_path: Path) -> None:
    calls: list[object] = []

    def run(cmd, **kwargs):
        calls.append((cmd, kwargs.get("cwd")))
        return SimpleNamespace(returncode=0)

    monkeypatch.setattr("rocci_ops.pr_checkout.subprocess.run", run)
    assert list_open_prs(root=tmp_path) == 0
    assert calls == [(["gh", "pr", "list", "--state", "open"], tmp_path)]


def test_main_without_ref_lists_prs(monkeypatch) -> None:
    monkeypatch.setattr("rocci_ops.pr_checkout.list_open_prs", lambda: 0)
    monkeypatch.setattr(
        "rocci_ops.pr_checkout.checkout_pr",
        lambda *args, **kwargs: (_ for _ in ()).throw(AssertionError("checkout")),
    )
    assert main([]) == 0


def test_dry_run_skips_git_mutators(monkeypatch, tmp_path: Path, capsys) -> None:
    monkeypatch.setattr(
        "rocci_ops.pr_checkout._git",
        lambda *args, **kwargs: (_ for _ in ()).throw(AssertionError(args)),
    )
    monkeypatch.setattr(
        "rocci_ops.pr_checkout._gh_head_ref",
        lambda *_: "feat/example-source-sidebar",
    )

    branch = checkout_pr(parse_pr_ref("39"), root=tmp_path, dry_run=True)
    assert branch == "pr/feat/example-source-sidebar"
    assert "#39" in capsys.readouterr().out
