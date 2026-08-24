import tomllib
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
    assert "- \"playground/**\"" in workflow
    assert "dist/examples-live/live-counter/server" in workflow
    assert "dist/examples-live/datastar/server" in workflow
    assert Path(root / "examples/rocci/apps.toml").is_file()


def test_retired_news_urls_have_exact_origin_dispositions() -> None:
    hybrid = (repo_root() / "docker/cdn/Caddyfile").read_text(encoding="utf-8")
    assert "redir " not in hybrid
    assert "@retired_news path /news /news/ /news/feed.xml" in hybrid
    assert "handle @retired_news" in hybrid
    assert "respond 410" in hybrid
    assert "respond @retired_news 410" not in hybrid
    assert "Content-Type application/wasm" in hybrid
    assert "path /news/*" not in hybrid


def test_examples_nav_matches_site_true_catalog_ids() -> None:
    root = repo_root()
    catalog = tomllib.loads((root / "examples/rocci/apps.toml").read_text(encoding="utf-8"))
    site_ids = {app["id"] for app in catalog["app"] if app.get("site", True)}
    excluded = {app["id"] for app in catalog["app"] if not app.get("site", True)}
    site_cfg = tomllib.loads((root / "site/rocdown.toml").read_text(encoding="utf-8"))
    examples = next(nav for nav in site_cfg["nav"] if nav.get("label") == "Examples")
    items = examples["items"]
    assert items[0] == "examples/index"
    nav_ids = []
    for item in items[1:]:
        assert item.startswith("examples/")
        assert item.endswith("/index")
        nav_ids.append(item.removeprefix("examples/").removesuffix("/index"))
    assert set(nav_ids) == site_ids
    assert excluded.isdisjoint(nav_ids)


def test_package_and_build_site_do_not_pass_all() -> None:
    local = (repo_root() / "tools/rocci-ops/src/rocci_ops/local.py").read_text(encoding="utf-8")
    assert '"--all"' not in local
    assert "'--all'" not in local
