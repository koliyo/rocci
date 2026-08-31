import subprocess
from pathlib import Path

from rocci_ops.ghutil import DEFAULT_CHECKS
from rocci_ops.release import (
    RELEASE_USAGE,
    push_version_update,
    release_command,
    run_release,
    wait_for_release_ci,
)
from rocci_ops.version import first_package_version, release_files_match

MINIMAL_CARGO = """\
[workspace]
members = [
    "crates/rocci-cli",
]

[workspace.package]
version = "0.1.0"
"""

MINIMAL_LOCK = """\
[[package]]
name = "rocci-cli"
version = "0.1.0"
"""


def test_release_usage() -> None:
    try:
        release_command([])
    except SystemExit as exc:
        assert str(exc) == RELEASE_USAGE
    else:
        raise AssertionError("expected SystemExit")


def test_release_command_routes(monkeypatch) -> None:
    called: list[str] = []
    monkeypatch.setattr(
        "rocci_ops.release.run_release",
        lambda tag, from_ref="main", force=False, dry_run=False: called.append(
            f"{tag}:{from_ref}:{force}:{dry_run}"
        )
        or 0,
    )
    assert release_command(["v1.2.3"]) == 0
    assert release_command(["v1.2.3", "--from", "release"]) == 0
    assert release_command(["v1.2.3", "--force"]) == 0
    assert release_command(["patch"]) == 0
    assert release_command(["patch", "--dry-run"]) == 0
    assert called == [
        "v1.2.3:main:False:False",
        "v1.2.3:release:False:False",
        "v1.2.3:main:True:False",
        "patch:main:False:False",
        "patch:main:False:True",
    ]


def test_run_release_dispatches_release_from_actions(monkeypatch, tmp_path) -> None:
    released: list[str] = []
    monkeypatch.setenv("GITHUB_ACTIONS", "true")
    monkeypatch.setattr("rocci_ops.release.repo_root", lambda: tmp_path)
    monkeypatch.setattr(
        "rocci_ops.release.git_capture",
        lambda *args, **kwargs: type("Result", (), {"returncode": 0, "stdout": "abc\n"})(),
    )
    monkeypatch.setattr("rocci_ops.release.run", lambda argv, cwd=None: None)
    monkeypatch.setattr(
        "rocci_ops.release.push_version_update",
        lambda version, from_ref, remote_sha: "newsha",
    )
    monkeypatch.setattr(
        "rocci_ops.release.wait_for_release_ci",
        lambda sha, from_ref="main": None,
    )
    monkeypatch.setattr("rocci_ops.release.dispatch_hosted_release", released.append)
    assert run_release("v1.2.3") == 0
    assert released == ["v1.2.3"]


def test_run_release_pushes_version_then_tags(monkeypatch, tmp_path) -> None:
    calls: list[list[str]] = []
    waited: list[str] = []
    monkeypatch.delenv("GITHUB_ACTIONS", raising=False)
    monkeypatch.setattr("rocci_ops.release.repo_root", lambda: tmp_path)
    monkeypatch.setattr(
        "rocci_ops.release.git_capture",
        lambda *args, **kwargs: type("Result", (), {"returncode": 0, "stdout": "abc\n"})(),
    )
    monkeypatch.setattr(
        "rocci_ops.release.run",
        lambda argv, cwd=None: calls.append(list(argv)),
    )
    monkeypatch.setattr(
        "rocci_ops.release.push_version_update",
        lambda version, from_ref, remote_sha: f"{version}:{from_ref}:{remote_sha}",
    )
    monkeypatch.setattr(
        "rocci_ops.release.wait_for_release_ci",
        lambda sha, from_ref="main": waited.append(sha),
    )

    assert run_release("v1.2.3") == 0
    assert waited == ["1.2.3:main:abc"]
    assert calls == [
        ["git", "fetch", "origin", "refs/heads/main:refs/remotes/origin/main"],
        ["git", "tag", "-a", "v1.2.3", "-m", "v1.2.3", "1.2.3:main:abc"],
        ["git", "push", "origin", "v1.2.3"],
    ]


def test_run_release_force_overwrites_versioned_tag(monkeypatch, tmp_path) -> None:
    calls: list[list[str]] = []
    monkeypatch.delenv("GITHUB_ACTIONS", raising=False)
    monkeypatch.setattr("rocci_ops.release.repo_root", lambda: tmp_path)
    monkeypatch.setattr(
        "rocci_ops.release.git_capture",
        lambda *args, **kwargs: type("Result", (), {"returncode": 0, "stdout": "abc\n"})(),
    )
    monkeypatch.setattr(
        "rocci_ops.release.run",
        lambda argv, cwd=None: calls.append(list(argv)),
    )
    monkeypatch.setattr(
        "rocci_ops.release.push_version_update",
        lambda version, from_ref, remote_sha: "newsha",
    )
    monkeypatch.setattr(
        "rocci_ops.release.wait_for_release_ci",
        lambda sha, from_ref="main": None,
    )

    assert run_release("v1.2.3", force=True) == 0
    assert calls == [
        ["git", "fetch", "origin", "refs/heads/main:refs/remotes/origin/main"],
        ["git", "tag", "-a", "-f", "v1.2.3", "-m", "v1.2.3", "newsha"],
        ["git", "push", "--force", "origin", "v1.2.3"],
    ]


def test_run_release_force_moves_dev(monkeypatch, tmp_path) -> None:
    calls: list[list[str]] = []
    monkeypatch.delenv("GITHUB_ACTIONS", raising=False)
    monkeypatch.setattr("rocci_ops.release.repo_root", lambda: tmp_path)
    monkeypatch.setattr(
        "rocci_ops.release.git_capture",
        lambda *args, **kwargs: type("Result", (), {"returncode": 0, "stdout": "abc\n"})(),
    )
    monkeypatch.setattr(
        "rocci_ops.release.run",
        lambda argv, cwd=None: calls.append(list(argv)),
    )
    monkeypatch.setattr(
        "rocci_ops.release.wait_for_release_ci",
        lambda sha, from_ref="main": None,
    )
    monkeypatch.setattr(
        "rocci_ops.release.push_version_update",
        lambda *args, **kwargs: (_ for _ in ()).throw(AssertionError("dev must not bump versions")),
    )

    assert run_release("dev") == 0
    assert calls == [
        ["git", "fetch", "origin", "refs/heads/main:refs/remotes/origin/main"],
        ["git", "tag", "-a", "-f", "dev", "-m", "dev", "abc"],
        ["git", "push", "--force", "origin", "dev"],
    ]


def test_run_release_does_not_push_when_ci_fails(monkeypatch, tmp_path) -> None:
    calls: list[list[str]] = []
    monkeypatch.setattr("rocci_ops.release.repo_root", lambda: tmp_path)
    monkeypatch.setattr(
        "rocci_ops.release.git_capture",
        lambda *args, **kwargs: type("Result", (), {"returncode": 0, "stdout": "abc\n"})(),
    )
    monkeypatch.setattr(
        "rocci_ops.release.run",
        lambda argv, cwd=None: calls.append(list(argv)),
    )
    monkeypatch.setattr(
        "rocci_ops.release.push_version_update",
        lambda version, from_ref, remote_sha: "newsha",
    )
    monkeypatch.setattr(
        "rocci_ops.release.wait_for_release_ci",
        lambda sha, from_ref="main": (_ for _ in ()).throw(SystemExit(f"CI failed for {sha}")),
    )
    try:
        run_release("v1.2.3")
    except SystemExit as exc:
        assert "newsha" in str(exc)
    else:
        raise AssertionError("expected SystemExit")
    assert calls == [["git", "fetch", "origin", "refs/heads/main:refs/remotes/origin/main"]]


def test_wait_for_release_ci_waits_default_checks(monkeypatch) -> None:
    seen: list[str] = []
    dispatched: list[list[str]] = []
    monkeypatch.delenv("GITHUB_ACTIONS", raising=False)
    monkeypatch.setattr("rocci_ops.release.github_repo", lambda: "koliyo/rocci")
    monkeypatch.setattr("rocci_ops.release.gh_run", lambda args: type("R", (), {"stdout": ""})())
    monkeypatch.setattr(
        "rocci_ops.release.dispatch_hosted_ci",
        lambda from_ref: dispatched.append([from_ref]),
    )
    monkeypatch.setattr(
        "rocci_ops.release.wait_for_check",
        lambda **kwargs: seen.append(kwargs["check"]),
    )
    wait_for_release_ci("abc")
    assert dispatched == []
    assert seen == list(DEFAULT_CHECKS)


def test_wait_for_release_ci_dispatches_from_actions(monkeypatch) -> None:
    seen: list[str] = []
    dispatched: list[str] = []
    monkeypatch.setenv("GITHUB_ACTIONS", "true")
    monkeypatch.setattr("rocci_ops.release.github_repo", lambda: "koliyo/rocci")
    monkeypatch.setattr("rocci_ops.release.gh_run", lambda args: type("R", (), {"stdout": ""})())
    monkeypatch.setattr(
        "rocci_ops.release.dispatch_hosted_ci",
        lambda from_ref: dispatched.append(from_ref),
    )
    monkeypatch.setattr(
        "rocci_ops.release.wait_for_check",
        lambda **kwargs: seen.append(kwargs["check"]),
    )
    wait_for_release_ci("abc", from_ref="release")
    assert dispatched == ["release"]
    assert seen == list(DEFAULT_CHECKS)


def _git(cwd: Path, *args: str) -> subprocess.CompletedProcess[str]:
    return subprocess.run(["git", *args], cwd=cwd, check=True, text=True, capture_output=True)


def test_push_version_update_commits_and_pushes(tmp_path: Path, monkeypatch) -> None:
    origin = tmp_path / "origin.git"
    repo = tmp_path / "repo"
    subprocess.run(["git", "init", "--bare", "-b", "main", str(origin)], check=True)
    subprocess.run(["git", "clone", str(origin), str(repo)], check=True)
    _git(repo, "config", "user.email", "ops@example.com")
    _git(repo, "config", "user.name", "ops")
    _git(repo, "config", "commit.gpgsign", "false")
    (repo / "Cargo.toml").write_text(MINIMAL_CARGO, encoding="utf-8")
    (repo / "Cargo.lock").write_text(MINIMAL_LOCK, encoding="utf-8")
    _git(repo, "add", ".")
    _git(repo, "commit", "-m", "seed")
    _git(repo, "push", "origin", "HEAD:main")
    sha = _git(repo, "rev-parse", "HEAD").stdout.strip()
    monkeypatch.setattr("rocci_ops.release.repo_root", lambda: repo)

    new_sha = push_version_update("2.3.4", "main", sha)
    assert new_sha != sha
    _git(repo, "fetch", "origin")
    checkout = tmp_path / "check"
    subprocess.run(["git", "clone", str(origin), str(checkout)], check=True)
    assert release_files_match(checkout, "2.3.4")
    assert first_package_version((checkout / "Cargo.toml").read_text(encoding="utf-8")) == "2.3.4"
    same = push_version_update("2.3.4", "main", new_sha)
    assert same == new_sha


def _show_result(text: str):
    return type("Result", (), {"returncode": 0, "stdout": text})()


def test_run_release_dry_run_does_not_push(monkeypatch, tmp_path, capsys) -> None:
    calls: list[list[str]] = []

    def capture(argv, cwd=None):
        if argv[:2] == ["git", "rev-parse"]:
            return type("Result", (), {"returncode": 0, "stdout": "abc\n"})()
        path = argv[2].split(":", 1)[1]
        if path == "Cargo.lock":
            return _show_result(MINIMAL_LOCK.replace("0.1.0", "1.2.3"))
        return _show_result(MINIMAL_CARGO.replace("0.1.0", "1.2.3"))

    monkeypatch.setattr("rocci_ops.release.repo_root", lambda: tmp_path)
    monkeypatch.setattr("rocci_ops.release.git_capture", capture)
    monkeypatch.setattr(
        "rocci_ops.release.run",
        lambda argv, cwd=None: calls.append(list(argv)),
    )
    monkeypatch.setattr(
        "rocci_ops.release.push_version_update",
        lambda *args, **kwargs: (_ for _ in ()).throw(AssertionError("dry-run must not bump")),
    )
    monkeypatch.setattr(
        "rocci_ops.release.wait_for_release_ci",
        lambda sha: (_ for _ in ()).throw(AssertionError("dry-run must not wait")),
    )

    assert run_release("v1.2.3", dry_run=True) == 0
    assert calls == [["git", "fetch", "origin", "refs/heads/main:refs/remotes/origin/main"]]
    out = capsys.readouterr().out
    assert "rocci-ops release v1.2.3" in out
    assert "dry-run: release files match=true" in out


def test_run_release_requires_v_prefix_or_dev() -> None:
    try:
        run_release("1.2.3")
    except SystemExit as exc:
        assert "dev" in str(exc)
    else:
        raise AssertionError("expected SystemExit")


def test_run_release_patch_resolves_from_sha(monkeypatch, tmp_path, capsys) -> None:
    calls: list[list[str]] = []
    waited: list[str] = []
    monkeypatch.delenv("GITHUB_ACTIONS", raising=False)
    cargo = MINIMAL_CARGO.replace("0.1.0", "0.1.2")
    lock = MINIMAL_LOCK.replace("0.1.0", "0.1.2")

    def capture(argv, cwd=None):
        if argv[:2] == ["git", "rev-parse"]:
            return type("Result", (), {"returncode": 0, "stdout": "abc\n"})()
        path = argv[2].split(":", 1)[1]
        if path == "Cargo.lock":
            return _show_result(lock)
        return _show_result(cargo)

    monkeypatch.setattr("rocci_ops.release.repo_root", lambda: tmp_path)
    monkeypatch.setattr("rocci_ops.release.git_capture", capture)
    monkeypatch.setattr(
        "rocci_ops.release.run",
        lambda argv, cwd=None: calls.append(list(argv)),
    )
    monkeypatch.setattr(
        "rocci_ops.release.push_version_update",
        lambda version, from_ref, remote_sha: f"{version}:{from_ref}:{remote_sha}",
    )
    monkeypatch.setattr(
        "rocci_ops.release.wait_for_release_ci",
        lambda sha, from_ref="main": waited.append(sha),
    )

    assert run_release("patch") == 0
    assert waited == ["0.1.3:main:abc"]
    assert calls == [
        ["git", "fetch", "origin", "refs/heads/main:refs/remotes/origin/main"],
        ["git", "tag", "-a", "v0.1.3", "-m", "v0.1.3", "0.1.3:main:abc"],
        ["git", "push", "origin", "v0.1.3"],
    ]
    assert "rocci-ops release v0.1.3" in capsys.readouterr().out
