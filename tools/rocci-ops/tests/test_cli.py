from rocci_ops.cli import CHECK_USAGE, USAGE, check_main, main


def test_top_level_usage_lists_grouped_commands() -> None:
    assert "check         deps | docs | zed" in USAGE
    assert "install       cli | vscode | cursor" in USAGE
    assert "promote       staging | production | tag" in USAGE
    assert "check-deps" not in USAGE
    assert "verify-zed" not in USAGE
    assert "install-cli" not in USAGE
    assert "promote-staging" not in USAGE


def test_check_usage_without_subcommand() -> None:
    assert check_main([]) == 2


def test_check_unknown_subcommand() -> None:
    assert check_main(["wasm"]) == 2


def test_check_help_is_ok() -> None:
    assert check_main(["--help"]) == 0
    assert "deps" in CHECK_USAGE


def test_origin_does_not_require_h35_desktop(monkeypatch) -> None:
    def boom() -> None:
        raise AssertionError("must not clone h35-desktop")

    monkeypatch.setattr("rocci_ops.cli.ensure_h35_desktop", boom)
    monkeypatch.setattr("rocci_ops.origin.main", lambda _argv: 0)
    try:
        main(["origin", "publish", "abc"])
    except SystemExit as exc:
        assert exc.code == 0
    else:
        raise AssertionError("expected SystemExit")


def test_unknown_top_level_command() -> None:
    try:
        main(["verify-zed"])
    except SystemExit as exc:
        assert exc.code == 2
    else:
        raise AssertionError("expected SystemExit")
