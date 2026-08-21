from rocci_ops.local import CLI_CRATES, build_site, parse_worktrees, render_brand_icons, require_darwin


def test_cli_crates() -> None:
    assert CLI_CRATES[0] == ("rocci-cli", "rocci")
    assert {binary for _, binary in CLI_CRATES} == {"rocci", "rocdown", "rocci-okf"}


def test_package_site_usage() -> None:
    from rocci_ops.local import main

    try:
        main(["package"])
    except SystemExit as exc:
        assert "site" in str(exc)
    else:
        raise AssertionError("expected SystemExit")


def test_build_site_stages_checks_tests_and_builds(monkeypatch, tmp_path) -> None:
    calls: list[list[str]] = []
    monkeypatch.setattr("rocci_ops.local.repo_root", lambda: tmp_path)
    monkeypatch.setattr(
        "rocci_ops.local.run",
        lambda argv, cwd=None, env=None: calls.append(list(argv)),
    )

    assert build_site() == 0
    assert calls[0][4] == "rocci-docs"
    assert [call[-2] for call in calls[1:]] == ["check", "test", "build"]
    assert all(call[-1] == "site" for call in calls[1:])


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


def test_render_brand_icons_requires_rsvg(monkeypatch) -> None:
    monkeypatch.setattr("rocci_ops.local.shutil.which", lambda _: None)
    try:
        render_brand_icons()
    except SystemExit as exc:
        assert "rsvg-convert" in str(exc)
    else:
        raise AssertionError("expected SystemExit")


def test_render_brand_icons_invokes_rsvg(monkeypatch, tmp_path) -> None:
    brand = tmp_path / "brand"
    brand.mkdir()
    (brand / "rocci-app.svg").write_text("<svg/>", encoding="utf-8")
    (brand / "rocci-file.svg").write_text("<svg/>", encoding="utf-8")
    (brand / "rocci-mark.svg").write_text("<svg/>", encoding="utf-8")
    (tmp_path / "crates/rocci-desktop/assets").mkdir(parents=True)
    (tmp_path / "site/assets").mkdir(parents=True)
    calls: list[list[str]] = []
    copies: list[tuple[str, str]] = []

    monkeypatch.setattr("rocci_ops.local.shutil.which", lambda _: "/usr/bin/rsvg-convert")
    monkeypatch.setattr("rocci_ops.local.repo_root", lambda: tmp_path)
    monkeypatch.setattr(
        "rocci_ops.local.run",
        lambda argv, cwd=None, env=None: calls.append(list(argv)),
    )
    monkeypatch.setattr(
        "rocci_ops.local.shutil.copy2",
        lambda src, dst: copies.append((str(src), str(dst))),
    )

    assert render_brand_icons() == 0
    assert len(calls) == 3
    assert all(call[0] == "rsvg-convert" for call in calls)
    assert any("rocci-icon.png" in call[-1] for call in calls)
    assert any("apple-touch-icon.png" in call[-1] for call in calls)
    assert len(copies) == 3
