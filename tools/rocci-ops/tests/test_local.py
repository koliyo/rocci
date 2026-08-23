from rocci_ops.local import (
    CLI_CRATES,
    _require_playground_dist,
    playground_wasm_artifact,
    build_site,
    package_site,
    parse_worktrees,
    promote_production,
    promote_staging,
    render_brand_icons,
    require_darwin,
)


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
    monkeypatch.setattr("rocci_ops.local.build_playground", lambda: 0)
    monkeypatch.setattr(
        "rocci_ops.local.run",
        lambda argv, cwd=None, env=None: calls.append(list(argv)),
    )

    assert build_site() == 0
    assert calls[0][4] == "rocci-docs"
    assert [call[-2] for call in calls[1:]] == ["check", "test", "build"]
    assert all(call[-1] == "site" for call in calls[1:])


def test_package_site_builds_playground_before_cargo(monkeypatch, tmp_path) -> None:
    calls: list[object] = []
    monkeypatch.setattr("rocci_ops.local.repo_root", lambda: tmp_path)
    monkeypatch.setattr(
        "rocci_ops.local.build_playground",
        lambda: calls.append("playground") or 0,
    )
    monkeypatch.setattr(
        "rocci_ops.local.stage_example_docs",
        lambda: calls.append("docs"),
    )
    monkeypatch.setattr(
        "rocci_ops.local.subprocess.run",
        lambda *args, **kwargs: type("Result", (), {"stdout": "\n", "returncode": 0})(),
    )
    monkeypatch.setattr(
        "rocci_ops.local.run",
        lambda argv, cwd=None, env=None: calls.append(list(argv)),
    )
    (tmp_path / "dist/examples-live").mkdir(parents=True)

    assert package_site(target="x64musl") == 0
    assert calls[0] == "playground"
    assert calls[1] == "docs"
    assert calls[2][4] == "rocci-rocdown-cli"
    assert calls[2][-4:] == ["package", "site", "--target", "x64musl"]


def test_require_playground_dist_rejects_empty_or_non_wasm(tmp_path) -> None:
    dist = tmp_path / "dist"
    dist.mkdir()
    (dist / "app.js").write_text("app", encoding="utf-8")
    (dist / "compiler-worker.js").write_text("worker", encoding="utf-8")
    (dist / "styles.css").write_text("css", encoding="utf-8")
    (dist / "compiler.wasm").write_bytes(b"")
    try:
        _require_playground_dist(dist)
    except SystemExit as exc:
        assert "compiler.wasm" in str(exc)
    else:
        raise AssertionError("expected SystemExit")

    (dist / "compiler.wasm").write_bytes(b"not-wasm")
    try:
        _require_playground_dist(dist)
    except SystemExit as exc:
        assert "WebAssembly" in str(exc)
    else:
        raise AssertionError("expected SystemExit")

    (dist / "compiler.wasm").write_bytes(b"\0asm" + b"\x01" * 8)
    _require_playground_dist(dist)


def test_playground_wasm_artifact_prefers_cargo_target_dir(monkeypatch, tmp_path) -> None:
    rel = "wasm32-unknown-unknown/release/rocci_playground_wasm.wasm"
    cached = tmp_path / "cache" / rel
    cached.parent.mkdir(parents=True)
    cached.write_bytes(b"\0asm" + b"\x01" * 8)
    workspace = tmp_path / "target" / rel
    workspace.parent.mkdir(parents=True)
    workspace.write_bytes(b"\0asm" + b"\x02" * 8)
    monkeypatch.setenv("CARGO_TARGET_DIR", str(tmp_path / "cache"))
    assert playground_wasm_artifact(tmp_path) == cached


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


def test_promote_staging_rebases_main_pushes_and_restores_branch(monkeypatch, tmp_path) -> None:
    calls: list[list[str]] = []
    monkeypatch.setattr("rocci_ops.local.repo_root", lambda: tmp_path)
    monkeypatch.setattr(
        "rocci_ops.local.subprocess.run",
        lambda *args, **kwargs: type("Result", (), {"stdout": "feature\n"})(),
    )
    monkeypatch.setattr("rocci_ops.local.run", lambda argv, cwd=None, env=None: calls.append(list(argv)))

    assert promote_staging() == 0
    assert calls == [
        ["git", "switch", "staging"],
        ["git", "rebase", "main"],
        ["git", "push", "origin", "staging"],
        ["git", "switch", "feature"],
    ]


def test_promote_production_pushes_origin_staging(monkeypatch, tmp_path) -> None:
    calls: list[list[str]] = []
    monkeypatch.setattr("rocci_ops.local.repo_root", lambda: tmp_path)
    monkeypatch.setattr(
        "rocci_ops.local.subprocess.run",
        lambda *args, **kwargs: type("Result", (), {"returncode": 0, "stdout": "abc\n"})(),
    )
    monkeypatch.setattr(
        "rocci_ops.local.run",
        lambda argv, cwd=None, env=None: calls.append(list(argv)),
    )

    assert promote_production() == 0
    assert calls == [
        ["git", "fetch", "origin"],
        ["git", "push", "origin", "origin/staging:refs/heads/production"],
    ]


def test_promote_production_requires_origin_staging(monkeypatch, tmp_path) -> None:
    monkeypatch.setattr("rocci_ops.local.repo_root", lambda: tmp_path)
    monkeypatch.setattr("rocci_ops.local.run", lambda argv, cwd=None, env=None: None)
    monkeypatch.setattr(
        "rocci_ops.local.subprocess.run",
        lambda *args, **kwargs: type("Result", (), {"returncode": 1, "stdout": ""})(),
    )
    try:
        promote_production()
    except SystemExit as exc:
        assert "origin/staging" in str(exc)
    else:
        raise AssertionError("expected SystemExit")


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
    (brand / "rocci-file-light.svg").write_text("<svg/>", encoding="utf-8")
    (brand / "rocci-mark.svg").write_text("<svg/>", encoding="utf-8")
    (tmp_path / "crates/rocci-desktop/assets").mkdir(parents=True)
    (tmp_path / "site/assets").mkdir(parents=True)
    calls: list[list[str]] = []
    copies: list[tuple[str, str]] = []

    monkeypatch.setattr(
        "rocci_ops.local.shutil.which",
        lambda name: "/usr/bin/rsvg-convert" if name == "rsvg-convert" else None,
    )
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
    assert len(copies) == 7
    assert any(dst.endswith("rocci-file-light.svg") for _, dst in copies)
    assert any(dst.endswith("text-x-rocci.svg") for _, dst in copies)


def test_render_brand_icons_generates_icns_when_iconutil_present(monkeypatch, tmp_path) -> None:
    from rocci_ops.local import ICONSET_SIZES

    brand = tmp_path / "brand"
    brand.mkdir()
    (brand / "rocci-app.svg").write_text("<svg/>", encoding="utf-8")
    (brand / "rocci-file.svg").write_text("<svg/>", encoding="utf-8")
    (brand / "rocci-file-light.svg").write_text("<svg/>", encoding="utf-8")
    (brand / "rocci-mark.svg").write_text("<svg/>", encoding="utf-8")
    (tmp_path / "crates/rocci-desktop/assets").mkdir(parents=True)
    (tmp_path / "site/assets").mkdir(parents=True)
    calls: list[list[str]] = []

    def which(name: str) -> str:
        return {
            "rsvg-convert": "/usr/bin/rsvg-convert",
            "iconutil": "/usr/bin/iconutil",
        }[name]

    monkeypatch.setattr("rocci_ops.local.shutil.which", which)
    monkeypatch.setattr("rocci_ops.local.repo_root", lambda: tmp_path)
    monkeypatch.setattr(
        "rocci_ops.local.run",
        lambda argv, cwd=None, env=None: calls.append(list(argv)),
    )
    monkeypatch.setattr("rocci_ops.local.shutil.copy2", lambda src, dst: None)

    assert render_brand_icons() == 0
    assert len(calls) == 3 + len(ICONSET_SIZES) + 1
    assert calls[-1][0] == "iconutil"
    assert calls[-1][-2] == str(brand / "rocci-app.icns")
    assert sum(1 for call in calls if call[0] == "rsvg-convert") == 3 + len(ICONSET_SIZES)
