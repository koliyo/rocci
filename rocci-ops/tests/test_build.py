from rocci_ops import build


def test_build_lsp_runs_debug_cargo(monkeypatch) -> None:
    captured: list[list[str]] = []
    monkeypatch.setattr("rocci_ops.build.run", lambda argv, cwd=None: captured.append(list(argv)))
    assert build.build_command(["lsp"]) == 0
    assert captured == [["cargo", "build", "-p", "rocci-rocdown-lsp"]]


def test_build_lsp_release(monkeypatch) -> None:
    captured: list[list[str]] = []
    monkeypatch.setattr("rocci_ops.build.run", lambda argv, cwd=None: captured.append(list(argv)))
    assert build.build_command(["lsp", "--release"]) == 0
    assert captured == [["cargo", "build", "--release", "-p", "rocci-rocdown-lsp"]]


def test_build_lsp_rejects_extra_args() -> None:
    try:
        build.build_command(["lsp", "--debug"])
    except SystemExit as exc:
        assert exc.code == build.BUILD_USAGE
    else:
        raise AssertionError("expected SystemExit")
