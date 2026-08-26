from pathlib import Path

from rocci_ops.local import (
    CLI_CRATES,
    INSTALL_USAGE,
    PROMOTE_USAGE,
    PROMOTE_TAG_USAGE,
    _require_playground_dist,
    playground_wasm_artifact,
    build_site,
    install_command,
    latest_vsix,
    package_site,
    parse_worktrees,
    promote_command,
    promote_production,
    promote_staging,
    promote_tag,
    wait_for_promote_ci,
    render_brand_icons,
    require_darwin,
)


def test_package_vscode_does_not_copy_language_server() -> None:
    source = Path(__file__).resolve().parents[1] / "src" / "rocci_ops" / "local.py"
    text = source.read_text(encoding="utf-8")
    start = text.index("def package_vscode")
    body = text[start : text.index("\ndef ", start + 1)]
    assert "rocci-language-server" not in body
    assert "dist / \"bin\"" not in body
    assert "cargo" not in body


def test_cli_crates() -> None:
    assert CLI_CRATES[0] == ("rocci-cli", "rocci")
    assert {binary for _, binary in CLI_CRATES} == {"rocci", "rocdown"}


def test_install_usage() -> None:
    try:
        install_command([])
    except SystemExit as exc:
        assert str(exc) == INSTALL_USAGE
    else:
        raise AssertionError("expected SystemExit")


def test_promote_usage() -> None:
    try:
        promote_command(["preview"])
    except SystemExit as exc:
        assert str(exc) == PROMOTE_USAGE
    else:
        raise AssertionError("expected SystemExit")


def test_promote_tag_usage() -> None:
    try:
        promote_command(["tag"])
    except SystemExit as exc:
        assert str(exc) == PROMOTE_TAG_USAGE
    else:
        raise AssertionError("expected SystemExit")


def test_install_vscode_and_cursor_use_vsix(monkeypatch, tmp_path) -> None:
    vscode = tmp_path / "editors" / "vscode"
    vscode.mkdir(parents=True)
    vsix = vscode / "rocci-0.1.0.vsix"
    vsix.write_bytes(b"vsix")
    calls: list[list[str]] = []
    monkeypatch.setattr("rocci_ops.local.repo_root", lambda: tmp_path)
    monkeypatch.setattr("rocci_ops.local.run", lambda argv, cwd=None, env=None: calls.append(list(argv)))
    monkeypatch.setattr("rocci_ops.local.Path.home", lambda: tmp_path / "home")

    assert install_command(["vscode"]) == 0
    assert calls == [["code", "--install-extension", str(vsix)]]
    calls.clear()
    assert install_command(["cursor"]) == 0
    assert calls == [
        [
            "code",
            "--extensions-dir",
            str(tmp_path / "home" / ".cursor" / "extensions"),
            "--install-extension",
            str(vsix),
        ]
    ]


def test_promote_command_routes(monkeypatch) -> None:
    called: list[str] = []
    monkeypatch.setattr("rocci_ops.local.promote_staging", lambda: called.append("staging") or 0)
    monkeypatch.setattr("rocci_ops.local.promote_production", lambda: called.append("production") or 0)
    monkeypatch.setattr(
        "rocci_ops.local.promote_tag",
        lambda tag, from_ref="main": called.append(f"{tag}:{from_ref}") or 0,
    )
    assert promote_command(["staging"]) == 0
    assert promote_command(["production"]) == 0
    assert promote_command(["tag", "v1.2.3"]) == 0
    assert promote_command(["tag", "v1.2.3", "--from", "staging"]) == 0
    assert called == ["staging", "production", "v1.2.3:main", "v1.2.3:staging"]


def test_latest_vsix_picks_newest(tmp_path) -> None:
    vscode = tmp_path / "editors" / "vscode"
    vscode.mkdir(parents=True)
    older = vscode / "rocci-0.1.0.vsix"
    newer = vscode / "rocci-0.2.0.vsix"
    older.write_bytes(b"old")
    newer.write_bytes(b"new")
    older.touch()
    newer.touch()
    assert latest_vsix(tmp_path) == newer


def test_bundle_usage() -> None:
    from rocci_ops.local import main

    try:
        main(["bundle"])
    except SystemExit as exc:
        assert "macos" in str(exc)
        assert "okf" in str(exc)
    else:
        raise AssertionError("expected SystemExit")


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


def test_promote_tag_annotates_and_pushes_origin_main(monkeypatch, tmp_path) -> None:
    calls: list[list[str]] = []
    waited: list[str] = []
    monkeypatch.setattr("rocci_ops.local.repo_root", lambda: tmp_path)
    monkeypatch.setattr(
        "rocci_ops.local.subprocess.run",
        lambda *args, **kwargs: type("Result", (), {"returncode": 0, "stdout": "abc\n"})(),
    )
    monkeypatch.setattr(
        "rocci_ops.local.run",
        lambda argv, cwd=None, env=None: calls.append(list(argv)),
    )
    monkeypatch.setattr("rocci_ops.local.wait_for_promote_ci", waited.append)

    assert promote_tag("v1.2.3") == 0
    assert waited == ["abc"]
    assert calls == [
        ["git", "fetch", "origin"],
        ["git", "tag", "-a", "v1.2.3", "-m", "v1.2.3", "origin/main"],
        ["git", "push", "origin", "v1.2.3"],
    ]


def test_promote_tag_from_is_configurable(monkeypatch, tmp_path) -> None:
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
    monkeypatch.setattr("rocci_ops.local.wait_for_promote_ci", lambda sha: None)

    assert promote_tag("v1.2.3", from_ref="staging") == 0
    assert ["git", "tag", "-a", "v1.2.3", "-m", "v1.2.3", "origin/staging"] in calls


def test_promote_tag_force_moves_dev(monkeypatch, tmp_path) -> None:
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
    monkeypatch.setattr("rocci_ops.local.wait_for_promote_ci", lambda sha: None)

    assert promote_tag("dev") == 0
    assert calls == [
        ["git", "fetch", "origin"],
        ["git", "tag", "-a", "-f", "dev", "-m", "dev", "origin/main"],
        ["git", "push", "--force", "origin", "dev"],
    ]


def test_promote_tag_does_not_push_when_ci_fails(monkeypatch, tmp_path) -> None:
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
    monkeypatch.setattr(
        "rocci_ops.local.wait_for_promote_ci",
        lambda sha: (_ for _ in ()).throw(SystemExit(f"CI failed for {sha}")),
    )
    try:
        promote_tag("v1.2.3")
    except SystemExit as exc:
        assert "abc" in str(exc)
    else:
        raise AssertionError("expected SystemExit")
    assert calls == [["git", "fetch", "origin"]]


def test_wait_for_promote_ci_waits_default_checks(monkeypatch) -> None:
    from rocci_ops.release import DEFAULT_CHECKS

    seen: list[str] = []
    monkeypatch.setattr("rocci_ops.local.github_repo", lambda: "koliyo/rocci")
    monkeypatch.setattr("rocci_ops.local.gh_run", lambda args: type("R", (), {"stdout": ""})())
    monkeypatch.setattr(
        "rocci_ops.local.wait_for_check",
        lambda **kwargs: seen.append(kwargs["check"]),
    )
    wait_for_promote_ci("abc")
    assert seen == list(DEFAULT_CHECKS)


def test_promote_tag_requires_v_prefix_or_dev() -> None:
    try:
        promote_tag("1.2.3")
    except SystemExit as exc:
        assert "dev" in str(exc)
    else:
        raise AssertionError("expected SystemExit")


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
