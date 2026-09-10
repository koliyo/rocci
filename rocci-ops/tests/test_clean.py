import os
from datetime import datetime, timezone

from rocci_ops.clean import (
    CACHE_DIR_NAME,
    clean_cursor_sandbox_cache,
    format_bytes,
    hashes_referenced_by_env,
    main,
    sandbox_cache_roots,
)
from rocci_ops.cli import main as cli_main


def _cache_tree(tmp_path, *, newer: str, older: str):
    root = tmp_path / CACHE_DIR_NAME
    new_dir = root / newer
    old_dir = root / older
    (new_dir / "cargo-target").mkdir(parents=True)
    (old_dir / "cargo-target").mkdir(parents=True)
    (new_dir / "cargo-target" / "a").write_bytes(b"n" * 100)
    (old_dir / "cargo-target" / "b").write_bytes(b"o" * 400)
    old_mtime = datetime(2026, 1, 1, tzinfo=timezone.utc).timestamp()
    new_mtime = datetime(2026, 9, 10, tzinfo=timezone.utc).timestamp()
    os.utime(old_dir, (old_mtime, old_mtime))
    os.utime(new_dir, (new_mtime, new_mtime))
    return root, new_dir, old_dir


def test_format_bytes() -> None:
    assert format_bytes(0) == "0 B"
    assert format_bytes(1024) == "1.0 KiB"
    assert format_bytes(int(2.5 * 1024**3)).startswith("2.5 GiB")


def test_hashes_referenced_by_env() -> None:
    env = {
        "CARGO_TARGET_DIR": "/var/folders/x/T/cursor-sandbox-cache/abc123/cargo-target",
        "PATH": "/usr/bin",
        "OTHER": "/tmp/cursor-sandbox-cache/def456/cargo",
    }
    assert hashes_referenced_by_env(env) == {"abc123", "def456"}


def test_sandbox_cache_roots_uses_tmp_arg(tmp_path) -> None:
    cache = tmp_path / CACHE_DIR_NAME
    cache.mkdir()
    roots = sandbox_cache_roots(tmp=tmp_path)
    assert cache in roots


def test_dry_run_does_not_delete(tmp_path, capsys) -> None:
    _root, new_dir, old_dir = _cache_tree(tmp_path, newer="aaaa", older="bbbb")
    assert (
        clean_cursor_sandbox_cache(
            apply=False,
            keep=[],
            keep_newest=1,
            tmp=tmp_path,
            environ={},
        )
        == 0
    )
    out = capsys.readouterr().out
    assert "keep (newest)" in out
    assert "drop" in out
    assert "dry-run" in out
    assert new_dir.is_dir()
    assert old_dir.is_dir()


def test_apply_keeps_newest_deletes_older(tmp_path) -> None:
    _root, new_dir, old_dir = _cache_tree(tmp_path, newer="aaaa", older="bbbb")
    assert (
        clean_cursor_sandbox_cache(
            apply=True,
            keep=[],
            keep_newest=1,
            tmp=tmp_path,
            environ={},
        )
        == 0
    )
    assert new_dir.is_dir()
    assert not old_dir.exists()


def test_env_protects_older_hash(tmp_path) -> None:
    _root, new_dir, old_dir = _cache_tree(tmp_path, newer="aaaa", older="bbbb")
    env = {"CARGO_TARGET_DIR": str(old_dir / "cargo-target")}
    assert (
        clean_cursor_sandbox_cache(
            apply=True,
            keep=[],
            keep_newest=1,
            tmp=tmp_path,
            environ=env,
        )
        == 0
    )
    assert new_dir.is_dir()
    assert old_dir.is_dir()


def test_keep_flag_protects_named_hash(tmp_path) -> None:
    _root, new_dir, old_dir = _cache_tree(tmp_path, newer="aaaa", older="bbbb")
    assert (
        clean_cursor_sandbox_cache(
            apply=True,
            keep=["bbbb"],
            keep_newest=1,
            tmp=tmp_path,
            environ={},
        )
        == 0
    )
    assert new_dir.is_dir()
    assert old_dir.is_dir()


def test_missing_target_prints_usage(capsys) -> None:
    assert main([]) == 2
    assert "cursor-sandbox-cache" in capsys.readouterr().out


def test_cli_dispatches_clean(monkeypatch) -> None:
    monkeypatch.setattr("rocci_ops.clean.main", lambda _argv: 0)
    try:
        cli_main(["clean", "cursor-sandbox-cache"])
    except SystemExit as exc:
        assert exc.code == 0
    else:
        raise AssertionError("expected SystemExit")
