from pathlib import Path

from rocci_ops.paths import repo_root


def test_example_caddy_routes_by_host_without_stealing_site_actions() -> None:
    root = repo_root()
    hybrid = (root / "docker/cdn/Caddyfile").read_text(encoding="utf-8")
    examples = (root / "docker/examples/Caddyfile").read_text(encoding="utf-8")
    compose = (root / "docker/compose.examples.yml").read_text(encoding="utf-8")
    assert "handle /actions/*" in hybrid
    assert "handle /sse" in hybrid
    assert "handle /actions/*" not in examples
    assert "live-counter.examples.rocci.dev" in examples
    assert "datastar.examples.rocci.dev" in examples
    assert "live-counter.examples.staging.rocci.dev" in examples
    assert "datastar:" in compose
    assert "live-counter:" in compose
    assert "snake" not in compose
    workflow = (root / ".github/workflows/site.yml").read_text(encoding="utf-8")
    assert '"examples/**"' in workflow or "- \"examples/**\"" in workflow
    assert "dist/examples-live/live-counter/server" in workflow
    assert "dist/examples-live/datastar/server" in workflow
    assert Path(root / "examples/rocci/apps.toml").is_file()


def test_retired_news_urls_have_exact_origin_dispositions() -> None:
    hybrid = (repo_root() / "docker/cdn/Caddyfile").read_text(encoding="utf-8")
    assert "redir /news/introducing-rocci/ /docs/start/what-is-rocci/ 308" in hybrid
    assert "redir /news/rocdown-static-collections/ /rocdown/site-config/ 308" in hybrid
    assert "redir /news/rocci-desktop-apps/ /docs/tutorials/ship/ 308" in hybrid
    assert "@retired_news path /news/ /news/feed.xml" in hybrid
    assert "respond @retired_news 410" in hybrid
    assert "path /news/*" not in hybrid
